use neo4rs::{ConfigBuilder, Graph};

#[derive(Debug, Clone)]
pub struct Neo4jConfig {
    pub uri: String,
    pub user: String,
    pub password: String,
}

pub async fn connect(config: &Neo4jConfig) -> anyhow::Result<Graph> {
    let graph_config = ConfigBuilder::default()
        .uri(&config.uri)
        .user(&config.user)
        .password(&config.password)
        .build()?;

    let graph = Graph::connect(graph_config).await?;

    tracing::info!("Neo4j connected");
    Ok(graph)
}
