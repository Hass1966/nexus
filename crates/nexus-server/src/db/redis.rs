use ::redis::aio::ConnectionManager;

pub async fn connect(redis_url: &str) -> anyhow::Result<ConnectionManager> {
    let client = ::redis::Client::open(redis_url)?;
    let manager = ConnectionManager::new(client).await?;

    tracing::info!("Redis connected");
    Ok(manager)
}
