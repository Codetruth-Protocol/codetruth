# Keyword Dictionary Contribution Guidelines

## Overview

The CodeTruth keyword dictionary (`spec/keywords.csv`) provides extensible keyword management for code analysis. This document explains how to contribute and maintain the dictionary.

## CSV Format

The dictionary follows this CSV format:
```
category,term,weight/score,context,synonyms,notes
```

### Categories

1. **domain_keywords** - For intent classification
   - `weight`: 0.0-1.0 (confidence weight)
   - `context`: Domain area (security, billing, etc.)
   - `synonyms`: Space-separated synonyms
   - `notes`: Description

2. **critical_domains** - For criticality assessment
   - `weight/score`: 0-10 (criticality score)
   - `context`: Reason for criticality
   - `synonyms`: (ignored)
   - `notes`: Detailed explanation

3. **stopwords** - Common words to ignore
   - `weight/score`: 1.0 (uniform weight)
   - `context`: Description
   - `synonyms`: (ignored)
   - `notes`: (ignored)

4. **project_type_keywords** - Project-specific keywords
   - `weight/score`: 1.0 (uniform weight)
   - `context`: Project type (cli_tool, web_app, library, api_service)
   - `synonyms`: (ignored)
   - `notes`: (ignored)

## Contribution Guidelines

### Adding New Keywords

1. **Check for duplicates** - Search existing terms before adding
2. **Use lowercase** - All terms should be lowercase
3. **Provide context** - Include meaningful descriptions
4. **Weight appropriately** - Use weights that reflect importance
5. **Include synonyms** - Add common variations and abbreviations

### Domain Keywords

```csv
domain_keywords,blockchain,1.0,cryptography,crypto distributed,"Blockchain and distributed ledger"
domain_keywords,microservice,0.9,architecture,service,"Microservice architecture"
```

### Critical Domains

```csv
critical_domains,security,10,"Core system protection - vulnerabilities compromise entire system"
critical_domains,authentication,9,"Identity verification - system breaks without auth"
```

### Stopwords

```csv
stopwords,however,1.0,"Common transition word"
stopwords,therefore,1.0,"Common transition word"
```

### Project Type Keywords

```csv
project_type_keywords,container,1.0,cli_tool keywords
project_type_keywords,component,1.0,web_app keywords
```

## Validation

Run validation before submitting:

```bash
cargo test --package ctp-spec keyword_dictionary
```

The validator checks for:
- Empty terms
- Invalid weights (domain: 0-1, critical: 0-10)
- Duplicate entries
- Malformed CSV lines

## Best Practices

### 1. Be Conservative
- Start with lower weights and increase based on usage
- Only add keywords that have clear semantic meaning
- Avoid overly generic terms

### 2. Provide Rich Context
- Include specific use cases in notes
- Add relevant synonyms for better matching
- Use descriptive context fields

### 3. Consider Project Types
- Tag keywords for relevant project types
- Think about CLI vs web vs library contexts
- Consider industry-specific terminology

### 4. Maintain Balance
- Don't over-specialize (too many niche terms)
- Keep stopwords focused on common noise words
- Ensure critical domains reflect real system impact

## Review Process

1. **Submit PR** with keyword additions
2. **Automated validation** runs on CI
3. **Manual review** for semantic appropriateness
4. **Testing** on real codebases
5. **Merge** and version bump

## Version Management

- Update version in CSV header when making breaking changes
- Use semantic versioning (1.0.0, 1.1.0, etc.)
- Document breaking changes in this file

## Examples

### Good Addition
```csv
domain_keywords,observability,1.0,monitoring,telemetry metrics,"System observability and monitoring"
```

### Poor Addition
```csv
domain_keywords,stuff,0.5,general,"Generic stuff"  # Too generic
```

### Critical Domain Example
```csv
critical_domains,payment,10,"Financial transactions - payment failures are business-critical and may cause financial loss"
```

## Tools and Scripts

### Validate Dictionary
```bash
cargo run --bin validate-keywords
```

### Statistics
```bash
cargo run --bin keyword-stats
```

### Search Keywords
```bash
cargo run --bin search-keywords auth
```

## Troubleshooting

### Common Issues

1. **CSV parsing errors** - Check for unescaped commas
2. **Duplicate terms** - Use unique lowercase terms
3. **Weight validation** - Ensure correct ranges
4. **Missing fields** - All 6 columns must be present

### Debug Mode
```bash
RUST_LOG=debug cargo test keyword_dictionary
```

## Contact

For questions about keyword contributions:
- Create an issue in the repository
- Tag with `keyword-dictionary` label
- Provide examples of where the keyword would be useful
