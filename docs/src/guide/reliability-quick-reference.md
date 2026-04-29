# Reliability System Quick Reference

## Overview

This quick reference guide summarizes the CodeTruth Protocol reliability system improvements for detecting framework gotchas, code duplication, and critical functionality.

## Framework Gotchas

### Laravel Migrations

**Policy**: `policies/laravel-migrations.yaml`

**Checks**:
- ✅ Timestamps are monotonic (no out-of-order)
- ✅ Generated via `php artisan make:migration`
- ✅ No duplicate timestamps
- ✅ No future timestamps beyond buffer

**Remediation**:
```bash
php artisan make:migration create_table_name
```

### Laravel Queue/Horizon

**Policy**: `policies/laravel-queue.yaml`

**Checks**:
- ✅ Queue connection exists in `config/queue.php`
- ✅ Queue listed in `config/horizon.php`
- ✅ `QUEUE_CONNECTION` set in `.env`
- ✅ Jobs implement `ShouldQueue` interface

**Remediation**:
1. Add connection to `config/queue.php`
2. Add queue to `config/horizon.php` environments
3. Set `QUEUE_CONNECTION` in `.env`

### Database Topology

**Policy**: `policies/laravel-db-topology.yaml`

**Checks**:
- ✅ WHERE clauses have indexes
- ✅ Foreign keys match JOIN patterns
- ✅ Partition keys used in queries
- ✅ Sharding keys respected

**Remediation**:
1. Add indexes for frequently queried columns
2. Verify FK constraints match JOINs
3. Include partition keys in WHERE clauses
4. Respect sharding keys

## Duplication Detection

### Semantic Similarity

**Policy**: `policies/duplication-semantic.yaml`

**Thresholds**:
- WARNING: 70-89% similarity
- ERROR: 90%+ similarity

**Remediation**: Consolidate into shared utility

### Function Name Similarity

**Policy**: `policies/duplication-function-names.yaml`

**Detects**: Functions with similar names doing the same thing
- Example: `formatFileSize`, `formatFileBytes`, `formatBytes`

**Remediation**: Consolidate or rename to reflect differences

### UI Duplicate Effects

**Policy**: `policies/ui-duplicate-effects.yaml`

**Detects**: Multiple toast/notification calls in same action flow

**Remediation**: Use notification manager to deduplicate

## Critical Functionality

### Critical Weight Formula

```
CW = clamp(0.45*A + 0.35*H + 0.20*U, 0, 1) * W_product
```

**Signals**:
- **A** (0.45): Annotations/Specs/Globs
- **H** (0.35): Hot Path Discovery
- **U** (0.20): Usage & Comments
- **W_product**: Product Entity Weighting

### Classification

- **Critical**: CW ≥ 0.85
- **High**: 0.70 ≤ CW < 0.85
- **Medium**: 0.50 ≤ CW < 0.70
- **Low**: CW < 0.50

### Test Requirements

**Policy**: `policies/criticality-tests.yaml`

**Requirements**:
- Critical/High entities must have tests
- Coverage thresholds:
  - Critical: 90%
  - High: 80%
  - Medium: 70%
  - Low: 50%

**Policy**: `policies/criticality-coverage.yaml`

**Checks**:
- Coverage reports parsed (LCOV, Cobertura, JaCoCo, etc.)
- Thresholds enforced by criticality level
- Fails closed if reports missing

## Data Architecture Mapping

### Entity Extraction

1. Parse migrations/ORM models
2. Build entity graph
3. Map code to entities
4. Apply product weighting

### Product Weighting Examples

**E-commerce**:
- `orders`: 1.3
- `checkout/payments`: 1.3
- `catalog`: 1.15

**B2C SaaS**:
- `billing/subscription`: 1.25
- `authentication`: 1.2

**Financial Services**:
- `ledger/transactions`: 1.35
- `risk/fraud`: 1.3

## Adapter Architecture

### Framework Adapters

- **Laravel**: Migrations, Eloquent, Queue, Horizon
- **Django**: Migrations, Models, Celery
- **Rails**: Migrations, ActiveRecord, Sidekiq

### Language Parsers

- PHP, Python, TypeScript, Rust, Go, Java, Ruby

### Normalized Output

- AST Facts
- Symbol Table
- Call Graph
- Config Facts

## Policy Structure

### YAML Policy Format

```yaml
ctp_version: "1.0.0"
policy_schema_version: "1.0.0"
policy:
  id: "policy.id"
  name: "Policy Name"
  description: "Description"
  scope:
    include: ["**/*.php"]
    exclude: ["**/tests/**"]
  severity: "ERROR"
  rules:
    - rule_id: "rule-id"
      type: "behavior_pattern"
      detector: "detector_name"
      params:
        threshold: 0.8
      violation_message: "Violation message"
      remediation: "Remediation steps"
  enforcement:
    block_merge: true
    notify: ["@team"]
```

## Common Detectors

| Detector | Purpose | Policy |
|----------|---------|--------|
| `migrations_timestamp_monotonic` | Migration timestamp ordering | `laravel-migrations.yaml` |
| `queue_horizon_configured` | Queue configuration | `laravel-queue.yaml` |
| `db_topology_alignment` | Database structure alignment | `laravel-db-topology.yaml` |
| `semantic_similarity` | Code duplication | `duplication-semantic.yaml` |
| `function_name_similarity` | Function name duplication | `duplication-function-names.yaml` |
| `ui_duplicate_toast` | UI duplicate effects | `ui-duplicate-effects.yaml` |
| `critical_tests_present` | Test presence | `criticality-tests.yaml` |
| `critical_tests_coverage` | Test coverage | `criticality-coverage.yaml` |

## CI/CD Integration

### Basic Usage

```bash
# Load policies and analyze
ctp-cli check --policies ./policies --paths . --fail-on-violation

# With coverage reports
ctp-cli check \
  --policies ./policies \
  --paths . \
  --coverage coverage/lcov.info \
  --fail-on-violation
```

### GitHub Actions

```yaml
- name: CodeTruth Check
  uses: codetruth/action@v1
  with:
    policies: './policies'
    coverage: 'coverage/lcov.info'
    fail-on-violation: true
```

## Best Practices

1. **Explicit Annotations**: Use code annotations for critical functions
2. **Product Metadata**: Maintain accurate `product-metadata.json`
3. **Coverage Reports**: Generate coverage reports for test enforcement
4. **Git History**: Ensure git history available for hot path analysis
5. **Regular Updates**: Recalculate critical weights periodically
6. **Policy Tuning**: Adjust thresholds per repository needs
7. **Exception Management**: Use policy exceptions with expiry dates

## Troubleshooting

### False Positives

- Adjust similarity thresholds
- Add exceptions to policies
- Review detector parameters

### Missing Coverage

- Verify coverage report format
- Check coverage report paths
- Ensure tests are run before analysis

### Symbol Resolution Issues

- Use language servers (LSP) for accuracy
- Check path-based matching fallback
- Review confidence scores

## Related Documentation

- [Reliability System Guide](./reliability-system.md)
- [Critical Weight Scoring](./critical-weight-scoring.md)
- [Data Architecture Mapping](./data-architecture-mapping.md)
- [Policy Evaluation](./policy-evaluation.md)
