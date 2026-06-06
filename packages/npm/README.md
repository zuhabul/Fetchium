# fetchium-cli

**Fetchium** — the universal retrieval layer for humans and AI agents. Federated multi-backend
search, adaptive content extraction (5-layer CEP), multi-signal ranking, and cited research —
as a CLI, a REST API, and an MCP server.

```bash
npm install -g fetchium-cli
fetchium --help
```

The installed command is `fetchium`. The npm package downloads the prebuilt binary for your
platform from GitHub Releases.

## Usage

```bash
# Search across multiple backends with multi-signal ranking
fetchium search "rust async programming best practices"

# Research pipeline — deep multi-source investigation with citations
fetchium research "Compare pgvector vs Pinecone vs Weaviate for production"

# Fetch + extract clean content from a URL (5-layer CEP pipeline)
fetchium fetch https://tokio.rs/tokio/tutorial

# Run as a REST API / MCP server
fetchium serve --port 3050
```

## Other install options

```bash
# Cargo (from source)
cargo install --git https://github.com/zuhabul/Fetchium fetchium-cli

# Prebuilt binary (Linux x86-64)
curl -fsSL https://github.com/zuhabul/Fetchium/releases/latest/download/fetchium-linux-x64.tar.gz | tar xz
```

## Links

- **GitHub**: https://github.com/zuhabul/Fetchium
- **Issues**: https://github.com/zuhabul/Fetchium/issues
- **License**: MIT OR Apache-2.0
