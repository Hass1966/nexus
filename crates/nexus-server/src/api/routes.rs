use axum::{
    Json, Router,
    extract::{Path, State},
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::api::error::AppError;
use crate::api::middleware::AuthUser;
use crate::api::state::AppState;
use crate::api::websocket::ws_handler;
use crate::models::auth as jwt;
use crate::models::requests::*;
use crate::models::responses::*;

pub fn create_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        // Public routes.
        .route("/health", get(health_handler))
        .route("/api/v1/auth/register", post(register_handler))
        .route("/api/v1/auth/login", post(login_handler))
        // Protected routes (AuthUser extractor validates JWT).
        .route("/api/v1/chat", post(chat_handler))
        .route("/api/v1/analyze", post(analyze_handler))
        .route("/api/v1/beliefs/{user_id}", get(beliefs_handler))
        .route("/api/v1/consciousness/state", get(consciousness_handler))
        // WebSocket.
        .route("/ws/chat/{session_id}", get(ws_handler))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

// ── Health Check ──

async fn health_handler(State(state): State<AppState>) -> Json<HealthResponse> {
    let pg_status = match sqlx::query("SELECT 1").execute(&state.db.pg).await {
        Ok(_) => ServiceStatus::up(),
        Err(e) => ServiceStatus::down(e.to_string()),
    };

    let neo4j_status = match state.db.neo4j.run(neo4rs::query("RETURN 1")).await {
        Ok(_) => ServiceStatus::up(),
        Err(e) => ServiceStatus::down(e.to_string()),
    };

    let qdrant_status = match state.db.qdrant.list_collections().await {
        Ok(_) => ServiceStatus::up(),
        Err(e) => ServiceStatus::down(e.to_string()),
    };

    let influx_status = match state.db.influx.ready().await {
        Ok(_) => ServiceStatus::up(),
        Err(e) => ServiceStatus::down(e.to_string()),
    };

    let redis_status = {
        let mut conn = state.db.redis.clone();
        match ::redis::cmd("PING").query_async::<String>(&mut conn).await {
            Ok(_) => ServiceStatus::up(),
            Err(e) => ServiceStatus::down(e.to_string()),
        }
    };

    let ollama_status = match state.ollama.health().await {
        Ok(true) => ServiceStatus::up(),
        Ok(false) => ServiceStatus::down("Ollama not healthy".into()),
        Err(e) => ServiceStatus::down(e.to_string()),
    };

    let all_up = [
        &pg_status,
        &neo4j_status,
        &qdrant_status,
        &influx_status,
        &redis_status,
        &ollama_status,
    ]
    .iter()
    .all(|s| s.status == "up");

    Json(HealthResponse {
        status: if all_up { "healthy" } else { "degraded" }.into(),
        services: HealthServices {
            postgres: pg_status,
            neo4j: neo4j_status,
            qdrant: qdrant_status,
            influxdb: influx_status,
            redis: redis_status,
            ollama: ollama_status,
        },
    })
}

// ── Auth ──

async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    use nexus_common::error::NexusError;

    let password_hash = hash_password(req.password.as_bytes());

    let user_id = Uuid::new_v4();
    sqlx::query("INSERT INTO users (id, username, email, password_hash) VALUES ($1, $2, $3, $4)")
        .bind(user_id)
        .bind(&req.username)
        .bind(&req.email)
        .bind(&password_hash)
        .execute(&state.db.pg)
        .await
        .map_err(|e| NexusError::Database(format!("Failed to create user: {e}")))?;

    let token = jwt::create_token(
        user_id,
        &req.username,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        user_id,
        username: req.username,
    }))
}

async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    use nexus_common::error::NexusError;

    let password_hash = hash_password(req.password.as_bytes());

    let row = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT id, username FROM users WHERE email = $1 AND password_hash = $2",
    )
    .bind(&req.email)
    .bind(&password_hash)
    .fetch_optional(&state.db.pg)
    .await
    .map_err(|e| NexusError::Database(e.to_string()))?
    .ok_or_else(|| NexusError::Auth("Invalid credentials".into()))?;

    let token = jwt::create_token(
        row.0,
        &row.1,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(Json(AuthResponse {
        token,
        user_id: row.0,
        username: row.1,
    }))
}

fn hash_password(data: &[u8]) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}

// ── Chat ──

