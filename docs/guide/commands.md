# Command Reference

All 25+ HyperSearchX commands, grouped by category.

## Search Commands

### `hsx search`

Search the web using multiple engines with intelligent ranking.

```
hsx search <QUERY> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--format`, `-f` | `markdown` | Output format: `markdown`, `json`, `text`, `csv` |
| `--max-results`, `-n` | `10` | Maximum results to return |
| `--engines` | `auto` | Engines: `ddg`, `google`, `bing`, `scholar`, `all` |
| `--validate` | `standard` | Validation mode: `off`, `fast`, `standard`, `strict` |

**Examples:**
```bash
hsx search "Rust async runtime comparison"
hsx search "quantum computing 2026" --format json --max-results 5
hsx search "site:arxiv.org transformer attention" --engines scholar
```

### `hsx fetch`

Fetch and extract content from a URL using CEP.

```
hsx fetch <URL> [OPTIONS]
```

| Flag | Default | Description |
|------|---------|-------------|
| `--format`, `-f` | `markdown` | Output format |
| `--tier` | `summary` | PDS tier: `key_facts`, `summary`, `detailed`, `complete` |
| `--budget` | `4096` | Token budget |

**Examples:**
```bash
hsx fetch https://doc.rust-lang.org/book/ch04-01.html
hsx fetch https://arxiv.org/abs/2106.09685 --tier detailed --format json
```

### `hsx deep`

Multi-agent deep research using AMRS (Adaptive Multi-Agent Research Swarm).

```
hsx deep <QUERY> [OPTIONS]
```

**Examples:**
```bash
hsx deep "Compare tokio vs async-std performance in 2026"
hsx deep "Rust embedded systems best practices" --format json
```

## Agent Commands (JSON output for AI frameworks)

### `hsx agent-search`

Token-budgeted search returning structured JSON with segments.

```
hsx agent-search <QUERY> [--budget N]
```

### `hsx agent-fetch`

Fetch with semantic extraction, returning JSON + content hash.

```
hsx agent-fetch <URL> [--budget N] [--tier TIER]
```

### `hsx agent-research`

Full research pipeline returning structured AgentResearchOutput JSON.

```
hsx agent-research <QUERY> [--budget N]
```

## Research & AI Commands

### `hsx research`

Comprehensive multi-source research report with citations.

```
hsx research <QUERY> [--format FORMAT] [--cite-style STYLE]
```

Citation styles: `inline`, `footnote`, `apa`, `mla`, `chicago`, `ieee`, `bibtex`

### `hsx ai`

AI synthesis using Ollama. Streams response to stdout.

```
hsx ai <QUERY> [--model MODEL]
```

## Index & Comparison Commands

### `hsx compare`

Compare two or more items side-by-side.

```
hsx compare "Tokio vs async-std" [--format FORMAT]
```

### `hsx index`

Manage the local document index.

```
hsx index add <PATH>
hsx index search <QUERY>
hsx index stats
hsx index clear
```

### `hsx monitor`

Monitor URLs for changes.

```
hsx monitor add <URL> [--interval INTERVAL]
hsx monitor list
hsx monitor check
hsx monitor diff <URL>
hsx monitor remove <ID>
```

Interval format: `30s`, `5m`, `1h`, `7d`

## Intelligence Commands

### `hsx radar`

Personalized research radar based on your search history.

```
hsx radar [--limit N]
```

### `hsx digest`

Generate a research digest for topics.

```
hsx digest --period weekly --topics "rust,wasm,llm"
hsx digest --period daily --topics "security" --output digest.md
```

### `hsx subscribe`

Subscribe to topic alerts.

```
hsx subscribe add "Rust security advisories" --interval 1d
hsx subscribe list
hsx subscribe remove <ID>
```

## Export Commands

```
hsx export --format pdf --output report.pdf
hsx export --format docx --output report.docx
hsx export --format bibtex --output refs.bib
```

## System Commands

### `hsx doctor`

Check system health and dependencies.

### `hsx serve`

Start MCP server or REST API.

```
hsx serve --mcp           # MCP stdio server for AI frameworks
hsx serve --rest          # REST API on :8080
hsx serve --both          # Both simultaneously
```

### `hsx completions`

Generate shell completion scripts.

```
hsx completions bash > ~/.bash_completion.d/hsx
hsx completions zsh > ~/.zsh/completions/_hsx
hsx completions fish > ~/.config/fish/completions/hsx.fish
```
