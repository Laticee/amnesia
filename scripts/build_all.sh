#!/bin/bash
set -e

# Configuration
BINARY_NAME="amnesia"
RELEASE_DIR="releases"
TARGETS=(
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-unknown-linux-musl"
    "aarch64-unknown-linux-musl"
)

# Ensure release directory exists
mkdir -p "$RELEASE_DIR"

echo "Starting multi-platform build for $BINARY_NAME..."

for TARGET in "${TARGETS[@]}"; do
    echo "--- Building for $TARGET ---"
    
    # Check if we should use cross or cargo
    if [[ "$TARGET" == *"apple-darwin"* ]]; then
        # Use cargo locally for macOS targets
        cargo build --release --target "$TARGET"
    else
        # Use cross for Linux targets
        cross build --release --target "$TARGET"
    fi

    # Determine binary paths and asset names
    ASSET_OS=""
    ASSET_ARCH=""

    if [[ "$TARGET" == "x86_64-apple-darwin" ]]; then
        ASSET_OS="macos"
        ASSET_ARCH="x86_64"
    elif [[ "$TARGET" == "aarch64-apple-darwin" ]]; then
        ASSET_OS="macos"
        ASSET_ARCH="aarch64"
    elif [[ "$TARGET" == "x86_64-unknown-linux-musl" ]]; then
        ASSET_OS="linux"
        ASSET_ARCH="x86_64"
    elif [[ "$TARGET" == "aarch64-unknown-linux-musl" ]]; then
        ASSET_OS="linux"
        ASSET_ARCH="aarch64"
    fi

    ASSET_NAME="${BINARY_NAME}-${ASSET_OS}-${ASSET_ARCH}"
    
    # Copy and package
    SOURCE_PATH="target/$TARGET/release/$BINARY_NAME"
    DEST_PATH="$RELEASE_DIR/$ASSET_NAME"
    
    cp "$SOURCE_PATH" "$DEST_PATH"
    tar -czf "${DEST_PATH}.tar.gz" -C "$RELEASE_DIR" "$ASSET_NAME"
    rm "$DEST_PATH"

    echo "Finished $TARGET: ${ASSET_NAME}.tar.gz"
done

echo "All builds completed. Assets are in $RELEASE_DIR/"
