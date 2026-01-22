# CLI Reference

Complete reference for the CodeTruth Protocol CLI.

## Synopsis

```bash
ctp [OPTIONS] <COMMAND>
```

## Global Options

| Option | Description |
|--------|-------------|
| `-v, --verbose` | Enable verbose output |
| `-h, --help` | Print help information |
| `-V, --version` | Print version |
| `--config <PATH>` | Path to config file |

## Commands

### init

Initialize CTP in a repository.

```bash
ctp init [PATH]
```

**Arguments:**
- `PATH` - Directory to initialize (default: current directory)

**Example:**
```bash
ctp init
ctp init /path/to/project
```

**Creates:**
- `.ctp/config.yaml` - Configuration file
- `.ctp/policies/` - Policy directory
- `.ctp/analyses/` - Analysis storage

---

### analyze

Analyze files or directories.

```bash
ctp analyze [OPTIONS] <PATHS>...
```

**Arguments:**
- `PATHS` - Files or directories to analyze

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `-f, --format` | simple | Output format (simple, json, yaml) |
| `-o, --output` | stdout | Output file |
| `--enhance` | false | Use LLM enhancement |
| `--min-drift-level` | low | Minimum drift to report |

**Examples:**
```bash
# Analyze single file
ctp analyze src/main.py

# Analyze directory
ctp analyze src/

# JSON output
ctp analyze src/ --format json

# Save to file
ctp analyze src/ --format json --output analysis.json

# With LLM enhancement
ctp analyze src/ --enhance
```

---

### explain

Get detailed explanation for a single file.

```bash
ctp explain [OPTIONS] <FILE>
```

**Arguments:**
- `FILE` - File to explain

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `-f, --format` | json | Output format |

**Example:**
```bash
ctp explain src/payments/handler.py
ctp explain src/handler.py --format yaml
```

---

### check

Check policy compliance.

```bash
ctp check [OPTIONS] [PATHS]...
```

**Arguments:**
- `PATHS` - Files or directories to check (default: current directory)

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `-p, --policies` | .ctp/policies | Policy path |
| `--fail-on-violation` | false | Exit with error on violations |
| `--min-severity` | info | Minimum severity to report |

**Examples:**
```bash
# Check all files
ctp check

# Check specific directory
ctp check src/payments/

# Fail on violations
ctp check --fail-on-violation

# Custom policy path
ctp check --policies ./policies/
```

---

### diff

Compare drift between versions.

```bash
ctp diff <BASE> [HEAD]
```

**Arguments:**
- `BASE` - Base reference (commit, branch, tag)
- `HEAD` - Head reference (default: HEAD)

**Examples:**
```bash
# Compare with previous commit
ctp diff HEAD~1

# Compare branches
ctp diff main feature-branch

# Compare specific commits
ctp diff abc123 def456
```

---

### audit

Generate audit report.

```bash
ctp audit [OPTIONS]
```

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `-f, --format` | json | Output format (json, pdf, html) |
| `-o, --output` | stdout | Output file |

**Examples:**
```bash
# JSON report
ctp audit --format json --output audit.json

# PDF report
ctp audit --format pdf --output audit-2026-01.pdf
```

---

### ci-check

CI/CD integration check.

```bash
ctp ci-check [OPTIONS] [PATHS]...
```

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `--min-drift-level` | high | Minimum drift to fail |
| `--fail-on-violation` | false | Fail on policy violations |

**Examples:**
```bash
# Standard CI check
ctp ci-check

# Strict mode
ctp ci-check --min-drift-level medium --fail-on-violation

# Check specific paths
ctp ci-check src/critical/
```

**Exit Codes:**
- `0` - Success, no issues
- `1` - Drift or violations detected

---

### lsp

Start Language Server Protocol server.

```bash
ctp lsp [OPTIONS]
```

**Options:**
| Option | Default | Description |
|--------|---------|-------------|
| `-p, --port` | 9999 | Port to listen on |

**Example:**
```bash
ctp lsp --port 9999
```

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CTP_CONFIG` | Config file path |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `CTP_LOG_LEVEL` | Log level |

## Output Formats

### Simple (Default)

Human-readable output:
```
✓ src/utils.py [python]
  Intent: Utility functions
  Behavior: 5 functions
  Drift: NONE (confidence: 95%)
```

### JSON

Machine-readable:
```json
{
  "ctp_version": "1.0.0",
  "module": {...},
  "intent": {...},
  "drift": {...}
}
```

### YAML

```yaml
ctp_version: "1.0.0"
module:
  name: utils.py
  language: python
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Drift/violations detected |
| 2 | Configuration error |
| 3 | Parse error |
| 4 | I/O error |
