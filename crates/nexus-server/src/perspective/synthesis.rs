use anyhow::Result;
use serde::Deserialize;

use crate::api::state::AppState;
use nexus_common::types::{
    AlternativeFraming, BeneficiaryAnalysis, CriticalSynthesis, HiddenContext, NaturalisedClaim,
};

/// Layer 4: Critical synthesis via a single Ollama call.
/// This layer produces the highest-level critical insights.
pub async fn analyze(state: &AppState, text: &str) -> Result<CriticalSynthesis> {
    let system = r#"Perform a critical synthesis of the given text. Return a single JSON object with these four arrays:

1. "claims": Naturalised claims — claims presented as natural/obvious but actually contestable. Each entry:
   - "claim": the naturalised claim
   - "how_naturalised": how it's made to seem natural
   - "counter_evidence": evidence that challenges this claim

2. "beneficiaries": Who benefits and who is disadvantaged by the framing. Each entry:
   - "who_benefits": who gains from this framing
   - "how": how they benefit
   - "who_is_disadvantaged": who loses or is marginalized

3. "contexts": Hidden contexts — background information not mentioned but significant. Each entry:
   - "context": the hidden context
   - "relevance": why it's relevant
   - "why_hidden": why this context might be omitted

4. "framings": Alternative framings using the same facts. Each entry:
   - "original_frame": how it's currently framed
   - "alternative": the alternative framing
   - "same_facts_used": which facts from the original are used

Limit each array to at most 3 entries. Focus on the most significant findings."#;

    let result: CombinedSynthesisResponse = state
        .ollama
        .generate_json(text, Some(system))
        .await
        .unwrap_or_else(|_| CombinedSynthesisResponse::default());

    Ok(CriticalSynthesis {
        naturalised_claims: result
            .claims
            .into_iter()
            .map(|c| NaturalisedClaim {
                claim: c.claim,
                how_naturalised: c.how_naturalised,
                counter_evidence: c.counter_evidence,
            })
            .collect(),
        beneficiary_analysis: result
            .beneficiaries
            .into_iter()
            .map(|b| BeneficiaryAnalysis {
                who_benefits: b.who_benefits,
                how: b.how,
                who_is_disadvantaged: b.who_is_disadvantaged,
            })
            .collect(),
        hidden_contexts: result
            .contexts
            .into_iter()
            .map(|c| HiddenContext {
                context: c.context,
                relevance: c.relevance,
                why_hidden: c.why_hidden,
            })
            .collect(),
        alternative_framings: result
            .framings
            .into_iter()
            .map(|f| AlternativeFraming {
                original_frame: f.original_frame,
                alternative: f.alternative,
                same_facts_used: f.same_facts_used,
            })
            .collect(),
    })
}

#[derive(Default, Deserialize)]
struct CombinedSynthesisResponse {
    #[serde(default)]
    claims: Vec<NaturalisedEntry>,
    #[serde(default)]
    beneficiaries: Vec<BeneficiaryEntry>,
    #[serde(default)]
    contexts: Vec<ContextEntry>,
    #[serde(default)]
    framings: Vec<FramingEntry>,
}

#[derive(Deserialize)]
struct NaturalisedEntry {
    claim: String,
    how_naturalised: String,
    counter_evidence: String,
}

#[derive(Deserialize)]
struct BeneficiaryEntry {
    who_benefits: String,
    how: String,
    who_is_disadvantaged: String,
}

#[derive(Deserialize)]
struct ContextEntry {
    context: String,
    relevance: String,
    why_hidden: String,
}

#[derive(Deserialize)]
struct FramingEntry {
    original_frame: String,
    alternative: String,
    same_facts_used: String,
}
