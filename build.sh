#!/bin/bash
set -e

# Build mode: default to 'release' unless overridden
BUILD_MODE=${BUILD_MODE:-release}
echo "📦 Building in $BUILD_MODE mode..."

if [ "$BUILD_MODE" = "debug" ]; then
    cargo build
    BIN_PATH="target/debug/partydeck-rs"
else
    cargo build --release
    BIN_PATH="target/release/partydeck-rs"
fi

# Verify binary exists
if [ ! -f "$BIN_PATH" ]; then
    echo "❌ Error: Binary not found at $BIN_PATH"
    exit 1
fi

# Prepare build output directory
echo "📁 Preparing build output..."
rm -rf build/
mkdir -p build

# Copy binary and required assets
cp "$BIN_PATH" build/partydeck-rs
cp -r res build/
cp res/PartyDeckKWinLaunch.sh build/

echo "✅ Build complete – files are in ./build"
