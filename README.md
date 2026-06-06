<div align="center">

```
███████╗███████╗████████╗ ██████╗██╗  ██╗██╗██╗   ██╗███╗   ███╗
██╔════╝██╔════╝╚══██╔══╝██╔════╝██║  ██║██║██║   ██║████╗ ████║
█████╗  █████╗     ██║   ██║     ███████║██║██║   ██║██╔████╔██║
██╔══╝  ██╔══╝     ██║   ██║     ██╔══██║██║██║   ██║██║╚██╔╝██║
██║     ███████╗   ██║   ╚██████╗██║  ██║██║╚██████╔╝██║ ╚═╝ ██║
╚═╝     ╚══════╝   ╚═╝    ╚═════╝╚═╝  ╚═╝╚═╝ ╚═════╝ ╚═╝     ╚═╝
```

### The universal retrieval layer for humans and AI agents

Rust-native search, fetch, extract, and synthesis — as a CLI, a REST API, and an MCP server.

[![CI](https://github.com/zuhabul/Fetchium/actions/workflows/ci.yml/badge.svg)](https://github.com/zuhabul/Fetchium/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust 1.75+](https://img.shields.io/badge/rust-1.75%2B-orange.svg?logo=rust)](https://www.rust-lang.org)

[Install](#installation) · [Quick start](#quick-start) · [For AI agents](#for-ai-agents) · [Configuration](#configuration) · [Docs](docs/) · [Contributing](CONTRIBUTING.md)

</div>

---

## What is Fetchium?

Fetchium finds, fetches, and makes sense of information from the open web. It runs as a single
binary and exposes the same engine three ways:

- **CLI** — search the web, extract clean content from any URL, summarize, compare, and run
  multi-step research from your terminal.
- **REST API** — call the engine over HTTP from any language.
- **MCP server** — plug it into AI coding agents (Codex, Claude, and other MCP clients) as a tool.

It is written in Rust, has no required runtime dependencies, and is dual-licensed MIT/Apache-2.0.

## Installation

> Requires [Rust 1.75+](https://rustup.rs) for the source/Cargo methods.

**With Cargo (recommended)**

```bash
cargo install --git https://github.com/zuhabul/Fetchium fetchium-cli
fetchium --version
```

**From source**

```bash
git clone https://github.com/zuhabul/Fetchium
cd Fetchium
cargo build -p fetchium-cli --release
./target/release/fetchium --version
```

**Prebuilt binary (Linux x86-64)** — from the [latest release](https://github.com/zuhabul/Fetchium/releases/latest):

```bash
curl -fsSL https://github.com/zuhabul/Fetchium/releases/latest/download/fetchium-linux-x64.tar.gz | tar xz
sudo mv fetchium /usr/local/bin/
fetchium --version
```

> Publishing to crates.io, npm, and Homebrew is wired through the release pipeline and will be
> enabled as those registries are configured. Run `fetchium doctor` to check optional tools.

## Quick start

```bash
# Web search across multiple backends
fetchium search "best rust async runtimes"

# Fetch and extract clean content from a URL
fetchium fetch https://example.com

# Summarize a URL or text
fetchium summarize https://example.com

# Compare options (generates a structured comparison)
fetchium compare "rust vs go vs python"

# Multi-step research report
fetchium research "impact of LLMs on software engineering"

# AI answer grounded in retrieved sources (needs an AI provider — see Configuration)
fetchium ai "what causes the northern lights?"

# Platform-specific retrieval
fetchium reddit search "mechanical keyboards"
fetchium hackernews top
fetchium youtube transcript https://www.youtube.com/watch?v=...
```

Run `fetchium --help` for the full command list, or see [docs/guide/commands.md](docs/guide/commands.md).

## For AI agents

Fetchium is built to be called by agents, not just humans.

- **MCP server** (`fetchium-mcp`) exposes retrieval as Model Context Protocol tools, so MCP-aware
  agents (Codex, Claude, and others) can search and fetch with citations.
- **REST API** (`fetchium-api`) serves the same engine over HTTP — start it with `fetchium serve`.
- **Framework adapters** for [LangChain](adapters/langchain) and [CrewAI](adapters/crewai) are
  included under `adapters/`.

See [docs/guide/agent-integration.md](docs/guide/agent-integration.md) for setup.

## Configuration

Fetchium reads configuration from `~/.fetchium/config.toml` (with environment-variable overrides).
API keys you provide are stored locally and are never committed to the repository.

```bash
fetchium doctor   # check installed optional tools and provider setup
fetchium config   # view/edit configuration
```

Optional integrations (only needed for specific features): an AI provider such as
[Ollama](https://ollama.com) or a hosted model for `ai`/`research`/`deep`, and
[Chromium](https://www.chromium.org) for JavaScript-rendered pages. Details in
[docs/guide/configuration.md](docs/guide/configuration.md).

## Architecture

Fetchium is a Cargo workspace:

| Crate | Role |
|-------|------|
| [`fetchium-core`](crates/fetchium-core) | The engine: search, extract, rank, validate, cache |
| [`fetchium-cli`](crates/fetchium-cli)   | The `fetchium` command-line binary |
| [`fetchium-mcp`](crates/fetchium-mcp)   | Model Context Protocol server for AI agents |
| [`fetchium-api`](crates/fetchium-api)   | REST API server |

Architecture notes live in [docs/architecture/](docs/architecture/).

## Contributing

Contributions are welcome — see [CONTRIBUTING.md](CONTRIBUTING.md) for setup, the workspace map,
and the exact checks CI runs. Please also read our [Code of Conduct](CODE_OF_CONDUCT.md). For
security reports, see [SECURITY.md](SECURITY.md).

## License

Licensed under either of

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option. This dual license is the Rust ecosystem standard: MIT is short and permissive,
while Apache-2.0 adds an explicit patent grant. Downstream users may choose whichever fits their
needs.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
this project by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without
any additional terms or conditions.
