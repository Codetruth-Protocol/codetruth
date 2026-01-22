# Architecture

CodeTruth Protocol follows a layered architecture designed for performance, extensibility, and interoperability.

## Protocol Stack

```
┌────────────────────────────────────────────────────────┐
│           CodeTruth Protocol (CTP) Stack               │
├────────────────────────────────────────────────────────┤
│  Layer 4: Presentation Layer                           │
│  - CLI Interface (ctp-cli)                             │
│  - IDE Extensions (VS Code, JetBrains)                │
│  - Web Dashboard                                       │
│  - CI/CD Integrations                                  │
├────────────────────────────────────────────────────────┤
│  Layer 3: Protocol API Layer                           │
│  - JSON-RPC 2.0 over HTTP/WebSocket                   │
│  - REST API for simple queries                         │
│  - Language Server Protocol (LSP)                      │
├────────────────────────────────────────────────────────┤
│  Layer 2: Core Intelligence Layer                      │
│  - Intent Extraction Engine (ctp-core)                │
│  - Drift Detection Engine (ctp-drift)                 │
│  - Policy Evaluation Engine (ctp-policy)              │
│  - LLM Integration (ctp-llm)                          │
├────────────────────────────────────────────────────────┤
│  Layer 1: Data Ingestion Layer                         │
│  - AST Parsing (ctp-parser, tree-sitter)             │
│  - Git History Analysis                                │
│  - Comment/Doc Extraction                              │
│  - Metadata Collection                                 │
└────────────────────────────────────────────────────────┘
```

## Crate Structure

```
crates/
├── ctp-core/      # Main analysis engine
├── ctp-cli/       # Command-line interface
├── ctp-parser/    # Tree-sitter parsing layer
├── ctp-drift/     # Drift detection algorithms
├── ctp-policy/    # Policy Definition Language engine
└── ctp-llm/       # LLM provider integrations
```

### ctp-core

The orchestration layer that coordinates all analysis:

- Manages analysis pipeline
- Coordinates parser, drift, and policy engines
- Generates explanation graphs
- Handles configuration

### ctp-parser

Multi-language AST parsing using tree-sitter:

- Supports Python, JavaScript, TypeScript, Rust, Go, Java
- Extracts functions, classes, imports, comments
- Language-agnostic interface

### ctp-drift

Drift detection algorithms:

- Intent drift (declared vs inferred)
- Behavior drift (intent vs implementation)
- Version drift (historical comparison)
- Configurable thresholds

### ctp-policy

Policy Definition Language (PDL) engine:

- YAML-based policy definitions
- Pattern matching rules
- Scope filtering
- Violation reporting

### ctp-llm

LLM integration for enhanced analysis:

- Anthropic Claude support
- OpenAI GPT support
- Ollama (local models)
- Graceful fallback to rule-based

## Data Flow

```
Source Code
    │
    ▼
┌─────────────┐
│  ctp-parser │ ──► AST + Comments + Metadata
└─────────────┘
    │
    ▼
┌─────────────┐
│  ctp-core   │ ──► Intent Extraction
│             │ ──► Behavior Analysis
└─────────────┘
    │
    ├──────────────────┐
    ▼                  ▼
┌─────────────┐  ┌─────────────┐
│  ctp-drift  │  │ ctp-policy  │
└─────────────┘  └─────────────┘
    │                  │
    └────────┬─────────┘
             ▼
    ┌─────────────────┐
    │ Explanation     │
    │ Graph           │
    └─────────────────┘
```

## Design Decisions

### Why Rust?

| Benefit | Impact |
|---------|--------|
| **Performance** | 10-100x faster than Python for parsing |
| **Memory Safety** | No GC pauses, predictable performance |
| **Single Binary** | Easy distribution, no dependency hell |
| **WASM Target** | Browser and edge deployment |
| **Fearless Concurrency** | Parallel file analysis |

### Why Tree-sitter?

- Industry standard for IDE tooling
- 40+ language support
- Incremental parsing for performance
- Used by GitHub, VS Code, Neovim
- Consistent API across languages

### Why Modular LLM?

- Optional enhancement (not required)
- Multiple provider support
- Local model option (privacy)
- Graceful degradation to rule-based

### Protocol-First Design

- JSON Schema for all data structures
- RFC-style specification
- Versioned protocol messages
- Extension mechanism for custom data

## Performance Characteristics

| Operation | Target | Typical |
|-----------|--------|---------|
| Parse 1K LOC file | <50ms | ~15ms |
| Analyze 10K files | <30s | ~15s |
| Memory per file | <10MB | ~2MB |
| Binary size | <20MB | ~5MB |

## Extensibility

### Custom Parsers

Implement the `Parser` trait for new languages:

```rust
pub trait Parser {
    fn parse(&mut self, source: &str) -> Result<ParsedAST>;
    fn language(&self) -> &str;
}
```

### Custom Policies

Define policies in YAML PDL format.

### Custom LLM Providers

Implement the `LLMProvider` trait:

```rust
pub trait LLMProvider {
    async fn infer_intent(&self, code: &str, context: &str) 
        -> Result<IntentInference>;
}
```

## Deployment Options

### CLI Binary

Single binary, no dependencies:

```bash
curl -fsSL https://codetruth.dev/install.sh | sh
```

### Docker

```bash
docker run codetruth/ctp analyze /workspace
```

### WebAssembly

Run in browser or edge functions:

```javascript
import { analyze } from '@codetruth/wasm';
const result = await analyze(code, 'python');
```

### Library

Embed in Rust applications:

```rust
use ctp_core::CodeTruthEngine;
let engine = CodeTruthEngine::default();
let graph = engine.analyze_file(path).await?;
```
