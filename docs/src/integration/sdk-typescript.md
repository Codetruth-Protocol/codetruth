# TypeScript/Node.js SDK

Complete guide for the CodeTruth TypeScript SDK.

## Installation

```bash
npm install @codetruth/sdk
# or
yarn add @codetruth/sdk
# or
pnpm add @codetruth/sdk
```

## Quick Start

```typescript
import { CodeTruth } from '@codetruth/sdk';

const ctp = new CodeTruth();

// Analyze a file
const result = await ctp.analyzeFile('src/handler.py');
console.log(result.drift.drift_severity);
```

## Configuration

```typescript
import { CodeTruth, CodeTruthConfig } from '@codetruth/sdk';

const config: CodeTruthConfig = {
  // API key for cloud features
  apiKey: process.env.CTP_API_KEY,
  
  // Use local binary instead of WASM
  useLocalBinary: true,
  binaryPath: '/usr/local/bin/ctp',
  
  // LLM settings
  enableLLM: false,
  llmProvider: 'anthropic',
  llmApiKey: process.env.ANTHROPIC_API_KEY,
  
  // Analysis settings
  minDriftLevel: 'low',
  maxFileSize: 10 * 1024 * 1024,
};

const ctp = new CodeTruth(config);
```

## API Reference

### CodeTruth Class

#### Constructor

```typescript
new CodeTruth(config?: CodeTruthConfig)
```

#### Methods

##### analyzeFile

Analyze a single file.

```typescript
async analyzeFile(path: string): Promise<ExplanationGraph>
```

```typescript
const result = await ctp.analyzeFile('src/handler.py');
```

##### analyzeString

Analyze code from a string.

```typescript
async analyzeString(
  content: string,
  language: string,
  name?: string
): Promise<ExplanationGraph>
```

```typescript
const code = `def hello(): print("Hello")`;
const result = await ctp.analyzeString(code, 'python', 'hello.py');
```

##### analyzeDirectory

Analyze all files in a directory.

```typescript
async analyzeDirectory(
  path: string,
  options?: AnalyzeOptions
): Promise<ExplanationGraph[]>
```

```typescript
const results = await ctp.analyzeDirectory('src/', {
  recursive: true,
  exclude: ['**/test/**'],
  languages: ['python', 'typescript'],
});
```

##### checkPolicies

Check policy compliance.

```typescript
async checkPolicies(
  files: FileInput[],
  policies: string[]
): Promise<PolicyCheckResult>
```

```typescript
const result = await ctp.checkPolicies(
  [{ path: 'src/handler.py', content: '...' }],
  ['documentation-required']
);
```

##### analyzeStream

Stream analysis results.

```typescript
analyzeStream(
  options: AnalyzeOptions
): AsyncGenerator<AnalysisUpdate>
```

```typescript
for await (const update of ctp.analyzeStream({ path: 'src/' })) {
  if (update.type === 'progress') {
    console.log(`Progress: ${update.progress}%`);
  } else if (update.type === 'result') {
    console.log(`Analyzed: ${update.file}`);
  }
}
```

### Types

```typescript
interface ExplanationGraph {
  ctp_version: string;
  explanation_id: string;
  module: Module;
  intent: Intent;
  behavior: Behavior;
  drift: DriftAnalysis;
  policies: PolicyResults;
  history: History;
  metadata: Metadata;
}

interface Module {
  name: string;
  path: string;
  language: string;
  lines_of_code: number;
  complexity_score: number;
}

interface Intent {
  declared_intent: string;
  inferred_intent: string;
  confidence: number;
  business_context: string;
  technical_rationale: string;
}

interface DriftAnalysis {
  drift_detected: boolean;
  drift_severity: DriftSeverity;
  drift_details: DriftDetail[];
}

type DriftSeverity = 'NONE' | 'LOW' | 'MEDIUM' | 'HIGH' | 'CRITICAL';
```

## Examples

### CI/CD Integration

```typescript
import { CodeTruth } from '@codetruth/sdk';

async function ciCheck() {
  const ctp = new CodeTruth();
  
  const results = await ctp.analyzeDirectory('src/');
  
  const criticalDrift = results.filter(
    r => r.drift.drift_severity === 'CRITICAL'
  );
  
  if (criticalDrift.length > 0) {
    console.error('Critical drift detected:');
    criticalDrift.forEach(r => {
      console.error(`  ${r.module.path}: ${r.drift.drift_severity}`);
    });
    process.exit(1);
  }
  
  console.log('All checks passed!');
}

ciCheck();
```

### VS Code Extension

```typescript
import * as vscode from 'vscode';
import { CodeTruth } from '@codetruth/sdk';

export function activate(context: vscode.ExtensionContext) {
  const ctp = new CodeTruth();
  const diagnostics = vscode.languages.createDiagnosticCollection('ctp');
  
  vscode.workspace.onDidSaveTextDocument(async (doc) => {
    const result = await ctp.analyzeString(
      doc.getText(),
      getLanguageId(doc.languageId),
      doc.fileName
    );
    
    const diags: vscode.Diagnostic[] = [];
    
    if (result.drift.drift_detected) {
      for (const detail of result.drift.drift_details) {
        diags.push(new vscode.Diagnostic(
          new vscode.Range(
            detail.location.line_start - 1, 0,
            detail.location.line_end - 1, 100
          ),
          detail.remediation,
          vscode.DiagnosticSeverity.Warning
        ));
      }
    }
    
    diagnostics.set(doc.uri, diags);
  });
}
```

### Custom Report Generator

```typescript
import { CodeTruth, ExplanationGraph } from '@codetruth/sdk';
import * as fs from 'fs';

async function generateReport(directory: string) {
  const ctp = new CodeTruth();
  const results = await ctp.analyzeDirectory(directory);
  
  const report = {
    generated_at: new Date().toISOString(),
    summary: {
      total_files: results.length,
      drift_detected: results.filter(r => r.drift.drift_detected).length,
      by_severity: {
        NONE: results.filter(r => r.drift.drift_severity === 'NONE').length,
        LOW: results.filter(r => r.drift.drift_severity === 'LOW').length,
        MEDIUM: results.filter(r => r.drift.drift_severity === 'MEDIUM').length,
        HIGH: results.filter(r => r.drift.drift_severity === 'HIGH').length,
        CRITICAL: results.filter(r => r.drift.drift_severity === 'CRITICAL').length,
      },
    },
    files: results.map(r => ({
      path: r.module.path,
      language: r.module.language,
      drift: r.drift.drift_severity,
      confidence: r.intent.confidence,
    })),
  };
  
  fs.writeFileSync('ctp-report.json', JSON.stringify(report, null, 2));
  console.log('Report generated: ctp-report.json');
}

generateReport('src/');
```

## Error Handling

```typescript
import { CodeTruth, CTPError } from '@codetruth/sdk';

try {
  const result = await ctp.analyzeFile('nonexistent.py');
} catch (error) {
  if (error instanceof CTPError) {
    switch (error.code) {
      case 'PARSE_ERROR':
        console.error('Failed to parse file:', error.message);
        break;
      case 'UNSUPPORTED_LANGUAGE':
        console.error('Language not supported:', error.message);
        break;
      default:
        console.error('CTP error:', error.message);
    }
  } else {
    throw error;
  }
}
```
