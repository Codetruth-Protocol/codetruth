# Drift Detection

Drift detection is CTP's mechanism for identifying mismatches between what code claims to do and what it actually does.

## What is Drift?

**Drift** occurs when there's a discrepancy between:
- Declared intent (documentation) and inferred intent (analysis)
- Intent and actual implementation behavior
- Current code and previous versions

## Drift Types

### Intent Drift

Mismatch between declared and inferred intent.

**Example:**
```python
def add_numbers(a, b):
    """Add two numbers together."""  # Declared: addition
    return a - b  # Actual: subtraction!
```

**Detection:** CTP compares documentation semantics with code behavior.

### Policy Drift

Code violates organizational policies.

**Example:**
```python
def delete_user(user_id):
    # No confirmation, no logging, no soft-delete
    db.execute(f"DELETE FROM users WHERE id = {user_id}")
```

**Detection:** Policy rules check for required patterns.

### Assumption Drift

Code makes invalid or outdated assumptions.

**Example:**
```python
def get_user_email(user):
    return user.email  # Assumes user always has email
```

**Detection:** Analysis of null checks, type guards, error handling.

### Implementation Drift

Behavior changed without intent update.

**Example:**
```python
# v1: Simple calculation
def calculate_tax(amount):
    return amount * 0.1

# v2: Added side effect without doc update
def calculate_tax(amount):
    log_calculation(amount)  # New side effect!
    return amount * 0.1
```

**Detection:** Version comparison of behavior analysis.

## Severity Levels

| Level | Similarity | Action |
|-------|------------|--------|
| **NONE** | ≥90% | No action needed |
| **LOW** | 70-90% | Review recommended |
| **MEDIUM** | 50-70% | Investigation required |
| **HIGH** | 30-50% | Immediate attention |
| **CRITICAL** | <30% | Urgent fix required |

## Detection Algorithm

CTP uses a multi-factor approach:

### 1. Semantic Similarity

Compares meaning of declared vs inferred intent using word overlap (Jaccard similarity):

```
similarity = |A ∩ B| / |A ∪ B|
```

### 2. Structural Analysis

Checks for:
- Undocumented side effects
- Missing error handling
- Dangerous operations without documentation

### 3. Pattern Matching

Identifies risky patterns:
- `delete`, `drop`, `truncate` without documentation
- Network calls without timeout handling
- Database operations without transactions

## Configuration

```yaml
drift:
  # Minimum severity to report
  min_severity: low
  
  # Similarity threshold for "no drift"
  similarity_threshold: 0.7
  
  # Weight for semantic vs structural analysis
  semantic_weight: 0.6
  structural_weight: 0.4
```

## CLI Usage

```bash
# Analyze with drift detection
ctp analyze src/

# Set minimum drift level to report
ctp analyze src/ --min-drift-level medium

# Compare versions
ctp diff HEAD~1 HEAD

# CI mode - fail on high drift
ctp ci-check --min-drift-level high
```

## Drift Report

```json
{
  "drift_detected": true,
  "overall_severity": "MEDIUM",
  "findings": [
    {
      "drift_type": "INTENT",
      "severity": "MEDIUM",
      "expected": "Calculate user discount",
      "actual": "Modify user account balance",
      "location": {
        "file": "src/billing.py",
        "line_start": 45,
        "line_end": 52
      },
      "remediation": "Update documentation to match actual behavior"
    }
  ],
  "confidence": 0.72
}
```

## Handling Drift

### Low Drift

Usually acceptable. Consider:
- Updating documentation for clarity
- Adding more specific comments

### Medium Drift

Requires investigation:
- Review code and documentation
- Determine if behavior or docs need updating
- Add tests to verify intended behavior

### High/Critical Drift

Immediate action needed:
- Stop deployment if in CI
- Review with team lead
- Determine root cause
- Fix code or documentation
- Add regression tests

## Best Practices

1. **Document Intent Clearly**
   ```python
   def process_payment(amount: float) -> PaymentResult:
       """
       Process a payment transaction.
       
       Side Effects:
           - Writes to payments table
           - Calls Stripe API
           - Sends confirmation email
       """
   ```

2. **Update Docs with Code**
   - Treat documentation as code
   - Review docs in PRs
   - Use CTP in CI to catch drift

3. **Set Appropriate Thresholds**
   - Stricter for critical paths (payments, auth)
   - More lenient for internal utilities

4. **Monitor Drift Over Time**
   - Track drift trends
   - Address systemic issues
   - Celebrate improvements
