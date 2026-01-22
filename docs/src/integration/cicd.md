# CI/CD Integration

Integrate CodeTruth into your continuous integration pipeline.

## Overview

CTP can be integrated into CI/CD to:
- Analyze code changes in pull requests
- Check policy compliance before merge
- Detect drift between versions
- Generate audit reports

## Quick Start

### GitHub Actions

```yaml
name: CodeTruth Analysis

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Run CodeTruth
        uses: codetruth/ctp-action@v1
        with:
          analyze: true
          check-policies: true
          fail-on-violation: false
```

### GitLab CI

```yaml
codetruth:
  image: codetruth/ctp:latest
  script:
    - ctp analyze src/
    - ctp check --fail-on-violation
  only:
    - merge_requests
```

### Generic CI

```bash
# Install CTP
curl -fsSL https://codetruth.dev/install.sh | sh

# Run analysis
ctp ci-check --min-drift-level high --fail-on-violation
```

## Configuration

### Environment Variables

| Variable | Description |
|----------|-------------|
| `CTP_API_KEY` | API key for LLM features |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `CTP_CONFIG` | Path to config file |

### CI-Specific Config

Create `.ctp/ci-config.yaml`:

```yaml
version: "1.0"

ci:
  # Only analyze changed files
  changed_files_only: true
  
  # Compare with base branch
  compare_with_base: true
  
  # Minimum drift level to fail
  min_drift_level: high
  
  # Fail on policy violations
  fail_on_violation: true
  
  # Post results as PR comment
  post_comment: true

output:
  format: json
  store_results: true
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success, no issues |
| 1 | Drift or violations detected |
| 2 | Configuration error |
| 3 | Parse error |

## Best Practices

1. **Start Non-Blocking**
   - Begin with `fail-on-violation: false`
   - Review results before enforcing

2. **Analyze Changed Files Only**
   - Faster CI runs
   - Focus on new code

3. **Set Appropriate Thresholds**
   - Critical paths: stricter
   - Utilities: more lenient

4. **Cache Results**
   - Store analysis for unchanged files
   - Speed up subsequent runs

5. **Gradual Rollout**
   - Start with warnings
   - Move to blocking over time

## Detailed Guides

- [GitHub Actions](./github-actions.md)
- [GitLab CI](./gitlab-ci.md)
