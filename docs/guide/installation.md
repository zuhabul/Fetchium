# Installation

Fetchium (`fetchium`) installs via Cargo, from source, or as a prebuilt binary.
All Cargo/source methods require [Rust 1.75+](https://rustup.rs).

## With Cargo (recommended)

```bash
cargo install --git https://github.com/zuhabul/Fetchium fetchium-cli
fetchium --version
```

## From source

```bash
git clone https://github.com/zuhabul/Fetchium
cd Fetchium
cargo build -p fetchium-cli --release
./target/release/fetchium --version
```

## Prebuilt binary (Linux x86-64)

Download the latest from [GitHub Releases](https://github.com/zuhabul/Fetchium/releases/latest):

| Platform | File |
|----------|------|
| Linux x86-64 | `fetchium-linux-x64.tar.gz` |

```bash
curl -fsSL https://github.com/zuhabul/Fetchium/releases/latest/download/fetchium-linux-x64.tar.gz | tar xz
sudo mv fetchium /usr/local/bin/
fetchium --version
```

> Additional prebuilt targets (macOS, Windows, ARM) and registry distribution (crates.io, npm,
> Homebrew) are produced by the release pipeline and will be published as they are enabled. Until
> then, use Cargo or build from source on those platforms.

## Optional Dependencies

| Tool | Purpose | Install |
|------|---------|---------|
| [Ollama](https://ollama.com) | Local AI synthesis (`fetchium ai`, `fetchium deep`) | `brew install ollama` |
| [Chromium](https://chromium.org) | JavaScript-rendered pages (CEP Layer 3) | `brew install chromium` |
| [Pandoc](https://pandoc.org) | Advanced document conversion for research workflows | `brew install pandoc` |
| [Typst](https://typst.app) | Fast PDF/report generation in custom pipelines | `brew install typst` |
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
