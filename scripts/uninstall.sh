#!/usr/bin/env sh
# Fetchium CLI uninstaller
set -eu

BIN_NAME="fetchium"
LOCATIONS="/usr/local/bin /usr/bin /opt/homebrew/bin $HOME/.local/bin $HOME/.cargo/bin"
FOUND=0

echo "Searching for fetchium installations..."
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
  echo "fetchium not found in standard locations."
  echo "If you installed via npm: npm uninstall -g fetchium"
  echo "If you installed via brew: brew uninstall fetchium"
else
  echo "✓ fetchium uninstalled successfully."
fi
