use anyhow::{Context, Result};
use chrono::Utc;
use uuid::Uuid;

use crate::api::state::AppState;
use nexus_common::types::ConsciousnessState;

/// Log a consciousness metrics snapshot to InfluxDB.
pub async fn log_metrics(state: &AppState, metrics: &ConsciousnessState) -> Result<()> {
    use influxdb2::models::DataPoint;

    let point = DataPoint::builder("consciousness")
        .tag("user_id", metrics.user_id.to_string())
        .tag("session_id", metrics.session_id.to_string())
        .field("epistemic_humility", metrics.epistemic_humility)
        .field("belief_volatility", metrics.belief_volatility)
        .field("contradiction_awareness", metrics.contradiction_awareness)
        .field("depth_of_inquiry", metrics.depth_of_inquiry)
        .build()
        .context("Failed to build InfluxDB data point")?;

    state
        .db
        .influx
        .write(
            &state.config.influxdb.bucket,
            futures::stream::iter(vec![point]),
        )
        .await
        .context("Failed to write consciousness metrics to InfluxDB")?;

    tracing::debug!(
        user_id = %metrics.user_id,
        "Logged consciousness metrics"
    );

    Ok(())
}

/// Get the current consciousness state by computing metrics from recent activity.
pub async fn get_current_state(state: &AppState, user_id: Uuid) -> Result<ConsciousnessState> {
    // Query the most recent metrics from InfluxDB using Flux.
    let flux_query = format!(
        r#"from(bucket: "{}")
            |> range(start: -24h)
            |> filter(fn: (r) => r._measurement == "consciousness")
            |> filter(fn: (r) => r.user_id == "{}")
            |> last()"#,
        state.config.influxdb.bucket, user_id,
    );

    let query = influxdb2::models::Query::new(flux_query);

    let raw_results = state
        .db
        .influx
        .query_raw(Some(query))
        .await
        .unwrap_or_default();

    // Parse results if available.
    if !raw_results.is_empty() {
        let mut epistemic_humility = 0.5;
        let mut belief_volatility = 0.0;
        let mut contradiction_awareness = 0.0;
        let mut depth_of_inquiry = 0.0;

        for record in &raw_results {
            let field = record
                .values
                .get("_field")
                .and_then(|v| v.string())
                .unwrap_or_default();

            let value = record
                .values
                .get("_value")
                .and_then(|v| v.f64())
                .unwrap_or(0.0);

            match field.as_str() {
                "epistemic_humility" => epistemic_humility = value,
                "belief_volatility" => belief_volatility = value,
                "contradiction_awareness" => contradiction_awareness = value,
                "depth_of_inquiry" => depth_of_inquiry = value,
                _ => {}
            }
        }

        return Ok(ConsciousnessState {
            user_id,
            session_id: Uuid::nil(),
            epistemic_humility,
            belief_volatility,
            contradiction_awareness,
            depth_of_inquiry,
            timestamp: Utc::now(),
        });
    }

    // Return defaults if no data.
    Ok(ConsciousnessState {
        user_id,
        session_id: Uuid::nil(),
        epistemic_humility: 0.5,
        belief_volatility: 0.0,
        contradiction_awareness: 0.0,
        depth_of_inquiry: 0.0,
        timestamp: Utc::now(),
    })
}

/// Compute consciousness metrics from the user's interaction data.
pub async fn compute_metrics(
    state: &AppState,
    user_id: Uuid,
    session_id: Uuid,
    beliefs_count: usize,
    contradictions_count: usize,
    questions_asked: usize,
    beliefs_revised: usize,
) -> Result<ConsciousnessState> {
    let epistemic_humility = if beliefs_count > 0 {
        ((questions_asked + beliefs_revised) as f64 / beliefs_count as f64).min(1.0)
    } else {
        0.5
    };

    let belief_volatility = if beliefs_count > 0 {
        (beliefs_revised as f64 / beliefs_count as f64).min(1.0)
    } else {
        0.0
    };

    let contradiction_awareness = if beliefs_count > 1 {
        (contradictions_count as f64 / (beliefs_count as f64 - 1.0)).min(1.0)
    } else {
        0.0
    };

    let depth_of_inquiry = (questions_asked as f64 / 10.0).min(1.0);

    let metrics = ConsciousnessState {
        user_id,
        session_id,
        epistemic_humility,
        belief_volatility,
        contradiction_awareness,
        depth_of_inquiry,
        timestamp: Utc::now(),
    };

    log_metrics(state, &metrics).await?;

    Ok(metrics)
}
