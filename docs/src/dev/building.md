# Building from Source

Complete guide for building CodeTruth from source.

## Prerequisites

### Required

- **Rust 1.75+**: Install via [rustup](https://rustup.rs/)
- **Git**: For cloning the repository
- **C compiler**: For tree-sitter (gcc/clang/MSVC)

### Optional

- **Node.js 18+**: For TypeScript SDK
- **Python 3.10+**: For Python SDK
- **wasm-pack**: For WebAssembly build

## Clone Repository

```bash
git clone https://github.com/codetruth/codetruth.git
cd codetruth
```

## Build CLI

### Debug Build

```bash
cargo build
```

Binary at `target/debug/ctp`

### Release Build

```bash
cargo build --release
```

Binary at `target/release/ctp`

### With All Languages

```bash
cargo build --release --features all-languages
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `python` | Python language support (default) |
| `javascript` | JavaScript support (default) |
| `typescript` | TypeScript support (default) |
| `rust-lang` | Rust language support |
| `go` | Go language support |
| `java` | Java language support |
| `llm` | LLM enhancement support |
| `all-languages` | All language parsers |

Example:

```bash
cargo build --release --features "python,rust-lang,llm"
```

## Install Locally

```bash
cargo install --path crates/ctp-cli
```

Or with features:

```bash
cargo install --path crates/ctp-cli --features all-languages
```

## Build Individual Crates

```bash
# Core library
cargo build -p ctp-core

# Parser
cargo build -p ctp-parser

# Drift detection
cargo build -p ctp-drift

# Policy engine
cargo build -p ctp-policy

# LLM integration
cargo build -p ctp-llm --features llm
```

## Cross-Compilation

### Linux (from macOS/Windows)

```bash
rustup target add x86_64-unknown-linux-gnu
cargo build --release --target x86_64-unknown-linux-gnu
```

### Windows (from macOS/Linux)

```bash
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

### macOS (from Linux/Windows)

Requires macOS SDK. Use cross:

```bash
cargo install cross
cross build --release --target x86_64-apple-darwin
```

## WebAssembly Build

```bash
# Install wasm-pack
cargo install wasm-pack

# Build WASM package
cd packages/ctp-wasm
wasm-pack build --target web
```

## Build SDKs

### TypeScript SDK

```bash
cd packages/ctp-typescript
npm install
npm run build
```

### Python SDK

```bash
cd packages/ctp-python
pip install maturin
maturin build --release
```

## Build Documentation

```bash
cd docs
mdbook build
```

Serve locally:

```bash
mdbook serve
# Open http://localhost:3000
```

## Docker Build

```bash
docker build -t codetruth/ctp:latest .
```

Multi-platform:

```bash
docker buildx build --platform linux/amd64,linux/arm64 -t codetruth/ctp:latest .
```

## Troubleshooting

### Tree-sitter Build Errors

Ensure C compiler is installed:

```bash
# Ubuntu/Debian
sudo apt install build-essential

# macOS
xcode-select --install

# Windows
# Install Visual Studio Build Tools
```

### Linking Errors on Linux

Install required libraries:

```bash
sudo apt install pkg-config libssl-dev
```

### Out of Memory

Reduce parallelism:

```bash
cargo build --release -j 2
```

### Slow Builds

Use sccache:

```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
cargo build --release
```

## Verification

```bash
# Run tests
cargo test

# Run clippy
cargo clippy

# Check formatting
cargo fmt --check

# Run benchmarks
cargo bench
```
