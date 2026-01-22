# Explanation Graph Schema

The Explanation Graph is the core data structure of CTP. This document provides the complete JSON Schema specification.

## JSON Schema

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://codetruth.dev/schema/v1/explanation-graph.json",
  "title": "CTP Explanation Graph",
  "description": "CodeTruth Protocol Explanation Graph v1.0",
  "type": "object",
  "required": ["ctp_version", "explanation_id", "module", "intent", "behavior", "drift"],
  
  "properties": {
    "ctp_version": {
      "type": "string",
      "pattern": "^\\d+\\.\\d+\\.\\d+$",
      "description": "Protocol version (semver)"
    },
    
    "explanation_id": {
      "type": "string",
      "description": "Unique identifier (content hash)"
    },
    
    "module": {
      "$ref": "#/$defs/Module"
    },
    
    "intent": {
      "$ref": "#/$defs/Intent"
    },
    
    "behavior": {
      "$ref": "#/$defs/Behavior"
    },
    
    "drift": {
      "$ref": "#/$defs/DriftAnalysis"
    },
    
    "policies": {
      "$ref": "#/$defs/PolicyResults"
    },
    
    "history": {
      "$ref": "#/$defs/History"
    },
    
    "metadata": {
      "$ref": "#/$defs/Metadata"
    }
  },
  
  "$defs": {
    "Module": {
      "type": "object",
      "required": ["name", "path", "language"],
      "properties": {
        "name": { "type": "string" },
        "path": { "type": "string" },
        "language": { "type": "string" },
        "lines_of_code": { "type": "integer", "minimum": 0 },
        "complexity_score": { "type": "number", "minimum": 0 }
      }
    },
    
    "Intent": {
      "type": "object",
      "required": ["declared_intent", "inferred_intent", "confidence"],
      "properties": {
        "declared_intent": { "type": "string" },
        "inferred_intent": { "type": "string" },
        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
        "business_context": { "type": "string" },
        "technical_rationale": { "type": "string" }
      }
    },
    
    "Behavior": {
      "type": "object",
      "required": ["actual_behavior"],
      "properties": {
        "actual_behavior": { "type": "string" },
        "entry_points": {
          "type": "array",
          "items": { "$ref": "#/$defs/EntryPoint" }
        },
        "exit_points": {
          "type": "array",
          "items": { "$ref": "#/$defs/ExitPoint" }
        },
        "side_effects": {
          "type": "array",
          "items": { "$ref": "#/$defs/SideEffect" }
        },
        "dependencies": {
          "type": "array",
          "items": { "$ref": "#/$defs/Dependency" }
        }
      }
    },
    
    "EntryPoint": {
      "type": "object",
      "properties": {
        "function": { "type": "string" },
        "parameters": { "type": "array", "items": { "type": "string" } },
        "preconditions": { "type": "array", "items": { "type": "string" } }
      }
    },
    
    "ExitPoint": {
      "type": "object",
      "properties": {
        "return_type": { "type": "string" },
        "possible_values": { "type": "array", "items": { "type": "string" } },
        "postconditions": { "type": "array", "items": { "type": "string" } }
      }
    },
    
    "SideEffect": {
      "type": "object",
      "required": ["effect_type", "description", "risk_level"],
      "properties": {
        "effect_type": {
          "type": "string",
          "enum": ["io", "network", "database", "state_mutation"]
        },
        "description": { "type": "string" },
        "risk_level": {
          "type": "string",
          "enum": ["LOW", "MEDIUM", "HIGH"]
        }
      }
    },
    
    "Dependency": {
      "type": "object",
      "properties": {
        "module": { "type": "string" },
        "reason": { "type": "string" },
        "coupling_type": {
          "type": "string",
          "enum": ["tight", "loose"]
        }
      }
    },
    
    "DriftAnalysis": {
      "type": "object",
      "required": ["drift_detected", "drift_severity"],
      "properties": {
        "drift_detected": { "type": "boolean" },
        "drift_severity": {
          "type": "string",
          "enum": ["NONE", "LOW", "MEDIUM", "HIGH", "CRITICAL"]
        },
        "drift_details": {
          "type": "array",
          "items": { "$ref": "#/$defs/DriftDetail" }
        }
      }
    },
    
    "DriftDetail": {
      "type": "object",
      "required": ["drift_type", "expected", "actual"],
      "properties": {
        "drift_type": {
          "type": "string",
          "enum": ["INTENT", "POLICY", "ASSUMPTION", "IMPLEMENTATION"]
        },
        "expected": { "type": "string" },
        "actual": { "type": "string" },
        "location": { "$ref": "#/$defs/Location" },
        "impact": { "$ref": "#/$defs/Impact" },
        "remediation": { "type": "string" }
      }
    },
    
    "Location": {
      "type": "object",
      "properties": {
        "file": { "type": "string" },
        "line_start": { "type": "integer", "minimum": 1 },
        "line_end": { "type": "integer", "minimum": 1 }
      }
    },
    
    "Impact": {
      "type": "object",
      "properties": {
        "functional": { "type": "string" },
        "security": { "type": "string" },
        "performance": { "type": "string" },
        "maintainability": { "type": "string" }
      }
    },
    
    "PolicyResults": {
      "type": "object",
      "properties": {
        "evaluated_at": { "type": "string", "format": "date-time" },
        "policy_results": {
          "type": "array",
          "items": { "$ref": "#/$defs/PolicyResult" }
        }
      }
    },
    
    "PolicyResult": {
      "type": "object",
      "properties": {
        "policy_id": { "type": "string" },
        "policy_name": { "type": "string" },
        "status": {
          "type": "string",
          "enum": ["PASS", "FAIL", "WARNING", "SKIP"]
        },
        "violations": {
          "type": "array",
          "items": { "$ref": "#/$defs/Violation" }
        }
      }
    },
    
    "Violation": {
      "type": "object",
      "properties": {
        "rule": { "type": "string" },
        "severity": {
          "type": "string",
          "enum": ["INFO", "WARNING", "ERROR", "CRITICAL"]
        },
        "message": { "type": "string" },
        "location": { "$ref": "#/$defs/Location" },
        "evidence": { "type": "string" }
      }
    },
    
    "History": {
      "type": "object",
      "properties": {
        "previous_versions": {
          "type": "array",
          "items": { "$ref": "#/$defs/PreviousVersion" }
        },
        "evolution": { "$ref": "#/$defs/Evolution" }
      }
    },
    
    "PreviousVersion": {
      "type": "object",
      "properties": {
        "version_id": { "type": "string" },
        "analyzed_at": { "type": "string", "format": "date-time" },
        "commit_hash": { "type": "string" },
        "drift_from_previous": { "type": "string" }
      }
    },
    
    "Evolution": {
      "type": "object",
      "properties": {
        "created_at": { "type": "string", "format": "date-time" },
        "last_modified": { "type": "string", "format": "date-time" },
        "modification_count": { "type": "integer", "minimum": 0 },
        "stability_score": { "type": "number", "minimum": 0, "maximum": 1 }
      }
    },
    
    "Metadata": {
      "type": "object",
      "properties": {
        "generated_at": { "type": "string", "format": "date-time" },
        "generator": { "$ref": "#/$defs/Generator" },
        "extensions": { "type": "object" }
      }
    },
    
    "Generator": {
      "type": "object",
      "properties": {
        "name": { "type": "string" },
        "version": { "type": "string" },
        "llm_provider": { "type": "string" },
        "llm_model": { "type": "string" }
      }
    }
  }
}
```

## Minimal Schema

For lightweight use cases:

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://codetruth.dev/schema/v1/minimal-analysis.json",
  "title": "CTP Minimal Analysis",
  "type": "object",
  "required": ["ctp_version", "file_hash", "intent", "behavior", "drift", "confidence"],
  
  "properties": {
    "ctp_version": { "type": "string" },
    "file_hash": { "type": "string" },
    "intent": { "type": "string", "maxLength": 280 },
    "behavior": { "type": "string", "maxLength": 500 },
    "drift": {
      "type": "string",
      "enum": ["NONE", "LOW", "MEDIUM", "HIGH", "CRITICAL"]
    },
    "confidence": { "type": "number", "minimum": 0, "maximum": 1 }
  }
}
```

## Validation

### Rust

```rust
use ctp_core::ExplanationGraph;
use serde_json;

let json = r#"{ ... }"#;
let graph: ExplanationGraph = serde_json::from_str(json)?;
```

### TypeScript

```typescript
import { ExplanationGraph } from '@codetruth/sdk';
import Ajv from 'ajv';

const ajv = new Ajv();
const validate = ajv.compile(schema);
const valid = validate(data);
```

### Python

```python
from jsonschema import validate
import json

with open('schema.json') as f:
    schema = json.load(f)

validate(instance=data, schema=schema)
```
