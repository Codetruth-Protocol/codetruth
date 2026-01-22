# Installation

## System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| OS | Windows 10, macOS 10.15, Linux (glibc 2.17+) | Latest stable |
| RAM | 512 MB | 2 GB |
| Disk | 50 MB | 100 MB |
| Rust | 1.75+ (for building) | Latest stable |

## Pre-built Binaries

### macOS

**Homebrew:**
```bash
brew install codetruth/tap/ctp
```

**Direct Download:**
```bash
curl -fsSL https://codetruth.dev/install.sh | sh
```

### Linux

**Debian/Ubuntu:**
```bash
curl -fsSL https://codetruth.dev/install.sh | sh
```

**Arch Linux (AUR):**
```bash
yay -S ctp-bin
```

**Alpine Linux:**
```bash
apk add --no-cache ctp
```

### Windows

**PowerShell:**
```powershell
irm https://codetruth.dev/install.ps1 | iex
```

**Scoop:**
```powershell
scoop bucket add codetruth https://github.com/codetruth/scoop-bucket
scoop install ctp
```

**Chocolatey:**
```powershell
choco install ctp
```

## Cargo Install

If you have Rust installed:

```bash
cargo install ctp-cli
```

To install with all language support:

```bash
cargo install ctp-cli --features all-languages
```

## Building from Source

### Clone the Repository

```bash
git clone https://github.com/codetruth/codetruth.git
cd codetruth
```

### Build Release Binary

```bash
cargo build --release
```

### Install Locally

```bash
cargo install --path crates/ctp-cli
```

### Feature Flags

| Feature | Description |
|---------|-------------|
| `python` | Python language support (default) |
| `javascript` | JavaScript support (default) |
| `typescript` | TypeScript support (default) |
| `rust-lang` | Rust language support |
| `go` | Go language support |
| `java` | Java language support |
| `llm` | LLM enhancement support |
| `all-languages` | All language parsers |

Example with specific features:

```bash
cargo build --release --features "python,rust-lang,llm"
```

## SDK Installation

### TypeScript/Node.js

```bash
npm install @codetruth/sdk
# or
yarn add @codetruth/sdk
# or
pnpm add @codetruth/sdk
```

### Python

```bash
pip install codetruth
# or
poetry add codetruth
```

## Docker

```bash
docker pull codetruth/ctp:latest

# Run analysis
docker run --rm -v $(pwd):/workspace codetruth/ctp analyze /workspace/src
```

## Verifying Installation

```bash
# Check version
ctp --version

# Run self-test
ctp --help

# Analyze a test file
echo 'def hello(): """Say hello."""; print("Hello!")' > test.py
ctp analyze test.py
rm test.py
```

## Troubleshooting

### "Command not found"

Ensure the installation directory is in your PATH:

```bash
# Unix
export PATH="$HOME/.codetruth/bin:$PATH"

# Windows (PowerShell)
$env:PATH += ";$env:USERPROFILE\.codetruth\bin"
```

### Permission Denied

On Unix systems, you may need to make the binary executable:

```bash
chmod +x ~/.codetruth/bin/ctp
```

### Missing Language Support

If you get "Unsupported language" errors, rebuild with the appropriate feature:

```bash
cargo install ctp-cli --features all-languages
```
