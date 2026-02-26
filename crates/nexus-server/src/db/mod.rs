pub mod influxdb;
pub mod neo4j;
pub mod postgres;
pub mod qdrant;
pub mod redis;

use std::sync::Arc;

/// All database connections bundled together.
#[derive(Clone)]
pub struct DatabaseConnections {
    pub pg: sqlx::PgPool,
    pub neo4j: Arc<neo4rs::Graph>,
    pub qdrant: Arc<qdrant_client::Qdrant>,
    pub influx: Arc<influxdb2::Client>,
    pub redis: ::redis::aio::ConnectionManager,
}

impl DatabaseConnections {
    pub async fn connect(config: &crate::config::AppConfig) -> anyhow::Result<Self> {
        let (pg, neo4j, qdrant, influx, redis) = tokio::try_join!(
            self::postgres::connect(&config.database_url),
            self::neo4j::connect(&config.neo4j),
            self::qdrant::connect(&config.qdrant_url),
            self::influxdb::connect(&config.influxdb),
            self::redis::connect(&config.redis_url),
        )?;

        Ok(Self {
            pg,
            neo4j: Arc::new(neo4j),
            qdrant: Arc::new(qdrant),
            influx: Arc::new(influx),
            redis,
        })
    }
}
