# SDKs

CTP provides SDKs for programmatic integration.

## Available SDKs

| SDK | Package | Status |
|-----|---------|--------|
| TypeScript/Node.js | `@codetruth/sdk` | ✅ Stable |
| Python | `codetruth` | ✅ Stable |
| Rust | `ctp-core` | ✅ Stable |
| WebAssembly | `@codetruth/wasm` | 🚧 Beta |

## Quick Comparison

```typescript
// TypeScript
import { CodeTruth } from '@codetruth/sdk';
const ctp = new CodeTruth();
const result = await ctp.analyzeFile('src/handler.py');
```

```python
# Python
from codetruth import CodeTruth
ctp = CodeTruth()
result = ctp.analyze_file('src/handler.py')
```

```rust
// Rust
use ctp_core::CodeTruthEngine;
let engine = CodeTruthEngine::default();
let result = engine.analyze_file(path).await?;
```

## Use Cases

### CI/CD Integration

```typescript
const ctp = new CodeTruth();
const results = await ctp.analyzeDirectory('src/');

const hasCriticalDrift = results.some(
  r => r.drift.drift_severity === 'CRITICAL'
);

if (hasCriticalDrift) {
  process.exit(1);
}
```

### Custom Tooling

```python
from codetruth import CodeTruth

ctp = CodeTruth()

# Analyze and filter
results = ctp.analyze_directory('src/')
high_drift = [r for r in results if r.drift.drift_severity in ['HIGH', 'CRITICAL']]

# Generate report
for r in high_drift:
    print(f"{r.module.path}: {r.drift.drift_severity}")
```

### Editor Integration

```typescript
// VS Code extension example
import { CodeTruth } from '@codetruth/sdk';

const ctp = new CodeTruth();

vscode.workspace.onDidSaveTextDocument(async (doc) => {
  const result = await ctp.analyzeString(
    doc.getText(),
    getLanguage(doc),
    doc.fileName
  );
  
  updateDiagnostics(doc, result);
});
```

## Detailed Guides

- [TypeScript/Node.js SDK](./sdk-typescript.md)
- [Python SDK](./sdk-python.md)
