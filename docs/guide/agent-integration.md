# Agent Integration

HyperSearchX supports 6 integration modes for AI frameworks.

## MCP (Model Context Protocol)

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "hypersearchx": {
      "command": "hsx",
      "args": ["serve", "--mcp"]
    }
  }
}
```

### Claude Code

Add to `.mcp.json` in your project root:

```json
{
  "servers": {
    "hypersearchx": {
      "command": "hsx",
      "args": ["serve", "--mcp"],
      "env": {}
    }
  }
}
```

### Available MCP Tools

| Tool | Description |
|------|-------------|
| `hypersearch_search` | Token-budgeted web search |
| `hypersearch_fetch` | Query-aware content extraction |
| `hypersearch_research` | Multi-source research with citations |
| `hypersearch_estimate` | Pre-fetch token estimation |
| `hypersearch_expand` | Tier expansion without re-fetching |

## REST API

Start the REST server:
```bash
hsx serve --rest --port 8080
```

### Endpoints

```
POST /api/search
POST /api/fetch
POST /api/research
POST /api/estimate
GET  /health
```

Example:
```bash
curl -X POST http://localhost:8080/api/search \
  -H 'Content-Type: application/json' \
  -d '{"query": "Rust ownership", "max_results": 5}'
```

## LangChain (Python)

```python
from langchain.tools import Tool
import subprocess, json

def hsx_search(query: str) -> str:
    result = subprocess.run(
        ["hsx", "agent-search", query, "--budget", "2000"],
        capture_output=True, text=True
    )
    return result.stdout

search_tool = Tool(
    name="HyperSearchX",
    func=hsx_search,
    description="Search the web with AI-native token-budgeted results"
)
```

## CrewAI (Python)

```python
from crewai_tools import tool
import subprocess

@tool("Web Search")
def web_search(query: str) -> str:
    """Search the web using HyperSearchX and return structured results."""
    result = subprocess.run(
        ["hsx", "agent-search", query],
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
            ["hsx", "agent-search", args["query"], "--budget", str(args.get("budget", 2000))],
            capture_output=True, text=True
        )
        return result.stdout
```

## Troubleshooting

**`hsx: command not found`** — Add the binary to PATH or use the full path.

**`Ollama not available`** — Start with `ollama serve`. Install at https://ollama.com.

**`Rate limited`** — HyperSearchX respects rate limits. Wait and retry, or configure multiple backends.

**`Content not found`** — The page may require JavaScript. Install Chromium for CEP Layer 3.
