# CodeTruth Protocol: Reliability, Criticality, and Rule-Packs

This document specifies how to reliably detect framework gotchas, duplication/bloat, and critical functionality across languages and frameworks using a singular adapter with pluggable rule-packs that emit protocol-native outputs.

The approach is fully aligned with the existing CodeTruth Protocol (CTP) artifacts:

- Explanation Graph (behavior, dependencies, side effects, drift)
- Policy Engine (YAML/JSON rule definitions; severities, enforcement, exceptions)
- Product Metadata (critical paths/map, product type, deployment)
- CLI (`ctp-cli analyze|explain|check`) for local and CI usage

---

## 1. Architecture Overview

- **Singular Adapter**
  - A single analyzer binary/library abstracts language/framework specifics via modular detectors.
  - Detectors produce a normalized internal model (AST facts, symbols, calls, config facts) that the adapter translates into CTP-native outputs.
  - Output stability: all findings are expressed as `policies.policy_results` and/or `drift` entries in the Explanation Graph.

- **Rule-Packs (Business Intelligence Rules)**
  - Packaged as files in `policies/<pack>` and may be authored in YAML, JSON, or Markdown (with frontmatter).
  - Contain rules that operate on adapter-provided facts (e.g., migrations ordering, queue configuration, duplication scores, coverage metrics).
  - Enforcement and exceptions are defined in the policy documents; the adapter only supplies evidence and context.

- **Data Flow**
  1) Source code, configs, coverage reports → Singular Adapter → Facts
  2) Facts → Rule Engine (policies from rule-packs) → PolicyResults
  3) PolicyResults + Detected Drifts → ExplanationGraph
  4) CLI renders results and provides CI exit codes.

---

## 2. Rule-Pack Format

Rule-packs are folders containing one or more policy documents in YAML/JSON or Markdown.

- **YAML/JSON Policy**
  - Keys: `id`, `name`, `description`, `scope.include[]`, `severity`, `rules[]`, `enforcement`, `exceptions[]`, `metadata`.
  - `rules[]` specify `rule_type` and parameters that the adapter understands.
  - Example (YAML):

```yaml
ctp_version: 1.0.0
policy_schema_version: 1.0.0
policy:
  id: laravel.migrations.timestamp_monotonic
  name: Laravel Migration Timestamps Must Be Monotonic
  description: Detect out-of-order/duplicate migration timestamps.
  scope:
    include: ["**/database/migrations/*.php"]
  severity: ERROR
  rules:
    - rule_type: behavior_pattern
      params:
        detector: migrations_timestamp_monotonic
  enforcement:
    block_merge: true
```

- **Markdown Policy (with frontmatter)**
  - Use YAML frontmatter to define policy metadata; body may include rationale and remediation.

```markdown
---
ctp_version: 1.0.0
policy_schema_version: 1.0.0
policy:
  id: duplication.semantic_similarity
  name: Duplicate Code Similarity Thresholds
  description: Flag near-duplicate functions across the codebase.
  scope:
    include: ["**/*.*"]
  severity: WARNING
  rules:
    - rule_type: behavior_pattern
      params:
        detector: semantic_similarity
        warn_threshold: 0.7
        error_threshold: 0.9
---

Rationale: Reduces maintenance risk. Remediation: consolidate into a single utility.
```

- **Detectors**
  - Named in `params.detector`. The singular adapter offers a stable set (e.g., `migrations_timestamp_monotonic`, `queue_horizon_configured`, `semantic_similarity`, `ui_duplicate_toast`, `coverage_min_thresholds`, `critical_tests_present`).
  - Detectors emit structured evidence consumed by the policy engine to create violations.

---

## 3. Framework Gotchas via Generic Detectors

While examples reference Laravel, detectors remain framework-agnostic:

- **Migrations Timestamp Monotonicity** (`migrations_timestamp_monotonic`)
  - Inputs: migration filenames, timestamps, neighboring order.
  - Violations: out-of-order or duplicate timestamps; remediation suggests regenerating with official tooling.

- **Queue/Horizon Configuration** (`queue_horizon_configured`)
  - Inputs: code references to job dispatch, queue config files, environment settings.
  - Violations: missing connection/queue, mismatched names, Horizon not tracking queue.

- **DB Topology & Query Alignment** (`db_topology_alignment`)
  - Inputs: schema facts (tables, PKs, FKs, indexes, partitions/shards) + query patterns.
  - Violations: missing indexes for common predicates, FK direction/cascade mismatches, partition misuse.

