# Policy Definition Language (PDL)

The Policy Definition Language (PDL) is CTP's YAML-based format for defining code governance rules.

## Overview

PDL allows organizations to define:
- Code quality standards
- Security requirements
- Documentation policies
- Compliance rules

## Schema

```yaml
ctp_version: "1.0.0"
policy_schema_version: "1.0.0"

policy:
  id: string           # Unique identifier
  name: string         # Human-readable name
  description: string  # Detailed description
  
  scope:
    include: [glob]    # Files to include
    exclude: [glob]    # Files to exclude
  
  severity: enum       # INFO, WARNING, ERROR, CRITICAL
  
  rules:
    - rule_id: string
      type: string     # Rule type
      requires: [...]  # Conditions
      violation_message: string
      remediation: string
  
  enforcement:
    block_merge: bool
    notify: [string]
    require_approval_from: [string]
  
  exceptions:
    - file: string
      reason: string
      approved_by: string
      expires_at: date
  
  metadata:
    created_by: string
    created_at: date
    last_updated: date
    references: [string]
```

## Complete Example

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
      - "**/fixtures/**"
  
  severity: "CRITICAL"
  
  rules:
    - rule_id: "idempotency-key-check"
      type: "behavior_pattern"
      
      requires:
        - pattern: "retry"
          context: "payment|transaction|charge"
        
        - pattern: "idempotency_key|idempotent_id|request_id"
          must_exist: true
          
        - pattern: "exponential_backoff|exp_backoff|jittered_retry"
          must_exist: true
      
      violation_message: |
        Payment retry logic detected without idempotency safeguards.
        All payment retries MUST include:
        1. Unique idempotency key generation
        2. Exponential backoff with jitter
        3. Maximum retry limit
      
      remediation: |
        Add idempotency key handling:
        ```python
        idempotency_key = f"{user_id}_{transaction_id}_{attempt}"
        retry_with_backoff(
            operation=charge_payment,
            idempotency_key=idempotency_key,
            max_retries=3,
            backoff_multiplier=2.0
        )
        ```
  
  enforcement:
    block_merge: true
    notify:
      - "@payments-team"
      - "@security-team"
    require_approval_from:
      - "CODEOWNER"
      - "payments-lead"
  
  exceptions:
    - file: "services/payments/legacy_processor.py"
      reason: "Legacy system, scheduled for replacement Q2 2026"
      approved_by: "cto@company.com"
      expires_at: "2026-06-30"

  metadata:
    created_by: "security-team"
    created_at: "2026-01-15"
    last_updated: "2026-01-16"
    references:
      - "https://stripe.com/docs/api/idempotent_requests"
      - "internal-docs/payment-standards.md"
```

## Rule Types

### behavior_pattern

Check for code patterns:

```yaml
rules:
  - rule_id: "no-hardcoded-secrets"
    type: "behavior_pattern"
    requires:
      - pattern: "(api_key|secret|password)\\s*=\\s*['\"][^'\"]+['\"]"
        must_exist: false
```

### documentation

Require documentation:

```yaml
rules:
  - rule_id: "docstring-required"
    type: "documentation"
    requires:
      - pattern: "def |function |fn "
        must_have_doc: true
```

### side_effect

Check for side effects:

```yaml
rules:
  - rule_id: "no-undocumented-io"
    type: "side_effect"
    requires:
      - effect_type: "io"
        must_be_documented: true
```

### dependency

Check dependencies:

```yaml
rules:
  - rule_id: "no-deprecated-libs"
    type: "dependency"
    requires:
      - module: "requests"
        version: ">=2.28.0"
```

## Pattern Syntax

Patterns use regex with these extensions:

| Syntax | Description |
|--------|-------------|
| `pattern` | Regex to match |
| `context` | Required surrounding context |
| `must_exist` | Pattern must be present |
| `must_not_exist` | Pattern must be absent |

## Scope Patterns

Glob patterns for file matching:

| Pattern | Matches |
|---------|---------|
| `**/*.py` | All Python files |
| `src/**` | All files in src/ |
| `!**/test/**` | Exclude test directories |
| `*.{js,ts}` | JS and TS files |

## Severity Levels

| Level | Description | CI Behavior |
|-------|-------------|-------------|
| `INFO` | Informational | No action |
| `WARNING` | Review recommended | Warning in logs |
| `ERROR` | Must be addressed | Fail if configured |
| `CRITICAL` | Immediate action | Always fail |

## Enforcement Options

### block_merge

Block PR merge on violation:

```yaml
enforcement:
  block_merge: true
```

### notify

Send notifications:

```yaml
enforcement:
  notify:
    - "@security-team"
    - "security@company.com"
```

### require_approval_from

Require specific approvers:

```yaml
enforcement:
  require_approval_from:
    - "CODEOWNER"
    - "security-lead"
```

## Exceptions

Allow exceptions with audit trail:

```yaml
exceptions:
  - file: "legacy/old_code.py"
    reason: "Legacy code, migration planned"
    approved_by: "tech-lead@company.com"
    expires_at: "2026-12-31"
```

## Validation

Validate policy files:

```bash
ctp policy validate .ctp/policies/my-policy.yaml
```

## Best Practices

1. **Start Simple**: Begin with 3-5 key policies
2. **Clear Messages**: Provide actionable violation messages
3. **Include Remediation**: Show how to fix issues
4. **Use Exceptions Sparingly**: Document and expire them
5. **Version Control**: Store policies in git
6. **Team Review**: Discuss policies before enforcement
