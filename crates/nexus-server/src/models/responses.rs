use nexus_common::types::{AnalysisResult, Belief, ConsciousnessState, Contradiction};
use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ChatResponse {
    pub session_id: Uuid,
    pub message: String,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<AnalysisResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contradictions: Option<Vec<Contradiction>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub beliefs_updated: Option<Vec<Belief>>,
}

#[derive(Debug, Serialize)]
pub struct AnalyzeResponse {
    pub analysis: AnalysisResult,
}

#[derive(Debug, Serialize)]
pub struct BeliefsResponse {
    pub user_id: Uuid,
    pub beliefs: Vec<Belief>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ConsciousnessResponse {
    pub state: ConsciousnessState,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub services: HealthServices,
}

#[derive(Debug, Serialize)]
pub struct HealthServices {
    pub postgres: ServiceStatus,
    pub neo4j: ServiceStatus,
    pub qdrant: ServiceStatus,
    pub influxdb: ServiceStatus,
    pub redis: ServiceStatus,
    pub ollama: ServiceStatus,
}

#[derive(Debug, Serialize)]
pub struct ServiceStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ServiceStatus {
    pub fn up() -> Self {
        Self {
            status: "up".into(),
            error: None,
        }
    }

    pub fn down(error: String) -> Self {
        Self {
            status: "down".into(),
            error: Some(error),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}
