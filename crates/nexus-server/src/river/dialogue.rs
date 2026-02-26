use anyhow::{Context, Result};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::river::{beliefs, consciousness, episodic};
use crate::shared::ollama::ChatMessage;

/// Process a user message through the River epistemic dialogue engine.
///
/// Flow:
/// 1. Recall relevant past memories
/// 2. Extract beliefs from the current message
/// 3. Check for contradictions against existing beliefs
/// 4. Store new beliefs and memory
/// 5. Generate a Socratic response using all context
/// 6. Update consciousness metrics
pub async fn process_message(
    state: &AppState,
    session_id: Uuid,
    user_id: Uuid,
    message: &str,
) -> Result<String> {
    let message_id = Uuid::new_v4();

    // 1. Recall relevant past conversations.
    let memories = episodic::recall_similar(state, user_id, message, 5)
        .await
        .unwrap_or_default();

    let memory_context = if memories.is_empty() {
        String::new()
    } else {
        let mem_texts: Vec<String> = memories
            .iter()
            .map(|m| format!("[{}] {}: {}", m.timestamp, m.role, m.content))
            .collect();
        format!("\n\nRelevant past conversations:\n{}", mem_texts.join("\n"))
    };

    // 2. Extract beliefs from the message.
    let extracted = beliefs::extract_beliefs(state, message)
        .await
        .unwrap_or_default();

    // 3. Check for contradictions.
    let mut all_contradictions = Vec::new();
    for claim in &extracted {
        let contras = beliefs::detect_contradictions(state, user_id, &claim.claim)
            .await
            .unwrap_or_default();
        all_contradictions.extend(contras);
    }

    let contradiction_context = if all_contradictions.is_empty() {
        String::new()
    } else {
        let contra_texts: Vec<String> = all_contradictions
            .iter()
            .map(|c| {
                format!(
                    "- Current: \"{}\" contradicts previous: \"{}\" ({})",
                    c.belief_b.claim, c.belief_a.claim, c.explanation
                )
            })
            .collect();
        format!("\n\nContradictions detected:\n{}", contra_texts.join("\n"))
    };

    // 4. Store new beliefs.
    let mut stored_beliefs = Vec::new();
    for claim in &extracted {
        match beliefs::store_belief(state, user_id, claim, message_id).await {
            Ok(b) => stored_beliefs.push(b),
            Err(e) => tracing::warn!("Failed to store belief: {e}"),
        }
    }

    // Link contradictions in Neo4j.
    for contra in &all_contradictions {
        let new_belief = stored_beliefs
            .iter()
            .find(|b| b.claim == contra.belief_b.claim);
        if let Some(new_b) = new_belief {
            let _ = beliefs::link_contradiction(
                state,
                contra.belief_a.id,
                new_b.id,
                &contra.explanation,
                contra.severity,
            )
            .await;
        }
    }

    // 5. Store this message as episodic memory.
    let _ = episodic::store_memory(state, user_id, session_id, message_id, message, "user").await;

    // 6. Retrieve existing beliefs for context.
    let existing_beliefs = beliefs::get_user_beliefs(state, user_id)
        .await
        .unwrap_or_default();

    let beliefs_context = if existing_beliefs.is_empty() {
        String::new()
    } else {
        let belief_texts: Vec<String> = existing_beliefs
            .iter()
            .take(20)
            .map(|b| format!("- \"{}\" (confidence: {:.1})", b.claim, b.confidence))
            .collect();
        format!(
            "\n\nUser's current belief network:\n{}",
            belief_texts.join("\n")
        )
    };

    // 7. Generate Socratic response.
    let system_prompt = format!(
        r#"You are a Socratic dialogue partner focused on epistemic exploration. Your role is NOT to provide answers but to ask questions that help the user examine their own beliefs, assumptions, and reasoning.

Guidelines:
- Ask ONE focused question at a time
- Target the user's actual epistemic gaps — what they haven't considered, not what they already know
- When contradictions are detected, gently surface them without judgment
- Reference past conversations when relevant to show continuity of thought
- Never lecture or give opinions — only ask questions
- Be genuinely curious, not rhetorical
- If the user makes a universal claim, probe the boundaries
- If the user uses loaded language, ask them to define their terms{memory_context}{beliefs_context}{contradiction_context}"#
    );

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: system_prompt,
        },
        ChatMessage {
            role: "user".into(),
            content: message.to_string(),
        },
    ];

    let response = state
        .ollama
        .chat(&messages)
        .await
        .context("Failed to generate Socratic response")?;

    // Store assistant response as memory too.
    let response_id = Uuid::new_v4();
    let _ = episodic::store_memory(
        state,
        user_id,
        session_id,
        response_id,
        &response,
        "assistant",
    )
    .await;

    // 8. Update consciousness metrics.
    let _ = consciousness::compute_metrics(
        state,
        user_id,
        session_id,
        existing_beliefs.len() + stored_beliefs.len(),
        all_contradictions.len(),
        1, // This message counts as engagement.
        0, // Beliefs revised is tracked separately.
    )
    .await;

    Ok(response)
}

/// Load session context from Redis for continuity.
pub async fn get_session_context(state: &AppState, session_id: Uuid) -> Result<Vec<ChatMessage>> {
    let mut conn = state.db.redis.clone();
    let key = format!("session:{session_id}:messages");

    let raw: Option<String> = ::redis::cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap_or(None);

    match raw {
        Some(json) => {
            let messages: Vec<ChatMessage> = serde_json::from_str(&json).unwrap_or_default();
            Ok(messages)
        }
        None => Ok(Vec::new()),
    }
}

/// Save session context to Redis.
pub async fn save_session_context(
    state: &AppState,
    session_id: Uuid,
    messages: &[ChatMessage],
) -> Result<()> {
    let mut conn = state.db.redis.clone();
    let key = format!("session:{session_id}:messages");
    let json = serde_json::to_string(messages)?;

    // Expire after 24 hours.
    ::redis::cmd("SET")
        .arg(&key)
        .arg(&json)
        .arg("EX")
        .arg(86400)
        .query_async::<()>(&mut conn)
        .await
        .context("Failed to save session to Redis")?;

    Ok(())
}
