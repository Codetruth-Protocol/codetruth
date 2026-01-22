# GitHub Actions

Detailed guide for integrating CTP with GitHub Actions.

## Basic Setup

Create `.github/workflows/codetruth.yml`:

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
          fetch-depth: 0  # Full history for drift detection
      
      - name: Run CodeTruth Analysis
        uses: codetruth/ctp-action@v1
        with:
          analyze: true
          check-policies: true
          fail-on-violation: false
```

## Full Configuration

```yaml
name: CodeTruth Analysis

on:
  pull_request:
    types: [opened, synchronize, reopened]
  push:
    branches: [main, develop]

jobs:
  analyze:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      pull-requests: write  # For PR comments
    
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      
      - name: Run CodeTruth Analysis
        uses: codetruth/ctp-action@v1
        with:
          # Analysis options
          analyze: true
          changed-files-only: true
          
          # Policy checking
          check-policies: true
          policies-path: .ctp/policies/
          
          # Drift detection
          detect-drift: true
          compare-with: ${{ github.base_ref }}
          min-drift-level: medium
          
          # Enforcement
          fail-on-violation: true
          fail-on-drift: high
          
          # Output
          post-comment: true
          upload-artifacts: true
          
          # LLM enhancement (optional)
          enable-llm: false
          # api-key: ${{ secrets.CTP_API_KEY }}
        
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
      
      - name: Upload Analysis Artifacts
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: ctp-analysis
          path: .ctp/output/
          retention-days: 30
```

## Action Inputs

| Input | Default | Description |
|-------|---------|-------------|
| `analyze` | true | Run code analysis |
| `check-policies` | false | Check policy compliance |
| `detect-drift` | false | Detect drift from base |
| `changed-files-only` | true | Only analyze changed files |
| `policies-path` | .ctp/policies/ | Path to policies |
| `compare-with` | base branch | Reference for drift |
| `min-drift-level` | low | Minimum drift to report |
| `fail-on-violation` | false | Fail on policy violations |
| `fail-on-drift` | critical | Drift level to fail on |
| `post-comment` | true | Post results as PR comment |
| `upload-artifacts` | false | Upload analysis artifacts |
| `enable-llm` | false | Enable LLM enhancement |
| `api-key` | - | API key for LLM |

## Action Outputs

| Output | Description |
|--------|-------------|
| `drift-detected` | Whether drift was detected |
| `drift-level` | Highest drift level found |
| `violations-count` | Number of policy violations |
| `files-analyzed` | Number of files analyzed |
| `report-path` | Path to JSON report |

## PR Comment Example

The action posts a comment like:

```markdown
## CodeTruth Analysis Results

**Files Analyzed:** 12
**Drift Detected:** 2 files
**Policy Violations:** 1

### Drift Summary

| File | Drift Level | Confidence |
|------|-------------|------------|
| src/payments/handler.py | LOW | 78% |
| src/api/routes.py | MEDIUM | 65% |

### Policy Violations

❌ **documentation-required** - FAIL
- `src/utils/helpers.py:45` - Function missing docstring

### Recommendations

1. Add documentation to `process_payment` function
2. Review drift in `src/api/routes.py`
```

## Workflow Examples

### PR Analysis Only

```yaml
on:
  pull_request:

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: codetruth/ctp-action@v1
        with:
          analyze: true
          post-comment: true
```

### Strict Enforcement

```yaml
jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: codetruth/ctp-action@v1
        with:
          analyze: true
          check-policies: true
          detect-drift: true
          fail-on-violation: true
          fail-on-drift: high
```

### Scheduled Audit

```yaml
on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: codetruth/ctp-action@v1
        with:
          analyze: true
          changed-files-only: false
          upload-artifacts: true
      - uses: actions/upload-artifact@v4
        with:
          name: weekly-audit
          path: .ctp/output/
```

## Caching

Speed up analysis with caching:

```yaml
- name: Cache CTP
  uses: actions/cache@v4
  with:
    path: |
      ~/.ctp
      .ctp/analyses
    key: ctp-${{ hashFiles('**/*.py', '**/*.js', '**/*.ts') }}
    restore-keys: |
      ctp-

- uses: codetruth/ctp-action@v1
```

## Matrix Builds

Analyze multiple directories:

```yaml
jobs:
  analyze:
    strategy:
      matrix:
        path: [services/api, services/payments, lib]
    
    steps:
      - uses: actions/checkout@v4
      - uses: codetruth/ctp-action@v1
        with:
          paths: ${{ matrix.path }}
```

## Troubleshooting

### Permission Denied

Ensure workflow has permissions:

```yaml
permissions:
  contents: read
  pull-requests: write
```

### No Files Analyzed

Check file patterns in config:

```yaml
analysis:
  exclude:
    - "**/node_modules/**"
```

### LLM Errors

Verify API key is set:

```yaml
env:
  ANTHROPIC_API_KEY: ${{ secrets.ANTHROPIC_API_KEY }}
```
