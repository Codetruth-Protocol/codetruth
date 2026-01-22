# Python SDK

Complete guide for the CodeTruth Python SDK.

## Installation

```bash
pip install codetruth
# or
poetry add codetruth
# or
pipx install codetruth
```

## Quick Start

```python
from codetruth import CodeTruth

ctp = CodeTruth()

# Analyze a file
result = ctp.analyze_file('src/handler.py')
print(result.drift.drift_severity)
```

## Configuration

```python
from codetruth import CodeTruth, Config

config = Config(
    # API key for cloud features
    api_key=os.environ.get('CTP_API_KEY'),
    
    # Use local binary
    use_local_binary=True,
    binary_path='/usr/local/bin/ctp',
    
    # LLM settings
    enable_llm=False,
    llm_provider='anthropic',
    llm_api_key=os.environ.get('ANTHROPIC_API_KEY'),
    
    # Analysis settings
    min_drift_level='low',
    max_file_size=10 * 1024 * 1024,
)

ctp = CodeTruth(config)
```

## API Reference

### CodeTruth Class

#### Constructor

```python
CodeTruth(config: Optional[Config] = None)
```

#### Methods

##### analyze_file

Analyze a single file.

```python
def analyze_file(self, path: str) -> ExplanationGraph
```

```python
result = ctp.analyze_file('src/handler.py')
```

##### analyze_string

Analyze code from a string.

```python
def analyze_string(
    self,
    content: str,
    language: str,
    name: Optional[str] = None
) -> ExplanationGraph
```

```python
code = 'def hello(): print("Hello")'
result = ctp.analyze_string(code, 'python', 'hello.py')
```

##### analyze_directory

Analyze all files in a directory.

```python
def analyze_directory(
    self,
    path: str,
    recursive: bool = True,
    exclude: Optional[List[str]] = None,
    languages: Optional[List[str]] = None
) -> List[ExplanationGraph]
```

```python
results = ctp.analyze_directory(
    'src/',
    recursive=True,
    exclude=['**/test/**'],
    languages=['python', 'javascript']
)
```

##### check_policies

Check policy compliance.

```python
def check_policies(
    self,
    files: List[FileInput],
    policies: List[str]
) -> PolicyCheckResult
```

```python
result = ctp.check_policies(
    files=[FileInput(path='src/handler.py', content='...')],
    policies=['documentation-required']
)
```

##### analyze_stream

Stream analysis results.

```python
def analyze_stream(
    self,
    path: str,
    **options
) -> Iterator[AnalysisUpdate]
```

```python
for update in ctp.analyze_stream('src/'):
    if update.type == 'progress':
        print(f"Progress: {update.progress}%")
    elif update.type == 'result':
        print(f"Analyzed: {update.file}")
```

### Data Classes

```python
from dataclasses import dataclass
from typing import List, Optional
from enum import Enum

class DriftSeverity(Enum):
    NONE = "NONE"
    LOW = "LOW"
    MEDIUM = "MEDIUM"
    HIGH = "HIGH"
    CRITICAL = "CRITICAL"

@dataclass
class ExplanationGraph:
    ctp_version: str
    explanation_id: str
    module: Module
    intent: Intent
    behavior: Behavior
    drift: DriftAnalysis
    policies: PolicyResults
    history: History
    metadata: Metadata

@dataclass
class Module:
    name: str
    path: str
    language: str
    lines_of_code: int
    complexity_score: float

@dataclass
class Intent:
    declared_intent: str
    inferred_intent: str
    confidence: float
    business_context: str
    technical_rationale: str

@dataclass
class DriftAnalysis:
    drift_detected: bool
    drift_severity: DriftSeverity
    drift_details: List[DriftDetail]
```

## Examples

### CI/CD Integration

```python
#!/usr/bin/env python3
import sys
from codetruth import CodeTruth

def ci_check():
    ctp = CodeTruth()
    
    results = ctp.analyze_directory('src/')
    
    critical = [r for r in results if r.drift.drift_severity.value == 'CRITICAL']
    
    if critical:
        print("Critical drift detected:")
        for r in critical:
            print(f"  {r.module.path}: {r.drift.drift_severity.value}")
        sys.exit(1)
    
    print("All checks passed!")

if __name__ == '__main__':
    ci_check()
```

