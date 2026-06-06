# Troubleshooting

## Common Issues

### Ollama Not Found

**Symptom:** `fetchium ai` or `fetchium deep` fails with "AI engine unavailable"

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
fetchium ai "Hello"
```

### Chromium Not Found

**Symptom:** JavaScript-rendered pages return empty content

**Fix:**
```bash
brew install chromium  # macOS
# or: apt install chromium-browser  # Debian/Ubuntu

# Test
fetchium fetch https://react.dev --tier complete
```

### Rate Limiting

**Symptom:** `HTTP 429 Too Many Requests` errors

**Fix:**
- Wait 60 seconds and retry
- Configure backup search engines in `~/.fetchium/config.toml`
- Use `--engines brave` if you have a Brave Search API key

### Slow Performance

**Symptom:** Searches taking >10 seconds

**Diagnostics:**
```bash
fetchium doctor  # Check resource tier and available tools
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
cargo build -p fetchium-cli
```

### Token Budget Exceeded

**Symptom:** `Token budget exceeded: used X, budget Y`

**Fix:**
```bash
fetchium fetch <url> --budget 8000   # Increase budget
fetchium fetch <url> --tier key_facts  # Use smaller tier
```

### Timeout Errors

**Symptom:** `Operation timed out after Nms`

**Fix:**
```bash
# Increase timeout globally
echo 'timeout_secs = 30' >> ~/.fetchium/config.toml

# Or per-command (if supported)
export FETCHIUM_TIMEOUT=30
```

## Diagnostic Commands

```bash
# Full system health check
fetchium doctor

# Check version
fetchium --version

# Test basic search (offline)
fetchium search "test" --format json 2>&1 | head -20

# Check config
cat ~/.fetchium/config.toml
```

## Getting Help

- GitHub Issues: https://github.com/zuhabul/Fetchium/issues
- Discussions: https://github.com/zuhabul/Fetchium/discussions
