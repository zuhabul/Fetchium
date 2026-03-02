# Installation

Fetchium (`fetchium`) is available via npm, Cargo, or as a pre-built binary.

## npm (Recommended)

The easiest way to install on any platform:

```bash
npm install -g fetchium
fetchium --version
```

This downloads a pre-built binary for your platform (Linux x64/ARM64, macOS x64/ARM64, Windows x64).

## Cargo

Install from crates.io (requires Rust 1.75+):

```bash
cargo install fetchium-cli
fetchium --version
```

## Pre-built Binaries

Download the latest release from [GitHub Releases](https://github.com/fetchium/fetchium/releases).

| Platform | File |
|----------|------|
| Linux x64 | `fetchium-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 | `fetchium-aarch64-unknown-linux-gnu.tar.gz` |
| macOS x64 | `fetchium-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 | `fetchium-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `fetchium-x86_64-pc-windows-msvc.zip` |

```bash
# Example: macOS ARM64
curl -fsSL https://github.com/fetchium/fetchium/releases/latest/download/fetchium-aarch64-apple-darwin.tar.gz | tar xz
sudo mv fetchium /usr/local/bin/
fetchium --version
```

## Optional Dependencies

| Tool | Purpose | Install |
|------|---------|---------|
| [Ollama](https://ollama.com) | Local AI synthesis (`fetchium ai`, `fetchium deep`) | `brew install ollama` |
| [Chromium](https://chromium.org) | JavaScript-rendered pages (CEP Layer 3) | `brew install chromium` |
| [Pandoc](https://pandoc.org) | PDF/DOCX export (`fetchium export`) | `brew install pandoc` |
| [Typst](https://typst.app) | Fast PDF export (preferred over LaTeX) | `brew install typst` |
| [Tesseract](https://github.com/tesseract-ocr/tesseract) | OCR for image-heavy pages (CEP Layer 5) | `brew install tesseract` |

Run `fetchium doctor` to check which tools are available on your system.

## Configuration

Fetchium reads configuration from `~/.fetchium/config.toml`:

```toml
[general]
max_results = 10

[fetch]
timeout_secs = 15

[ai]
ollama_host = "http://localhost:11434"
default_model = "deepseek-r1:7b"
```

See [Configuration](configuration.md) for all options.
