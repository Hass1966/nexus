use nexus_common::types::ChatMode;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
    #[serde(default)]
    pub mode: ChatMode,
    pub session_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct AnalyzeRequest {
    pub text: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}
