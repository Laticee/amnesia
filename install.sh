#!/bin/bash
set -e

# Configuration
REPO="laticee/amnesia"
BINARY_NAME="amnesia"
INSTALL_DIR="/usr/local/bin"

# Detect OS and Architecture
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

case "$OS" in
  linux)
    ASSET_OS="linux"
    ;;
  darwin)
    ASSET_OS="macos"
    ;;
  *)
    echo "Error: Unsupported OS: $OS"
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64)
    ASSET_ARCH="x86_64"
    ;;
  arm64|aarch64)
    ASSET_ARCH="aarch64"
    ;;
  *)
    echo "Error: Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

# Fallback for Intel Macs if needed
if [ "$ASSET_OS" == "macos" ] && [ "$ASSET_ARCH" == "aarch64" ]; then
    # Check if we should use x86_64 or aarch64
    if [[ "$(sysctl -n hw.optional.arm64 2>/dev/null)" != "1" ]]; then
        ASSET_ARCH="x86_64"
    fi
fi

# Construct filename
ASSET_NAME="amnesia-${ASSET_OS}-${ASSET_ARCH}.tar.gz"

echo "Downloading ${ASSET_NAME} from ${REPO}..."

# Get latest release tag
LATEST_TAG=$(curl -s "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"tag_name": "([^"]+)".*/\1/')

if [ -z "$LATEST_TAG" ]; then
    echo "Error: Could not find latest release for ${REPO}"
    exit 1
fi

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST_TAG}/${ASSET_NAME}"

# Download and extract
tmp_dir=$(mktemp -d)
curl -L "$DOWNLOAD_URL" -o "${tmp_dir}/${ASSET_NAME}"
tar -xzf "${tmp_dir}/${ASSET_NAME}" -C "${tmp_dir}"

echo "Installing ${BINARY_NAME} to ${INSTALL_DIR}..."
if [ -w "$INSTALL_DIR" ]; then
    mv "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/"
else
    echo "Requesting sudo for installation to ${INSTALL_DIR}"
    sudo mv "${tmp_dir}/${BINARY_NAME}" "${INSTALL_DIR}/"
fi

rm -rf "$tmp_dir"

echo "Successfully installed amnesia!"
amnesia --version
