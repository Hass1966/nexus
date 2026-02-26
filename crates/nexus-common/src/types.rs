use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Chat mode determines which engine processes the message.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChatMode {
    /// Pure River: epistemic dialogue with belief tracking.
    Conversation,
    /// Pure Perspective: 4-layer critical discourse analysis.
    Analysis,
    /// River + Perspective combined: claims are extracted and analyzed inline.
    #[default]
    Integrated,
}

/// A single message in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: Uuid,
    pub session_id: Uuid,
    pub user_id: Uuid,
    pub role: MessageRole,
    pub content: String,
    pub mode: ChatMode,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

/// A belief node in the user's epistemic graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub id: Uuid,
    pub user_id: Uuid,
    pub claim: String,
    pub confidence: f64,
    pub source_message_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A contradiction detected between two beliefs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contradiction {
    pub belief_a: Belief,
    pub belief_b: Belief,
    pub explanation: String,
    pub severity: f64,
}

/// Consciousness metrics snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsciousnessState {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub epistemic_humility: f64,
    pub belief_volatility: f64,
    pub contradiction_awareness: f64,
    pub depth_of_inquiry: f64,
    pub timestamp: DateTime<Utc>,
}

// ── Perspective Analysis Types ──

/// Complete 4-layer analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    pub id: Uuid,
    pub input_text: String,
    pub syntactic: SyntacticAnalysis,
    pub semantic: SemanticAnalysis,
    pub discourse: DiscourseAnalysis,
    pub critical_synthesis: CriticalSynthesis,
    pub created_at: DateTime<Utc>,
}

/// Layer 1: Syntactic analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyntacticAnalysis {
    pub voice_analysis: Vec<VoiceInstance>,
    pub sentence_complexity: Vec<SentenceComplexity>,
    pub nominalisations: Vec<Nominalisation>,
    pub transitivity: Vec<TransitivityInstance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceInstance {
    pub sentence: String,
    pub voice: VoiceType,
    pub significance: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoiceType {
    Active,
    Passive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentenceComplexity {
    pub sentence: String,
    pub score: f64,
    pub clause_count: u32,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nominalisation {
    pub original: String,
    pub verb_form: String,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitivityInstance {
    pub sentence: String,
    pub actor: String,
    pub process: String,
    pub affected: String,
    pub analysis: String,
}

/// Layer 2: Semantic analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticAnalysis {
    pub presuppositions: Vec<Presupposition>,
    pub implicatures: Vec<Implicature>,
    pub power_hierarchies: Vec<PowerHierarchy>,
    pub lexical_fields: Vec<LexicalField>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presupposition {
    pub trigger: String,
    pub presupposed_content: String,
    pub significance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Implicature {
    pub statement: String,
    pub implied_meaning: String,
    pub mechanism: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerHierarchy {
    pub dominant: String,
    pub subordinate: String,
    pub linguistic_markers: Vec<String>,
    pub analysis: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexicalField {
    pub field_name: String,
    pub terms: Vec<String>,
    pub connotation: String,
}

/// Layer 3: Discourse analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscourseAnalysis {
    pub framing: Vec<FramingInstance>,
    pub strategic_omissions: Vec<StrategicOmission>,
    pub collocations: Vec<CollocationPattern>,
    pub intertextuality: Vec<IntertextualityMarker>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FramingInstance {
    pub frame_name: String,
    pub evidence: String,
    pub effect: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategicOmission {
    pub what_is_missing: String,
    pub why_it_matters: String,
    pub who_benefits: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollocationPattern {
    pub pattern: String,
    pub frequency_note: String,
    pub ideological_loading: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntertextualityMarker {
    pub reference: String,
    pub source_discourse: String,
    pub function: String,
}

/// Layer 4: Critical synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalSynthesis {
    pub naturalised_claims: Vec<NaturalisedClaim>,
    pub beneficiary_analysis: Vec<BeneficiaryAnalysis>,
    pub hidden_contexts: Vec<HiddenContext>,
    pub alternative_framings: Vec<AlternativeFraming>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaturalisedClaim {
    pub claim: String,
    pub how_naturalised: String,
    pub counter_evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeneficiaryAnalysis {
    pub who_benefits: String,
    pub how: String,
    pub who_is_disadvantaged: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenContext {
    pub context: String,
    pub relevance: String,
    pub why_hidden: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeFraming {
    pub original_frame: String,
    pub alternative: String,
    pub same_facts_used: String,
}
