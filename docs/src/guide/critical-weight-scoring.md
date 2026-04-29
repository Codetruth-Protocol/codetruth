# Critical Weight Scoring System

## Overview

The Critical Weight Scoring System is a three-pronged approach to accurately identify critical functionality in codebases. It combines static analysis, usage patterns, and explicit annotations to produce a reliable criticality classification.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│              Critical Weight Calculator                  │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  CW = clamp(0.45*A + 0.35*H + 0.20*U, 0, 1) * W_product │
│                                                          │
│  Where:                                                  │
│  - A = Annotation/Spec/Glob signal (0-1)                │
│  - H = Hot path score (0-1)                             │
│  - U = Usage & comment signal (0-1)                      │
│  - W_product = Product entity weighting (0.8-1.4)       │
│                                                          │
└─────────────────────────────────────────────────────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
    ┌────────┐   ┌──────────┐  ┌──────────┐  ┌──────────┐
    │   A    │   │    H     │  │    U     │  │ W_product│
    │Signal  │   │  Signal  │  │  Signal  │  │  Signal  │
    └────────┘   └──────────┘  └──────────┘  └──────────┘
```

## Signal 1: Hot Path Discovery (H)

### Purpose

Identifies functions and code paths that are frequently executed or frequently modified, indicating high importance.

### Components

#### 1. Static Call Graph Frequency

**Method**:
- Build call graph from AST analysis
- Count function call frequency across entire codebase
- Normalize to percentile (0-1)

**Implementation**:
```rust
fn calculate_call_frequency(&self, symbol: &Symbol) -> f64 {
    let total_calls = self.call_graph.total_calls();
    let symbol_calls = self.call_graph.calls_to(symbol);
    
    if total_calls == 0 {
        return 0.0;
    }
    
    // Normalize to 0-1 range using percentile
    symbol_calls as f64 / total_calls as f64
}
```

#### 2. Git History Analysis

**Method**:
- Analyze git commit history for file change frequency
- Identify "hotspot" files (frequently modified)
- Calculate churn score based on:
  - Number of commits touching the file
  - Recency of changes
  - Number of contributors

**Implementation**:
```rust
fn calculate_git_churn(&self, file_path: &Path) -> f64 {
    let commits = self.git_history.commits_for_file(file_path);
    let recent_commits = commits.iter()
        .filter(|c| c.date > self.cutoff_date)
        .count();
    
    // Weight recent changes more heavily
    let recency_weight = 0.6;
    let total_weight = 0.4;
    
    let recent_score = (recent_commits as f64 / self.max_recent_commits).min(1.0);
    let total_score = (commits.len() as f64 / self.max_total_commits).min(1.0);
    
    recency_weight * recent_score + total_weight * total_score
}
```

#### 3. Runtime Profile Integration (Optional)

**Method**:
- Parse runtime profiling data (if available)
- Extract function execution frequency
- Normalize to 0-1 range

**Sources**:
- Application Performance Monitoring (APM) data
- Profiling tools (perf, pprof, etc.)
- Custom instrumentation

### Hot Path Score Formula

```
H = 0.5 * call_frequency_percentile 
  + 0.3 * git_churn_score 
  + 0.2 * runtime_profile_score
```

**Fallback**: If runtime profile is unavailable:
```
H = 0.6 * call_frequency_percentile 
  + 0.4 * git_churn_score
```

## Signal 2: Comments + Significant Usage (U)

### Purpose

Identifies critical functionality through explicit comments and high usage frequency across the codebase.

### Components

#### 1. Comment Markers

**Detection**:
- Parse comments for explicit criticality markers
- Support language-idiomatic annotations
- Locale-agnostic keyword matching

**Markers**:
```yaml
comment_markers:
  explicit:
    - "@critical"
    - "@mission-critical"
    - "@security-sensitive"
    - "@production-critical"
    - "@must-not-fail"
  keywords:
    en:
      - "critical"
      - "mission-critical"
      - "security-sensitive"
      - "production-critical"
      - "must not fail"
      - "do not modify"
      - "core functionality"
    es:
      - "crítico"
      - "crítico para la misión"
      - "sensible a la seguridad"
    # ... other locales
```

**Scoring**:
- Explicit marker: 1.0
- Keyword match: 0.6
- No match: 0.0

#### 2. Symbol Reference Count

**Method**:
- Count all references to symbol across codebase
- Normalize to percentile (0-1)
- Weight by reference type (direct call > import > comment)

**Implementation**:
```rust
fn calculate_usage_percentile(&self, symbol: &Symbol) -> f64 {
    let references = self.symbol_table.references_to(symbol);
    let total_references = self.symbol_table.total_references();
    
    if total_references == 0 {
        return 0.0;
    }
    
    // Weight by reference type
    let weighted_count = references.iter()
        .map(|r| match r.reference_type {
            ReferenceType::DirectCall => 1.0,
            ReferenceType::Import => 0.7,
            ReferenceType::Comment => 0.3,
        })
        .sum::<f64>();
    
    weighted_count / total_references as f64
}
```

#### 3. Keyword Density

**Method**:
- Count criticality keywords in comments near symbol
- Calculate density (keywords per comment length)
- Normalize to 0-1

### Usage Score Formula

```
U = 0.4 * comment_signal 
  + 0.4 * usage_percentile 
  + 0.2 * keyword_density
```

## Signal 3: Annotations + Specs/Globs (A)

### Purpose

Uses explicit declarations of criticality from annotations, product metadata, and policy globs.

### Components

#### 1. Code Annotations

**Language Support**:
- Python: `@critical`, `@mission_critical`
- TypeScript/JavaScript: `@critical`, `@CTPCritical`
- Rust: `#[critical]`, `#[ctp_critical]`
- Java: `@Critical`, `@MissionCritical`
- PHP: `@critical`, `@ctp-critical`