async fn chat_handler(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, AppError> {
    let session_id = req.session_id.unwrap_or_else(Uuid::new_v4);
    let user_id = claims.sub;
    let mode_str = match req.mode {
        nexus_common::types::ChatMode::Conversation => "conversation",
        nexus_common::types::ChatMode::Analysis => "analysis",
        nexus_common::types::ChatMode::Integrated => "integrated",
    };

    // Ensure session exists.
    ensure_session(&state, session_id, user_id, mode_str).await?;

    // Save user message.
    save_message(&state, session_id, user_id, "user", &req.message, mode_str).await?;

    match req.mode {
        nexus_common::types::ChatMode::Conversation => {
            let response =
                crate::river::dialogue::process_message(&state, session_id, user_id, &req.message)
                    .await?;

            save_message(&state, session_id, user_id, "assistant", &response, mode_str).await?;

            Ok(Json(ChatResponse {
                session_id,
                message: response,
                mode: mode_str.into(),
                analysis: None,
                contradictions: None,
                beliefs_updated: None,
            }))
        }
        nexus_common::types::ChatMode::Analysis => {
            let analysis = crate::perspective::engine::analyze_text(&state, &req.message).await?;

            let summary = "Analysis complete.";
            save_message(&state, session_id, user_id, "assistant", summary, mode_str).await?;

            Ok(Json(ChatResponse {
                session_id,
                message: summary.into(),
                mode: mode_str.into(),
                analysis: Some(analysis),
                contradictions: None,
                beliefs_updated: None,
            }))
        }
        nexus_common::types::ChatMode::Integrated => {
            let (response, analysis) = crate::river::integrated::process_integrated(
                &state,
                session_id,
                user_id,
                &req.message,
            )
            .await?;

            save_message(&state, session_id, user_id, "assistant", &response, mode_str).await?;

            Ok(Json(ChatResponse {
                session_id,
                message: response,
                mode: mode_str.into(),
                analysis: Some(analysis),
                contradictions: None,
                beliefs_updated: None,
            }))
        }
    }
}

async fn ensure_session(
    state: &AppState,
    session_id: Uuid,
    user_id: Uuid,
    mode: &str,
) -> Result<(), AppError> {
    use nexus_common::error::NexusError;
    sqlx::query(
        "INSERT INTO sessions (id, user_id, mode) VALUES ($1, $2, $3) ON CONFLICT (id) DO NOTHING",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(mode)
    .execute(&state.db.pg)
    .await
    .map_err(|e| NexusError::Database(format!("Failed to ensure session: {e}")))?;
    Ok(())
}

async fn save_message(
    state: &AppState,
    session_id: Uuid,
    user_id: Uuid,
    role: &str,
    content: &str,
    mode: &str,
) -> Result<(), AppError> {
    use nexus_common::error::NexusError;
    sqlx::query(
        "INSERT INTO messages (id, session_id, user_id, role, content, mode) VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::new_v4())
    .bind(session_id)
    .bind(user_id)
    .bind(role)
    .bind(content)
    .bind(mode)
    .execute(&state.db.pg)
    .await
    .map_err(|e| NexusError::Database(format!("Failed to save message: {e}")))?;
    Ok(())
}

// ── Analyze ──

async fn analyze_handler(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Json(req): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, AppError> {
    let analysis = crate::perspective::engine::analyze_text(&state, &req.text).await?;
    Ok(Json(AnalyzeResponse { analysis }))
}

// ── Beliefs ──

async fn beliefs_handler(
    State(state): State<AppState>,
    AuthUser(_claims): AuthUser,
    Path(user_id): Path<Uuid>,
) -> Result<Json<BeliefsResponse>, AppError> {
    let beliefs = crate::river::beliefs::get_user_beliefs(&state, user_id).await?;
    let total = beliefs.len();
    Ok(Json(BeliefsResponse {
        user_id,
        beliefs,
        total,
    }))
}

// ── Consciousness ──

async fn consciousness_handler(
    State(state): State<AppState>,
    AuthUser(claims): AuthUser,
) -> Result<Json<ConsciousnessResponse>, AppError> {
    let consciousness_state =
        crate::river::consciousness::get_current_state(&state, claims.sub).await?;
    Ok(Json(ConsciousnessResponse {
        state: consciousness_state,
    }))
}
