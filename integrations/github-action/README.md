# CodeTruth GitHub Action

Run CodeTruth Protocol analysis on your pull requests.

## Usage

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

      - name: Run CodeTruth Analysis
        uses: codetruth/ctp-action@v1
        with:
          analyze: true
          check-policies: true
          detect-drift: true
          fail-on-violation: false
          min-drift-level: medium
          post-comment: true
```

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `analyze` | Run analysis on changed files | `true` |
| `check-policies` | Check policy compliance | `true` |
| `policies-path` | Path to policy files | `.ctp/policies/` |
| `detect-drift` | Detect drift from base branch | `true` |
| `fail-on-violation` | Fail on policy violations | `false` |
| `min-drift-level` | Minimum drift level to report | `medium` |
| `api-key` | API key for LLM enhancement | - |
| `post-comment` | Post results as PR comment | `true` |

## Outputs

| Output | Description |
|--------|-------------|
| `drift-detected` | Whether drift was detected |
| `violations-count` | Number of policy violations |
| `analysis-report` | Path to analysis report JSON |
