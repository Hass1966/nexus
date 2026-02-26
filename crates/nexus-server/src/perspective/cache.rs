use anyhow::{Context, Result};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::api::state::AppState;
use nexus_common::types::AnalysisResult;

/// Cache analysis results in Redis with a TTL of 1 hour.
const CACHE_TTL_SECS: u64 = 3600;

/// Generate a cache key for a given text input.
fn cache_key(text: &str) -> String {
    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();
    format!("analysis:{hash:x}")
}

/// Try to retrieve a cached analysis result.
pub async fn get_cached(state: &AppState, text: &str) -> Result<Option<AnalysisResult>> {
    let mut conn = state.db.redis.clone();
    let key = cache_key(text);

    let raw: Option<String> = redis::cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .unwrap_or(None);

    match raw {
        Some(json) => {
            let result: AnalysisResult =
                serde_json::from_str(&json).context("Failed to deserialize cached analysis")?;
            tracing::debug!("Cache hit for analysis");
            Ok(Some(result))
        }
        None => Ok(None),
    }
}

/// Store an analysis result in the cache.
pub async fn set_cached(state: &AppState, text: &str, result: &AnalysisResult) -> Result<()> {
    let mut conn = state.db.redis.clone();
    let key = cache_key(text);
    let json = serde_json::to_string(result)?;

    redis::cmd("SET")
        .arg(&key)
        .arg(&json)
        .arg("EX")
        .arg(CACHE_TTL_SECS)
        .query_async::<()>(&mut conn)
        .await
        .context("Failed to cache analysis result")?;

    tracing::debug!("Cached analysis result");
    Ok(())
}
