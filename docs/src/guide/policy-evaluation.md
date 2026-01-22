# Policy Evaluation

Policies define organizational rules for code quality, security, and compliance. CTP evaluates code against these policies automatically.

## Policy Definition Language (PDL)

Policies are defined in YAML using CTP's Policy Definition Language:

```yaml
ctp_version: "1.0.0"
policy_schema_version: "1.0.0"

policy:
  id: "payment-idempotency-001"
  name: "Payment Retry Idempotency"
  description: |
    All payment retry logic must implement idempotency
    safeguards to prevent duplicate charges.
  
  scope:
    include:
      - "services/payments/**/*.{py,js,ts,go}"
      - "lib/payment_processing/**"
    exclude:
      - "**/*_test.*"
      - "**/*.spec.*"
  
  severity: "CRITICAL"
  
  rules:
    - rule_id: "idempotency-key-check"
      type: "behavior_pattern"
      requires:
        - pattern: "retry"
          context: "payment|transaction|charge"
        - pattern: "idempotency_key|idempotent_id|request_id"
          must_exist: true
      
      violation_message: |
        Payment retry logic detected without idempotency safeguards.
      
      remediation: |
        Add idempotency key handling to prevent duplicate charges.
  
  enforcement:
    block_merge: true
    notify:
      - "@payments-team"
      - "@security-team"
```

## Policy Structure

### Metadata

| Field | Required | Description |
|-------|----------|-------------|
| `id` | Yes | Unique identifier |
| `name` | Yes | Human-readable name |
| `description` | Yes | Detailed description |
| `severity` | Yes | INFO, WARNING, ERROR, CRITICAL |

### Scope

Define which files the policy applies to:

```yaml
scope:
  include:
    - "src/**/*.py"           # All Python files in src
    - "lib/**/*.{js,ts}"      # JS/TS in lib
  exclude:
    - "**/*_test.*"           # Exclude tests
    - "**/fixtures/**"        # Exclude fixtures
```

### Rules

Each policy contains one or more rules:

```yaml
rules:
  - rule_id: "unique-rule-id"
    type: "behavior_pattern"    # Rule type
    requires:                   # Conditions
      - pattern: "regex"
        must_exist: true
    violation_message: "..."    # Error message
    remediation: "..."          # How to fix
```

### Rule Types

| Type | Description |
|------|-------------|
| `behavior_pattern` | Check for code patterns |
| `documentation` | Require documentation |
| `side_effect` | Check side effects |
| `dependency` | Check dependencies |

## Built-in Policies

CTP includes common policies:

### Documentation Required

```yaml
policy:
  id: "documentation-required"
  name: "Documentation Required"
  rules:
    - rule_id: "docstring-required"
      type: "documentation"
      requires:
        - pattern: "def |function |fn "
          must_have_doc: true
```

### No Hardcoded Secrets

```yaml
policy:
  id: "no-hardcoded-secrets"
  name: "No Hardcoded Secrets"
  rules:
    - rule_id: "no-api-keys"
      type: "behavior_pattern"
      requires:
        - pattern: "(api_key|apikey|secret|password)\\s*=\\s*['\"][^'\"]+['\"]"
          must_exist: false
```

### Error Handling Required

```yaml
policy:
  id: "error-handling"
  name: "Error Handling Required"
  rules:
    - rule_id: "try-catch-required"
      type: "behavior_pattern"
      requires:
        - pattern: "fetch|axios|http"
          context: "try|catch|error"
```

## Using Policies

### Create Policy File

Save to `.ctp/policies/my-policy.yaml`

### Check Compliance

```bash
# Check all policies
ctp check

# Check specific policy
ctp check --policy .ctp/policies/payment-idempotency.yaml

# Check specific files
ctp check src/payments/
```

### Output

```
Policy Check Results
====================

✓ documentation-required: PASS
  Files checked: 45
  Violations: 0

✗ payment-idempotency: FAIL
  Files checked: 12
  Violations: 2

  src/payments/retry.py:45
    Rule: idempotency-key-check
    Severity: CRITICAL
    Message: Payment retry logic detected without idempotency safeguards.
    Remediation: Add idempotency key handling to prevent duplicate charges.

  src/payments/processor.py:123
    Rule: idempotency-key-check
    Severity: CRITICAL
    Message: Payment retry logic detected without idempotency safeguards.
```

## CI/CD Integration

### GitHub Actions

```yaml
- name: Check Policies
  uses: codetruth/ctp-action@v1
  with:
    check-policies: true
    fail-on-violation: true
```

### CLI

```bash
# Fail on any violation
ctp check --fail-on-violation

# Fail only on critical
ctp check --min-severity critical --fail-on-violation
```

## Policy Exceptions

Allow exceptions with justification:

```yaml
policy:
  exceptions:
    - file: "services/payments/legacy_processor.py"
      reason: "Legacy system, scheduled for replacement Q2 2026"
      approved_by: "cto@company.com"
      expires_at: "2026-06-30"
```

## Best Practices

1. **Start Simple**
   - Begin with 3-5 key policies
   - Add more as team matures

2. **Severity Matters**
   - CRITICAL: Block deployment
   - ERROR: Require review
   - WARNING: Informational
   - INFO: Suggestions

3. **Clear Remediation**
   - Always provide fix guidance
   - Include code examples

4. **Team Buy-in**
   - Discuss policies with team
   - Document rationale
   - Review periodically

5. **Gradual Enforcement**
   - Start with warnings
   - Move to blocking over time
