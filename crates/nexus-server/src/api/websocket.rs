use axum::{
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::state::AppState;
use nexus_common::types::ChatMode;

#[derive(Debug, Deserialize)]
struct WsIncoming {
    message: String,
    #[serde(default)]
    mode: ChatMode,
}

#[derive(Debug, Serialize)]
struct WsOutgoing {
    #[serde(rename = "type")]
    msg_type: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    analysis: Option<serde_json::Value>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(session_id): Path<Uuid>,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, session_id, state))
}

async fn handle_socket(socket: WebSocket, session_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();

    tracing::info!(%session_id, "WebSocket connected");

    // Send welcome message.
    let welcome = WsOutgoing {
        msg_type: "connected".into(),
        content: format!("Session {session_id} established"),
        analysis: None,
    };
    if let Ok(json) = serde_json::to_string(&welcome) {
        let _ = sender.send(Message::Text(json.into())).await;
    }

    while let Some(Ok(msg)) = receiver.next().await {
        match msg {
            Message::Text(text) => {
                let incoming: WsIncoming = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        let err = WsOutgoing {
                            msg_type: "error".into(),
                            content: format!("Invalid message format: {e}"),
                            analysis: None,
                        };
                        if let Ok(json) = serde_json::to_string(&err) {
                            let _ = sender.send(Message::Text(json.into())).await;
                        }
                        continue;
                    }
                };

                // Send thinking indicator.
                let thinking = WsOutgoing {
                    msg_type: "thinking".into(),
                    content: "Processing...".into(),
                    analysis: None,
                };
                if let Ok(json) = serde_json::to_string(&thinking) {
                    let _ = sender.send(Message::Text(json.into())).await;
                }

                // Process through the appropriate engine.
                let response = process_ws_message(&state, session_id, &incoming).await;

                if let Ok(json) = serde_json::to_string(&response) {
                    let _ = sender.send(Message::Text(json.into())).await;
                }
            }
            Message::Close(_) => {
                tracing::info!(%session_id, "WebSocket closed");
                break;
            }
            _ => {}
        }
    }
}

async fn process_ws_message(
    state: &AppState,
    session_id: Uuid,
    incoming: &WsIncoming,
) -> WsOutgoing {
    match incoming.mode {
        ChatMode::Conversation => {
            match crate::river::dialogue::process_message(
                state,
                session_id,
                // Use a placeholder user_id for WS (auth should be added).
                Uuid::nil(),
                &incoming.message,
            )
            .await
            {
                Ok(response) => WsOutgoing {
                    msg_type: "response".into(),
                    content: response,
                    analysis: None,
                },
                Err(e) => WsOutgoing {
                    msg_type: "error".into(),
                    content: format!("River error: {e}"),
                    analysis: None,
                },
            }
        }
        ChatMode::Analysis => {
            match crate::perspective::engine::analyze_text(state, &incoming.message).await {
                Ok(result) => WsOutgoing {
                    msg_type: "analysis".into(),
                    content: "Analysis complete".into(),
                    analysis: serde_json::to_value(&result).ok(),
                },
                Err(e) => WsOutgoing {
                    msg_type: "error".into(),
                    content: format!("Perspective error: {e}"),
                    analysis: None,
                },
            }
        }
        ChatMode::Integrated => {
            match crate::river::integrated::process_integrated(
                state,
                session_id,
                Uuid::nil(),
                &incoming.message,
            )
            .await
            {
                Ok((response, analysis)) => WsOutgoing {
                    msg_type: "integrated".into(),
                    content: response,
                    analysis: serde_json::to_value(&analysis).ok(),
                },
                Err(e) => WsOutgoing {
                    msg_type: "error".into(),
                    content: format!("Integrated mode error: {e}"),
                    analysis: None,
                },
            }
        }
    }
}
