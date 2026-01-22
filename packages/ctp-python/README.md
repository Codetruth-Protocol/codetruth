# CodeTruth Python SDK

Python bindings for the CodeTruth Protocol (CTP).

## Installation

```bash
pip install codetruth
```

## Quick Start

```python
from codetruth import CTPAnalyzer

analyzer = CTPAnalyzer()

# Analyze a file
result = analyzer.analyze_file("src/main.py")
print(f"Intent: {result.intent.inferred_intent}")
print(f"Drift: {result.drift.drift_severity}")

# Analyze code string
result = analyzer.analyze_code('''
def factorial(n):
    """Calculate factorial of n."""
    if n <= 1:
        return 1
    return n * factorial(n - 1)
''', language="python")

# Minimal analysis (faster)
minimal = analyzer.analyze_minimal("src/main.py")
print(f"Drift: {minimal.drift}")
```

## Using the API Client

```python
from codetruth import CTPClient

client = CTPClient(base_url="http://localhost:9999")
result = client.analyze("src/main.py")
```

## License

MIT
