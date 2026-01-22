# JetBrains Plugin

Guide for the CodeTruth JetBrains plugin (IntelliJ IDEA, PyCharm, WebStorm, etc.).

## Installation

### From Plugin Repository

1. Open Settings (Ctrl+Alt+S)
2. Go to Plugins > Marketplace
3. Search "CodeTruth"
4. Click Install
5. Restart IDE

### From Disk

1. Download `.zip` from releases
2. Settings > Plugins > ⚙️ > Install from Disk
3. Select the `.zip` file
4. Restart IDE

## Supported IDEs

- IntelliJ IDEA (2023.1+)
- PyCharm (2023.1+)
- WebStorm (2023.1+)
- GoLand (2023.1+)
- RustRover (2023.1+)
- CLion (2023.1+)

## Features

### Editor Annotations

Gutter icons show drift status:

- ✓ Green: No drift
- ○ Yellow: Low drift
- ● Orange: Medium drift
- ✗ Red: High/critical drift

### Inspections

CTP issues appear as inspections:

```
⚠ CTP: Function missing documentation
   Inspection: CodeTruth > Documentation Required
```

### Quick Fixes

Alt+Enter on issues for fixes:

- Generate documentation
- Add error handling
- Suppress warning

### Tool Window

View > Tool Windows > CodeTruth

Shows:
- Current file analysis
- Workspace summary
- Policy violations
- History

### Actions

Find Action (Ctrl+Shift+A):

| Action | Description |
|--------|-------------|
| Analyze File | Analyze current file |
| Analyze Project | Analyze entire project |
| Check Policies | Run policy check |
| Show Explanation | View full analysis |

## Configuration

### Settings

Settings > Tools > CodeTruth

| Setting | Default | Description |
|---------|---------|-------------|
| Enable | true | Enable plugin |
| Analyze on Save | true | Auto-analyze on save |
| Min Drift Level | LOW | Minimum level to show |
| Show Gutter Icons | true | Show status icons |
| Enable LLM | false | Use LLM enhancement |

### Project Settings

Create `.idea/codetruth.xml`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<project version="4">
  <component name="CodeTruthSettings">
    <option name="enabled" value="true" />
    <option name="minDriftLevel" value="MEDIUM" />
    <option name="excludePatterns">
      <list>
        <option value="**/generated/**" />
      </list>
    </option>
  </component>
</project>
```

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Alt+C` | Analyze current file |
| `Ctrl+Alt+Shift+C` | Analyze project |

Customize in Settings > Keymap.

## Integration

### Run Configurations

Add CTP check to run configurations:

1. Run > Edit Configurations
2. Add > CodeTruth Check
3. Configure paths and options

### Build Integration

Add to build process:

```groovy
// build.gradle.kts
tasks.register("ctpCheck") {
    doLast {
        exec {
            commandLine("ctp", "check", "--fail-on-violation")
        }
    }
}

tasks.named("check") {
    dependsOn("ctpCheck")
}
```

## Troubleshooting

### Plugin Not Loading

1. Check IDE version compatibility
2. View > Tool Windows > Event Log
3. Help > Show Log in Explorer

### Slow Performance

1. Increase excluded patterns
2. Disable "Analyze on Save"
3. Increase min drift level

### CTP Not Found

Set binary path in settings:
- Settings > Tools > CodeTruth > Binary Path

## Uninstall

1. Settings > Plugins
2. Find CodeTruth
3. Click ⚙️ > Uninstall
4. Restart IDE
