# hypersearchx

**HyperSearchX CLI** — AI-native federated search engine with neural ranking and token-budgeted extraction.

```bash
npm install -g hypersearchx
hsx --help
```

## Usage

```bash
# Search across multiple backends with neural ranking
hsx search "rust async programming best practices"

# Research pipeline — deep multi-source investigation
hsx research "Compare pgvector vs Pinecone vs Weaviate for production"

# Extract content from a URL (5-layer CEP pipeline)
hsx scrape https://tokio.rs/tokio/tutorial

# Run as API server
hsx serve --port 3050
```

## Install options

```bash
# npm (cross-platform)
npm install -g hypersearchx
npx hypersearchx --help

# Shell (Linux / macOS)
curl -sSf https://install.hypersearchx.zuhabul.com | sh

# Homebrew (macOS / Linux)
brew install zuhabul/tap/hsx

# cargo-binstall
cargo binstall hsx
```

## Links

- **Docs**: https://hypersearchx.zuhabul.com/docs
- **API dashboard**: https://app.hypersearchx.zuhabul.com
- **GitHub**: https://github.com/zuhabul/HyperSearchX
