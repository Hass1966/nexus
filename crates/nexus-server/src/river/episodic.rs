use anyhow::{Context, Result};
use qdrant_client::qdrant::{
    CreateCollectionBuilder, Distance, PointStruct, SearchPointsBuilder, UpsertPointsBuilder,
    VectorParamsBuilder,
};
use serde_json::json;
use uuid::Uuid;

use crate::api::state::AppState;

const COLLECTION_NAME: &str = "episodic_memory";

/// Ensure the episodic memory collection exists in Qdrant.
pub async fn ensure_collection(state: &AppState) -> Result<()> {
    let collections = state.db.qdrant.list_collections().await?;

    let exists = collections
        .collections
        .iter()
        .any(|c| c.name == COLLECTION_NAME);

    if !exists {
        let dim = state.embeddings.dimension();
        state
            .db
            .qdrant
            .create_collection(
                CreateCollectionBuilder::new(COLLECTION_NAME)
                    .vectors_config(VectorParamsBuilder::new(dim, Distance::Cosine)),
            )
            .await
            .context("Failed to create episodic memory collection")?;

        tracing::info!("Created Qdrant collection: {COLLECTION_NAME}");
    }

    Ok(())
}

/// Store a message as an episodic memory with its embedding.
pub async fn store_memory(
    state: &AppState,
    user_id: Uuid,
    session_id: Uuid,
    message_id: Uuid,
    content: &str,
    role: &str,
) -> Result<()> {
    let embedding = state
        .embeddings
        .embed(content)
        .await
        .context("Failed to generate embedding for memory")?;

    let payload: serde_json::Map<String, serde_json::Value> = serde_json::from_value(json!({
        "user_id": user_id.to_string(),
        "session_id": session_id.to_string(),
        "message_id": message_id.to_string(),
        "content": content,
        "role": role,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))?;

    let point = PointStruct::new(message_id.to_string(), embedding, payload);

    state
        .db
        .qdrant
        .upsert_points(UpsertPointsBuilder::new(COLLECTION_NAME, vec![point]))
        .await
        .context("Failed to store episodic memory")?;

    Ok(())
}

/// Search for relevant past memories using semantic similarity.
pub async fn recall_similar(
    state: &AppState,
    user_id: Uuid,
    query_text: &str,
    limit: u64,
) -> Result<Vec<MemoryResult>> {
    let query_embedding = state
        .embeddings
        .embed(query_text)
        .await
        .context("Failed to generate query embedding")?;

    let filter = qdrant_client::qdrant::Filter::must([qdrant_client::qdrant::Condition::matches(
        "user_id",
        user_id.to_string(),
    )]);

    let results = state
        .db
        .qdrant
        .search_points(
            SearchPointsBuilder::new(COLLECTION_NAME, query_embedding, limit)
                .filter(filter)
                .with_payload(true),
        )
        .await
        .context("Failed to search episodic memory")?;

    let memories = results
        .result
        .into_iter()
        .filter_map(|point| {
            let payload = &point.payload;
            let content = payload.get("content")?.as_str()?.to_string();
            let role = payload.get("role")?.as_str()?.to_string();
            let timestamp = payload.get("timestamp")?.as_str()?.to_string();

            Some(MemoryResult {
                content,
                role,
                timestamp,
                score: point.score,
            })
        })
        .collect();

    Ok(memories)
}

#[derive(Debug, Clone)]
pub struct MemoryResult {
    pub content: String,
    pub role: String,
    pub timestamp: String,
    pub score: f32,
}
