#!/usr/bin/bash

set -euo pipefail

OS="$1"

case "$OS" in
  "ubuntu")
    if ! sudo apt install -y clang lld; then
      echo "LLD installation failed"
      exit 1
    fi
    ;;
  "macos")
    if ! brew install llvm lld; then
      echo "LLVM/LLD installation failed"
      exit 1
    fi


    if [ -n "${GITHUB_ENV:-}" ]; then
        echo "CC=clang" >> "$GITHUB_ENV"
        echo "CXX=clang++" >> "$GITHUB_ENV"
    fi
    ;;
  *)
    echo "Invalid OS: $OS"
    exit 1
esac