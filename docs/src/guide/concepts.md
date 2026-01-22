# Core Concepts

Understanding the fundamental concepts behind CodeTruth Protocol.

## The Problem: AI Code Trust

As AI-generated code becomes ubiquitous, a critical question emerges: **How do you trust code you didn't write?**

Traditional code review assumes human-written code with clear intent. AI-generated code often lacks:
- Clear documentation of intent
- Explanation of design decisions
- Visibility into assumptions made
- Audit trail for compliance

## CTP's Solution

CodeTruth Protocol addresses this through three core mechanisms:

1. **Explanation Graphs** - Machine-readable descriptions of code intent and behavior
2. **Drift Detection** - Automated comparison of declared intent vs actual implementation
3. **Policy Evaluation** - Governance rules for code quality and compliance

## Key Concepts

### Intent

**Intent** is what the code is supposed to do. CTP distinguishes between:

- **Declared Intent**: What the code claims to do (from comments, docstrings, documentation)
- **Inferred Intent**: What CTP determines the code is trying to do (from analysis)

```python
def calculate_discount(price: float, percentage: float) -> float:
    """Calculate discounted price."""  # ← Declared Intent
    return price * (1 - percentage / 100)  # ← Actual Implementation
```

### Behavior

**Behavior** is what the code actually does. CTP analyzes:

- **Entry Points**: Functions, methods, endpoints
- **Exit Points**: Return values, exceptions
- **Side Effects**: I/O, network, database, state mutations
- **Dependencies**: External modules and services

### Drift

**Drift** is the mismatch between intent and behavior. Types include:

| Type | Description | Example |
|------|-------------|---------|
| **Intent Drift** | Declared ≠ Inferred intent | Comment says "add" but code subtracts |
| **Policy Drift** | Code violates policies | Missing error handling |
| **Assumption Drift** | Invalid assumptions | Assumes non-null input |
| **Implementation Drift** | Behavior changed | New side effects added |

### Confidence

**Confidence** (0.0 - 1.0) indicates how certain CTP is about its analysis:

- **0.9+**: High confidence, well-documented code
- **0.7-0.9**: Good confidence, some ambiguity
- **0.5-0.7**: Moderate confidence, needs review
- **<0.5**: Low confidence, significant uncertainty

## The Explanation Graph

The **Explanation Graph** is CTP's core output - a structured representation of code understanding:

```json
{
  "ctp_version": "1.0.0",
  "module": {
    "name": "handler.py",
    "language": "python",
    "lines_of_code": 150
  },
  "intent": {
    "declared_intent": "Process payment transactions",
    "inferred_intent": "Handle payments with retry logic",
    "confidence": 0.85
  },
  "behavior": {
    "actual_behavior": "3 functions, database ops, network calls",
    "side_effects": [
      {"type": "database", "risk_level": "HIGH"},
      {"type": "network", "risk_level": "MEDIUM"}
    ]
  },
  "drift": {
    "drift_detected": true,
    "drift_severity": "LOW",
    "drift_details": [...]
  }
}
```

## Analysis Modes

### Minimal Mode (Default)

For 90% of use cases. Fast, lightweight analysis:

```json
{
  "file_hash": "sha256:...",
  "intent": "Process payments",
  "behavior": "3 functions, database ops",
  "drift": "LOW",
  "confidence": 0.85
}
```

### Standard Mode

Adds policy evaluation and historical context:

```bash
ctp analyze src/ --mode standard
```

### Advanced Mode

Full explanation graph with cross-file analysis:

```bash
ctp analyze src/ --mode advanced --enhance
```

## The Analysis Pipeline

```
Source Code
    ↓
┌─────────────────┐
│  AST Parsing    │  ← tree-sitter
│  (Layer 1)      │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Intent Extract  │  ← Comments, docstrings
│ Behavior Analyze│  ← Side effects, deps
│  (Layer 2)      │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Drift Detection │  ← Compare intent/behavior
│ Policy Evaluate │  ← Check rules
│  (Layer 2)      │
└────────┬────────┘
         ↓
┌─────────────────┐
│ Explanation     │  ← Generate output
│ Graph           │
│  (Output)       │
└─────────────────┘
```

## Next Steps

- [Explanation Graphs](./explanation-graphs.md) - Deep dive into the output format
- [Drift Detection](./drift-detection.md) - How drift is detected
- [Policy Evaluation](./policy-evaluation.md) - Writing and using policies
