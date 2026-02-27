# fetchium

**Fetchium CLI** — AI-native federated search engine with neural ranking and token-budgeted extraction.

```bash
npm install -g fetchium
fetchium --help
```

## Usage

```bash
# Search across multiple backends with neural ranking
fetchium search "rust async programming best practices"

# Research pipeline — deep multi-source investigation
fetchium research "Compare pgvector vs Pinecone vs Weaviate for production"

# Extract content from a URL (5-layer CEP pipeline)
fetchium scrape https://tokio.rs/tokio/tutorial

# Run as API server
fetchium serve --port 3050
```

## Install options

```bash
# npm (cross-platform)
npm install -g fetchium
npx fetchium --help

# Shell (Linux / macOS)
curl -sSf https://install.fetchium.com | sh

# Homebrew (macOS / Linux)
brew install zuhabul/tap/fetchium

# cargo-binstall
cargo binstall fetchium
```

## Links

- **Docs**: https://fetchium.com/docs
- **API dashboard**: https://app.fetchium.com
- **GitHub**: https://github.com/zuhabul/fetchium
