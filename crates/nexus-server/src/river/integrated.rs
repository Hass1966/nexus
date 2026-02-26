use anyhow::{Context, Result};
use uuid::Uuid;

use crate::api::state::AppState;
use crate::perspective::engine as perspective;
use crate::river::{beliefs, consciousness, episodic};
use crate::shared::ollama::ChatMessage;
use nexus_common::types::AnalysisResult;

/// Integrated mode: River + Perspective combined.
///
/// Flow:
/// 1. Extract claims from the user's message
/// 2. Run Perspective 4-layer analysis on the claims
/// 3. Recall relevant episodic memories
/// 4. Detect contradictions with existing beliefs
/// 5. Generate a Socratic question informed by the discourse analysis insights
/// 6. Store everything and update metrics
pub async fn process_integrated(
    state: &AppState,
    session_id: Uuid,
    user_id: Uuid,
    message: &str,
) -> Result<(String, AnalysisResult)> {
    let message_id = Uuid::new_v4();

    // Run Perspective analysis and memory recall in parallel.
    let (analysis_result, memories, extracted_beliefs) = tokio::try_join!(
        perspective::analyze_text(state, message),
        async {
            episodic::recall_similar(state, user_id, message, 5)
                .await
                .or_else(|_| Ok(Vec::new()))
        },
        async {
            beliefs::extract_beliefs(state, message)
                .await
                .or_else(|_| Ok(Vec::new()))
        },
    )?;

    // Detect contradictions for extracted beliefs.
    let mut contradictions = Vec::new();
    for claim in &extracted_beliefs {
        let contras = beliefs::detect_contradictions(state, user_id, &claim.claim)
            .await
            .unwrap_or_default();
        contradictions.extend(contras);
    }

    // Store beliefs.
    for claim in &extracted_beliefs {
        let _ = beliefs::store_belief(state, user_id, claim, message_id).await;
    }

    // Store episodic memory.
    let _ = episodic::store_memory(state, user_id, session_id, message_id, message, "user").await;

    // Build rich context from Perspective analysis.
    let analysis_insights = build_analysis_context(&analysis_result);

    let memory_context = if memories.is_empty() {
        String::new()
    } else {
        let mem_texts: Vec<String> = memories
            .iter()
            .map(|m| format!("[{}] {}: {}", m.timestamp, m.role, m.content))
            .collect();
        format!("\n\nRelevant past conversations:\n{}", mem_texts.join("\n"))
    };

    let contradiction_context = if contradictions.is_empty() {
        String::new()
    } else {
        let texts: Vec<String> = contradictions
            .iter()
            .map(|c| {
                format!(
                    "- \"{}\" contradicts \"{}\" ({})",
                    c.belief_b.claim, c.belief_a.claim, c.explanation
                )
            })
            .collect();
        format!("\n\nContradictions detected:\n{}", texts.join("\n"))
    };

    // Generate integrated Socratic response.
    let system_prompt = format!(
        r#"You are an integrated epistemic dialogue partner that combines Socratic questioning with critical discourse analysis. You have performed a deep analysis of the user's statement and discovered specific linguistic patterns and hidden assumptions.

DISCOURSE ANALYSIS INSIGHTS:
{analysis_insights}
{memory_context}
{contradiction_context}

Your task:
1. Use the discourse analysis to identify the most significant epistemic gap in the user's statement
2. Ask ONE precise Socratic question that helps the user examine their own framing
3. Reference specific findings (e.g., "You used the word 'always' — what exceptions might exist?")
4. Do NOT lecture about discourse analysis — use the insights to ask better questions
5. Be genuinely curious and non-judgmental
6. If contradictions were found, gently surface the most significant one

The question should be something the user has NOT considered, directly informed by the analysis."#
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
        .context("Failed to generate integrated response")?;

    // Store response as memory.
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

    // Update consciousness metrics.
    let existing = beliefs::get_user_beliefs(state, user_id)
        .await
        .unwrap_or_default();
    let _ = consciousness::compute_metrics(
        state,
        user_id,
        session_id,
        existing.len(),
        contradictions.len(),
        1,
        0,
    )
    .await;

    Ok((response, analysis_result))
}

/// Build a human-readable summary of Perspective analysis for the Socratic prompt.
fn build_analysis_context(analysis: &AnalysisResult) -> String {
    let mut parts = Vec::new();

    // Syntactic highlights.
    if !analysis.syntactic.nominalisations.is_empty() {
        let noms: Vec<String> = analysis
            .syntactic
            .nominalisations
            .iter()
            .map(|n| format!("\"{}\" (hides verb: {})", n.original, n.verb_form))
            .collect();
        parts.push(format!("Nominalisations found: {}", noms.join(", ")));
    }

    let passive_count = analysis
        .syntactic
        .voice_analysis
        .iter()
        .filter(|v| v.voice == nexus_common::types::VoiceType::Passive)
        .count();
    if passive_count > 0 {
        parts.push(format!(
            "Passive voice used {passive_count} time(s) — agency is obscured"
        ));
    }

    // Semantic highlights.
    if !analysis.semantic.presuppositions.is_empty() {
        let presups: Vec<String> = analysis
            .semantic
            .presuppositions
            .iter()
            .map(|p| format!("\"{}\" presupposes: {}", p.trigger, p.presupposed_content))
            .collect();
        parts.push(format!("Presuppositions: {}", presups.join("; ")));
    }

    if !analysis.semantic.power_hierarchies.is_empty() {
        let powers: Vec<String> = analysis
            .semantic
            .power_hierarchies
            .iter()
            .map(|p| format!("{} > {}", p.dominant, p.subordinate))
            .collect();
        parts.push(format!("Power hierarchies implied: {}", powers.join(", ")));
    }

    // Discourse highlights.
    if !analysis.discourse.framing.is_empty() {
        let frames: Vec<String> = analysis
            .discourse
            .framing
            .iter()
            .map(|f| format!("{}: {}", f.frame_name, f.effect))
            .collect();
        parts.push(format!("Framing patterns: {}", frames.join("; ")));
    }

    if !analysis.discourse.strategic_omissions.is_empty() {
        let omissions: Vec<String> = analysis
            .discourse
            .strategic_omissions
            .iter()
            .map(|o| o.what_is_missing.clone())
            .collect();
        parts.push(format!("Strategic omissions: {}", omissions.join("; ")));
    }

    // Critical synthesis highlights.
    if !analysis.critical_synthesis.naturalised_claims.is_empty() {
        let claims: Vec<String> = analysis
            .critical_synthesis
            .naturalised_claims
            .iter()
            .map(|c| format!("\"{}\"", c.claim))
            .collect();
        parts.push(format!(
            "Claims presented as natural/obvious: {}",
            claims.join(", ")
        ));
    }

    if !analysis.critical_synthesis.alternative_framings.is_empty() {
        let alts: Vec<String> = analysis
            .critical_synthesis
            .alternative_framings
            .iter()
            .map(|a| a.alternative.clone())
            .collect();
        parts.push(format!(
            "Alternative framings possible: {}",
            alts.join("; ")
        ));
    }

    if parts.is_empty() {
        "No significant discourse patterns detected.".to_string()
    } else {
        parts.join("\n")
    }
}
