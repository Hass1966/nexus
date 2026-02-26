use qdrant_client::Qdrant;

pub async fn connect(url: &str) -> anyhow::Result<Qdrant> {
    let client = Qdrant::from_url(url).build()?;

    // Verify connectivity by listing collections.
    client.list_collections().await?;

    tracing::info!("Qdrant connected");
    Ok(client)
}
