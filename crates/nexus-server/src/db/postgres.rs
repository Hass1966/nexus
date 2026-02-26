use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(database_url)
        .await?;

    tracing::info!("PostgreSQL connected");
    Ok(pool)
}
