#!/bin/bash
set -e

# Update package index and install required system packages
if command -v apt-get >/dev/null 2>&1; then
    sudo apt-get update
    sudo apt-get install -y build-essential pkg-config libssl-dev libarchive-dev curl git
elif command -v pacman >/dev/null 2>&1; then
    sudo pacman -Sy --noconfirm --needed base-devel pkgconf openssl libarchive curl git
else
    echo "Please install build tools, pkg-config, openssl, libarchive, curl, and git using your system's package manager." >&2
fi

# Install rustup and the stable toolchain if not already installed
if ! command -v rustup >/dev/null 2>&1; then
    curl https://sh.rustup.rs -sSf | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Ensure required rust components are installed
rustup toolchain install stable
rustup component add --toolchain stable rustfmt clippy

echo "Setup complete."

