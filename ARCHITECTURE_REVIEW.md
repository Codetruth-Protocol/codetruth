# CodeTruth Protocol - Architecture Review & Refactoring Plan

## Executive Summary

This document addresses the critical challenge: **How to handle large context and summarization without losing essential information** when analyzing codebases with 1000+ components.

The analogy: Like documenting body parts for a doctor - we need a system where:
- Essential information is **never lost** (a hand's function is always preserved)
- Redundancy is **prevented** (no duplicate hands doing leg work)
- Context is **hierarchically compressed** (2 pages → 1 sentence when appropriate)
- Relationships are **explicitly tracked** (hand connects to arm, not to foot)

---

## Current Architecture Analysis

### Strengths
1. **Rust core** - Performance for large codebases
2. **Modular crates** - `ctp-core`, `ctp-drift`, `ctp-llm`, `ctp-policy`, `ctp-parser`
3. **Minimal mode** - `MinimalAnalysis` struct for 90% use cases
4. **Tree-sitter parsing** - Fast, incremental AST analysis

### Critical Gaps Identified

#### Gap 1: No Hierarchical Context Model
**Current state**: Each file analyzed in isolation
```rust
// engine.rs:135 - analyze_file operates on single files
pub async fn analyze_file(&self, path: &Path) -> CTPResult<ExplanationGraph>
```
**Problem**: With 1000 components, there's no way to understand:
- How components relate to each other
- Which components are "critical" vs "peripheral"
- System-level intent vs file-level intent

#### Gap 2: No Semantic Compression Layer
**Current state**: Intent stored as raw strings (280 char limit)
```rust
// engine.rs:449 - Truncation without semantic preservation
return docstring.trim().chars().take(280).collect();
```
**Problem**: Truncation loses meaning. "This function handles payment retry with idempotency keys using exponential backoff" becomes "This function handles payment retry with idempotency keys using expo..."

#### Gap 3: No Component Registry / Ontology
**Current state**: No tracking of component types, roles, or relationships
**Problem**: Can't prevent "adding new hands to the body" - no way to detect when new code duplicates existing functionality

#### Gap 4: Flat LLM Context Management
**Current state**: LLM prompts built ad-hoc with simple string formatting
```rust
// ctp-llm/src/lib.rs:62-76 - Simple prompt construction
let prompt = format!(r#"Analyze this code and infer its intent...
```
**Problem**: For large codebases, can't fit all context in LLM window. No strategy for:
- What to include vs exclude
- How to summarize related components
- Maintaining consistency across analyses

#### Gap 5: No Incremental/Delta Analysis
**Current state**: Full re-analysis on every run
**Problem**: Analyzing 1000 files repeatedly is wasteful and loses historical context

---

## Proposed Architecture: Hierarchical Semantic Compression (HSC)

### Core Concept: The "Anatomy Model"

Like medical documentation, we need:

```
┌─────────────────────────────────────────────────────────────────┐
│                    SYSTEM LEVEL (The Body)                       │
│  "E-commerce platform handling payments, inventory, users"       │
│  Essential invariants: "All payments must be idempotent"         │
├─────────────────────────────────────────────────────────────────┤
│                   DOMAIN LEVEL (Organ Systems)                   │
│  payments/     → "Handles all financial transactions"            │
│  inventory/    → "Manages product stock and availability"        │
│  users/        → "Authentication and user management"            │
├─────────────────────────────────────────────────────────────────┤
│                  MODULE LEVEL (Organs)                           │
│  payments/retry.rs → "Retry logic with idempotency"              │
│  payments/charge.rs → "Core charging logic"                      │
├─────────────────────────────────────────────────────────────────┤
│                 FUNCTION LEVEL (Cells)                           │
│  retry_with_backoff() → "Exponential backoff, max 3 retries"     │
│  generate_idempotency_key() → "Creates unique key per attempt"   │
└─────────────────────────────────────────────────────────────────┘
```

### New Data Structures

```rust
/// Hierarchical context that can be compressed without losing essence
pub struct SemanticContext {
    /// Unique identifier for this context node
    pub id: ContextId,
    
    /// Level in the hierarchy
    pub level: ContextLevel,
    
    /// The "essence" - irreducible core meaning (NEVER truncated)
    pub essence: Essence,
    
    /// Expanded detail (can be compressed/expanded as needed)
    pub detail: Option<DetailLevel>,
    
    /// Relationships to other contexts
    pub relationships: Vec<Relationship>,
    
    /// Invariants that must be preserved (like "hand must grip")
    pub invariants: Vec<Invariant>,
    
    /// Hash for change detection
    pub content_hash: String,
}

#[derive(Debug, Clone)]
pub enum ContextLevel {
    System,      // Entire codebase
    Domain,      // Major subsystem (payments, auth, etc.)
    Module,      // Single file/module
    Function,    // Individual function/method
    Block,       // Code block within function
}

/// The irreducible core meaning - like "hand grips objects"
#[derive(Debug, Clone)]
pub struct Essence {
    /// One-sentence purpose (MUST fit in 100 chars)
    pub purpose: String,
    
    /// Role in the system (what would break if this disappeared?)
    pub role: ComponentRole,
    
    /// Critical constraints (things that MUST remain true)
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ComponentRole {
    /// Entry point - receives external input
    Boundary { direction: BoundaryDirection },
    
    /// Core logic - implements business rules
    Core { domain: String },
    
    /// Utility - supports other components
    Utility { used_by: Vec<ContextId> },
    
    /// Orchestrator - coordinates other components
    Orchestrator { coordinates: Vec<ContextId> },
    
    /// Data - holds/transforms data
    Data { schema_hash: String },
}

/// Relationship between components (prevents "hand doing leg work")
#[derive(Debug, Clone)]
pub struct Relationship {
    pub target: ContextId,
    pub relationship_type: RelationshipType,
    pub strength: f64, // 0.0-1.0, how tightly coupled
}

#[derive(Debug, Clone)]
pub enum RelationshipType {
    /// A calls B
    Calls,
    /// A depends on B's types/interfaces
    DependsOn,
    /// A and B serve same purpose (REDUNDANCY WARNING)
    Duplicates,
    /// A extends/implements B
    Extends,
    /// A and B must change together
    CoChanges,
}

/// Invariant that must be preserved across changes
#[derive(Debug, Clone)]
pub struct Invariant {
    pub id: String,
    pub description: String,
    pub severity: InvariantSeverity,
    /// Code patterns that indicate violation
    pub violation_patterns: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum InvariantSeverity {
    /// System will crash/corrupt data
    Critical,
    /// Feature will malfunction
    High,
    /// Degraded behavior
    Medium,
    /// Style/convention violation
    Low,
}
```

### Compression Strategy

```rust
/// Compresses context to fit within token budget while preserving essence
pub struct ContextCompressor {
    /// Maximum tokens for LLM context
    token_budget: usize,
    
    /// Priority rules for what to keep
    priority_rules: Vec<PriorityRule>,
}

impl ContextCompressor {
    /// Compress a set of contexts to fit budget
    pub fn compress(
        &self,
        contexts: &[SemanticContext],
        focus: &ContextId,  // What we're currently analyzing
        budget: usize,
    ) -> CompressedContext {
        // 1. Always include essence of focus context (full detail)
        // 2. Include essence of direct relationships
        // 3. Include essence of ancestors (system → domain → module)
        // 4. Summarize siblings at same level
        // 5. Omit unrelated contexts entirely
        
        let mut result = CompressedContext::new(budget);
        
        // Phase 1: Mandatory inclusions (essence only)
        let focus_ctx = self.find_context(contexts, focus);
        result.add_full(focus_ctx);
        
        for rel in &focus_ctx.relationships {
            if rel.strength > 0.7 {
                result.add_essence(self.find_context(contexts, &rel.target));
            }
        }
        
        // Phase 2: Hierarchical context (ancestors)
        for ancestor in self.get_ancestors(contexts, focus) {
            result.add_essence(&ancestor);
        }
        
        // Phase 3: Fill remaining budget with relevant siblings
        let remaining = budget - result.token_count();
        let siblings = self.get_siblings(contexts, focus);
        result.add_summaries(siblings, remaining);
        
        result
    }
    
    /// Generate a summary that preserves essence
    pub fn summarize_group(
        &self,
        contexts: &[SemanticContext],
    ) -> String {
        // Group by role
        let by_role = self.group_by_role(contexts);
        
        // Generate summary like:
        // "5 utility functions for string manipulation, 
        //  2 core handlers for payment processing,
        //  1 boundary for API input validation"
        
        by_role.iter()
            .map(|(role, ctxs)| {
                format!("{} {} for {}", 
                    ctxs.len(),
                    role.category(),
                    self.common_purpose(ctxs))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}
```

### Component Registry (Prevents Redundancy)

```rust
/// Registry of all components - prevents "adding new hands"
pub struct ComponentRegistry {
    /// All known components indexed by semantic hash
    components: HashMap<SemanticHash, RegistryEntry>,
    
    /// Similarity index for finding duplicates
    similarity_index: SimilarityIndex,
}

impl ComponentRegistry {
    /// Check if new code duplicates existing functionality
    pub fn check_redundancy(
        &self,
        new_essence: &Essence,
    ) -> RedundancyReport {
        // Find semantically similar components
        let similar = self.similarity_index.find_similar(
            &new_essence.purpose,
            threshold: 0.8,
        );
        
        if similar.is_empty() {
            return RedundancyReport::Unique;
        }
        
        // Check if truly redundant or just related
        for candidate in similar {
            let existing = self.components.get(&candidate.hash);
            
            if self.is_true_duplicate(new_essence, &existing.essence) {
                return RedundancyReport::Duplicate {
                    existing: existing.id.clone(),
                    similarity: candidate.score,
                    recommendation: format!(
                        "Consider using existing component '{}' instead",
                        existing.essence.purpose
                    ),
                };
            }
        }
        
        RedundancyReport::Related { similar }
    }
    
    /// Register a new component
    pub fn register(
        &mut self,
        context: SemanticContext,
    ) -> Result<(), RegistryError> {
        // Check for redundancy first
        let redundancy = self.check_redundancy(&context.essence);
        
        if let RedundancyReport::Duplicate { existing, .. } = redundancy {
            return Err(RegistryError::RedundantComponent {
                new: context.id,
                existing,
            });
        }
        
        // Add to registry
        let hash = self.compute_semantic_hash(&context.essence);
        self.components.insert(hash.clone(), RegistryEntry {
            id: context.id.clone(),
            essence: context.essence.clone(),
            hash,
            registered_at: Utc::now(),
        });
        
        // Update similarity index
        self.similarity_index.add(&context.essence);
        
        Ok(())
    }
}
```

---

## Implementation Phases

### Phase 1: Foundation (Week 1-2)
Create new crate `ctp-context` with:
- [ ] `SemanticContext` and related types
- [ ] `Essence` extraction from existing `ExplanationGraph`
- [ ] Basic `ContextLevel` hierarchy detection

### Phase 2: Compression (Week 3-4)
- [ ] `ContextCompressor` implementation
- [ ] Token counting utilities
- [ ] Priority rules engine
- [ ] Integration with `ctp-llm` for smarter prompts

### Phase 3: Registry (Week 5-6)
- [ ] `ComponentRegistry` implementation
- [ ] Semantic similarity index (consider using embeddings)
- [ ] Redundancy detection in `ctp-drift`

### Phase 4: Integration (Week 7-8)
- [ ] Update `CodeTruthEngine` to build hierarchical context
- [ ] Update CLI for multi-file analysis with context
- [ ] Add incremental analysis support

---

## Migration Strategy

### Backward Compatibility
- Existing `ExplanationGraph` remains valid
- New `SemanticContext` can be derived from `ExplanationGraph`
- Minimal mode continues to work unchanged

### Gradual Adoption
```rust
// Old API still works
let graph = engine.analyze_file(path).await?;

// New API for context-aware analysis
let context = engine.analyze_with_context(path, &registry).await?;

// Batch analysis with full hierarchy
let system_context = engine.analyze_codebase(root_path).await?;
```

---

## Key Principles

1. **Essence is Sacred**: Never truncate or lose the core purpose
2. **Relationships are Explicit**: No implicit dependencies
3. **Hierarchy Enables Compression**: System → Domain → Module → Function
4. **Redundancy is Drift**: Duplicate functionality = architectural drift
5. **Invariants are Contracts**: Breaking an invariant = breaking the system

---

## Success Metrics

| Metric | Current | Target |
|--------|---------|--------|
| Context loss on summarization | ~40% | <5% |
| Redundancy detection rate | 0% | >90% |
| Analysis time (1000 files) | N/A | <60s |
| LLM token efficiency | ~20% | >80% |
| False positive drift rate | ~30% | <10% |

---

## Next Steps

1. Review this document with stakeholders
2. Create `ctp-context` crate skeleton
3. Implement `Essence` extraction as proof of concept
4. Validate on real codebase (suggest: fastapi or express)
