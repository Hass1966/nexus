mod api;
mod config;
mod db;
mod models;
mod perspective;
mod river;
mod shared;

use tracing_subscriber::{EnvFilter, fmt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file.
    dotenvy::dotenv().ok();

    // Initialize tracing.
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_target(true)
        .with_thread_ids(true)
        .init();

    tracing::info!("Starting NEXUS platform");

    // Load configuration.
    let config = config::AppConfig::from_env()?;
    tracing::info!("Configuration loaded");

    // Connect to all databases.
    let db = db::DatabaseConnections::connect(&config).await?;
    tracing::info!("All database connections established");

    // Run PostgreSQL migrations.
    sqlx::migrate!("../../migrations").run(&db.pg).await?;
    tracing::info!("PostgreSQL migrations applied");

    // Ensure Qdrant collection exists.
    river::episodic::ensure_collection(&api::state::AppState::new(db.clone(), config.clone()))
        .await?;
    tracing::info!("Qdrant collections initialized");

    // Build application state.
    let state = api::state::AppState::new(db, config.clone());

    // Build the router.
    let app = api::build_router(state);

    // Start server.
    let bind_addr = config.bind_addr();
    tracing::info!("Listening on {bind_addr}");

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
