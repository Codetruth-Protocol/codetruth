# IDE Extensions

CTP provides IDE extensions for real-time code analysis.

## Overview

IDE extensions provide:
- Real-time drift detection
- Inline intent annotations
- Policy violation warnings
- Quick fixes and suggestions

## Supported IDEs

| IDE | Status | Install |
|-----|--------|---------|
| VS Code | ✅ Available | [Marketplace](https://marketplace.visualstudio.com/items?itemName=codetruth.ctp) |
| JetBrains | 🚧 Beta | [Plugin Repository](https://plugins.jetbrains.com/plugin/codetruth) |
| Neovim | 🚧 Beta | LSP configuration |
| Emacs | 🚧 Beta | LSP configuration |

## Features

### Inline Annotations

See intent and drift inline:

```python
def process_payment(amount):  # CTP: Intent matches ✓
    """Process a payment transaction."""
    # ...
```

### Diagnostics

Policy violations appear as warnings:

```
⚠️ CTP: Function missing documentation (documentation-required)
```

### Hover Information

Hover over functions to see:
- Declared intent
- Inferred intent
- Confidence score
- Side effects

### Code Actions

Quick fixes for common issues:
- Generate documentation template
- Add error handling
- Mark as intentional

## Installation

### VS Code

1. Open Extensions (Ctrl+Shift+X)
2. Search "CodeTruth"
3. Click Install

Or via command line:

```bash
code --install-extension codetruth.ctp
```

### JetBrains

1. Open Settings > Plugins
2. Search "CodeTruth"
3. Click Install

### LSP (Neovim/Emacs)

Start the LSP server:

```bash
ctp lsp --port 9999
```

Configure your editor to connect.

## Configuration

### VS Code Settings

```json
{
  "codetruth.enable": true,
  "codetruth.analyzeOnSave": true,
  "codetruth.minDriftLevel": "low",
  "codetruth.showInlineAnnotations": true,
  "codetruth.enableLLM": false
}
```

### JetBrains Settings

Settings > Tools > CodeTruth

## Detailed Guides

- [VS Code Extension](./vscode.md)
- [JetBrains Plugin](./jetbrains.md)
