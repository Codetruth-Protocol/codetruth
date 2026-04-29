# CodeTruth GitHub Action

Run CodeTruth Protocol analysis on your pull requests using the MCP server (ctp-mcp).

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

## How It Works

This action:
1. Builds the `ctp-mcp` MCP server from source
2. Invokes the `analyze_codebase` MCP tool on the repository
3. Parses the JSON-RPC response for violations and drift
4. Posts results as a PR comment (if enabled)
5. Fails the check if violations exceed threshold (if enabled)

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `analyze` | Run analysis on changed files | `true` |
| `check-policies` | Check policy compliance | `true` |
| `policies-path` | Path to policy files | `.ctp/policies/` |
| `detect-drift` | Detect drift from base branch | `true` |
| `fail-on-violation` | Fail on policy violations | `false` |
| `min-drift-level` | Minimum drift level to report | `medium` |
| `post-comment` | Post results as PR comment | `true` |

## Outputs

| Output | Description |
|--------|-------------|
| `drift-detected` | Whether drift was detected |
| `violations-count` | Number of policy violations |
| `analysis-report` | Path to analysis report JSON |

## Requirements

- Rust toolchain (installed via actions-rs/toolchain)
- GitHub CLI (gh) for PR comment posting (optional)
- jq for JSON parsing (optional, fallback provided)

## Notes

- The action builds ctp-mcp from source to ensure compatibility
- Analysis results are saved to `.ctp/output/analysis.json`
- Requires the repository to have policy files at `.ctp/policies/` (if check-policies is enabled)
