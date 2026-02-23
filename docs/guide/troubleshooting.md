# Troubleshooting

## Common Issues

### Ollama Not Found

**Symptom:** `hsx ai` or `hsx deep` fails with "AI engine unavailable"

**Fix:**
```bash
# Install Ollama
brew install ollama  # macOS
# or: curl -fsSL https://ollama.com/install.sh | sh  # Linux

# Start the server
ollama serve

# Pull a model
ollama pull deepseek-r1:7b

# Test
hsx ai "Hello"
```

### Chromium Not Found

**Symptom:** JavaScript-rendered pages return empty content

**Fix:**
```bash
brew install chromium  # macOS
# or: apt install chromium-browser  # Debian/Ubuntu

# Test
hsx fetch https://react.dev --tier complete
```

### Rate Limiting

**Symptom:** `HTTP 429 Too Many Requests` errors

**Fix:**
- Wait 60 seconds and retry
- Configure backup search engines in `~/.hypersearchx/config.toml`
- Use `--engines brave` if you have a Brave Search API key

### Slow Performance

**Symptom:** Searches taking >10 seconds

**Diagnostics:**
```bash
hsx doctor  # Check resource tier and available tools
```

**Fixes:**
- Reduce `--max-results` (default 10)
- Use `--tier key_facts` for faster, smaller output
- Ensure Ollama model is already loaded: `ollama run deepseek-r1:7b`

### macOS SDK Issues (C dependencies)

**Symptom:** Build error: "architecture not supported" when compiling from source

**Fix:**
```bash
export SDKROOT=$(xcrun --sdk macosx --show-sdk-path)
cargo build -p hsx-cli
```

### Token Budget Exceeded

**Symptom:** `Token budget exceeded: used X, budget Y`

**Fix:**
```bash
hsx fetch <url> --budget 8000   # Increase budget
hsx fetch <url> --tier key_facts  # Use smaller tier
```

### Timeout Errors

**Symptom:** `Operation timed out after Nms`

**Fix:**
```bash
# Increase timeout globally
echo 'timeout_secs = 30' >> ~/.hypersearchx/config.toml

# Or per-command (if supported)
export HSX_TIMEOUT=30
```

## Diagnostic Commands

```bash
# Full system health check
hsx doctor

# Check version
hsx --version

# Test basic search (offline)
hsx search "test" --format json 2>&1 | head -20

# Check config
cat ~/.hypersearchx/config.toml
```

## Getting Help

- GitHub Issues: https://github.com/hypersearchx/hypersearchx/issues
- Discussions: https://github.com/hypersearchx/hypersearchx/discussions
