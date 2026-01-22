# API Reference

CTP provides multiple API interfaces for integration.

## REST API

Base URL: `https://api.codetruth.dev/v1`

### Authentication

```bash
curl -H "Authorization: Bearer $CTP_API_KEY" \
     https://api.codetruth.dev/v1/analyze
```

### Endpoints

#### POST /analyze

Analyze code files.

**Request:**
```json
{
  "files": [
    {
      "path": "src/handler.py",
      "content": "def hello(): ...",
      "language": "python"
    }
  ],
  "options": {
    "enable_llm": false,
    "min_drift_level": "low"
  }
}
```

**Response:**
```json
{
  "success": true,
  "results": [
    {
      "file": "src/handler.py",
      "explanation_graph": { ... }
    }
  ]
}
```

#### POST /check

Check policy compliance.

**Request:**
```json
{
  "files": [...],
  "policies": ["documentation-required"]
}
```

**Response:**
```json
{
  "success": true,
  "passed": false,
  "violations": [...]
}
```

#### POST /diff

Compare versions.

**Request:**
```json
{
  "base": { "ref": "main", "files": [...] },
  "head": { "ref": "feature", "files": [...] }
}
```

#### GET /health

Health check.

**Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0"
}
```

## WebSocket API

For real-time streaming analysis.

**Connect:**
```javascript
const ws = new WebSocket('wss://api.codetruth.dev/v1/ws');
```

**Send Request:**
```javascript
ws.send(JSON.stringify({
  type: 'analyze',
  payload: { files: [...] }
}));
```

**Receive Updates:**
```javascript
ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  if (msg.type === 'progress') {
    console.log(`Progress: ${msg.progress}%`);
  } else if (msg.type === 'result') {
    console.log('Result:', msg.payload);
  }
};
```

## TypeScript SDK

```bash
npm install @codetruth/sdk
```

### Usage

```typescript
import { CodeTruth } from '@codetruth/sdk';

const ctp = new CodeTruth({
  apiKey: process.env.CTP_API_KEY
});

// Analyze file
const result = await ctp.analyze({
  files: [{ path: 'src/handler.py', content: '...' }]
});

// Check policies
const check = await ctp.check({
  files: [...],
  policies: ['documentation-required']
});

// Stream analysis
for await (const update of ctp.analyzeStream({ files: [...] })) {
  console.log(update.progress);
}
```

### Types

```typescript
interface AnalyzeOptions {
  files: FileInput[];
  enableLlm?: boolean;
  minDriftLevel?: DriftLevel;
}

interface FileInput {
  path: string;
  content: string;
  language?: string;
}

interface ExplanationGraph {
  ctpVersion: string;
  explanationId: string;
  module: Module;
  intent: Intent;
  behavior: Behavior;
  drift: DriftAnalysis;
}

type DriftLevel = 'NONE' | 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
```

## Python SDK

```bash
pip install codetruth
```

### Usage

```python
from codetruth import CodeTruth

ctp = CodeTruth(api_key=os.environ['CTP_API_KEY'])

# Analyze file
result = ctp.analyze(
    files=[{'path': 'src/handler.py', 'content': '...'}]
)

# Analyze local file
result = ctp.analyze_file('src/handler.py')

# Check policies
check = ctp.check(
    files=[...],
    policies=['documentation-required']
)

# Stream analysis
for update in ctp.analyze_stream(files=[...]):
    print(f"Progress: {update.progress}%")
```

### Types

```python
from dataclasses import dataclass
from typing import List, Optional
from enum import Enum

class DriftLevel(Enum):
    NONE = "NONE"
    LOW = "LOW"
    MEDIUM = "MEDIUM"
    HIGH = "HIGH"
    CRITICAL = "CRITICAL"

@dataclass
class ExplanationGraph:
    ctp_version: str
    explanation_id: str
    module: Module
    intent: Intent
    behavior: Behavior
    drift: DriftAnalysis
```

## Rust Library

```toml
[dependencies]
ctp-core = "0.1"
```

### Usage

```rust
use ctp_core::{CodeTruthEngine, EngineConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let engine = CodeTruthEngine::new(EngineConfig::default());
    
    // Analyze file
    let graph = engine.analyze_file(Path::new("src/handler.py")).await?;
    
    // Analyze string
    let graph = engine.analyze_string(code, "python", "handler.py").await?;
    
    println!("Drift: {:?}", graph.drift.drift_severity);
    Ok(())
}
```

## Rate Limits

| Tier | Requests/min | Files/request |
|------|--------------|---------------|
| Free | 10 | 10 |
| Pro | 100 | 100 |
| Enterprise | 1000 | 1000 |

## Error Handling

All APIs return errors in this format:

```json
{
  "success": false,
  "error": {
    "code": "PARSE_ERROR",
    "message": "Failed to parse file",
    "details": { ... }
  }
}
```

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `PARSE_ERROR` | 400 | Invalid code |
| `INVALID_REQUEST` | 400 | Bad request |
| `UNAUTHORIZED` | 401 | Invalid API key |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |
