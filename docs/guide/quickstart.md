# Quickstart

Get from zero to your first search in under 60 seconds.

## 1. Install

**Shell installer** (Linux + macOS):
```bash
curl -sSfL https://install.fetchium.com | sh
```

**From crates.io**:
```bash
cargo install fetchium-cli
```

**npm / npx**:
```bash
npm install -g fetchium-cli
npx fetchium-cli --help
```

**macOS via Homebrew**:
```bash
brew install zuhabul/fetchium/fetchium
```

**From source**:
```bash
git clone https://github.com/zuhabul/Fetchium
cd Fetchium
cargo build --release -p fetchium-cli
```

## 2. Your First Search

```bash
fetchium search "Rust async runtime comparison"
```

Output: Ranked results with titles, URLs, snippets, and relevance scores.

## 3. Fetch a URL

```bash
fetchium fetch https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html
```

Output: Clean structured content with boilerplate removed (CEP extraction).

## 4. Research with Citations

```bash
fetchium research "tradeoffs between async Rust runtimes in 2026"
```

Output: Multi-source cited report with evidence validation. Add `--no-ai` to skip AI synthesis.

## 5. YouTube Intelligence

```bash
# Search YouTube and rank by educational value
fetchium youtube search "Rust programming tutorial" -n 5

# Extract transcript from any video
fetchium youtube transcript https://www.youtube.com/watch?v=dQw4w9WgXcQ

# Full video analysis (metadata, engagement, credibility)
fetchium youtube analyze https://www.youtube.com/watch?v=dQw4w9WgXcQ
```

## 6. Social Media Intelligence

```bash
# Unified search across all platforms at once
fetchium social "open source database tools"

# Platform-specific
fetchium reddit search "mechanical keyboards" -n 10
fetchium hackernews search "systems programming" -n 5
fetchium twitter search "Rust 2026"
fetchium tiktok search "coding tips" -n 5

# Transcribe audio/video from any URL
fetchium transcribe https://www.youtube.com/watch?v=dQw4w9WgXcQ
```

## 7. Compare & Deep Research

```bash
# Side-by-side structured comparison
fetchium compare "tokio vs async-std vs smol"

# Deep multi-agent research (Mode E)
fetchium deep "history of memory-safe systems languages" --max-depth 3

# AI-grounded answer (requires Ollama or other AI provider)
fetchium ai "what causes the northern lights?"
```

## 8. Monitor & Productivity

```bash
# Watch a URL for content changes
fetchium monitor add https://blog.rust-lang.org
fetchium monitor check
fetchium monitor diff

# Generate a research digest
fetchium digest "AI and Rust weekly"

# Personalized radar from your history
fetchium radar
```

## 9. Agent Mode (for AI frameworks)

```bash
# Token-budgeted JSON output for LLM consumption
fetchium agent-search "latest Rust async developments" --budget 2000
fetchium agent-fetch https://blog.rust-lang.org
fetchium agent-research "Rust async runtimes"
```

## 10. REST API & MCP Server

```bash
# Start REST API (port 3000 by default)
fetchium serve

# Start MCP server (stdio — for Codex, Claude, etc.)
fetchium serve --mode mcp

# Start both
fetchium serve --mode both --port 3000
```

## 11. Check Your Setup

```bash
fetchium doctor
```

Shows available AI providers, search backends, system resources, and configuration status.

## Next Steps

- [Command Reference](commands.md) — full 30+ command surface
- [Configuration](configuration.md) — API keys, token budgets, backends
- [Agent Integration](agent-integration.md) — MCP, LangChain, CrewAI, REST