**Scoring**:
- Explicit annotation: 1.0

#### 2. Product Metadata Matching

**Method**:
- Match symbol against `product-metadata.json` critical paths
- Check `core_functionalities` entries
- Verify `criticality_map` globs

**Scoring**:
- Exact match in critical paths: 0.9
- Match in core functionalities: 0.7
- Match in criticality map: 0.8

#### 3. Policy Globs

**Method**:
- Match file path against policy-defined criticality globs
- Check policy `scope.include` patterns
- Verify explicit criticality declarations

**Scoring**:
- Explicit glob match: 0.8
- Policy scope match: 0.6

#### 4. Comment Tags

**Method**:
- Parse comment tags (e.g., `// CTP: critical`)
- Extract explicit declarations
- Normalize to 0-1

**Scoring**:
- Comment tag: 0.6

### Annotation Score Formula

```
A = max(
    explicit_annotation_score,      // 1.0 if present
    product_metadata_match_score,   // 0.7-0.9
    policy_glob_match_score,        // 0.6-0.8
    comment_tag_score              // 0.6
)
```

## Product Entity Weighting (W_product)

### Purpose

Adjusts critical weight based on business centrality of entities the code operates on.

### Method

1. **Extract Domain Entities**: From `product-metadata.json` or schema analysis
2. **Map Code to Entities**: Identify which entities code operates on
3. **Apply Weighting**: Multiply CW by entity weight

### Default Weights by Product Type

```yaml
product_weighting:
  ecommerce:
    orders/order-lines: 1.3
    checkout/payments/refunds: 1.3
    catalog/pricing/inventory: 1.15
    customers: 1.05
    reviews/ratings: 0.95
  
  b2c_saas:
    authentication/session: 1.2
    billing/subscription: 1.25
    core_feature_modules: 1.2
    ancillary_modules: 0.95
  
  b2b_enterprise:
    access_control/permissions: 1.25
    integrations/connectors: 1.2
    reporting/compliance_exports: 1.15
  
  financial_services:
    ledger/transactions: 1.35
    reconciliation/reporting: 1.25
    risk/fraud_checks: 1.3
  
  healthcare:
    PHI_handling: 1.35
    scheduling/clinical_workflows: 1.25
    audit/compliance: 1.25
  
  api_platform:
    request_routing/throttling: 1.25
    auth/token_services: 1.25
    top_endpoints_by_qps: 1.2-1.3
```

### Implementation

```rust
fn calculate_product_weight(&self, symbol: &Symbol, product_type: &str) -> f64 {
    let entities = self.entity_mapper.entities_for_symbol(symbol);
    let weights = self.product_weights.get(product_type)?;
    
    let max_weight = entities.iter()
        .filter_map(|e| weights.get(e))
        .copied()
        .fold(1.0, f64::max);
    
    // Clamp to reasonable range
    max_weight.clamp(0.8, 1.4)
}
```

## Final Critical Weight Calculation

### Formula

```
CW = clamp(0.45*A + 0.35*H + 0.20*U, 0, 1) * W_product
```

### Classification

- **Critical**: CW ≥ 0.85
- **High**: 0.70 ≤ CW < 0.85
- **Medium**: 0.50 ≤ CW < 0.70
- **Low**: CW < 0.50

### Confidence Score

Confidence is calculated based on signal availability and quality:

```
Confidence = min(
    signal_availability_score,    // All 3 signals present = 1.0
    signal_stability_score,        // Consistent across runs
    resolution_quality_score       // Exact match vs. path-level
)
```

**Signal Availability**:
- All 3 signals (A, H, U): 1.0
- 2 signals: 0.7
- 1 signal: 0.4

**Resolution Quality**:
- Exact symbol match: 1.0
- Path-level match: 0.7
- Pattern match: 0.5

## Example Calculation

### Scenario: E-commerce Order Processing Function

**Given**:
- Function: `processOrder()` in `src/orders/processor.ts`
- Product type: `ecommerce`
- Entity: `orders`

**Signals**:
- **A (Annotation)**: 0.9 (matches product metadata critical path)
- **H (Hot Path)**: 0.85 (high call frequency, frequently modified)
- **U (Usage)**: 0.75 (many references, some critical comments)
- **W_product**: 1.3 (operates on `orders` entity)

**Calculation**:
```
CW = clamp(0.45*0.9 + 0.35*0.85 + 0.20*0.75, 0, 1) * 1.3
  = clamp(0.405 + 0.2975 + 0.15, 0, 1) * 1.3
  = clamp(0.8525, 0, 1) * 1.3
  = 0.8525 * 1.3
  = 1.10825
  = 1.0 (clamped to max)
```

**Result**: **Critical** (CW = 1.0, exceeds 0.85 threshold)

**Confidence**: 0.95 (all signals present, exact symbol match)

## Configuration

All thresholds and weights are configurable via policy or configuration file:

```yaml
critical_weight_config:
  signal_weights:
    annotation: 0.45
    hot_path: 0.35
    usage: 0.20
  classification_thresholds:
    critical: 0.85
    high: 0.70
    medium: 0.50
  product_weighting:
    # Override defaults per repository
    ecommerce:
      orders: 1.3
      # ...
```

## Best Practices

1. **Explicit Annotations**: Use code annotations for critical functions to ensure accurate classification
2. **Product Metadata**: Maintain accurate `product-metadata.json` with critical paths
3. **Coverage Reports**: Generate coverage reports for accurate test enforcement
4. **Git History**: Ensure git history is available for hot path analysis
5. **Regular Updates**: Recalculate critical weights periodically as codebase evolves
