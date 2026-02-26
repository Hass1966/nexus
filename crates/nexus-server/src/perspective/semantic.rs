use anyhow::Result;
use serde::Deserialize;

use crate::api::state::AppState;
use nexus_common::types::{
    Implicature, LexicalField, PowerHierarchy, Presupposition, SemanticAnalysis,
};

/// Layer 2: Semantic analysis via a single Ollama call.
pub async fn analyze(state: &AppState, text: &str) -> Result<SemanticAnalysis> {
    let system = r#"Perform a comprehensive semantic analysis of the given text. Return a single JSON object with these four arrays:

1. "presuppositions": Linguistic presuppositions (things taken for granted). Each entry:
   - "trigger": the linguistic trigger
   - "presupposed_content": what is presupposed
   - "significance": why this matters

2. "implicatures": Conversational implicatures (meanings implied but not stated). Each entry:
   - "statement": the statement
   - "implied_meaning": what is implied
   - "mechanism": how the implicature works

3. "hierarchies": Power hierarchies encoded in the text. Each entry:
   - "dominant": who/what holds power
   - "subordinate": who/what is subordinated
   - "linguistic_markers": specific words/phrases that encode this (array)
   - "analysis": brief analysis

4. "fields": Lexical fields (semantic clusters of related words). Each entry:
   - "field_name": the domain
   - "terms": array of related words
   - "connotation": what this lexical field implies

Limit each array to at most 3 entries. Focus on the most significant findings."#;

    let result: CombinedSemanticResponse = state
        .ollama
        .generate_json(text, Some(system))
        .await
        .unwrap_or_else(|_| CombinedSemanticResponse::default());

    Ok(SemanticAnalysis {
        presuppositions: result
            .presuppositions
            .into_iter()
            .map(|p| Presupposition {
                trigger: p.trigger,
                presupposed_content: p.presupposed_content,
                significance: p.significance,
            })
            .collect(),
        implicatures: result
            .implicatures
            .into_iter()
            .map(|i| Implicature {
                statement: i.statement,
                implied_meaning: i.implied_meaning,
                mechanism: i.mechanism,
            })
            .collect(),
        power_hierarchies: result
            .hierarchies
            .into_iter()
            .map(|p| PowerHierarchy {
                dominant: p.dominant,
                subordinate: p.subordinate,
                linguistic_markers: p.linguistic_markers,
                analysis: p.analysis,
            })
            .collect(),
        lexical_fields: result
            .fields
            .into_iter()
            .map(|f| LexicalField {
                field_name: f.field_name,
                terms: f.terms,
                connotation: f.connotation,
            })
            .collect(),
    })
}

#[derive(Default, Deserialize)]
struct CombinedSemanticResponse {
    #[serde(default)]
    presuppositions: Vec<PresupEntry>,
    #[serde(default)]
    implicatures: Vec<ImplicatureEntry>,
    #[serde(default)]
    hierarchies: Vec<PowerEntry>,
    #[serde(default)]
    fields: Vec<LexicalEntry>,
}

#[derive(Deserialize)]
struct PresupEntry {
    trigger: String,
    presupposed_content: String,
    significance: String,
}

#[derive(Deserialize)]
struct ImplicatureEntry {
    statement: String,
    implied_meaning: String,
    mechanism: String,
}

#[derive(Deserialize)]
struct PowerEntry {
    dominant: String,
    subordinate: String,
    linguistic_markers: Vec<String>,
    analysis: String,
}

#[derive(Deserialize)]
struct LexicalEntry {
    field_name: String,
    terms: Vec<String>,
    connotation: String,
}
