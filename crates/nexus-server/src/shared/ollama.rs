use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Client for the Ollama HTTP API.
#[derive(Clone)]
pub struct OllamaClient {
    http: Client,
    base_url: String,
    model: String,
}

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    system: Option<&'a str>,
    stream: bool,
    format: Option<&'a str>,
    options: Option<GenerateOptions>,
}

#[derive(Serialize)]
struct GenerateOptions {
    temperature: f32,
    num_predict: i32,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    stream: bool,
    format: Option<&'a str>,
    options: Option<GenerateOptions>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: ChatMessage,
}

impl OllamaClient {
    pub fn new(base_url: &str, model: &str) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .expect("Failed to build HTTP client");
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            model: model.to_string(),
        }
    }

    /// Generate a completion with an optional system prompt. Returns raw text.
    pub async fn generate(&self, prompt: &str, system: Option<&str>) -> Result<String> {
        let req = GenerateRequest {
            model: &self.model,
            prompt,
            system,
            stream: false,
            format: None,
            options: Some(GenerateOptions {
                temperature: 0.7,
                num_predict: 2048,
            }),
        };

        let resp = self
            .http
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await
            .context("Failed to reach Ollama")?
            .error_for_status()
            .context("Ollama returned error")?
            .json::<GenerateResponse>()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(resp.response)
    }

    /// Generate a completion and parse the response as JSON.
    pub async fn generate_json<T: serde::de::DeserializeOwned>(
        &self,
        prompt: &str,
        system: Option<&str>,
    ) -> Result<T> {
        let req = GenerateRequest {
            model: &self.model,
            prompt,
            system,
            stream: false,
            format: Some("json"),
            options: Some(GenerateOptions {
                temperature: 0.3,
                num_predict: 4096,
            }),
        };

        let resp = self
            .http
            .post(format!("{}/api/generate", self.base_url))
            .json(&req)
            .send()
            .await
            .context("Failed to reach Ollama")?
            .error_for_status()
            .context("Ollama returned error")?
            .json::<GenerateResponse>()
            .await
            .context("Failed to parse Ollama response")?;

        let parsed: T = serde_json::from_str(&resp.response)
            .context("Failed to parse JSON from LLM response")?;

        Ok(parsed)
    }

    /// Multi-turn chat completion.
    pub async fn chat(&self, messages: &[ChatMessage]) -> Result<String> {
        let req = ChatRequest {
            model: &self.model,
            messages,
            stream: false,
            format: None,
            options: Some(GenerateOptions {
                temperature: 0.7,
                num_predict: 2048,
            }),
        };

        let resp = self
            .http
            .post(format!("{}/api/chat", self.base_url))
            .json(&req)
            .send()
            .await
            .context("Failed to reach Ollama")?
            .error_for_status()
            .context("Ollama returned error")?
            .json::<ChatResponse>()
            .await
            .context("Failed to parse Ollama chat response")?;

        Ok(resp.message.content)
    }

    /// Multi-turn chat with JSON output parsing.
    pub async fn chat_json<T: serde::de::DeserializeOwned>(
        &self,
        messages: &[ChatMessage],
    ) -> Result<T> {
        let req = ChatRequest {
            model: &self.model,
            messages,
            stream: false,
            format: Some("json"),
            options: Some(GenerateOptions {
                temperature: 0.3,
                num_predict: 4096,
            }),
        };

        let resp = self
            .http
            .post(format!("{}/api/chat", self.base_url))
            .json(&req)
            .send()
            .await
            .context("Failed to reach Ollama")?
            .error_for_status()
            .context("Ollama returned error")?
            .json::<ChatResponse>()
            .await
            .context("Failed to parse Ollama chat response")?;

        let parsed: T = serde_json::from_str(&resp.message.content)
            .context("Failed to parse JSON from LLM chat response")?;

        Ok(parsed)
    }

    /// Health check: verify Ollama is reachable and the model is available.
    pub async fn health(&self) -> Result<bool> {
        let resp = self
            .http
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await?;

        Ok(resp.status().is_success())
    }
}
