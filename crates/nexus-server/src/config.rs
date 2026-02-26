use crate::db::{influxdb::InfluxConfig, neo4j::Neo4jConfig};

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub neo4j: Neo4jConfig,
    pub qdrant_url: String,
    pub influxdb: InfluxConfig,
    pub redis_url: String,
    pub ollama_url: String,
    pub ollama_model: String,
    pub ollama_embed_model: String,
    pub jwt_secret: String,
    pub jwt_expiry_hours: u64,
}

impl AppConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".into()),
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "3001".into())
                .parse()?,
            database_url: std::env::var("DATABASE_URL")?,
            neo4j: Neo4jConfig {
                uri: std::env::var("NEO4J_URI")?,
                user: std::env::var("NEO4J_USER")?,
                password: std::env::var("NEO4J_PASSWORD")?,
            },
            qdrant_url: std::env::var("QDRANT_URL")?,
            influxdb: InfluxConfig {
                url: std::env::var("INFLUXDB_URL")?,
                token: std::env::var("INFLUXDB_TOKEN")?,
                org: std::env::var("INFLUXDB_ORG")?,
                bucket: std::env::var("INFLUXDB_BUCKET")?,
            },
            redis_url: std::env::var("REDIS_URL")?,
            ollama_url: std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".into()),
            ollama_model: std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.1:8b".into()),
            ollama_embed_model: std::env::var("OLLAMA_EMBED_MODEL")
                .unwrap_or_else(|_| "nomic-embed-text".into()),
            jwt_secret: std::env::var("JWT_SECRET")?,
            jwt_expiry_hours: std::env::var("JWT_EXPIRY_HOURS")
                .unwrap_or_else(|_| "24".into())
                .parse()?,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
