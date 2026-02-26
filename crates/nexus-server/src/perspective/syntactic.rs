use anyhow::Result;
use regex::Regex;
use serde::Deserialize;

use crate::api::state::AppState;
use nexus_common::types::{
    Nominalisation, SentenceComplexity, SyntacticAnalysis, TransitivityInstance, VoiceInstance,
    VoiceType,
};

/// Layer 1: Syntactic analysis.
/// Uses regex for simple pattern matching (voice, nominalisations)
/// and a single Ollama call for deeper analysis (transitivity + complexity combined).
pub async fn analyze(state: &AppState, text: &str) -> Result<SyntacticAnalysis> {
    // Run regex-based analysis locally.
    let voice_analysis = detect_voice(text);
    let nominalisations = detect_nominalisations(text);

    // Single combined Ollama call for complexity + transitivity.
    let (complexity, transitivity) = analyze_combined(state, text).await?;

    Ok(SyntacticAnalysis {
        voice_analysis,
        sentence_complexity: complexity,
        nominalisations,
        transitivity,
    })
}

/// Detect active/passive voice using regex patterns.
fn detect_voice(text: &str) -> Vec<VoiceInstance> {
    let passive_re =
        Regex::new(r"(?i)\b(was|were|is|are|been|being|be)\s+(\w+ed|made|done|given|taken|seen|known|found|told|shown|built|kept|left|held|brought|set|put|run|cut|let|lost|paid|met|hit|shut|hurt|read|thought|felt|bought|caught|taught|fought|sought|spent|sent|lent|bent|dealt|meant|dreamt|learnt|burnt|spoilt|spilt|smelt|built|understood|stood|sat|lay|led|fed|bid|rid|shed|split|spread|thrust|cast|cost|knit)\b")
            .expect("passive voice regex");

    let sentences = split_sentences(text);
    let mut results = Vec::new();

    for sentence in &sentences {
        let trimmed = sentence.trim();
        if trimmed.is_empty() {
            continue;
        }

        if passive_re.is_match(trimmed) {
            results.push(VoiceInstance {
                sentence: trimmed.to_string(),
                voice: VoiceType::Passive,
                significance: "Agent is obscured or de-emphasised".into(),
            });
        } else {
            results.push(VoiceInstance {
                sentence: trimmed.to_string(),
                voice: VoiceType::Active,
                significance: "Clear agent-action relationship".into(),
            });
        }
    }

    results
}