All above emit:
- `PolicyResult` with deterministic `id` and evidence.
- Optional `drift` entries (type `Implementation`) with remediation text.

---

## 4. Duplication and Bloat Detection

- **Semantic Similarity** (`semantic_similarity`)
  - AST-normalized hashing and token shingles to compute similarity between functions/modules.
  - Configurable thresholds (e.g., WARNING ≥ 0.7; ERROR ≥ 0.9).
  - Evidence lists symbol pairs, files, and score.

- **UI Duplicate Effects** (`ui_duplicate_toast`)
  - Detect repeated side-effect calls within single action flows (e.g., `toast()` multiple times along one path).

- **Maintainability/Bloat** (`maintainability_limits`)
  - Thresholds on lines of code, cyclomatic complexity, file size.
  - Optionally scope by folders (e.g., utilities) to focus on consolidation.

Mapping:
- Findings appear in `policies.policy_results`.
- Where appropriate, also reported as `drift` with type `Implementation`.

---

## 5. Critical Functionality: Registry and Enforcement

A deterministic, cross-language approach based on three prongs with confidence scoring.

### 5.1 Sources of Criticality

1. **Hot Path Discovery**
   - Static call graph frequency + git history signals (e.g., change hotspots) + optional runtime profiles where available.
   - Output: per-symbol call frequency percentile (0–1).

2. **Comments + Significant Usage/Occurrences**
   - Comment markers (e.g., `ctp:critical`, `mission-critical`, `security-sensitive`) and frequency of symbol references across repo.
   - Output: textual signal (0–1) from comment semantics + usage count percentile.

3. **Annotations + Specs/Globs**
   - Code annotations (language-idiomatic), comment tags, and `product-metadata` critical paths/map.
   - Output: binary/weighted signal (0–1) depending on explicitness (explicit IDs/globs score higher).

### 5.2 Critical Weight Formula

- Let:
  - `H` = Hot path score (0–1)
  - `U` = Usage & comment signal (0–1)
  - `A` = Annotation/spec/glob signal (0–1)
  - `W_product` = Product entity weighting (see §6)

- Critical Weight `CW`:

```
CW = clamp( 0.45*A + 0.35*H + 0.20*U, 0, 1 ) * W_product
```

- Classification thresholds (configurable):
  - Critical: CW ≥ 0.85
  - High: 0.70 ≤ CW < 0.85
  - Medium: 0.50 ≤ CW < 0.70
  - Low: CW < 0.50

- Confidence score reported alongside classification using signal quality and feature availability.

### 5.3 Registry Emission

- The adapter emits a registry of critical entities (symbol, file, classification, CW, confidence, entry points, dependencies).
- The registry is linked to each file’s `ExplanationGraph` via `behavior.entry_points` and `behavior.dependencies` when relevant.

### 5.4 Test Enforcement Policies

- `critical.tests.present`
  - For every Critical/High entity, at least one test references its symbol or mapped path.
- `critical.tests.coverage.min`
  - Coverage thresholds by class: Critical ≥ 90%, High ≥ 80%, Medium ≥ 70% (configurable).
- `critical.tests.runtime.path` (optional)
  - Enforce branch/path coverage for designated entry points when the coverage format supports it.

If coverage artifacts are missing for a critical file, the detector fails closed (ERROR with remediation).

---

## 6. Data Architecture Mapping and Product Correlation

- **Entity Extraction**
  - Parse schema definitions (migrations, ORM models, schema dumps) to build an entity graph (tables/collections, relationships, indexes, partitions).

- **Product Correlation**
  - Use `product-metadata` to identify domain entities (e.g., `orders`, `customers`, `invoices`).
  - Compute focus weighting `W_product` for symbols primarily operating on highly central entities.
  - Example heuristic: In an ecommerce product, entities related to `orders` receive `W_product` = 1.1–1.3, while peripheral entities get 0.9–1.0.

- **Effect on Critical Weight**
  - `W_product` multiplies `CW` to reflect business centrality, pushing order-related functions into higher criticality classes when appropriate.

---

### 6.1 Default W_product Map by Product Type

These are sensible defaults; tune in rule-pack configuration per repo. Values multiply the computed CW and should generally stay in [0.8, 1.4].

