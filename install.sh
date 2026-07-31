#!/usr/bin/env bash
set -euo pipefail

REPO="once2027/amnesia"
BINARY_NAME="amnesia"
LOCAL_BIN="$HOME/.local/bin"

log()  { printf '\033[1;32m%s\033[0m\n' "$*"; }
warn() { printf '\033[1;33m%s\033[0m\n' "$*" >&2; }
die()  { printf '\033[1;31mERROR: %s\033[0m\n' "$*" >&2; exit 1; }

# --- Windows (Git Bash / MSYS / Cygwin) ---
case "$(uname -s)" in
  MINGW*|MSYS*|CYGWIN*)
    warn "this installer is for Linux/macOS. on Windows run:"
    warn "  powershell -ExecutionPolicy Bypass -Command \"irm https://raw.githubusercontent.com/$REPO/master/install.ps1 | iex\""
    exit 1
    ;;
esac

api() { # prints response body
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- "$1"
  else
    die "curl or wget is required"
  fi
}

download() { # <url> <output file>
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$1" -o "$2"
  elif command -v wget >/dev/null 2>&1; then
    wget -q -O "$2" "$1"
  else
    die "curl or wget is required"
  fi
}

OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)  ASSET_OS="linux" ;;
  darwin) ASSET_OS="macos" ;;
  *) die "unsupported OS: $OS" ;;
esac

case "$ARCH" in
  amd64)          ASSET_ARCH="x86_64" ;;
  arm64)          ASSET_ARCH="aarch64" ;;
  x86_64|aarch64) ASSET_ARCH="$ARCH" ;;
  *) die "unsupported architecture: $ARCH" ;;
esac

# Intel Mac fallback
if [ "$ASSET_OS" = "macos" ] && [ "$ASSET_ARCH" = "aarch64" ] && [ "$(sysctl -n hw.optional.arm64 2>/dev/null)" != "1" ]; then
  ASSET_ARCH="x86_64"
fi

ASSET_NAME="amnesia-${ASSET_OS}-${ASSET_ARCH}.tar.gz"

# Resolve version (latest release by default, or pin with VERSION=v1.1)
TAG="${VERSION:-}"
if [ -z "$TAG" ]; then
  TAG="$(api "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n1 || true)"
fi
[ -n "$TAG" ] || die "could not determine the latest release (set VERSION=v1.1 to pin one)"

URL="https://github.com/$REPO/releases/download/$TAG/$ASSET_NAME"

# Pick a writable directory that is already on PATH
INSTALL_DIR=""
for dir in "$LOCAL_BIN" /usr/local/bin /opt/homebrew/bin /usr/bin; do
  if [ -d "$dir" ] && [ -w "$dir" ] && [[ ":$PATH:" == *":$dir:"* ]]; then
    INSTALL_DIR="$dir"
    break
  fi
done

# Fall back to /usr/local/bin (with sudo if needed), else ~/.local/bin
USE_SUDO=0
if [ -z "$INSTALL_DIR" ] && [ -d /usr/local/bin ]; then
  if [ -w /usr/local/bin ]; then
    INSTALL_DIR=/usr/local/bin
  elif command -v sudo >/dev/null 2>&1; then
    INSTALL_DIR=/usr/local/bin
    USE_SUDO=1
  fi
fi
INSTALL_DIR="${INSTALL_DIR:-$LOCAL_BIN}"

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

log "==> downloading $ASSET_NAME ($TAG)"
download "$URL" "$TMP/$ASSET_NAME" || die "download failed: $URL"

# Verify checksum if the release ships a SHA256SUMS file
if SUMS="$(api "https://github.com/$REPO/releases/download/$TAG/SHA256SUMS" 2>/dev/null || true)"; then
  EXPECTED="$(printf '%s\n' "$SUMS" | awk -v name="$ASSET_NAME" '$2 == name || $2 ~ ("/" name "$") { print $1; exit }')"
  if [ -n "$EXPECTED" ]; then
    ACTUAL="$(sha256sum "$TMP/$ASSET_NAME" | awk '{print $1}')"
    [ "$EXPECTED" = "$ACTUAL" ] || die "checksum mismatch for $ASSET_NAME"
    log "==> checksum verified"
  fi
fi

mkdir -p "$INSTALL_DIR"
tar -xzf "$TMP/$ASSET_NAME" -C "$TMP"

if [ "$USE_SUDO" = "1" ]; then
  sudo mv "$TMP/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
else
  mv "$TMP/$BINARY_NAME" "$INSTALL_DIR/$BINARY_NAME"
fi
chmod +x "$INSTALL_DIR/$BINARY_NAME"

log "==> installed amnesia to $INSTALL_DIR"

if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
  warn "==> $INSTALL_DIR is not on your PATH."
  warn "    to use amnesia, add it to your shell config, e.g.:"
  warn "      echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.bashrc"
  warn "      echo 'export PATH=\"\$PATH:$INSTALL_DIR\"' >> ~/.zshrc"
  export PATH="$PATH:$INSTALL_DIR"
fi

"$INSTALL_DIR/$BINARY_NAME" --version
log "==> done. type 'amnesia' to get started."
