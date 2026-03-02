# Command Reference

This reference is generated from the live CLI surface. For full flags and examples per command, run:

```bash
fetchium <command> --help
```

## Core Retrieval

- `fetchium search` — Search the web (human-friendly output)
- `fetchium fetch` — Fetch and extract URL content
- `fetchium view` — Alias for `fetchium fetch`
- `fetchium research` — Multi-source research with citations
- `fetchium ai` — AI analysis over retrieved evidence
- `fetchium deep` — Deep multi-agent research

## Agent JSON Commands

- `fetchium agent-search` — Token-budgeted agent search output
- `fetchium agent-fetch` — Agent-optimized URL extraction
- `fetchium agent-research` — Agent-optimized research output

## API / Server

- `fetchium serve --mode rest --port 3000` — REST API server
- `fetchium serve --mode mcp` — MCP server (stdio)
- `fetchium serve --mode both --port 3000` — REST + MCP

Common REST endpoints:

- `GET /health`
- `GET /v1/health`
- `POST /v1/search`
- `POST /v1/fetch` (alias: `POST /v1/scrape`)
- `POST /v1/research`
- `POST /v1/estimate`
- `POST /v1/youtube/search`
- `POST /v1/youtube/analyze`
- `POST /v1/social/research`
- `POST /v1/social/reddit`
- `POST /v1/social/hackernews`
- `GET /v1/usage` (Bearer auth)
- `POST /v1/keys` (admin)
- `GET /v1/keys` (admin)
- `DELETE /v1/keys/:id` (admin)

## Intelligence / Productivity

- `fetchium compare` — Side-by-side comparison research
- `fetchium monitor` — URL change monitoring
- `fetchium index` — Local document index management
- `fetchium intelligence` — PIE/intelligence tools
- `fetchium workspace` — Workspace create/fork/merge/sync
- `fetchium subscribe` — Topic subscriptions
- `fetchium radar` — Personalized research radar
- `fetchium digest` — Research digests

## Platform / Specialized

- `fetchium youtube` — YouTube intelligence suite
- `fetchium social` — Unified social intelligence
- `fetchium twitter` — X/Twitter-specific tooling
- `fetchium reddit` — Reddit-specific tooling
- `fetchium hackernews` — Hacker News tooling
- `fetchium facebook` — Facebook tooling
- `fetchium tiktok` — TikTok tooling
- `fetchium transcribe` — Audio/video transcription
- `fetchium summarize` — AI summarization

## System / Setup

- `fetchium help` — Show help for commands and subcommands
- `fetchium doctor` — Environment and dependency checks
- `fetchium setup` — Guided setup (Chromium/SearXNG/etc.)
- `fetchium provider` — AI provider auth and routing chain
- `fetchium plugin` — Plugin lifecycle commands
- `fetchium config` — Configuration management
- `fetchium cache` — Cache management
- `fetchium tui` — Interactive terminal UI
- `fetchium completions` — Shell completions

## Completion Examples

```bash
fetchium completions bash > ~/.bash_completion.d/fetchium
fetchium completions zsh > ~/.zsh/completions/_fetchium
fetchium completions fish > ~/.config/fish/completions/fetchium.fish
```
