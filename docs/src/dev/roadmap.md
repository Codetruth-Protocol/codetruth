# Roadmap

CodeTruth Protocol development roadmap.

## Current Status: v0.1.0 (Alpha)

Core functionality implemented:
- ✅ Basic AST parsing (Python, JavaScript, TypeScript)
- ✅ Intent extraction from comments/docstrings
- ✅ Rule-based behavior analysis
- ✅ Simple drift detection
- ✅ CLI interface
- ✅ Basic policy engine

## Q1 2026: v0.2.0 (Beta)

### Core Engine
- [ ] Enhanced tree-sitter integration
- [ ] Cyclomatic complexity calculation
- [ ] Dependency graph extraction
- [ ] Cross-file analysis

### Drift Detection
- [ ] Semantic similarity using embeddings
- [ ] Historical drift tracking
- [ ] Configurable thresholds

### Policy Engine
- [ ] Full PDL implementation
- [ ] Built-in policy library
- [ ] Policy validation tool

### LLM Integration
- [ ] Anthropic Claude integration
- [ ] OpenAI GPT integration
- [ ] Ollama local models
- [ ] Graceful fallback

## Q2 2026: v0.3.0 (Release Candidate)

### IDE Extensions
- [ ] VS Code extension
- [ ] JetBrains plugin
- [ ] LSP server

### CI/CD Integration
- [ ] GitHub Action
- [ ] GitLab CI template
- [ ] Generic webhook

### SDKs
- [ ] TypeScript SDK
- [ ] Python SDK
- [ ] WebAssembly package

## Q3 2026: v1.0.0 (Stable)

### Protocol Finalization
- [ ] JSON Schema v1.0
- [ ] RFC-style specification
- [ ] Backward compatibility guarantees

### Enterprise Features
- [ ] Team management
- [ ] SSO integration
- [ ] Audit logging
- [ ] Custom policies

### Performance
- [ ] Parallel analysis
- [ ] Incremental analysis
- [ ] Caching layer

## Q4 2026: v1.1.0

### Advanced Analysis
- [ ] Cross-repository analysis
- [ ] Dependency vulnerability detection
- [ ] Code evolution tracking

### Integrations
- [ ] Slack notifications
- [ ] Jira integration
- [ ] Custom webhooks

### Analytics
- [ ] Drift trends dashboard
- [ ] Team metrics
- [ ] Quality reports

## Future (2027+)

### Standards Track
- [ ] IETF RFC submission
- [ ] Linux Foundation project
- [ ] Industry adoption

### Advanced Features
- [ ] AI-powered remediation
- [ ] Automated documentation
- [ ] Code generation validation

## Contributing

See [Contributing Guide](./contributing.md) to help with roadmap items.

## Feature Requests

Submit feature requests via [GitHub Issues](https://github.com/codetruth/codetruth/issues).

## Versioning

CTP follows [Semantic Versioning](https://semver.org/):

- **Major**: Breaking protocol changes
- **Minor**: New features, backward compatible
- **Patch**: Bug fixes

## Release Schedule

- **Alpha**: Monthly releases
- **Beta**: Bi-weekly releases
- **Stable**: Quarterly releases
