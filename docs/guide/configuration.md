# Configuration

Fetchium reads config from `~/.fetchium/config.toml`. All settings can be
overridden via environment variables.

## Config File Location

```bash
~/.fetchium/config.toml  # Primary config
```

Create the directory and file:
```bash
mkdir -p ~/.fetchium
cat > ~/.fetchium/config.toml << 'EOF'
[general]
max_results = 10

[fetch]
timeout_secs = 15

[ai]
ollama_host = "http://localhost:11434"
default_model = "deepseek-r1:7b"
EOF
```

## All Configuration Options

```toml
[general]
# Maximum search results to return (default: 10)
max_results = 10

# Directory for Fetchium data files (default: ~/.fetchium)
data_dir = "~/.fetchium"

[fetch]
# HTTP request timeout in seconds (default: 15)
timeout_secs = 15

# Maximum page size in bytes (default: 5MB)
max_page_size = 5242880

# Maximum HTTP redirects to follow (default: 5)
max_redirects = 5

# User agent string
user_agent = "Fetchium/0.1 (https://github.com/fetchium/fetchium)"

[ai]
# Ollama server address (default: http://localhost:11434)
ollama_host = "http://localhost:11434"

# Default model for AI synthesis (default: deepseek-r1:7b)
default_model = "deepseek-r1:7b"

# Fast model for latency-sensitive tasks like HyDE (default: same as default_model)
fast_model = "qwen3:0.6b"

# Maximum tokens per AI response (default: 4096)
max_tokens = 4096

[cache]
# Maximum cache entries (default: 1000)
max_entries = 1000

# Cache TTL in seconds (default: 3600 = 1 hour)
ttl_secs = 3600

[search]
# Search backends to use: ddg, google, bing, brave, scholar, etc.
backends = ["ddg"]
```

## Environment Variables

All settings can be overridden via environment variables:

| Variable | Config Key | Example |
|----------|-----------|---------|
| `HSX_MAX_RESULTS` | `general.max_results` | `HSX_MAX_RESULTS=20` |
| `HSX_TIMEOUT` | `fetch.timeout_secs` | `HSX_TIMEOUT=30` |
| `HSX_OLLAMA_HOST` | `ai.ollama_host` | `HSX_OLLAMA_HOST=http://192.168.1.5:11434` |
| `HSX_DEFAULT_MODEL` | `ai.default_model` | `HSX_DEFAULT_MODEL=llama3.2:3b` |
| `HSX_DATA_DIR` | `general.data_dir` | `HSX_DATA_DIR=/var/lib/hsx` |

## API Keys

Search engine API keys (optional, used when DDG quota is exceeded):

```toml
[keys]
brave_api_key = "BSA..."
google_api_key = "AIza..."
google_cx = "partner-pub-..."
```

Or via environment variables:
```bash
export HSX_BRAVE_API_KEY="BSA..."
export HSX_GOOGLE_API_KEY="AIza..."
```
