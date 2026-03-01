#!/usr/bin/env bash
# PostToolUse hook: run cargo check after editing Rust files or Cargo.toml
# Exit 2 to block Claude and show errors; exit 0 for silent pass.

set -euo pipefail

INPUT=$(cat)

# Extract file path from hook JSON input
FILE_PATH=$(echo "$INPUT" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    # PostToolUse input: tool_input.file_path or tool_input.path
    ti = d.get('tool_input', {})
    print(ti.get('file_path', ti.get('path', '')))
except Exception:
    print('')
" 2>/dev/null)

# Only trigger for Rust source files and Cargo manifests
if [[ "$FILE_PATH" != *.rs && "$FILE_PATH" != */Cargo.toml && "$FILE_PATH" != */Cargo.lock ]]; then
    exit 0
fi

# Run cargo check from workspace root
cd "$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

export PATH="$HOME/.cargo/bin:/usr/bin:/usr/local/bin:/bin:$PATH"

OUTPUT=$(cargo check --workspace -q 2>&1)
EXIT_CODE=$?

if [ $EXIT_CODE -ne 0 ]; then
    echo "cargo check failed after editing $FILE_PATH:"
    echo ""
    echo "$OUTPUT"
    exit 2
fi

exit 0
