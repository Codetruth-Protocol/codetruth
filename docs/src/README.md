# CodeTruth Protocol

> **Open Standard for AI Code Intelligence & Drift Monitoring**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)

## What is CodeTruth?

**CodeTruth Protocol (CTP)** is an open standard that enables:

1. **Machine-readable explanation** of code intent and behavior
2. **Detection of drift** between declared intent and actual implementation
3. **Automated product specification** generation from codebase analysis
4. **Policy-based governance** for AI-generated code
5. **Audit-ready artifacts** for regulatory compliance
6. **Interoperability** across all AI code generation tools

## Why CodeTruth?

As AI-generated code becomes ubiquitous, organizations face a critical challenge: **How do you trust code you didn't write?**

CTP solves this by providing:

- **Transparency**: Every piece of code gets an explanation graph describing what it does and why
- **Verification**: Automated drift detection catches when code doesn't match its documentation
- **Governance**: Policy evaluation ensures AI-generated code meets organizational standards
- **Auditability**: Complete history tracking for regulatory compliance

## Quick Example

```bash
# Initialize CTP in your repository
ctp init

# Analyze a file
ctp analyze src/payments/handler.py

# Output:
# ✓ src/payments/handler.py [python]
#   Intent: Process payment transactions with retry logic
#   Behavior: Performs 3 function(s), database operations, network calls
#   Drift: NONE (confidence: 92%)
```

## Protocol Design Principles

CTP follows the **Unix Philosophy**:

- **Do One Thing Well**: CTP explains code intent and detects drift
- **Compose with Other Tools**: Output feeds into linters, reviewers, monitors
- **Text-Based**: JSON over HTTP, Git-friendly formats
- **Silence is Golden**: Don't output unless there's something meaningful

## Getting Started

### Installation

```bash
# Install CLI (single binary, no dependencies)
curl -fsSL https://codetruth.dev/install.sh | sh

# Or with Cargo
cargo install ctp-cli
```

Ready to try CodeTruth? Head to the [Getting Started](./guide/getting-started.md) guide.

## Key Features

### Product Specification Generation
Automatically generates comprehensive product metadata from your codebase:
- **Smart Discovery**: Analyzes code to identify core functionalities
- **Evidence-Based Confidence**: Scores accuracy based on patterns and structure
- **Multi-Language Support**: Rust, JavaScript, Python, Go, Java, TypeScript

### Drift Detection
- **Intent vs. Behavior**: Compares documented intent with actual implementation
- **Real-time Monitoring**: Catches drift as it happens
- **Severity Assessment**: Prioritizes critical drift issues

### Policy Governance
- **Custom Rules**: Define organizational coding standards
- **Automated Enforcement**: Check compliance across the entire codebase
- **CI/CD Integration**: Prevent drift from entering production

## Community

- **GitHub**: [github.com/codetruth/codetruth](https://github.com/codetruth/codetruth)
- **Discord**: [discord.gg/codetruth](https://discord.gg/codetruth)
- **Twitter**: [@codetruth_dev](https://twitter.com/codetruth_dev)

## License

CodeTruth Protocol is released under the [MIT License](https://opensource.org/licenses/MIT).

---

*CodeTruth Protocol - Because AI-generated code must be auditable.*
