# Quickstart

Get from zero to your first search in under 60 seconds.

## 1. Install

```bash
npm install -g hypersearchx
```

## 2. Your First Search

```bash
hsx search "Rust async runtime comparison"
```

Output: Ranked search results with titles, URLs, and snippets.

## 3. Fetch a URL

```bash
hsx fetch https://doc.rust-lang.org/book/ch04-01.html
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
hsx research "What are the tradeoffs between async Rust runtimes in 2026?"
```

Output: Multi-source research report with citations.

## 5. Agent Mode (for AI frameworks)

```bash
# Machine-readable JSON output for LLM consumption
hsx agent-search "latest Rust async developments" --budget 2000

# Fetch with semantic extraction
hsx agent-fetch https://blog.rust-lang.org/inside-rust/2026/01/tokio-update
```

## 6. Check Your Setup

```bash
hsx doctor
```

Shows available tools, system resources, and configuration status.

## Next Steps

- [Command Reference](commands.md) — all 25+ commands with examples
- [Configuration](configuration.md) — customize behavior
- [Agent Integration](agent-integration.md) — use with Claude, LangChain, CrewAI
