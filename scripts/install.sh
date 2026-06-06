#!/usr/bin/env sh
# Fetchium CLI installer
# Usage: curl -sSfL https://install.fetchium.com | sh
# Options:
#   FETCHIUM_INSTALL_DIR  - Installation directory (default: /usr/local/bin)
#   FETCHIUM_VERSION      - Pin a specific version (default: latest)
set -eu

REPO="zuhabul/Fetchium"
BIN_NAME="fetchium"
INSTALL_DIR="${FETCHIUM_INSTALL_DIR:-/usr/local/bin}"
VERSION="${FETCHIUM_VERSION:-}"

# ── Terminal colours ──────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BOLD='\033[1m'; RESET='\033[0m'
info()    { printf "${BOLD}info${RESET}  %s\n" "$*"; }
success() { printf "${GREEN}✓${RESET}     %s\n" "$*"; }
warn()    { printf "${YELLOW}warn${RESET}  %s\n" "$*"; }
error()   { printf "${RED}error${RESET} %s\n" "$*" >&2; exit 1; }

# ── Platform detection ────────────────────────────────────────────────────────
detect_platform() {
  OS=$(uname -s | tr '[:upper:]' '[:lower:]')
  ARCH=$(uname -m)
  case "$ARCH" in
    x86_64|amd64) ARCH="x64" ;;
    aarch64|arm64) ARCH="arm64" ;;
    *) error "Unsupported architecture: $ARCH. See https://docs.fetchium.com/self-hosting" ;;
  esac
  case "$OS" in
    linux)  PLATFORM="linux"  ;;
    darwin) PLATFORM="darwin" ;;
    *)      error "Unsupported OS: $OS. On Windows use: npm install -g fetchium" ;;
  esac
  ARTIFACT="fetchium-${PLATFORM}-${ARCH}"
}

# ── Download helper ───────────────────────────────────────────────────────────
download() {
  URL="$1"; DEST="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --retry 3 --retry-delay 2 -o "$DEST" "$URL"
  elif command -v wget >/dev/null 2>&1; then
    wget -q --tries=3 --waitretry=2 -O "$DEST" "$URL"
  else
    error "curl or wget is required. Install one and try again."
  fi
}

# ── Version resolution ────────────────────────────────────────────────────────
resolve_version() {
  if [ -z "$VERSION" ]; then
    info "Fetching latest version..."
    API_URL="https://api.github.com/repos/${REPO}/releases/latest"
    if command -v curl >/dev/null 2>&1; then
      VERSION=$(curl -fsSL "$API_URL" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/' | tr -d ' ')
    else
      VERSION=$(wget -qO- "$API_URL" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/' | tr -d ' ')
    fi
    [ -z "$VERSION" ] && error "Could not resolve latest version. Check your internet connection."
  fi
  # Normalise: ensure it starts with v
  case "$VERSION" in
    v*) ;;
    *)  VERSION="v${VERSION}" ;;
  esac
}

# ── Checksum verification ─────────────────────────────────────────────────────
verify_checksum() {
  ARCHIVE="$1"; CHECKSUM_FILE="$2"
  EXPECTED=$(cat "$CHECKSUM_FILE" | awk '{print $1}')
  if command -v sha256sum >/dev/null 2>&1; then
    ACTUAL=$(sha256sum "$ARCHIVE" | awk '{print $1}')
  elif command -v shasum >/dev/null 2>&1; then
    ACTUAL=$(shasum -a 256 "$ARCHIVE" | awk '{print $1}')
  else
    warn "sha256sum/shasum not found — skipping checksum verification"
    return 0
  fi
  if [ "$EXPECTED" != "$ACTUAL" ]; then
    error "Checksum mismatch!\n  expected: $EXPECTED\n  got:      $ACTUAL\nDownload may be corrupted."
  fi
  success "Checksum verified"
}

# ── Main ──────────────────────────────────────────────────────────────────────
main() {
  printf "\n${BOLD}Fetchium CLI Installer${RESET}\n\n"

  detect_platform
  resolve_version

  SEMVER="${VERSION#v}"
  BASE_URL="https://github.com/${REPO}/releases/download/${VERSION}"
  ARCHIVE_URL="${BASE_URL}/${ARTIFACT}.tar.gz"
  SHA_URL="${BASE_URL}/${ARTIFACT}.tar.gz.sha256"

  TMP_DIR=$(mktemp -d)
  ARCHIVE="${TMP_DIR}/${ARTIFACT}.tar.gz"
  CHECKSUM="${TMP_DIR}/${ARTIFACT}.tar.gz.sha256"

  trap "rm -rf '$TMP_DIR'" EXIT

  info "Downloading fetchium ${VERSION} for ${PLATFORM}/${ARCH}..."
  download "$ARCHIVE_URL" "$ARCHIVE"
  download "$SHA_URL" "$CHECKSUM"
  verify_checksum "$ARCHIVE" "$CHECKSUM"

  info "Extracting..."
  tar -xzf "$ARCHIVE" -C "$TMP_DIR"

  # Install
  if [ -w "$INSTALL_DIR" ]; then
    mv "${TMP_DIR}/fetchium" "${INSTALL_DIR}/${BIN_NAME}"
  else
    info "Writing to ${INSTALL_DIR} requires sudo..."
    sudo mv "${TMP_DIR}/fetchium" "${INSTALL_DIR}/${BIN_NAME}"
  fi
  chmod +x "${INSTALL_DIR}/${BIN_NAME}"

  printf "\n"
  success "fetchium ${VERSION} installed to ${INSTALL_DIR}/fetchium"
  printf "\n"
  printf "  ${BOLD}Get started:${RESET}\n"
  printf "  fetchium --help\n"
  printf "  fetchium search \"your query\"\n"
  printf "\n"
  printf "  ${BOLD}Docs:${RESET} https://docs.fetchium.com\n"
  printf "  ${BOLD}API key:${RESET} https://app.fetchium.com\n\n"
}

main "$@"
