# FAQ

Frequently asked questions about CodeTruth Protocol.

## General

### What is CodeTruth Protocol?

CodeTruth Protocol (CTP) is an open standard for AI code intelligence and drift monitoring. It provides machine-readable explanations of code intent and behavior, detects drift between documentation and implementation, and enables policy-based governance.

### Why do I need CTP?

As AI-generated code becomes ubiquitous, organizations need to:
- Understand what AI-generated code does
- Verify it matches intended behavior
- Ensure compliance with policies
- Maintain audit trails

### How is CTP different from linters?

Linters check code style and common errors. CTP analyzes code **intent** and **behavior**, detecting semantic drift between what code claims to do and what it actually does.

### Is CTP open source?

Yes! CTP is released under the MIT license. The specification, reference implementation, and SDKs are all open source.

## Installation

### What platforms are supported?

- Linux (x86_64, ARM64)
- macOS (x86_64, ARM64)
- Windows (x86_64)
- WebAssembly (browser, edge)

### Do I need Rust installed?

No, pre-built binaries are available. Rust is only needed for building from source.

### How do I update CTP?

```bash
# If installed via curl
curl -fsSL https://codetruth.dev/install.sh | sh

# If installed via cargo
cargo install ctp-cli --force
```

## Usage

### What languages are supported?

- Python
- JavaScript
- TypeScript
- Rust
- Go
- Java

More languages coming soon.

### Does CTP require an internet connection?

No, CTP works entirely offline. LLM enhancement is optional and requires API access.

### How fast is CTP?

Typical performance:
- Single file: 10-50ms
- 1000 files: 10-30 seconds
- Memory: ~2MB per file

### Can I use CTP in CI/CD?

Yes! CTP provides:
- GitHub Action
- GitLab CI template
- Generic CLI for any CI system

## Drift Detection

### What is "drift"?

Drift is a mismatch between:
- Declared intent (documentation) and inferred intent
- Intent and actual implementation behavior
- Current code and previous versions

### What do drift levels mean?

| Level | Meaning |
|-------|---------|
| NONE | Perfect match |
| LOW | Minor discrepancies |
| MEDIUM | Notable differences |
| HIGH | Significant mismatch |
| CRITICAL | Severe drift |

### How accurate is drift detection?

Without LLM: ~80% accuracy on well-documented code
With LLM: ~90% accuracy

Accuracy depends on code documentation quality.

## LLM Integration

### Is LLM required?

No, CTP works without LLM using rule-based analysis. LLM enhancement is optional.

### Which LLM providers are supported?

- Anthropic Claude
- OpenAI GPT
- Ollama (local models)

### Is my code sent to external services?

Only if you enable LLM enhancement. By default, all analysis is local.

### Can I use local models?

Yes, via Ollama integration. Your code never leaves your machine.

## Policies

### What are policies?

Policies are rules that define code standards. CTP checks code against these rules and reports violations.

### Can I create custom policies?

Yes, using the Policy Definition Language (PDL) in YAML format.

### Are there built-in policies?

Yes, CTP includes policies for:
- Documentation requirements
- Error handling
- Security patterns
- And more

## Integration

### Does CTP work with my IDE?

VS Code and JetBrains plugins are available. LSP support enables integration with any LSP-compatible editor.

### Can I use CTP programmatically?

Yes, SDKs are available for:
- TypeScript/Node.js
- Python
- Rust (native)
- WebAssembly

## Troubleshooting

### CTP says "Unsupported language"

Ensure the language feature is enabled:
```bash
cargo install ctp-cli --features all-languages
```

### Analysis is slow

Try:
- Excluding large directories
- Reducing analyzed languages
- Using `--changed-files-only` in CI

### High false positive rate

Try:
- Increasing `min_drift_level`
- Improving code documentation
- Enabling LLM enhancement

## Contributing

### How can I contribute?

See our [Contributing Guide](./dev/contributing.md). We welcome:
- Bug reports
- Feature requests
- Code contributions
- Documentation improvements

### Where do I report bugs?

[GitHub Issues](https://github.com/codetruth/codetruth/issues)

### How do I request features?

[GitHub Discussions](https://github.com/codetruth/codetruth/discussions)

## Support

### Where can I get help?

- **Documentation**: You're here!
- **Discord**: [discord.gg/codetruth](https://discord.gg/codetruth)
- **GitHub**: Issues and Discussions

### Is commercial support available?

Enterprise support plans are available. Contact enterprise@codetruth.dev.
