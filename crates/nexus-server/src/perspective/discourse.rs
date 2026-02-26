use anyhow::Result;
use serde::Deserialize;

use crate::api::state::AppState;
use nexus_common::types::{
    CollocationPattern, DiscourseAnalysis, FramingInstance, IntertextualityMarker,
    StrategicOmission,
};

/// Layer 3: Discourse analysis via a single Ollama call.
pub async fn analyze(state: &AppState, text: &str) -> Result<DiscourseAnalysis> {
    let system = r#"Perform a comprehensive discourse analysis of the given text. Return a single JSON object with these four arrays:

1. "frames": How the text frames issues. Each entry:
   - "frame_name": name of the frame
   - "evidence": specific text evidence
   - "effect": how this frame shapes understanding

2. "omissions": What is strategically omitted. Each entry:
   - "what_is_missing": description of what's omitted
   - "why_it_matters": why this omission is significant
   - "who_benefits": who benefits from this omission

3. "collocations": Significant word pairings and their ideological implications. Each entry:
   - "pattern": the collocation
   - "frequency_note": how often/where this appears
   - "ideological_loading": what ideology this serves

4. "markers": Intertextual references (echoes of other texts/discourses). Each entry:
   - "reference": the intertextual element
   - "source_discourse": where it comes from
   - "function": what it does in this context

Limit each array to at most 3 entries. Focus on the most significant findings."#;

    let result: CombinedDiscourseResponse = state
        .ollama
        .generate_json(text, Some(system))
        .await
        .unwrap_or_else(|_| CombinedDiscourseResponse::default());

    Ok(DiscourseAnalysis {
        framing: result
            .frames
            .into_iter()
            .map(|f| FramingInstance {
                frame_name: f.frame_name,
                evidence: f.evidence,
                effect: f.effect,
            })
            .collect(),
        strategic_omissions: result
            .omissions
            .into_iter()
            .map(|o| StrategicOmission {
                what_is_missing: o.what_is_missing,
                why_it_matters: o.why_it_matters,
                who_benefits: o.who_benefits,
            })
            .collect(),
        collocations: result
            .collocations
            .into_iter()
            .map(|c| CollocationPattern {
                pattern: c.pattern,
                frequency_note: c.frequency_note,
                ideological_loading: c.ideological_loading,
            })
            .collect(),
        intertextuality: result
            .markers
            .into_iter()
            .map(|m| IntertextualityMarker {
                reference: m.reference,
                source_discourse: m.source_discourse,
                function: m.function,
            })
            .collect(),
    })
}

#[derive(Default, Deserialize)]
struct CombinedDiscourseResponse {
    #[serde(default)]
    frames: Vec<FramingEntry>,
    #[serde(default)]
    omissions: Vec<OmissionEntry>,
    #[serde(default)]
    collocations: Vec<CollocationEntry>,
    #[serde(default)]
    markers: Vec<IntertextEntry>,
}

#[derive(Deserialize)]
struct FramingEntry {
    frame_name: String,
    evidence: String,
    effect: String,
}

#[derive(Deserialize)]
struct OmissionEntry {
    what_is_missing: String,
    why_it_matters: String,
    who_benefits: String,
}

#[derive(Deserialize)]
struct CollocationEntry {
    pattern: String,
    frequency_note: String,
    ideological_loading: String,
}

#[derive(Deserialize)]
struct IntertextEntry {
    reference: String,
    source_discourse: String,
    function: String,
}