/// Detect nominalisations: nouns derived from verbs (e.g., "destruction" from "destroy").
fn detect_nominalisations(text: &str) -> Vec<Nominalisation> {
    let patterns = [
        (r"\b(\w+tion)\b", "tion"),
        (r"\b(\w+sion)\b", "sion"),
        (r"\b(\w+ment)\b", "ment"),
        (r"\b(\w+ance)\b", "ance"),
        (r"\b(\w+ence)\b", "ence"),
        (r"\b(\w+ity)\b", "ity"),
        (r"\b(\w+ness)\b", "ness"),
        (r"\b(\w+ism)\b", "ism"),
    ];

    // Common words that are NOT nominalisations.
    let exceptions = [
        "information",
        "situation",
        "question",
        "position",
        "condition",
        "mention",
        "nation",
        "station",
        "section",
        "attention",
        "addition",
        "fashion",
        "opinion",
        "religion",
        "version",
        "season",
        "reason",
        "person",
        "lesson",
        "television",
        "environment",
        "government",
        "department",
        "management",
        "moment",
        "element",
        "comment",
        "document",
        "statement",
        "treatment",
        "movement",
        "agreement",
        "development",
        "argument",
        "equipment",
        "experiment",
        "apartment",
        "importance",
        "performance",
        "appearance",
        "distance",
        "instance",
        "substance",
        "sentence",
        "evidence",
        "experience",
        "difference",
        "reference",
        "presence",
        "violence",
        "silence",
        "absence",
        "patience",
        "community",
        "opportunity",
        "security",
        "quality",
        "reality",
        "ability",
        "activity",
        "authority",
        "university",
        "majority",
        "identity",
        "property",
        "society",
        "variety",
        "business",
        "darkness",
        "fitness",
        "illness",
        "kindness",
        "madness",
        "sadness",
        "weakness",
        "awareness",
        "happiness",
        "loneliness",
        "mechanism",
        "organism",
        "capitalism",
        "socialism",
    ];

    let word_lower = text.to_lowercase();
    let mut results = Vec::new();

    for (pattern, suffix) in &patterns {
        let re = Regex::new(pattern).expect("nominalisation regex");
        for cap in re.captures_iter(&word_lower) {
            let word = cap[1].to_string();
            if exceptions.contains(&word.as_str()) {
                continue;
            }

            // Attempt to reconstruct the verb form.
            let verb_form = match *suffix {
                "tion" => word.trim_end_matches("tion").to_string() + "te",
                "sion" => word.trim_end_matches("sion").to_string() + "de",
                "ment" => word.trim_end_matches("ment").to_string(),
                "ance" | "ence" => word
                    .trim_end_matches("ance")
                    .trim_end_matches("ence")
                    .to_string(),
                "ity" => word.trim_end_matches("ity").to_string(),
                "ness" => word.trim_end_matches("ness").to_string(),
                "ism" => word.trim_end_matches("ism").to_string(),
                _ => word.clone(),
            };

            results.push(Nominalisation {
                original: word.clone(),
                verb_form,
                effect: "Converts a process into a thing, hiding who does the action".to_string(),
            });
        }
    }

    results
}

/// Combined Ollama call for complexity + transitivity analysis.
async fn analyze_combined(
    state: &AppState,
    text: &str,
) -> Result<(Vec<SentenceComplexity>, Vec<TransitivityInstance>)> {
    let system = r#"Perform two analyses on the given text and return a single JSON object with two arrays:

1. "sentences": Analyze sentence complexity. Each entry has:
   - "sentence": the sentence text
   - "score": complexity score 0.0-1.0
   - "clause_count": number of clauses
   - "note": brief note on complexity
   Limit to 5 most notable sentences.

2. "processes": Transitivity analysis (who does what to whom). Each entry has:
   - "sentence": the relevant sentence
   - "actor": who performs the action
   - "process": the action/verb
   - "affected": who/what is affected
   - "analysis": brief note on power/agency
   Limit to 5 most significant processes."#;

    let result: CombinedSyntacticResponse = state
        .ollama
        .generate_json(text, Some(system))
        .await
        .unwrap_or_else(|_| CombinedSyntacticResponse {
            sentences: Vec::new(),
            processes: Vec::new(),
        });

    let complexity = result
        .sentences
        .into_iter()
        .map(|s| SentenceComplexity {
            sentence: s.sentence,
            score: s.score,
            clause_count: s.clause_count,
            note: s.note,
        })
        .collect();

    let transitivity = result
        .processes
        .into_iter()
        .map(|t| TransitivityInstance {
            sentence: t.sentence,
            actor: t.actor,
            process: t.process,
            affected: t.affected,
            analysis: t.analysis,
        })
        .collect();

    Ok((complexity, transitivity))
}

fn split_sentences(text: &str) -> Vec<String> {
    let re = Regex::new(r"[.!?]+\s+|[.!?]+$").expect("sentence split regex");
    re.split(text)
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[derive(Deserialize)]
struct CombinedSyntacticResponse {
    #[serde(default)]
    sentences: Vec<ComplexityEntry>,
    #[serde(default)]
    processes: Vec<TransitivityEntry>,
}

#[derive(Deserialize)]
struct ComplexityEntry {
    sentence: String,
    score: f64,
    clause_count: u32,
    note: String,
}

#[derive(Deserialize)]
struct TransitivityEntry {
    sentence: String,
    actor: String,
    process: String,
    affected: String,
    analysis: String,
}
