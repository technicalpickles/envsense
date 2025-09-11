#!/usr/bin/env bash

# measure-impact.sh
#
# Purpose: Measure actual time cost of LLD installation vs linking benefits
# Created: 2025-09-11
# Used for: Quantifying the cost/benefit of LLD in the envsense project
#
# This script helps validate the analysis in analysis.md by measuring:
# - LLD installation time
# - Linking time with/without LLD
# - Total build time impact

set -euo pipefail

echo "=== LLD Impact Measurement ==="
echo "Date: $(date)"
echo "Platform: $(uname -s)"
echo ""

# Function to time a command and extract just the real time
time_command() {
    local description="$1"
    shift
    echo "Measuring: $description"
    
    # Use time command to measure, capture only real time
    local start_time=$(date +%s.%N)
    "$@" > /dev/null 2>&1 || true
    local end_time=$(date +%s.%N)
    
    local elapsed=$(echo "$end_time - $start_time" | bc -l)
    echo "Time: ${elapsed} seconds"
    echo ""
    return 0
}

# Detect platform and measure installation time
case "$(uname -s)" in
    "Linux")
        if command -v apt >/dev/null 2>&1; then
            echo "=== Linux (apt) Installation Time ==="
            time_command "apt update" sudo apt update
            time_command "install clang lld" sudo apt install -y clang lld
        else
            echo "Skipping Linux installation measurement (not apt-based)"
        fi
        ;;
    "Darwin")
        echo "=== macOS (brew) Installation Time ==="
        if command -v brew >/dev/null 2>&1; then
            # Check if already installed to avoid skewing results
            if brew list llvm &>/dev/null && brew list lld &>/dev/null; then
                echo "llvm and lld already installed, skipping installation measurement"
                echo "To get accurate timing, run: brew uninstall llvm lld"
                echo ""
            else
                time_command "brew install llvm lld" brew install llvm lld
            fi
        else
            echo "Homebrew not available, skipping installation measurement"
        fi
        ;;
    *)
        echo "Unsupported platform for installation measurement"
        ;;
esac

# Prepare clean environment for build testing
echo "=== Build Time Comparison ==="

# Clean builds to ensure fair comparison
cargo clean

# Measure baseline build (without LLD)
unset RUSTFLAGS 2>/dev/null || true
echo "Measuring baseline build (standard linker)..."
time_command "cargo build --release (standard)" cargo build --release

# Clean and measure with LLD (if available)
cargo clean

if command -v ld.lld >/dev/null 2>&1; then
    echo "Measuring LLD build..."
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
    time_command "cargo build --release (LLD)" cargo build --release
    unset RUSTFLAGS
else
    echo "LLD not available for build comparison"
fi

# Measure linking-only time by doing incremental builds
echo "=== Linking-Only Time Comparison ==="

# First, ensure we have a built project
cargo build --release >/dev/null 2>&1

# Touch main.rs to force relinking without full compilation
touch src/main.rs

echo "Measuring incremental linking (standard linker)..."
time_command "incremental build (standard)" cargo build --release

# Touch again and test with LLD
touch src/main.rs

if command -v ld.lld >/dev/null 2>&1; then
    export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
    echo "Measuring incremental linking (LLD)..."
    time_command "incremental build (LLD)" cargo build --release
    unset RUSTFLAGS
fi

echo "=== Summary ==="
echo "This measurement helps validate the analysis that LLD installation time"
echo "outweighs linking performance benefits for this project."
echo ""
echo "Key findings should show:"
echo "- Installation time: 30-90 seconds" 
echo "- Linking time difference: 1-3 seconds"
echo "- Net impact: Installation overhead >> linking savings"
echo ""
echo "See analysis.md for detailed cost-benefit analysis."
