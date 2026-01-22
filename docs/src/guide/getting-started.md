# Getting Started

This guide will help you get CodeTruth Protocol up and running in your project.

## Prerequisites

- **Rust 1.75+** (for building from source)
- **Git** (for version control integration)
- A supported programming language project (Python, JavaScript, TypeScript, Rust, Go, or Java)

## Installation Options

### Option 1: Pre-built Binary (Recommended)

**macOS/Linux:**
```bash
curl -fsSL https://codetruth.dev/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://codetruth.dev/install.ps1 | iex
```

### Option 2: Cargo Install

```bash
cargo install ctp-cli
```

### Option 3: Build from Source

```bash
git clone https://github.com/codetruth/codetruth.git
cd codetruth
cargo build --release
```

The binary will be at `target/release/ctp`.

## Verify Installation

```bash
ctp --version
# CodeTruth Protocol CLI v0.1.0
```

## Initialize Your Project

Navigate to your project directory and initialize CTP:

```bash
cd your-project
ctp init
```

This creates a `.ctp/` directory with:
- `config.yaml` - Configuration settings
- `policies/` - Policy definition files
- `analyses/` - Stored analysis results

## Your First Analysis

Analyze a single file:

```bash
ctp analyze src/main.py
```

Or analyze an entire directory:

```bash
ctp analyze src/
```

## Understanding the Output

```
✓ src/payments/handler.py [python]
  Intent: Process payment transactions with retry logic
  Behavior: Performs 3 function(s), database operations, network calls
  Drift: NONE (confidence: 92%)
```

- **✓** indicates no significant drift detected
- **Intent** shows what the code claims to do (from documentation)
- **Behavior** describes what the code actually does
- **Drift** indicates any mismatch between intent and behavior
- **Confidence** shows how certain the analysis is

## Next Steps

- [Installation Details](./installation.md) - Platform-specific installation guides
- [Quick Start](./quick-start.md) - 5-minute tutorial
- [Configuration](./configuration.md) - Customize CTP for your project
- [Core Concepts](./concepts.md) - Understand how CTP works
