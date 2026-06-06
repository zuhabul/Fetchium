# Agent Integration

Fetchium supports 6 integration modes for AI frameworks.

## MCP (Model Context Protocol)

Fetchium supports both local stdio MCP and an HTTP JSON-RPC MCP endpoint at `/mcp`.

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "fetchium": {
      "command": "fetchium",
      "args": ["serve", "--mode", "mcp"]
    }
  }
}
```

### Claude Code

Add to `.mcp.json` in your project root:

```json
{
  "servers": {
    "fetchium": {
      "command": "fetchium",
      "args": ["serve", "--mode", "mcp"],
      "env": {}
    }
  }
}
```

### HTTP MCP

Start the HTTP MCP server:

```bash
fetchium serve --mode mcp --transport http --port 3471
```

Probe it:

```bash
curl -X POST http://127.0.0.1:3471/mcp \
  -H 'Content-Type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}'
```

### Available MCP Tools

| Tool | Description |
|------|-------------|
| `fetchium_search` | Token-budgeted web search |
| `fetchium_fetch` | Query-aware content extraction |
| `fetchium_research` | Multi-source research with citations |
| `fetchium_estimate` | Pre-fetch token estimation |
| `fetchium_expand` | Tier expansion without re-fetching |
| `youtube_search` | YouTube ranked search |
| `youtube_analyze` | Single-video analysis |
| `youtube_watch` | Unified watch report |
| `youtube_transcript` | Transcript extraction |
| `social_research` | Unified cross-platform social research |
| `reddit_search` | Reddit search intelligence |
| `hackernews_search` | Hacker News search intelligence |

## REST API

Start the REST server:
```bash
fetchium serve --mode rest --port 3000
```

### Endpoints

```
POST /v1/search
POST /v1/fetch
POST /v1/research
POST /v1/estimate
GET  /health
```

Example:
```bash
curl -X POST http://localhost:3000/v1/search \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <your-api-key>' \
  -d '{"query": "Rust ownership", "max_results": 5}'
```

## LangChain (Python)

```python
from langchain.tools import Tool
import subprocess, json

def fetchium_search(query: str) -> str:
    result = subprocess.run(
        ["fetchium", "agent-search", query, "--budget", "2000"],
        capture_output=True, text=True
    )
    return result.stdout

search_tool = Tool(
    name="Fetchium",
    func=fetchium_search,
    description="Search the web with AI-native token-budgeted results"
)
```

## CrewAI (Python)

```python
from crewai_tools import tool
import subprocess

@tool("Web Search")
def web_search(query: str) -> str:
    """Search the web using Fetchium and return structured results."""
    result = subprocess.run(
        ["fetchium", "agent-search", query],
        capture_output=True, text=True, timeout=30
    )
    return result.stdout
```

## OpenAI Function Calling

```python
search_function = {
    "name": "web_search",
    "description": "Search the web for current information",
    "parameters": {
        "type": "object",
        "properties": {
            "query": {"type": "string", "description": "Search query"},
            "budget": {"type": "integer", "description": "Token budget", "default": 2000}
        },
        "required": ["query"]
    }
}

def handle_function_call(name, args):
    if name == "web_search":
        result = subprocess.run(
            ["fetchium", "agent-search", args["query"], "--budget", str(args.get("budget", 2000))],
            capture_output=True, text=True
        )
        return result.stdout
```

## Troubleshooting

**`fetchium: command not found`** — Add the binary to PATH or use the full path.

**`Ollama not available`** — Start with `ollama serve`. Install at https://ollama.com.

**`Rate limited`** — Fetchium respects rate limits. Wait and retry, or configure multiple backends.

**`Content not found`** — The page may require JavaScript. Install Chromium for CEP Layer 3.
