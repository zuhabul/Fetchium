# Quickstart

Get from zero to your first search in under 60 seconds.

## 1. Install

**From crates.io** (recommended):
```bash
cargo install fetchium-cli
```

**From source** (latest main branch):
```bash
git clone https://github.com/zuhabul/Fetchium
cd Fetchium
cargo build --release -p fetchium-cli
# Binary is at ./target/release/fetchium
```

**macOS via Homebrew** (when formula is available):
```bash
brew install zuhabul/fetchium/fetchium
```

## 2. Your First Search

```bash
fetchium search "Rust async runtime comparison"
```

Output: Ranked search results with titles, URLs, and snippets.

## 3. Fetch a URL

```bash
fetchium fetch https://doc.rust-lang.org/book/ch04-01.html
```

Output: Clean, structured content with boilerplate removed.

## 4. AI-Powered Research

First, start Ollama:
```bash
ollama serve
ollama pull deepseek-r1:7b
```

Then:
```bash
fetchium research "What are the tradeoffs between async Rust runtimes in 2026?"
```

Output: Multi-source research report with citations.

## 5. Agent Mode (for AI frameworks)

```bash
# Machine-readable JSON output for LLM consumption
fetchium agent-search "latest Rust async developments" --budget 2000

# Fetch with semantic extraction
fetchium agent-fetch https://blog.rust-lang.org/inside-rust/2026/01/tokio-update
```

## 6. Check Your Setup

```bash
fetchium doctor
```

Shows available tools, system resources, and configuration status.

## Next Steps

- [Command Reference](commands.md) — all 25+ commands with examples
- [Configuration](configuration.md) — customize behavior
- [Agent Integration](agent-integration.md) — use with Claude, LangChain, CrewAI
