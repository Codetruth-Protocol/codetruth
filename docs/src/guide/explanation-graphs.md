# Explanation Graphs

The Explanation Graph is CTP's core output format - a structured, machine-readable representation of code understanding.

## Overview

An Explanation Graph contains:

- **Module Information**: File metadata and metrics
- **Intent**: Declared and inferred purpose
- **Behavior**: What the code actually does
- **Drift Analysis**: Mismatches between intent and behavior
- **Policy Results**: Compliance evaluation
- **History**: Version tracking
- **Metadata**: Analysis information

## Full Schema

```json
{
  "ctp_version": "1.0.0",
  "explanation_id": "sha256:abc123...",
  
  "module": {
    "name": "payment_handler.py",
    "path": "src/payments/payment_handler.py",
    "language": "python",
    "lines_of_code": 150,
    "complexity_score": 12.5
  },
  
  "intent": {
    "declared_intent": "Process payment transactions with retry logic",
    "inferred_intent": "Handle payment processing with exponential backoff",
    "confidence": 0.85,
    "business_context": "Part of checkout flow",
    "technical_rationale": "Uses idempotency keys for safety"
  },
  
  "behavior": {
    "actual_behavior": "Performs 3 function(s), database operations, network calls",
    "entry_points": [
      {
        "function": "process_payment",
        "parameters": ["amount", "customer_id", "idempotency_key"],
        "preconditions": ["amount > 0", "customer_id is valid"]
      }
    ],
    "exit_points": [
      {
        "return_type": "PaymentResult",
        "possible_values": ["success", "failed", "pending"],
        "postconditions": ["transaction recorded"]
      }
    ],
    "side_effects": [
      {
        "effect_type": "database",
        "description": "Writes transaction record",
        "risk_level": "HIGH"
      },
      {
        "effect_type": "network",
        "description": "Calls payment gateway API",
        "risk_level": "MEDIUM"
      }
    ],
    "dependencies": [
      {
        "module": "stripe",
        "reason": "Payment processing",
        "coupling_type": "tight"
      }
    ]
  },
  
  "drift": {
    "drift_detected": false,
    "drift_severity": "NONE",
    "drift_details": []
  },
  
  "policies": {
    "evaluated_at": "2026-01-16T20:00:00Z",
    "policy_results": [
      {
        "policy_id": "payment-idempotency",
        "policy_name": "Payment Idempotency Required",
        "status": "PASS",
        "violations": []
      }
    ]
  },
  
  "history": {
    "previous_versions": [
      {
        "version_id": "sha256:def456...",
        "analyzed_at": "2026-01-15T10:00:00Z",
        "commit_hash": "abc123",
        "drift_from_previous": "NONE"
      }
    ],
    "evolution": {
      "created_at": "2026-01-01T00:00:00Z",
      "last_modified": "2026-01-16T20:00:00Z",
      "modification_count": 5,
      "stability_score": 0.95
    }
  },
  
  "metadata": {
    "generated_at": "2026-01-16T20:00:00Z",
    "generator": {
      "name": "CodeTruth",
      "version": "0.1.0",
      "llm_provider": "anthropic",
      "llm_model": "claude-sonnet-4-20250514"
    },
    "extensions": {}
  }
}
```

## Minimal Analysis

For lightweight use cases, CTP provides a minimal format:

```json
{
  "ctp_version": "1.0.0",
  "file_hash": "sha256:abc123...",
  "intent": "Process payment transactions",
  "behavior": "3 functions, database ops, network calls",
  "drift": "NONE",
  "confidence": 0.85
}
```

## Field Reference

### Module

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | File name |
| `path` | string | Full file path |
| `language` | string | Programming language |
| `lines_of_code` | int | Line count |
| `complexity_score` | float | Cyclomatic complexity |

### Intent

| Field | Type | Description |
|-------|------|-------------|
| `declared_intent` | string | From comments/docs |
| `inferred_intent` | string | From analysis |
| `confidence` | float | 0.0 - 1.0 |
| `business_context` | string | Business purpose |
| `technical_rationale` | string | Technical reasoning |

### Behavior

| Field | Type | Description |
|-------|------|-------------|
| `actual_behavior` | string | Summary description |
| `entry_points` | array | Function entry points |
| `exit_points` | array | Return points |
| `side_effects` | array | I/O, network, DB ops |
| `dependencies` | array | External dependencies |

### Side Effect Types

| Type | Description |
|------|-------------|
| `io` | File system operations |
| `network` | HTTP/socket calls |
| `database` | Database queries |
| `state_mutation` | Global state changes |

### Risk Levels

| Level | Description |
|-------|-------------|
| `LOW` | Minimal risk |
| `MEDIUM` | Moderate risk, review recommended |
| `HIGH` | Significant risk, requires attention |

### Drift Severity

| Level | Description |
|-------|-------------|
| `NONE` | No drift detected |
| `LOW` | Minor discrepancies |
| `MEDIUM` | Notable differences |
| `HIGH` | Significant mismatch |
| `CRITICAL` | Severe drift, immediate action needed |

## Working with Explanation Graphs

### Generate Graph

```bash
ctp explain src/handler.py --format json > graph.json
```

### Query with jq

```bash
# Get drift severity
cat graph.json | jq '.drift.drift_severity'

# List side effects
cat graph.json | jq '.behavior.side_effects[].effect_type'

# Check confidence
cat graph.json | jq '.intent.confidence'
```

### Programmatic Access (TypeScript)

```typescript
import { analyze } from '@codetruth/sdk';

const graph = await analyze('src/handler.py');
console.log(graph.drift.drift_severity);
```

### Programmatic Access (Python)

```python
from codetruth import analyze

graph = analyze('src/handler.py')
print(graph.drift.drift_severity)
```

## Storing Graphs

By default, CTP stores graphs in `.ctp/analyses/`:

```
.ctp/
└── analyses/
    └── sha256_abc123.json
```

Enable/disable via config:

```yaml
output:
  store_results: true
  results_path: .ctp/analyses/
```
