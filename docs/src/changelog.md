# Changelog

All notable changes to CodeTruth Protocol.

## [Unreleased]

### Added
- Initial documentation website structure
- mdBook-based documentation

## [0.1.0] - 2026-01-16

### Added
- Core analysis engine (`ctp-core`)
- Command-line interface (`ctp-cli`)
- Tree-sitter parsing layer (`ctp-parser`)
  - Python support
  - JavaScript support
  - TypeScript support
- Drift detection engine (`ctp-drift`)
  - Intent drift detection
  - Behavior drift detection
  - Version drift detection
- Policy evaluation engine (`ctp-policy`)
  - YAML-based Policy Definition Language
  - Scope filtering
  - Violation reporting
- LLM integration layer (`ctp-llm`)
  - Anthropic Claude support
  - OpenAI GPT support
  - Ollama local model support
- CLI commands:
  - `ctp init` - Initialize repository
  - `ctp analyze` - Analyze files
  - `ctp explain` - Detailed explanation
  - `ctp check` - Policy compliance
  - `ctp ci-check` - CI/CD integration
- Explanation Graph schema v1.0
- Minimal Analysis format
- JSON and YAML output formats

### Technical
- Rust workspace structure
- Async/await throughout
- Feature flags for language support
- Comprehensive error types

## [0.0.1] - 2026-01-01

### Added
- Initial project structure
- Basic README
- MIT License

---

## Version History

| Version | Date | Status |
|---------|------|--------|
| 0.1.0 | 2026-01-16 | Alpha |
| 0.0.1 | 2026-01-01 | Initial |

## Upgrade Guide

### 0.0.1 → 0.1.0

First public release. No migration needed.

## Release Notes Format

Each release includes:
- **Added**: New features
- **Changed**: Changes to existing features
- **Deprecated**: Features to be removed
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security fixes

Following [Keep a Changelog](https://keepachangelog.com/) format.
