use thiserror::Error;

/// Domain errors shared across the platform.
#[derive(Debug, Error)]
pub enum NexusError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Neo4j error: {0}")]
    Neo4j(String),

    #[error("Vector store error: {0}")]
    VectorStore(String),

    #[error("Time series error: {0}")]
    TimeSeries(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Embedding error: {0}")]
    Embedding(String),

    #[error("Analysis error: {0}")]
    Analysis(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Internal error: {0}")]
    Internal(String),
}
