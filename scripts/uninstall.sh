#!/usr/bin/env sh
# HyperSearchX CLI uninstaller
set -eu

BIN_NAME="hsx"
LOCATIONS="/usr/local/bin /usr/bin /opt/homebrew/bin $HOME/.local/bin $HOME/.cargo/bin"
FOUND=0

echo "Searching for hsx installations..."
for dir in $LOCATIONS; do
  BIN="${dir}/${BIN_NAME}"
  if [ -f "$BIN" ]; then
    echo "Found: $BIN"
    if [ -w "$dir" ]; then
      rm -f "$BIN"
    else
      sudo rm -f "$BIN"
    fi
    echo "✓ Removed: $BIN"
    FOUND=1
  fi
done

if [ "$FOUND" -eq 0 ]; then
  echo "hsx not found in standard locations."
  echo "If you installed via npm: npm uninstall -g hypersearchx"
  echo "If you installed via brew: brew uninstall hsx"
else
  echo "✓ hsx uninstalled successfully."
fi
