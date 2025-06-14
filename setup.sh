#!/bin/bash
set -e

# Update package index and install required system packages
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev libarchive-dev curl git

# Install rustup and the stable toolchain if not already installed
if ! command -v rustup >/dev/null 2>&1; then
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Ensure required rust components are installed
rustup toolchain install stable
rustup component add --toolchain stable rustfmt clippy

echo "Setup complete."

