# Testing

Guide for testing CodeTruth Protocol.

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Crate

```bash
cargo test -p ctp-core
cargo test -p ctp-parser
cargo test -p ctp-drift
cargo test -p ctp-policy
```

### Specific Test

```bash
cargo test test_drift_detection
cargo test -p ctp-core test_analyze_string
```

### With Output

```bash
cargo test -- --nocapture
```

## Test Categories

### Unit Tests

Located in `src/` files:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let engine = CodeTruthEngine::default();
        let hash = engine.hash_content("hello");
        assert!(hash.starts_with("sha256:"));
    }

    #[tokio::test]
    async fn test_analyze_string() {
        let engine = CodeTruthEngine::default();
        let result = engine.analyze_string("def foo(): pass", "python", "test.py").await;
        assert!(result.is_ok());
    }
}
```

### Integration Tests

Located in `tests/`:

```rust
// tests/integration_test.rs
use ctp_core::CodeTruthEngine;
use std::path::Path;

#[tokio::test]
async fn test_analyze_python_file() {
    let engine = CodeTruthEngine::default();
    let result = engine.analyze_file(Path::new("tests/fixtures/sample_python.py")).await;
    
    assert!(result.is_ok());
    let graph = result.unwrap();
    assert_eq!(graph.module.language, "python");
}

#[tokio::test]
async fn test_analyze_typescript_file() {
    let engine = CodeTruthEngine::default();
    let result = engine.analyze_file(Path::new("tests/fixtures/sample_typescript.ts")).await;
    
    assert!(result.is_ok());
}
```

### Test Fixtures

Located in `tests/fixtures/`:

```python
# tests/fixtures/sample_python.py
"""Sample Python module for testing."""

def factorial(n: int) -> int:
    """Calculate the factorial of a number.
    
    Args:
        n: The number to calculate factorial for.
        
    Returns:
        The factorial of n.
    """
    if n <= 1:
        return 1
    return n * factorial(n - 1)


class Calculator:
    """A simple calculator class."""
    
    def add(self, a: int, b: int) -> int:
        """Add two numbers."""
        return a + b
```

```typescript
// tests/fixtures/sample_typescript.ts
/**
 * Sample TypeScript module for testing.
 */

/**
 * Calculate the factorial of a number.
 * @param n - The number to calculate factorial for.
 * @returns The factorial of n.
 */
export function factorial(n: number): number {
    if (n <= 1) {
        return 1;
    }
    return n * factorial(n - 1);
}

/**
 * A simple calculator class.
 */
export class Calculator {
    /**
     * Add two numbers.
     */
    add(a: number, b: number): number {
        return a + b;
    }
}
```

## Benchmarks

Located in `benches/`:

```rust
// crates/ctp-core/benches/analysis.rs
use criterion::{criterion_group, criterion_main, Criterion};
use ctp_core::CodeTruthEngine;
use std::path::Path;

fn bench_analyze_file(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let engine = CodeTruthEngine::default();
    
    c.bench_function("analyze_python_file", |b| {
        b.to_async(&rt).iter(|| {
            engine.analyze_file(Path::new("tests/fixtures/sample_python.py"))
        });
    });
}

criterion_group!(benches, bench_analyze_file);
criterion_main!(benches);
```

Run benchmarks:

```bash
cargo bench
```

## Coverage

### Using Tarpaulin

```bash
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
# Open tarpaulin-report.html
```

### Using llvm-cov

```bash
cargo install cargo-llvm-cov
cargo llvm-cov --html
# Open target/llvm-cov/html/index.html
```

## Property-Based Testing

Using proptest:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_similarity_symmetric(a in "\\PC*", b in "\\PC*") {
        let detector = DriftDetector::default();
        let sim_ab = detector.calculate_similarity(&a, &b);
        let sim_ba = detector.calculate_similarity(&b, &a);
        prop_assert!((sim_ab - sim_ba).abs() < 0.001);
    }
}
```

## Mocking

Using mockall:

```rust
use mockall::automock;

#[automock]
trait LLMProvider {
    async fn infer_intent(&self, code: &str) -> Result<String>;
}

#[tokio::test]
async fn test_with_mock_llm() {
    let mut mock = MockLLMProvider::new();
    mock.expect_infer_intent()
        .returning(|_| Ok("Test intent".into()));
    
    // Use mock in tests
}
```

## CI Testing

### GitHub Actions

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --all-features
```

### Test Matrix

```yaml
jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        rust: [stable, beta]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@${{ matrix.rust }}
      - run: cargo test
```

## Writing Good Tests

### Test Naming

```rust
#[test]
fn test_drift_detection_returns_none_for_identical_intent() { }

#[test]
fn test_drift_detection_returns_critical_for_completely_different_intent() { }
```

### Test Structure (AAA)

```rust
#[test]
fn test_example() {
    // Arrange
    let detector = DriftDetector::default();
    let declared = "Calculate sum";
    let inferred = "Calculate sum";
    
    // Act
    let report = detector.detect_intent_drift(declared, inferred);
    
    // Assert
    assert_eq!(report.overall_severity, DriftSeverity::None);
}
```

### Test Edge Cases

```rust
#[test]
fn test_empty_input() {
    let detector = DriftDetector::default();
    let report = detector.detect_intent_drift("", "");
    assert_eq!(report.overall_severity, DriftSeverity::None);
}

#[test]
fn test_unicode_input() {
    let detector = DriftDetector::default();
    let report = detector.detect_intent_drift("计算总和", "计算总和");
    assert_eq!(report.overall_severity, DriftSeverity::None);
}
```
