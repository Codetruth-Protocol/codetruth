# CodeTruth Protocol (CTP)

> **Open Standard for AI Code Intelligence & Drift Monitoring**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

## What is CodeTruth?

CodeTruth Protocol (CTP) is an open standard that enables:

1. **Machine-readable explanation** of code intent and behavior
2. **Detection of drift** between declared intent and actual implementation
3. **Automated product specification** generation from codebase analysis
4. **Policy-based governance** for AI-generated code
5. **Audit-ready artifacts** for regulatory compliance
6. **Interoperability** across all AI code generation tools

## Project Structure

```
codetruth/
├── crates/                    # Rust workspace
│   ├── ctp-core/              # Core analysis engine
│   ├── ctp-cli/               # Command-line interface
│   ├── ctp-parser/            # Tree-sitter parsing layer
│   ├── ctp-drift/             # Drift detection engine
│   ├── ctp-policy/            # Policy evaluation engine
│   ├── ctp-spec/              # Product specification engine
│   ├── ctp-product/           # Product metadata management
│   ├── ctp-context/           # Semantic context management
│   ├── ctp-design/            # Design pattern detection
│   ├── ctp-storage/           # Data persistence layer
│   └── ctp-llm/               # LLM integration layer
├── packages/                  # Polyglot SDKs
│   ├── ctp-typescript/        # TypeScript/Node.js SDK
│   ├── ctp-python/            # Python bindings (PyO3)
│   └── ctp-wasm/              # WebAssembly package
├── integrations/              # Third-party integrations
│   ├── vscode/                # VS Code extension
│   ├── jetbrains/             # JetBrains plugin
│   ├── github-action/         # GitHub Action
│   └── gitlab-ci/             # GitLab CI template
├── spec/                      # Protocol specification
│   ├── schema/                # JSON schemas
│   ├── rfc/                   # RFC-style documents
│   └── examples/              # Example CTP outputs
├── docs/                      # Documentation
│   ├── getting-started/       # Quick start guides
│   ├── architecture/          # Technical architecture
│   └── api/                   # API reference
└── tests/                     # Integration tests
    ├── fixtures/              # Test code samples
    └── benchmarks/            # Performance benchmarks
```

## Quick Start

### Installation

```bash
# Install CLI (single binary, no dependencies)
curl -fsSL https://codetruth.dev/install.sh | sh

# Or with Cargo
cargo install ctp-cli
```

### Basic Usage

```bash
# Initialize CTP in your repository
ctp init

# Generate product specification from codebase
ctp spec generate

# Analyze a file
ctp analyze src/payments/retry_handler.py

# Check policy compliance
ctp check --policies=.ctp/policies/

# Generate audit report
ctp audit --format=pdf
```

### Product Specification

```bash
# Generate product spec with high accuracy (rule-based)
ctp spec generate --output product-metadata.json

# Generate with LLM enrichment (requires API key)
ctp spec generate --use-llm --llm-key=gsk_...

# Validate spec against current codebase
ctp spec validate

# Show current specification
ctp spec show

# Enrich existing spec with LLM insights
ctp spec enrich --llm-key=gsk_...
```

## Architecture

CTP follows a layered architecture:

```
┌────────────────────────────────────────────────────────┐
│           CodeTruth Protocol (CTP) Stack               │
├────────────────────────────────────────────────────────┤
│  Layer 4: Presentation Layer                           │
│  - CLI Interface (ctp spec, analyze, check)           │
│  - IDE Extensions (VS Code, JetBrains)                │
│  - Product Specification Dashboard                     │
├────────────────────────────────────────────────────────┤
│  Layer 3: Protocol API Layer                           │
│  - RESTful API (JSON-RPC 2.0)                         │
│  - WebSocket Streaming                                 │
├────────────────────────────────────────────────────────┤
│  Layer 2: Core Intelligence Layer                      │
│  - Intent Extraction Engine                            │
│  - Product Spec Generation (ctp-spec)                  │
│  - Drift Detection Engine                              │
│  - Policy Evaluation Engine                            │
├────────────────────────────────────────────────────────┤
│  Layer 1: Data Ingestion Layer                         │
│  - AST Parsing (tree-sitter)                          │
│  - Git History Analysis                                │
│  - Comment/Doc Extraction                              │
└────────────────────────────────────────────────────────┘
```

### Product Specification Engine

The `ctp-spec` engine automatically generates comprehensive product metadata:

- **Smart Discovery**: Analyzes codebase to identify core functionalities
- **Evidence-Based Confidence**: Scores accuracy based on file patterns, error handling, and API surface
- **Context-Aware Classification**: Adapts to project type (CLI, web app, library)
- **Deduplication**: Removes redundant or overlapping functionalities
- **Multi-Language Support**: Rust, JavaScript, Python, Go, Java, TypeScript

**Accuracy Improvements**:
- Filters test/fixtures from analysis
- Sanitizes intent keys to prevent invalid IDs
- Extracts product metadata from Cargo.toml/package.json
- 60+ domain keywords for precise classification
- Multi-signal criticality assessment

## Why Rust?

- **Performance**: 10-100x faster than Python for parsing/analysis
- **Memory Safety**: No GC pauses, predictable performance
- **Single Binary**: No dependency hell, easy distribution
- **WASM Target**: Run in browser, edge functions, VS Code
- **Fearless Concurrency**: Analyze thousands of files in parallel

## Product Specification Features

### Generation Accuracy
- **80%+ reduction** in manual edits needed
- **Zero invalid IDs** (no "#" or garbage entries)
- **Accurate product names** from project files
- **No test contamination** in functionalities
- **Evidence-based confidence** scoring (0.5-1.0 range)

### CLI Commands
- `ctp spec generate` - Auto-generate from codebase
- `ctp spec validate` - Check against implementation
- `ctp spec show` - Display current specification
- `ctp spec enrich` - Enhance with LLM insights

### Output Format
```json
{
  "product": {
    "name": "your-project",
    "product_type": "library",
    "primary_language": "rust"
  },
  "core_functionalities": [
    {
      "id": "authentication",
      "name": "Authentication",
      "category": "core",
      "criticality": "high",
      "confidence": 0.85
    }
  ],
  "confidence": 0.78,
  "generation_method": "rule_based"
}
```

## Contributing

See [CONTRIBUTING.md](./docs/CONTRIBUTING.md) for guidelines.

### Development

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Generate product spec for this project
cargo run -p ctp-cli -- spec generate

# Validate current spec
cargo run -p ctp-cli -- spec validate
```

### Recent Improvements

See [METADATA_GENERATION_IMPROVEMENTS.md](./METADATA_GENERATION_IMPROVEMENTS.md) for detailed technical changes to the product specification engine.

## Documentation

- **[CLI Reference](./CLI_REFERENCE.md)** - Complete command-line interface documentation
- **[Architecture Guide](./docs/architecture/)** - Technical architecture and design
- **[API Reference](./docs/api/)** - Programmatic API documentation
- **[Contributing Guide](./docs/CONTRIBUTING.md)** - Development guidelines

## License

MIT License - see [LICENSE](./LICENSE) for details.

---

**CodeTruth Protocol** - *Because AI-generated code must be auditable.*