### Pre-commit Hook

```python
#!/usr/bin/env python3
"""Pre-commit hook for CTP analysis."""
import subprocess
import sys
from codetruth import CodeTruth

def get_staged_files():
    result = subprocess.run(
        ['git', 'diff', '--cached', '--name-only', '--diff-filter=ACM'],
        capture_output=True, text=True
    )
    return [f for f in result.stdout.strip().split('\n') if f.endswith(('.py', '.js', '.ts'))]

def main():
    files = get_staged_files()
    if not files:
        return 0
    
    ctp = CodeTruth()
    
    for file in files:
        try:
            result = ctp.analyze_file(file)
            if result.drift.drift_severity.value in ['HIGH', 'CRITICAL']:
                print(f"❌ {file}: {result.drift.drift_severity.value} drift")
                return 1
        except Exception as e:
            print(f"⚠️ {file}: {e}")
    
    print("✅ All files passed CTP check")
    return 0

if __name__ == '__main__':
    sys.exit(main())
```

### Custom Report Generator

```python
import json
from datetime import datetime
from codetruth import CodeTruth

def generate_report(directory: str, output: str = 'ctp-report.json'):
    ctp = CodeTruth()
    results = ctp.analyze_directory(directory)
    
    severity_counts = {}
    for r in results:
        sev = r.drift.drift_severity.value
        severity_counts[sev] = severity_counts.get(sev, 0) + 1
    
    report = {
        'generated_at': datetime.now().isoformat(),
        'summary': {
            'total_files': len(results),
            'drift_detected': sum(1 for r in results if r.drift.drift_detected),
            'by_severity': severity_counts,
        },
        'files': [
            {
                'path': r.module.path,
                'language': r.module.language,
                'drift': r.drift.drift_severity.value,
                'confidence': r.intent.confidence,
            }
            for r in results
        ],
    }
    
    with open(output, 'w') as f:
        json.dump(report, f, indent=2)
    
    print(f"Report generated: {output}")

if __name__ == '__main__':
    generate_report('src/')
```

### Jupyter Notebook Integration

```python
from codetruth import CodeTruth
from IPython.display import display, HTML

ctp = CodeTruth()

def analyze_and_display(code: str, language: str = 'python'):
    result = ctp.analyze_string(code, language)
    
    color = {
        'NONE': 'green',
        'LOW': 'yellow',
        'MEDIUM': 'orange',
        'HIGH': 'red',
        'CRITICAL': 'darkred',
    }[result.drift.drift_severity.value]
    
    html = f"""
    <div style="border: 1px solid {color}; padding: 10px; margin: 10px 0;">
        <h4>CTP Analysis</h4>
        <p><b>Intent:</b> {result.intent.inferred_intent}</p>
        <p><b>Confidence:</b> {result.intent.confidence:.0%}</p>
        <p><b>Drift:</b> <span style="color: {color}">{result.drift.drift_severity.value}</span></p>
    </div>
    """
    display(HTML(html))

# Usage in notebook cell:
code = '''
def factorial(n):
    """Calculate factorial."""
    if n <= 1:
        return 1
    return n * factorial(n - 1)
'''
analyze_and_display(code)
```

## Error Handling

```python
from codetruth import CodeTruth, CTPError, ParseError, UnsupportedLanguageError

ctp = CodeTruth()

try:
    result = ctp.analyze_file('nonexistent.py')
except FileNotFoundError:
    print("File not found")
except ParseError as e:
    print(f"Failed to parse: {e}")
except UnsupportedLanguageError as e:
    print(f"Language not supported: {e}")
except CTPError as e:
    print(f"CTP error: {e}")
```

## Async Support

```python
import asyncio
from codetruth import AsyncCodeTruth

async def main():
    ctp = AsyncCodeTruth()
    
    # Analyze multiple files concurrently
    files = ['src/a.py', 'src/b.py', 'src/c.py']
    results = await asyncio.gather(*[
        ctp.analyze_file(f) for f in files
    ])
    
    for result in results:
        print(f"{result.module.path}: {result.drift.drift_severity.value}")

asyncio.run(main())
```
