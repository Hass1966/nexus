use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Embedding service using Ollama's embedding endpoint.
#[derive(Clone)]
pub struct EmbeddingService {
    http: Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

impl EmbeddingService {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            http: Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    /// Generate an embedding vector for the given text.
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let req = EmbedRequest {
            model: &self.model,
            input: text,
        };

        let resp = self
            .http
            .post(format!("{}/api/embed", self.base_url))
            .json(&req)
            .send()
            .await
            .context("Failed to reach Ollama embedding endpoint")?
            .error_for_status()
            .context("Ollama embedding returned error")?
            .json::<EmbedResponse>()
            .await
            .context("Failed to parse embedding response")?;

        resp.embeddings
            .into_iter()
            .next()
            .context("No embedding returned")
    }

    /// Get the embedding dimension (nomic-embed-text = 768).
    pub fn dimension(&self) -> u64 {
        768
    }
}
