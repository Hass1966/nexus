use std::sync::Arc;

use crate::config::AppConfig;
use crate::db::DatabaseConnections;
use crate::shared::embeddings::EmbeddingService;
use crate::shared::ollama::OllamaClient;

/// Shared application state injected into all handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnections,
    pub ollama: OllamaClient,
    pub embeddings: EmbeddingService,
    pub config: Arc<AppConfig>,
}

impl AppState {
    pub fn new(db: DatabaseConnections, config: AppConfig) -> Self {
        let ollama = OllamaClient::new(&config.ollama_url, &config.ollama_model);
        let embeddings = EmbeddingService::new(&config.ollama_url, &config.ollama_embed_model);

        Self {
            db,
            ollama,
            embeddings,
            config: Arc::new(config),
        }
    }
}
