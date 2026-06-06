# Fetchium API Inventory

Date: 2026-03-07

Base URLs:
- Public HTTPS: `https://api.fetchium.com`
- Local dev: `http://127.0.0.1:3050` or custom port via `fetchium serve --mode rest --port <port>`

Source references:
- REST routes: [crates/fetchium-api/src/routes.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/routes.rs#L10)
- Request types and validation: [crates/fetchium-api/src/types.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/types.rs#L26)
- Admin key endpoints: [crates/fetchium-api/src/handlers_auth.rs](/home/echo/projects/Fetchium/crates/fetchium-api/src/handlers_auth.rs#L28)
- MCP tool definitions: [crates/fetchium-mcp/src/tools.rs](/home/echo/projects/Fetchium/crates/fetchium-mcp/src/tools.rs#L8)

## Auth

- Public endpoints: `GET /`, `GET /health`, `GET /v1/health`, `GET /api/health`
- User endpoints: `Authorization: Bearer fetchium_...`
- Admin endpoints: `X-Admin-Secret: ...`

## Verified REST Endpoints

| Method | Path | Auth | Local test | Public HTTPS test | Notes |
|---|---|---|---|---|---|
| GET | `/` | none | 200 | 200 | API index |
| GET | `/health` | none | 200 | 200 | Health payload returned |
| GET | `/v1/health` | none | 200 | 200 | Health payload returned |
| GET | `/api/health` | none | 200 | not checked separately on public | Legacy alias |
| POST | `/v1/keys` | admin | 200 | 401 without admin secret | Public host enforces admin auth |
| GET | `/v1/keys` | admin | 200 | not tested | Lists masked active keys |
| DELETE | `/v1/keys/:id` | admin | route exists | not tested | Revoke key |
| GET | `/v1/usage` | bearer | 200 | 200 | Public key accepted |
| POST | `/v1/search` | bearer | 200 | 200 | Public key accepted |
| POST | `/v1/scrape` | bearer | 200 | timeout after 20s | Public host reachable but endpoint did not respond within timeout |
| POST | `/v1/fetch` | bearer | 200 | 200 | Alias for scrape handler; public test succeeded |
| POST | `/v1/estimate` | bearer | 200 | timeout after 20s | No public response within timeout |
| POST | `/v1/research` | bearer | 200 | timeout after 20s | No public response within timeout |
| POST | `/v1/social/hackernews` | bearer | 200 | timeout after 20s | No public response within timeout |
| POST | `/v1/social/reddit` | bearer | 200 | timeout after 20s | No public response within timeout |
| POST | `/v1/social/research` | bearer | 200 | timeout after 20s | No public response within timeout |
| POST | `/v1/youtube/search` | bearer | 200 | timeout after 20s | No public response within timeout |
| POST | `/v1/youtube/analyze` | bearer | 200 | timeout after 20s | No public response within timeout |

## Request Shapes

### Search
```json
{
  "query": "Rust programming language",
  "token_budget": 600,
  "tier": "summary",
  "max_sources": 2
}
```

### Scrape / Fetch
```json
{
  "url": "https://www.rust-lang.org/",
  "query": "optional query hint",
  "token_budget": 500,
  "format": "markdown"
}
```

### Research
```json
{
  "query": "What is Rust programming language?",
  "token_budget": 1500,
  "max_sources": 3,
  "depth": "standard",
  "strict_evidence": false,
  "citation_style": "inline"
}
```

### Estimate
```json
{
  "url": "https://www.rust-lang.org/"
}
```

### YouTube Search
```json
{
  "query": "Rust programming tutorial",
  "max_results": 2,
  "fact_check": false
}
```

### YouTube Analyze
```json
{
  "url": "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
  "transcript": false,
  "comments": false,
  "teaching": false
}
```

### Social Research
```json
{
  "query": "Rust programming",
  "platforms": ["reddit", "hackernews"],
  "max_per_platform": 2,
  "generate_ideas": false
}
```

### Reddit Search
```json
{
  "query": "Rust programming",
  "max_posts": 2
}
```

### Hacker News Search
```json
{
  "query": "Rust",
  "max_results": 2
}
```

## Example curl

```bash
export FETCHIUM_BASE="https://api.fetchium.com"
export FETCHIUM_API_KEY="fetchium_REPLACE_ME"

curl -s "$FETCHIUM_BASE/v1/usage" \
  -H "Authorization: Bearer $FETCHIUM_API_KEY"

curl -sX POST "$FETCHIUM_BASE/v1/search" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \
  -d '{"query":"Rust programming language","max_sources":2,"tier":"summary","token_budget":600}'

curl -sX POST "$FETCHIUM_BASE/v1/fetch" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $FETCHIUM_API_KEY" \
  -d '{"url":"https://example.com/","format":"text","token_budget":200}'
```

## Public HTTPS Result Summary

- Public host is live and returns health successfully.
- Public host rejects unauthenticated user requests with `401`.
- Public host rejects unauthenticated admin key creation with `401`.
- The dedicated bearer key created during testing worked on the public host for:
- `GET /v1/usage`
- `POST /v1/search`
- `POST /v1/fetch`
- The following public endpoints timed out after 20 seconds with no bytes received:
- `POST /v1/scrape`
- `POST /v1/estimate`
- `POST /v1/research`
- `POST /v1/social/hackernews`
- `POST /v1/social/reddit`
- `POST /v1/social/research`
- `POST /v1/youtube/search`
- `POST /v1/youtube/analyze`

## MCP Tools

- `fetchium_search`
- `fetchium_fetch`
- `fetchium_research`
- `fetchium_estimate`
- `fetchium_expand`
- `youtube_search`
- `youtube_analyze`
- `youtube_watch`
- `youtube_transcript`
- `social_research`
- `reddit_search`
- `hackernews_search`