- b2c_saas
  - authentication/session: 1.2
  - billing/subscription: 1.25
  - core feature modules (top 2 by hot path): 1.2
  - ancillary modules (settings, profile): 0.95

- b2b_enterprise
  - access control/permissions: 1.25
  - integrations/connectors: 1.2
  - reporting/compliance exports: 1.15

- internal_tool
  - workflows/automation core: 1.15
  - admin/scripting surfaces: 1.1

- mobile_app
  - offline sync/storage: 1.2
  - payments/in-app purchases: 1.25
  - push/notifications pipeline: 1.15

- api_platform
  - request routing/throttling: 1.25
  - auth/token services: 1.25
  - top 3 endpoints by QPS: 1.2–1.3

- ecommerce
  - orders/order-lines: 1.3
  - checkout/payments/refunds: 1.3
  - catalog/pricing/inventory: 1.15
  - customers: 1.05

- financial_services
  - ledger/transactions: 1.35
  - reconciliation/reporting: 1.25
  - risk/fraud checks: 1.3

- healthcare
  - PHI handling (storage/transit): 1.35
  - scheduling/clinical workflows: 1.25
  - audit/compliance: 1.25

- unknown
  - core hot paths (top 2): 1.15
  - others: 1.0

## 7. Protocol Mapping

- **ExplanationGraph.behavior**
  - `entry_points[]`, `dependencies[]`, and `side_effects[]` populated by adapter detectors.

- **ExplanationGraph.drift**
  - Implementation drift for framework gotchas, duplication/bloat, and deviations from declared intent/policies.

- **PolicyResults**
  - All rule-pack outcomes are represented as `policy_results[]` with clear IDs, severities, and evidence (file, line, message).

- **Product Metadata**
  - `critical_paths` and `criticality_map` seed explicit criticality; used in the `A` signal and scoping of stricter enforcements.

---

## 8. CI Integration

- Run analyzers and coverage before policy checks.
- Example CI step:

```bash
ctp-cli check --policies ./policies --paths . --fail-on-violation
```

- Ensure coverage artifacts (LCOV, Cobertura, JaCoCo, nyc, pytest-cov, tarpaulin) are generated to known paths; the adapter parses them to enforce `critical.tests.coverage.min`.
- Use policy `enforcement.block_merge: true` for hard gates and `exceptions` with expiry where justified.

---

## 9. Configuration

- Global config:
  - Paths to coverage artifacts
  - Detectors enable/disable and thresholds
  - Glob patterns for language/framework files
- Rule-pack config:
  - Detector parameters per policy (thresholds, scopes)
  - Severity and enforcement settings

---

## 10. Quality and Accuracy Guarantees

- Deterministic sources for criticality (globs/specs/annotations) take precedence over heuristics.
- Adapter fails closed for missing mandatory artifacts (e.g., required coverage for Critical entities).
- Clear evidence emission (file, line, symbol, score, remediation) to support dispute resolution.

---

## 11. Examples of Policies (IDs)

- Framework/Infra
  - `migrations.timestamp_monotonic`
  - `queue.horizon_configured`
  - `db.index.missing_for_predicates`
  - `db.foreign_key.misaligned`

- Duplication/Bloat
  - `duplication.semantic_similarity`
  - `ui.toast.duplicate`
  - `maintainability.complexity_exceeded`

- Criticality/Test
  - `critical.tests.present`
  - `critical.tests.coverage.min`
  - `critical.tests.runtime.path`

---

## 12. Roadmap (Incremental Adoption)

1) Introduce rule-packs (`policies/`) with default thresholds and scopes.
2) Enable detectors in singular adapter for migrations, queues, duplication, coverage.
3) Build critical registry (hot paths, usage/comments, annotations/specs) and emit CW/confidence.
4) Enforce test presence/coverage in CI; add exceptions only with expiry and justification.
5) Expand data architecture mapping to refine `W_product` for domain centrality.

---

## 13. Appendix: Signals and Scoring Details

- Hot Paths (H): frequency from static call graph + churn hotspots (normalized 0–1).
- Usage/Comments (U): symbol reference density + presence of critical language in comments (NLP keyword list; locale-agnostic mapping possible via config).
- Annotations/Specs (A): explicit tags and globs: explicit ID = 1.0, glob match = 0.8, comment tag = 0.6.
- Confidence: min over signal availability + stability of mapping; lowered when symbol resolution falls back to path-level.

All formulas and thresholds are configurable per repository and per product type via rule-pack parameters.
