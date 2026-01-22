# Configuration

CTP is configured via `.ctp/config.yaml` in your project root.

## Default Configuration

```yaml
# CodeTruth Protocol Configuration
version: "1.0"

# Analysis settings
analysis:
  # Languages to analyze
  languages:
    - python
    - javascript
    - typescript
    - rust
    - go
    - java

  # Files to exclude (glob patterns)
  exclude:
    - "**/node_modules/**"
    - "**/target/**"
    - "**/.git/**"
    - "**/dist/**"
    - "**/build/**"
    - "**/__pycache__/**"
    - "**/venv/**"

  # Maximum file size to analyze (bytes)
  max_file_size: 10485760  # 10MB

# LLM settings (optional)
llm:
  enabled: false
  provider: anthropic  # anthropic, openai, ollama
  model: claude-sonnet-4-20250514
  # api_key: $ANTHROPIC_API_KEY  # Use environment variable

# Drift detection
drift:
  # Minimum severity to report
  min_severity: low  # none, low, medium, high, critical
  
  # Similarity threshold for "no drift"
  similarity_threshold: 0.7

# Policy settings
policies:
  # Path to policy files
  path: .ctp/policies/
  
  # Fail CI on policy violations
  fail_on_violation: false

# Output settings
output:
  # Default format (simple, json, yaml)
  format: simple
  
  # Store analysis results
  store_results: true
  results_path: .ctp/analyses/
```

## Configuration Options

### Analysis Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `languages` | list | All supported | Languages to analyze |
| `exclude` | list | Common ignores | Glob patterns to exclude |
| `max_file_size` | int | 10485760 | Max file size in bytes |

### LLM Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | false | Enable LLM enhancement |
| `provider` | string | anthropic | LLM provider |
| `model` | string | claude-sonnet-4-20250514 | Model to use |
| `api_key` | string | - | API key (use env var) |

**Supported Providers:**
- `anthropic` - Claude models
- `openai` - GPT models
- `ollama` - Local models

### Drift Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `min_severity` | string | low | Minimum severity to report |
| `similarity_threshold` | float | 0.7 | Threshold for "no drift" |

**Severity Levels:**
- `none` - Report everything
- `low` - Minor discrepancies
- `medium` - Notable differences
- `high` - Significant drift
- `critical` - Severe mismatch

### Policy Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `path` | string | .ctp/policies/ | Policy files location |
| `fail_on_violation` | bool | false | Fail CI on violations |

## Environment Variables

CTP respects these environment variables:

| Variable | Description |
|----------|-------------|
| `CTP_CONFIG` | Path to config file |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `CTP_LOG_LEVEL` | Log level (debug, info, warn, error) |

## Per-Directory Configuration

You can override settings for specific directories by placing a `.ctp/config.yaml` in that directory. Settings are merged with parent configurations.

## CLI Overrides

Most settings can be overridden via CLI flags:

```bash
# Override output format
ctp analyze src/ --format json

# Enable LLM enhancement
ctp analyze src/ --enhance

# Set minimum drift level
ctp analyze src/ --min-drift-level high

# Specify config file
ctp --config /path/to/config.yaml analyze src/
```

## Example Configurations

### Minimal (No LLM)

```yaml
version: "1.0"
analysis:
  languages: [python, javascript]
drift:
  min_severity: medium
```

### With LLM Enhancement

```yaml
version: "1.0"
llm:
  enabled: true
  provider: anthropic
  model: claude-sonnet-4-20250514
drift:
  min_severity: low
```

### CI/CD Strict Mode

```yaml
version: "1.0"
drift:
  min_severity: high
policies:
  fail_on_violation: true
output:
  format: json
```

### Local Development

```yaml
version: "1.0"
llm:
  enabled: true
  provider: ollama
  model: codestral
  base_url: http://localhost:11434
output:
  format: simple
```
