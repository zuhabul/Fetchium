# Command Reference

All 25+ Fetchium commands, grouped by category.

## Search Commands

### `fetchium search`

Search the web using multiple engines with intelligent ranking.

```
fetchium search <QUERY> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--format`, `-f` | `markdown` | Output format: `markdown`, `json`, `text`, `csv` |
| `--max-results`, `-n` | `10` | Maximum results to return |
| `--engines` | `auto` | Engines: `ddg`, `google`, `bing`, `scholar`, `all` |
| `--validate` | `standard` | Validation mode: `off`, `fast`, `standard`, `strict` |

**Examples:**
```bash
fetchium search "Rust async runtime comparison"
fetchium search "quantum computing 2026" --format json --max-results 5
fetchium search "site:arxiv.org transformer attention" --engines scholar
```

### `fetchium fetch`

Fetch and extract content from a URL using CEP.

```
fetchium fetch <URL> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--format`, `-f` | `markdown` | Output format |
| `--tier` | `summary` | PDS tier: `key_facts`, `summary`, `detailed`, `complete` |
| `--budget` | `4096` | Token budget |

**Examples:**
```bash
fetchium fetch https://doc.rust-lang.org/book/ch04-01.html
fetchium fetch https://arxiv.org/abs/2106.09685 --tier detailed --format json
```

### `fetchium deep`

Multi-agent deep research using AMRS (Adaptive Multi-Agent Research Swarm).

```
fetchium deep <QUERY> [OPTIONS]
```

**Examples:**
```bash
fetchium deep "Compare tokio vs async-std performance in 2026"
fetchium deep "Rust embedded systems best practices" --format json
```

## Agent Commands (JSON output for AI frameworks)

### `fetchium agent-search`

Token-budgeted search returning structured JSON with segments.

```
fetchium agent-search <QUERY> [--budget N]
```

### `fetchium agent-fetch`

Fetch with semantic extraction, returning JSON + content hash.

```
fetchium agent-fetch <URL> [--budget N] [--tier TIER]
```

### `fetchium agent-research`

Full research pipeline returning structured AgentResearchOutput JSON.

```
fetchium agent-research <QUERY> [--budget N]
```

## Research & AI Commands

### `fetchium research`

Comprehensive multi-source research report with citations.

```
fetchium research <QUERY> [--format FORMAT] [--cite-style STYLE]
```

Citation styles: `inline`, `footnote`, `apa`, `mla`, `chicago`, `ieee`, `bibtex`

### `fetchium ai`

AI synthesis using Ollama. Streams response to stdout.

```
fetchium ai <QUERY> [--model MODEL]
```

## Index & Comparison Commands

### `fetchium compare`

Compare two or more items side-by-side.

```
fetchium compare "Tokio vs async-std" [--format FORMAT]
```

### `fetchium index`

Manage the local document index.

```
fetchium index add <PATH>
fetchium index search <QUERY>
fetchium index stats
fetchium index clear
```

### `fetchium monitor`

Monitor URLs for changes.

```
fetchium monitor add <URL> [--interval INTERVAL]
fetchium monitor list
fetchium monitor check
fetchium monitor diff <URL>
fetchium monitor remove <ID>
```

Interval format: `30s`, `5m`, `1h`, `7d`

## Intelligence Commands

### `fetchium radar`

Personalized research radar based on your search history.

```
fetchium radar [--limit N]
```

### `fetchium digest`

Generate a research digest for topics.

```
fetchium digest --period weekly --topics "rust,wasm,llm"
fetchium digest --period daily --topics "security" --output digest.md
```

### `fetchium subscribe`

Subscribe to topic alerts.

```
fetchium subscribe add "Rust security advisories" --interval 1d
fetchium subscribe list
fetchium subscribe remove <ID>
```

## Export Commands

```
fetchium export --format pdf --output report.pdf
fetchium export --format docx --output report.docx
fetchium export --format bibtex --output refs.bib
```

## System Commands

### `fetchium doctor`

Check system health and dependencies.

### `fetchium serve`

Start MCP server or REST API.

```
fetchium serve --mcp           # MCP stdio server for AI frameworks
fetchium serve --rest          # REST API on :8080
fetchium serve --both          # Both simultaneously
```

### `fetchium completions`

Generate shell completion scripts.

```
fetchium completions bash > ~/.bash_completion.d/fetchium
fetchium completions zsh > ~/.zsh/completions/_hsx
fetchium completions fish > ~/.config/fish/completions/fetchium.fish
```
