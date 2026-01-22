# VS Code Extension

Detailed guide for the CodeTruth VS Code extension.

## Installation

### From Marketplace

1. Open VS Code
2. Go to Extensions (Ctrl+Shift+X)
3. Search "CodeTruth"
4. Click Install

### From VSIX

```bash
code --install-extension codetruth-vscode-0.1.0.vsix
```

### From Source

```bash
git clone https://github.com/codetruth/codetruth
cd integrations/vscode
npm install
npm run package
code --install-extension codetruth-*.vsix
```

## Features

### Real-time Analysis

The extension analyzes files as you edit:

- Drift detection on save
- Inline annotations
- Problems panel integration

### Inline Annotations

See intent and drift status inline:

```python
def process_payment(amount):  # ✓ Intent: Process payment transaction
    """Process a payment transaction."""
    return charge(amount)
```

### Hover Information

Hover over any function to see:

```
CodeTruth Analysis
─────────────────────
Intent: Process payment transaction
Confidence: 92%
Drift: NONE

Side Effects:
• Database write (HIGH risk)
• Network call (MEDIUM risk)
```

### Diagnostics

Issues appear in the Problems panel:

```
⚠ CTP001: Function missing documentation [Ln 45, Col 1]
⚠ CTP002: Undocumented side effects detected [Ln 52, Col 5]
```

### Code Actions

Quick fixes available via lightbulb:

- **Generate Documentation** - Create docstring template
- **Add Error Handling** - Wrap in try/catch
- **Mark as Intentional** - Add CTP ignore comment
- **View Full Analysis** - Open explanation graph

### Commands

Access via Command Palette (Ctrl+Shift+P):

| Command | Description |
|---------|-------------|
| `CTP: Analyze Current File` | Analyze active file |
| `CTP: Analyze Workspace` | Analyze all files |
| `CTP: Check Policies` | Run policy check |
| `CTP: Show Explanation Graph` | View full analysis |
| `CTP: Toggle Annotations` | Show/hide inline annotations |

## Configuration

### Settings

Open Settings (Ctrl+,) and search "codetruth":

```json
{
  // Enable/disable extension
  "codetruth.enable": true,
  
  // Analyze on file save
  "codetruth.analyzeOnSave": true,
  
  // Analyze on file open
  "codetruth.analyzeOnOpen": false,
  
  // Minimum drift level to show
  "codetruth.minDriftLevel": "low",
  
  // Show inline annotations
  "codetruth.showInlineAnnotations": true,
  
  // Show hover information
  "codetruth.showHoverInfo": true,
  
  // Enable LLM enhancement
  "codetruth.enableLLM": false,
  
  // LLM provider
  "codetruth.llmProvider": "anthropic",
  
  // Languages to analyze
  "codetruth.languages": [
    "python",
    "javascript",
    "typescript",
    "rust",
    "go"
  ],
  
  // Files to exclude
  "codetruth.exclude": [
    "**/node_modules/**",
    "**/dist/**",
    "**/*.test.*"
  ],
  
  // Path to CTP binary
  "codetruth.binaryPath": "",
  
  // Path to config file
  "codetruth.configPath": ".ctp/config.yaml"
}
```

### Workspace Settings

Create `.vscode/settings.json`:

```json
{
  "codetruth.enable": true,
  "codetruth.minDriftLevel": "medium",
  "codetruth.exclude": [
    "**/generated/**"
  ]
}
```

## Status Bar

The status bar shows:

- **CTP ✓** - No drift detected
- **CTP ○** - Low drift
- **CTP ●** - Medium drift
- **CTP ✗** - High/critical drift

Click to see summary.

## Output Panel

View detailed logs in Output > CodeTruth:

```
[CTP] Analyzing src/handler.py...
[CTP] Found 3 functions, 2 classes
[CTP] Drift: NONE (confidence: 92%)
[CTP] Analysis complete in 45ms
```

## Keyboard Shortcuts

| Shortcut | Command |
|----------|---------|
| `Ctrl+Shift+C A` | Analyze current file |
| `Ctrl+Shift+C G` | Show explanation graph |
| `Ctrl+Shift+C T` | Toggle annotations |

Customize in Keyboard Shortcuts (Ctrl+K Ctrl+S).

## Troubleshooting

### Extension Not Working

1. Check Output panel for errors
2. Verify CTP is installed: `ctp --version`
3. Check `codetruth.binaryPath` setting

### Slow Analysis

1. Exclude large directories
2. Disable `analyzeOnOpen`
3. Increase `minDriftLevel`

### LLM Errors

1. Check API key in settings
2. Verify network connectivity
3. Check Output panel for details

### High CPU Usage

1. Reduce analyzed languages
2. Add more exclusions
3. Disable real-time analysis

## Uninstall

1. Go to Extensions
2. Find CodeTruth
3. Click Uninstall

Or via command line:

```bash
code --uninstall-extension codetruth.ctp
```
