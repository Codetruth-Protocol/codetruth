/**
 * Core CTP types matching the protocol specification
 */

export interface ExplanationGraph {
  ctp_version: string;
  explanation_id: string;
  module: Module;
  intent: Intent;
  behavior: Behavior;
  drift: DriftAnalysis;
  policies: PolicyResults;
  history: History;
  metadata: Metadata;
}

export interface Module {
  name: string;
  path: string;
  language: string;
  lines_of_code: number;
  complexity_score: number;
}

export interface Intent {
  declared_intent: string;
  inferred_intent: string;
  confidence: number;
  business_context: string;
  technical_rationale: string;
}

export interface Behavior {
  actual_behavior: string;
  entry_points: EntryPoint[];
  exit_points: ExitPoint[];
  side_effects: SideEffect[];
  dependencies: Dependency[];
}

export interface EntryPoint {
  function: string;
  parameters: string[];
  preconditions: string[];
}

export interface ExitPoint {
  return_type: string;
  possible_values: string[];
  postconditions: string[];
}

export interface SideEffect {
  effect_type: 'io' | 'network' | 'database' | 'state_mutation';
  description: string;
  risk_level: 'LOW' | 'MEDIUM' | 'HIGH';
}

export interface Dependency {
  module: string;
  reason: string;
  coupling_type: 'tight' | 'loose';
}

export interface DriftAnalysis {
  drift_detected: boolean;
  drift_severity: DriftSeverity;
  drift_details: DriftDetail[];
}

export type DriftSeverity = 'NONE' | 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';

export interface DriftDetail {
  drift_type: 'INTENT' | 'POLICY' | 'ASSUMPTION' | 'IMPLEMENTATION';
  expected: string;
  actual: string;
  location: Location;
  impact: Impact;
  remediation: string;
}

export interface Location {
  file: string;
  line_start: number;
  line_end: number;
}

export interface Impact {
  functional: string;
  security: string;
  performance: string;
  maintainability: string;
}

export interface PolicyResults {
  evaluated_at: string;
  policy_results: PolicyResult[];
}

export interface PolicyResult {
  policy_id: string;
  policy_name: string;
  status: 'PASS' | 'FAIL' | 'WARNING' | 'SKIP';
  violations: Violation[];
}

export interface Violation {
  rule: string;
  severity: 'INFO' | 'WARNING' | 'ERROR' | 'CRITICAL';
  message: string;
  location: Location;
  evidence: string;
}

export interface History {
  previous_versions: PreviousVersion[];
  evolution: Evolution;
}

export interface PreviousVersion {
  version_id: string;
  analyzed_at: string;
  commit_hash: string;
  drift_from_previous: string;
}

export interface Evolution {
  created_at: string;
  last_modified: string;
  modification_count: number;
  stability_score: number;
}

export interface Metadata {
  generated_at: string;
  generator: Generator;
  extensions: Record<string, unknown>;
}

export interface Generator {
  name: string;
  version: string;
  llm_provider?: string;
  llm_model?: string;
}

/** Minimal analysis for 90% of use cases */
export interface MinimalAnalysis {
  ctp_version: string;
  file_hash: string;
  intent: string;
  behavior: string;
  drift: DriftSeverity;
  confidence: number;
}
