# Quick Start

Get up and running with CodeTruth in 5 minutes.

## Step 1: Install CTP

```bash
# macOS/Linux
curl -fsSL https://codetruth.dev/install.sh | sh

# Windows
irm https://codetruth.dev/install.ps1 | iex
```

## Step 2: Initialize Your Project

```bash
cd your-project
ctp init
```

You'll see:
```
→ Initializing CodeTruth Protocol...
✓ Created .ctp/ directory with default configuration
→ Edit .ctp/config.yaml to customize settings
→ Add policies to .ctp/policies/
```

## Step 3: Analyze Your Code

```bash
# Analyze a single file
ctp analyze src/main.py

# Analyze a directory
ctp analyze src/

# Get detailed explanation
ctp explain src/payments/handler.py
```

## Step 4: Review Results

### Simple Output

```
✓ src/utils/helpers.py [python]
  Intent: Utility functions for string manipulation
  Behavior: Performs 5 function(s)
  Drift: NONE (confidence: 95%)

○ src/api/routes.py [python]
  Intent: API endpoint definitions
  Behavior: Performs 8 function(s), network calls
  Drift: LOW (confidence: 78%)
    → INTENT: Add documentation for error handling

✗ src/payments/processor.py [python]
  Intent: (none declared)
  Behavior: Performs 3 function(s), database operations, network calls
  Drift: HIGH (confidence: 45%)
    → INTENT: Undocumented side effects detected
    → POLICY: High-risk operations need review
```

### JSON Output

```bash
ctp analyze src/ --format json > analysis.json
```

```json
{
  "ctp_version": "1.0.0",
  "module": {
    "name": "handler.py",
    "language": "python",
    "lines_of_code": 150
  },
  "intent": {
    "declared_intent": "Process payment transactions",
    "inferred_intent": "Handle payment processing with retry logic",
    "confidence": 0.85
  },
  "drift": {
    "drift_detected": true,
    "drift_severity": "LOW",
    "drift_details": [...]
  }
}
```

## Step 5: Add to CI/CD

### GitHub Actions

Create `.github/workflows/codetruth.yml`:

```yaml
name: CodeTruth Analysis

on: [pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Run CodeTruth
        uses: codetruth/ctp-action@v1
        with:
          analyze: true
          check-policies: true
          fail-on-violation: false
```

## Step 6: Define Policies (Optional)

Create `.ctp/policies/documentation.yaml`:

```yaml
ctp_version: "1.0.0"
policy_schema_version: "1.0.0"

policy:
  id: "documentation-required"
  name: "Documentation Required"
  description: All public functions must have documentation.
  
  scope:
    include:
      - "**/*.py"
      - "**/*.js"
    exclude:
      - "**/*_test.*"
  
  severity: "WARNING"
  
  rules:
    - rule_id: "docstring-required"
      type: "documentation"
      violation_message: Function detected without documentation.
```

Run policy check:

```bash
ctp check --policies=.ctp/policies/
```

## Common Commands

| Command | Description |
|---------|-------------|
| `ctp init` | Initialize CTP in current directory |
| `ctp analyze <path>` | Analyze files or directories |
| `ctp explain <file>` | Get detailed explanation for a file |
| `ctp check` | Check policy compliance |
| `ctp diff <base> <head>` | Compare drift between versions |
| `ctp audit` | Generate audit report |

## Next Steps

- [Configuration](./configuration.md) - Customize CTP settings
- [Core Concepts](./concepts.md) - Understand explanation graphs and drift
- [CI/CD Integration](../integration/cicd.md) - Set up automated analysis
- [Writing Policies](./policy-evaluation.md) - Create custom policies
