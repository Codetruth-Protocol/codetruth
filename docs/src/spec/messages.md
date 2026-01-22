# Protocol Messages

CTP uses JSON-based messages for all communication. This document specifies the message formats.

## Message Envelope

All CTP messages follow this envelope:

```json
{
  "ctp_version": "1.0.0",
  "message_type": "analyze_request",
  "timestamp": "2026-01-16T20:00:00Z",
  "request_id": "uuid-v4",
  "payload": { ... }
}
```

## Request Messages

### AnalyzeRequest

Request to analyze code files.

```json
{
  "message_type": "analyze_request",
  "payload": {
    "files": [
      {
        "path": "src/handler.py",
        "content": "def hello(): ...",
        "language": "python",
        "metadata": {
          "git_commit": "abc123",
          "author": "developer@example.com"
        }
      }
    ],
    "context": {
      "repository": "github.com/org/repo",
      "branch": "main",
      "commit": "abc123def456",
      "pr_number": 42
    },
    "options": {
      "enable_llm": false,
      "min_drift_level": "low",
      "policies": ["documentation-required"]
    }
  }
}
```

### CheckRequest

Request to check policy compliance.

```json
{
  "message_type": "check_request",
  "payload": {
    "files": [...],
    "policies": [
      {
        "id": "documentation-required",
        "content": "... YAML policy ..."
      }
    ],
    "options": {
      "fail_on_violation": true,
      "min_severity": "warning"
    }
  }
}
```

### DiffRequest

Request to compare versions.

```json
{
  "message_type": "diff_request",
  "payload": {
    "base": {
      "ref": "main",
      "files": [...]
    },
    "head": {
      "ref": "feature-branch",
      "files": [...]
    }
  }
}
```

## Response Messages

### AnalyzeResponse

Response containing explanation graphs.

```json
{
  "message_type": "analyze_response",
  "payload": {
    "success": true,
    "results": [
      {
        "file": "src/handler.py",
        "explanation_graph": { ... }
      }
    ],
    "summary": {
      "files_analyzed": 10,
      "drift_detected": 2,
      "policy_violations": 0
    }
  }
}
```

### CheckResponse

Response with policy check results.

```json
{
  "message_type": "check_response",
  "payload": {
    "success": true,
    "passed": false,
    "results": [
      {
        "policy_id": "documentation-required",
        "policy_name": "Documentation Required",
        "status": "FAIL",
        "violations": [
          {
            "file": "src/handler.py",
            "line": 45,
            "rule": "docstring-required",
            "message": "Function missing docstring",
            "severity": "WARNING"
          }
        ]
      }
    ]
  }
}
```

### DiffResponse

Response with version comparison.

```json
{
  "message_type": "diff_response",
  "payload": {
    "drift_detected": true,
    "changes": [
      {
        "file": "src/handler.py",
        "change_type": "modified",
        "intent_drift": "LOW",
        "behavior_drift": "MEDIUM",
        "details": [...]
      }
    ]
  }
}
```

### ErrorResponse

Error response for failed requests.

```json
{
  "message_type": "error_response",
  "payload": {
    "success": false,
    "error": {
      "code": "PARSE_ERROR",
      "message": "Failed to parse src/handler.py",
      "details": {
        "line": 42,
        "column": 10,
        "expected": ")",
        "found": "EOF"
      }
    }
  }
}
```

## Error Codes

| Code | Description |
|------|-------------|
| `PARSE_ERROR` | Failed to parse source code |
| `UNSUPPORTED_LANGUAGE` | Language not supported |
| `POLICY_ERROR` | Invalid policy definition |
| `LLM_ERROR` | LLM provider error |
| `IO_ERROR` | File system error |
| `CONFIG_ERROR` | Configuration error |
| `RATE_LIMITED` | Too many requests |
| `INTERNAL_ERROR` | Internal server error |

## Streaming Messages

For long-running operations, CTP supports streaming:

### ProgressMessage

```json
{
  "message_type": "progress",
  "payload": {
    "request_id": "uuid-v4",
    "progress": 0.45,
    "current_file": "src/payments/handler.py",
    "files_completed": 45,
    "files_total": 100
  }
}
```

### PartialResult

```json
{
  "message_type": "partial_result",
  "payload": {
    "request_id": "uuid-v4",
    "file": "src/handler.py",
    "explanation_graph": { ... }
  }
}
```

## WebSocket Protocol

For real-time communication:

```javascript
const ws = new WebSocket('wss://api.codetruth.dev/v1/ws');

ws.send(JSON.stringify({
  ctp_version: "1.0.0",
  message_type: "analyze_request",
  payload: { ... }
}));

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  if (message.message_type === 'progress') {
    updateProgress(message.payload.progress);
  } else if (message.message_type === 'analyze_response') {
    handleResult(message.payload);
  }
};
```

## HTTP API

REST endpoints for simple operations:

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/v1/analyze` | Analyze code |
| POST | `/v1/check` | Check policies |
| POST | `/v1/diff` | Compare versions |
| GET | `/v1/health` | Health check |
| GET | `/v1/version` | Version info |
