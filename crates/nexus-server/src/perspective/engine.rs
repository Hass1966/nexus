use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::api::state::AppState;
use crate::perspective::{cache, discourse, semantic, syntactic, synthesis};
use nexus_common::types::AnalysisResult;

/// Run full 4-layer Perspective analysis on the given text.
/// Results are cached in Redis.
pub async fn analyze_text(state: &AppState, text: &str) -> Result<AnalysisResult> {
    // Check cache first.
    if let Ok(Some(cached)) = cache::get_cached(state, text).await {
        return Ok(cached);
    }

    tracing::info!("Running full 4-layer Perspective analysis");

    // Run all 4 layers in parallel.
    let (syntactic_result, semantic_result, discourse_result, synthesis_result) = tokio::try_join!(
        syntactic::analyze(state, text),
        semantic::analyze(state, text),
        discourse::analyze(state, text),
        synthesis::analyze(state, text),
    )?;

    let result = AnalysisResult {
        id: Uuid::new_v4(),
        input_text: text.to_string(),
        syntactic: syntactic_result,
        semantic: semantic_result,
        discourse: discourse_result,
        critical_synthesis: synthesis_result,
        created_at: Utc::now(),
    };

    // Cache the result (best effort).
    let _ = cache::set_cached(state, text, &result).await;

    // Store in PostgreSQL for persistence.
    let _ = store_analysis(state, &result).await;

    Ok(result)
}

/// Persist analysis result to PostgreSQL.
async fn store_analysis(state: &AppState, result: &AnalysisResult) -> Result<()> {
    let analysis_json = serde_json::to_value(result)?;

    sqlx::query(
        "INSERT INTO analyses (id, input_text, result, created_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(result.id)
    .bind(&result.input_text)
    .bind(&analysis_json)
    .bind(result.created_at)
    .execute(&state.db.pg)
    .await?;

    Ok(())
}
