export type ChatMode = "conversation" | "analysis" | "integrated";

export type MessageRole = "user" | "assistant" | "system";

export interface ChatMessage {
  id: string;
  role: MessageRole;
  content: string;
  mode: ChatMode;
  timestamp: string;
  analysis?: AnalysisResult;
}

export interface ChatRequest {
  message: string;
  mode: ChatMode;
  session_id?: string;
}

export interface ChatResponse {
  session_id: string;
  message: string;
  mode: string;
  analysis?: AnalysisResult;
  contradictions?: Contradiction[];
  beliefs_updated?: Belief[];
}

export interface AnalyzeRequest {
  text: string;
}

export interface AnalysisResult {
  id: string;
  input_text: string;
  syntactic: SyntacticAnalysis;
  semantic: SemanticAnalysis;
  discourse: DiscourseAnalysis;
  critical_synthesis: CriticalSynthesis;
  created_at: string;
}

// Layer 1: Syntactic
export interface SyntacticAnalysis {
  voice_analysis: VoiceInstance[];
  sentence_complexity: SentenceComplexity[];
  nominalisations: Nominalisation[];
  transitivity: TransitivityInstance[];
}

export interface VoiceInstance {
  sentence: string;
  voice: "active" | "passive";
  significance: string;
}

export interface SentenceComplexity {
  sentence: string;
  score: number;
  clause_count: number;
  note: string;
}

export interface Nominalisation {
  original: string;
  verb_form: string;
  effect: string;
}

export interface TransitivityInstance {
  sentence: string;
  actor: string;
  process: string;
  affected: string;
  analysis: string;
}

// Layer 2: Semantic
export interface SemanticAnalysis {
  presuppositions: Presupposition[];
  implicatures: Implicature[];
  power_hierarchies: PowerHierarchy[];
  lexical_fields: LexicalField[];
}

export interface Presupposition {
  trigger: string;
  presupposed_content: string;
  significance: string;
}

export interface Implicature {
  statement: string;
  implied_meaning: string;
  mechanism: string;
}

export interface PowerHierarchy {
  dominant: string;
  subordinate: string;
  linguistic_markers: string[];
  analysis: string;
}

export interface LexicalField {
  field_name: string;
  terms: string[];
  connotation: string;
}

// Layer 3: Discourse
export interface DiscourseAnalysis {
  framing: FramingInstance[];
  strategic_omissions: StrategicOmission[];
  collocations: CollocationPattern[];
  intertextuality: IntertextualityMarker[];
}

export interface FramingInstance {
  frame_name: string;
  evidence: string;
  effect: string;
}

export interface StrategicOmission {
  what_is_missing: string;
  why_it_matters: string;
  who_benefits: string;
}

export interface CollocationPattern {
  pattern: string;
  frequency_note: string;
  ideological_loading: string;
}

export interface IntertextualityMarker {
  reference: string;
  source_discourse: string;
  function: string;
}

// Layer 4: Critical Synthesis
export interface CriticalSynthesis {
  naturalised_claims: NaturalisedClaim[];
  beneficiary_analysis: BeneficiaryAnalysis[];
  hidden_contexts: HiddenContext[];
  alternative_framings: AlternativeFraming[];
}

export interface NaturalisedClaim {
  claim: string;
  how_naturalised: string;
  counter_evidence: string;
}

export interface BeneficiaryAnalysis {
  who_benefits: string;
  how: string;
  who_is_disadvantaged: string;
}

export interface HiddenContext {
  context: string;
  relevance: string;
  why_hidden: string;
}

export interface AlternativeFraming {
  original_frame: string;
  alternative: string;
  same_facts_used: string;
}

// Belief & Consciousness
export interface Belief {
  id: string;
  user_id: string;
  claim: string;
  confidence: number;
  source_message_id: string;
  created_at: string;
  updated_at: string;
}

export interface Contradiction {
  belief_a: Belief;
  belief_b: Belief;
  explanation: string;
  severity: number;
}

export interface ConsciousnessState {
  user_id: string;
  session_id: string;
  epistemic_humility: number;
  belief_volatility: number;
  contradiction_awareness: number;
  depth_of_inquiry: number;
  timestamp: string;
}

export interface HealthResponse {
  status: string;
  services: {
    postgres: ServiceStatus;
    neo4j: ServiceStatus;
    qdrant: ServiceStatus;
    influxdb: ServiceStatus;
    redis: ServiceStatus;
    ollama: ServiceStatus;
  };
}

export interface ServiceStatus {
  status: string;
  error?: string;
}

// WebSocket
export interface WsOutgoing {
  type: string;
  content: string;
  analysis?: AnalysisResult;
}

// Auth
export interface AuthResponse {
  token: string;
  user_id: string;
  username: string;
}
