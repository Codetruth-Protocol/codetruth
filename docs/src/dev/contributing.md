# Contributing

Thank you for your interest in contributing to CodeTruth Protocol!

## Code of Conduct

Please read and follow our [Code of Conduct](https://github.com/codetruth/codetruth/blob/main/CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites

- Rust 1.75+
- Node.js 18+ (for TypeScript SDK)
- Python 3.10+ (for Python SDK)
- Git

### Clone the Repository

```bash
git clone https://github.com/codetruth/codetruth.git
cd codetruth
```

### Build

```bash
cargo build
```

### Run Tests

```bash
cargo test
```

## Project Structure

```
codetruth/
├── crates/                 # Rust workspace
│   ├── ctp-core/           # Core analysis engine
│   ├── ctp-cli/            # Command-line interface
│   ├── ctp-parser/         # Tree-sitter parsing
│   ├── ctp-drift/          # Drift detection
│   ├── ctp-policy/         # Policy evaluation
│   └── ctp-llm/            # LLM integration
├── packages/               # Polyglot SDKs
│   ├── ctp-typescript/     # TypeScript SDK
│   └── ctp-python/         # Python SDK
├── integrations/           # IDE/CI integrations
├── spec/                   # Protocol specification
├── docs/                   # Documentation
└── tests/                  # Integration tests
```

## Development Workflow

### 1. Create an Issue

Before starting work, create or find an issue describing the change.

### 2. Fork and Branch

```bash
git checkout -b feature/your-feature
```

### 3. Make Changes

- Follow the coding style
- Add tests for new functionality
- Update documentation

### 4. Test

```bash
# Run all tests
cargo test

# Run specific crate tests
cargo test -p ctp-core

# Run with coverage
cargo tarpaulin
```

### 5. Commit

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add new drift detection algorithm
fix: handle empty files in parser
docs: update API reference
test: add tests for policy engine
```

### 6. Submit PR

- Fill out the PR template
- Link to the issue
- Wait for review

## Coding Guidelines

### Rust

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting
- Use `clippy` for linting
- Document public APIs

```rust
/// Analyze a file and generate an explanation graph.
///
/// # Arguments
///
/// * `path` - Path to the file to analyze
///
/// # Returns
///
/// An `ExplanationGraph` containing the analysis results.
///
/// # Errors
///
/// Returns `CTPError` if the file cannot be read or parsed.
pub async fn analyze_file(&self, path: &Path) -> CTPResult<ExplanationGraph> {
    // ...
}
```

### TypeScript

- Use TypeScript strict mode
- Follow ESLint rules
- Document with JSDoc

### Python

- Follow PEP 8
- Use type hints
- Document with docstrings

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_detection() {
        let detector = DriftDetector::default();
        let report = detector.detect_intent_drift("add numbers", "add numbers");
        assert_eq!(report.overall_severity, DriftSeverity::None);
    }
}
```

### Integration Tests

```rust
// tests/integration_test.rs
#[tokio::test]
async fn test_full_analysis() {
    let engine = CodeTruthEngine::default();
    let result = engine.analyze_file(Path::new("tests/fixtures/sample.py")).await;
    assert!(result.is_ok());
}
```

### Test Fixtures

Add test files to `tests/fixtures/`:

```
tests/fixtures/
├── sample_python.py
├── sample_typescript.ts
└── sample_rust.rs
```

## Documentation

### Code Documentation

- Document all public APIs
- Include examples in doc comments
- Keep docs up to date with code

### User Documentation

- Update `docs/` for user-facing changes
- Add examples and tutorials
- Test documentation builds

```bash
cd docs
mdbook build
mdbook serve
```

## Release Process

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create release PR
4. After merge, tag release
5. CI publishes packages

## Getting Help

- **Discord**: [discord.gg/codetruth](https://discord.gg/codetruth)
- **GitHub Discussions**: For questions and ideas
- **GitHub Issues**: For bugs and features

## Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md`
- Release notes
- Project website

Thank you for contributing to CodeTruth Protocol!
