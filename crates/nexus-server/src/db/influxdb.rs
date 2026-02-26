use influxdb2::Client;

#[derive(Debug, Clone)]
pub struct InfluxConfig {
    pub url: String,
    pub token: String,
    pub org: String,
    pub bucket: String,
}

pub async fn connect(config: &InfluxConfig) -> anyhow::Result<Client> {
    let client = Client::new(&config.url, &config.org, &config.token);

    // Verify connectivity.
    client
        .ready()
        .await
        .map_err(|e| anyhow::anyhow!("InfluxDB not ready: {e}"))?;

    tracing::info!("InfluxDB connected");
    Ok(client)
}
