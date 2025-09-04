#!/usr/bin/env bash

set -euo pipefail

# Detect OS using uname
UNAME_OS="$(uname -s)"

# Function to detect if we're on Ubuntu
is_ubuntu() {
    if command -v lsb_release >/dev/null 2>&1; then
        lsb_release -i | grep -q "Ubuntu"
    elif [ -f /etc/os-release ]; then
        grep -q "^ID=ubuntu" /etc/os-release
    elif [ -f /etc/lsb-release ]; then
        grep -q "DISTRIB_ID=Ubuntu" /etc/lsb-release
    else
        return 1
    fi
}

case "$UNAME_OS" in
  "Linux")
    if is_ubuntu; then
        echo "Detected Ubuntu Linux, installing clang and lld..."
        if ! sudo apt update && sudo apt install -y clang lld; then
            echo "Warning: LLD installation failed, will use system linker"
            echo "This is expected on some platforms like ARM64"
            exit 0  # Don't fail the build, just warn
        fi
    else
        echo "Error: Only Ubuntu Linux is supported, but detected a different Linux distribution"
        exit 1
    fi
    ;;
  "Darwin")
    echo "Detected macOS, installing llvm and lld..."
    if ! brew install llvm lld; then
        echo "LLVM installation failed"
        exit 1
    fi

    # Set compiler environment variables for GitHub Actions
    if [ -n "${GITHUB_ENV:-}" ]; then
        echo "CC=clang" >> "$GITHUB_ENV"
        echo "CXX=clang++" >> "$GITHUB_ENV"
    fi
    ;;
  *)
    echo "Error: Unsupported OS '$UNAME_OS'. Only macOS and Ubuntu Linux are supported."
    exit 1
    ;;
esac