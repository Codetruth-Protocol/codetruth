# GitLab CI

Guide for integrating CTP with GitLab CI/CD.

## Basic Setup

Add to `.gitlab-ci.yml`:

```yaml
codetruth:
  image: codetruth/ctp:latest
  stage: test
  script:
    - ctp analyze src/
    - ctp check
  only:
    - merge_requests
```

## Full Configuration

```yaml
stages:
  - test
  - analyze

variables:
  CTP_MIN_DRIFT_LEVEL: "medium"

codetruth:
  image: codetruth/ctp:latest
  stage: analyze
  
  before_script:
    - ctp --version
  
  script:
    # Analyze changed files
    - |
      if [ -n "$CI_MERGE_REQUEST_DIFF_BASE_SHA" ]; then
        CHANGED_FILES=$(git diff --name-only $CI_MERGE_REQUEST_DIFF_BASE_SHA HEAD -- '*.py' '*.js' '*.ts')
        if [ -n "$CHANGED_FILES" ]; then
          echo "$CHANGED_FILES" | xargs ctp analyze --format json --output analysis.json
        fi
      else
        ctp analyze src/ --format json --output analysis.json
      fi
    
    # Check policies
    - ctp check --policies .ctp/policies/ --fail-on-violation
    
    # Detect drift
    - ctp ci-check --min-drift-level $CTP_MIN_DRIFT_LEVEL
  
  artifacts:
    paths:
      - analysis.json
      - .ctp/output/
    reports:
      codequality: analysis.json
    expire_in: 30 days
  
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH

codetruth:audit:
  image: codetruth/ctp:latest
  stage: analyze
  script:
    - ctp analyze . --format json --output full-audit.json
    - ctp audit --format json --output audit-report.json
  artifacts:
    paths:
      - full-audit.json
      - audit-report.json
    expire_in: 90 days
  rules:
    - if: $CI_PIPELINE_SOURCE == "schedule"
```

## Environment Variables

Set in GitLab CI/CD Settings:

| Variable | Description |
|----------|-------------|
| `CTP_API_KEY` | API key for LLM features |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `CTP_MIN_DRIFT_LEVEL` | Minimum drift level |

## Merge Request Integration

### Code Quality Report

CTP can generate GitLab Code Quality reports:

```yaml
codetruth:
  script:
    - ctp analyze src/ --format gitlab-codequality --output gl-code-quality-report.json
  artifacts:
    reports:
      codequality: gl-code-quality-report.json
```

### Merge Request Comments

Use GitLab API to post comments:

```yaml
codetruth:
  script:
    - ctp analyze src/ --format json --output analysis.json
    - |
      if [ -n "$CI_MERGE_REQUEST_IID" ]; then
        COMMENT=$(ctp format-comment analysis.json)
        curl --request POST \
          --header "PRIVATE-TOKEN: $GITLAB_TOKEN" \
          --data-urlencode "body=$COMMENT" \
          "$CI_API_V4_URL/projects/$CI_PROJECT_ID/merge_requests/$CI_MERGE_REQUEST_IID/notes"
      fi
```

## Pipeline Templates

### Include Template

Create `.gitlab/codetruth.yml`:

```yaml
.codetruth:
  image: codetruth/ctp:latest
  stage: analyze
  script:
    - ctp analyze ${CTP_PATHS:-src/}
    - ctp check
  artifacts:
    paths:
      - .ctp/output/
```

Use in projects:

```yaml
include:
  - local: '.gitlab/codetruth.yml'

codetruth:
  extends: .codetruth
  variables:
    CTP_PATHS: "services/"
```

## Caching

```yaml
codetruth:
  cache:
    key: ctp-${CI_COMMIT_REF_SLUG}
    paths:
      - .ctp/analyses/
```

## Parallel Analysis

```yaml
codetruth:
  parallel:
    matrix:
      - PATH: ["services/api", "services/payments", "lib"]
  script:
    - ctp analyze $PATH
```

## Scheduled Audits

```yaml
# In GitLab CI/CD > Schedules, create weekly schedule

codetruth:weekly-audit:
  script:
    - ctp analyze . --format json
    - ctp audit --format pdf --output audit-$(date +%Y-%m-%d).pdf
  artifacts:
    paths:
      - "*.pdf"
  rules:
    - if: $CI_PIPELINE_SOURCE == "schedule"
```

## Docker-in-Docker

For custom Docker builds:

```yaml
codetruth:
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker run --rm -v $(pwd):/workspace codetruth/ctp analyze /workspace/src
```

## Troubleshooting

### Exit Code 1

Check if drift or violations were detected:

```yaml
script:
  - ctp analyze src/ || echo "Drift detected"
  - ctp check || exit 1  # Only fail on policy violations
```

### Large Repositories

Limit analysis scope:

```yaml
script:
  - ctp analyze src/ --max-files 1000
```

### Timeout Issues

Increase job timeout:

```yaml
codetruth:
  timeout: 30 minutes
```
