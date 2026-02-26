use anyhow::{Context, Result};
use chrono::Utc;
use neo4rs::query;
use serde::Deserialize;
use uuid::Uuid;

use crate::api::state::AppState;
use nexus_common::types::{Belief, Contradiction};

/// Extract claims/beliefs from a user message using Ollama.
pub async fn extract_beliefs(state: &AppState, message: &str) -> Result<Vec<ExtractedClaim>> {
    let system = r#"You are a belief extraction engine. Given a user's message, extract discrete claims or beliefs the user holds. Return a JSON object with a "claims" array. Each claim has:
- "claim": the belief statement
- "confidence": how confidently the user holds it (0.0-1.0)
- "is_explicit": whether they directly stated it (true) or it's implied (false)

Only extract genuine belief claims, not questions or meta-commentary. If there are no claims, return {"claims": []}."#;

    let prompt = format!("Extract beliefs from this message:\n\n\"{message}\"");

    let result: ClaimsResponse = state
        .ollama
        .generate_json(&prompt, Some(system))
        .await
        .context("Failed to extract beliefs")?;

    Ok(result.claims)
}

#[derive(Debug, Deserialize)]
struct ClaimsResponse {
    claims: Vec<ExtractedClaim>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExtractedClaim {
    pub claim: String,
    pub confidence: f64,
    #[serde(default)]
    pub is_explicit: bool,
}

/// Store a belief in Neo4j and return the Belief struct.
pub async fn store_belief(
    state: &AppState,
    user_id: Uuid,
    claim: &ExtractedClaim,
    source_message_id: Uuid,
) -> Result<Belief> {
    let belief_id = Uuid::new_v4();
    let now = Utc::now();

    let q = query(
        "MERGE (u:User {id: $user_id})
         CREATE (b:Belief {
             id: $belief_id,
             claim: $claim,
             confidence: $confidence,
             source_message_id: $source_msg_id,
             created_at: $created_at,
             updated_at: $updated_at
         })
         CREATE (u)-[:HOLDS]->(b)
         RETURN b.id AS id",
    )
    .param("user_id", user_id.to_string())
    .param("belief_id", belief_id.to_string())
    .param("claim", claim.claim.clone())
    .param("confidence", claim.confidence)
    .param("source_msg_id", source_message_id.to_string())
    .param("created_at", now.to_rfc3339())
    .param("updated_at", now.to_rfc3339());

    state
        .db
        .neo4j
        .run(q)
        .await
        .context("Failed to store belief in Neo4j")?;

    Ok(Belief {
        id: belief_id,
        user_id,
        claim: claim.claim.clone(),
        confidence: claim.confidence,
        source_message_id,
        created_at: now,
        updated_at: now,
    })
}

/// Retrieve all beliefs for a user from Neo4j.
pub async fn get_user_beliefs(state: &AppState, user_id: Uuid) -> Result<Vec<Belief>> {
    let q = query(
        "MATCH (u:User {id: $user_id})-[:HOLDS]->(b:Belief)
         RETURN b.id AS id, b.claim AS claim, b.confidence AS confidence,
                b.source_message_id AS source_message_id,
                b.created_at AS created_at, b.updated_at AS updated_at
         ORDER BY b.created_at DESC",
    )
    .param("user_id", user_id.to_string());

    let mut result = state
        .db
        .neo4j
        .execute(q)
        .await
        .context("Failed to query beliefs from Neo4j")?;

    let mut beliefs = Vec::new();
    while let Some(row) = result.next().await? {
        let id_str: String = row.get("id").unwrap_or_default();
        let claim: String = row.get("claim").unwrap_or_default();
        let confidence: f64 = row.get("confidence").unwrap_or(0.5);
        let source_str: String = row.get("source_message_id").unwrap_or_default();
        let created_str: String = row.get("created_at").unwrap_or_default();
        let updated_str: String = row.get("updated_at").unwrap_or_default();

        beliefs.push(Belief {
            id: id_str.parse().unwrap_or(Uuid::nil()),
            user_id,
            claim,
            confidence,
            source_message_id: source_str.parse().unwrap_or(Uuid::nil()),
            created_at: chrono::DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: chrono::DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        });
    }

    Ok(beliefs)
}

/// Detect contradictions between a new claim and existing beliefs.
pub async fn detect_contradictions(
    state: &AppState,
    user_id: Uuid,
    new_claim: &str,
) -> Result<Vec<Contradiction>> {
    let existing = get_user_beliefs(state, user_id).await?;
    if existing.is_empty() {
        return Ok(Vec::new());
    }

    let existing_claims: Vec<String> = existing.iter().map(|b| b.claim.clone()).collect();
    let existing_json = serde_json::to_string(&existing_claims)?;

    let system = r#"You are a contradiction detection engine. Given a new claim and a list of existing beliefs, identify any contradictions. Return a JSON object with a "contradictions" array. Each entry has:
- "existing_claim": the contradicted existing belief (exact text)
- "explanation": why these contradict
- "severity": how severe the contradiction is (0.0-1.0)

If no contradictions exist, return {"contradictions": []}."#;

    let prompt = format!("New claim: \"{new_claim}\"\n\nExisting beliefs:\n{existing_json}");

    let result: ContradictionResponse = state
        .ollama
        .generate_json(&prompt, Some(system))
        .await
        .unwrap_or_else(|_| ContradictionResponse {
            contradictions: Vec::new(),
        });

    let mut found = Vec::new();
    for c in result.contradictions {
        if let Some(existing_belief) = existing.iter().find(|b| b.claim == c.existing_claim) {
            found.push(Contradiction {
                belief_a: existing_belief.clone(),
                belief_b: Belief {
                    id: Uuid::nil(),
                    user_id,
                    claim: new_claim.to_string(),
                    confidence: 0.5,
                    source_message_id: Uuid::nil(),
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                },
                explanation: c.explanation,
                severity: c.severity,
            });
        }
    }

    Ok(found)
}

/// Create CONTRADICTS relationship in Neo4j between two beliefs.
pub async fn link_contradiction(
    state: &AppState,
    belief_a_id: Uuid,
    belief_b_id: Uuid,
    explanation: &str,
    severity: f64,
) -> Result<()> {
    let q = query(
        "MATCH (a:Belief {id: $a_id}), (b:Belief {id: $b_id})
         CREATE (a)-[:CONTRADICTS {explanation: $explanation, severity: $severity, detected_at: $now}]->(b)",
    )
    .param("a_id", belief_a_id.to_string())
    .param("b_id", belief_b_id.to_string())
    .param("explanation", explanation.to_string())
    .param("severity", severity)
    .param("now", Utc::now().to_rfc3339());

    state
        .db
        .neo4j
        .run(q)
        .await
        .context("Failed to create contradiction link")?;

    Ok(())
}

#[derive(Deserialize)]
struct ContradictionResponse {
    contradictions: Vec<ContradictionEntry>,
}

#[derive(Deserialize)]
struct ContradictionEntry {
    existing_claim: String,
    explanation: String,
    severity: f64,
}
