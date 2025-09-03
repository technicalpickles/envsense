#!/usr/bin/env bash

# test-release.sh
#
# Purpose: Test cross-compilation and release process locally before pushing
# Created: 2024-01-15
# Used for: Validating release workflow implementation
#
# This script tests the cross-compilation targets used in the release workflow
# to catch issues before they occur in CI.

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

print_header() {
    echo
    print_status "$BLUE" "=== $1 ==="
}

print_success() {
    print_status "$GREEN" "✓ $1"
}

print_warning() {
    print_status "$YELLOW" "⚠ $1"
}

print_error() {
    print_status "$RED" "✗ $1"
}

# Get current version from Cargo.toml
get_version() {
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

# Note: Build and binary preparation logic moved to modular scripts:
# - scripts/build-target.sh
# - scripts/prepare-binary.sh

# Main test function
main() {
    print_header "Release Build Test"
    
    # Check prerequisites
    if ! command -v cargo >/dev/null 2>&1; then
        print_error "cargo not found. Please install Rust."
        exit 1
    fi
    
    if ! command -v rustup >/dev/null 2>&1; then
        print_error "rustup not found. Please install rustup."
        exit 1
    fi
    
    # Get version
    VERSION=$(get_version)
    print_status "$GREEN" "Current version: $VERSION"
    
    # Create dist directory
    mkdir -p dist
    
    # Test targets (matching release.yml) - using arrays for bash 3.2 compatibility
    local targets=(
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
        "x86_64-apple-darwin"
        "aarch64-apple-darwin"
        "universal-apple-darwin"
        "x86_64-pc-windows-msvc"
    )
    
    local build_types=(
        "false"
        "true"
        "false"
        "false"
        "universal"
        "false"
    )
    
    local failed_targets=()
    local successful_targets=()
    
    for i in "${!targets[@]}"; do
        target="${targets[$i]}"
        build_type="${build_types[$i]}"
        
        print_header "Building $target"
        
        # Skip cross-compilation targets on non-Linux hosts for now
        if [ "$build_type" = "true" ] && [[ "$OSTYPE" != "linux-gnu"* ]]; then
            print_warning "Skipping cross-compilation target $target on non-Linux host"
            continue
        fi
        
        # Skip macOS targets on non-macOS hosts
        if [[ "$target" == *"apple"* ]] && [[ "$OSTYPE" != "darwin"* ]]; then
            print_warning "Skipping macOS target $target on non-macOS host"
            continue
        fi
        
        # Skip Windows targets on non-Windows hosts (unless using cross)
        if [[ "$target" == *"windows"* ]] && [[ "$OSTYPE" != "msys" ]] && [ "$build_type" = "false" ]; then
            print_warning "Skipping Windows target $target on non-Windows host"
            continue
        fi
        
        # Use the new modular scripts
        if ./scripts/build-target.sh "$target" "$build_type"; then
            print_success "Build successful for $target"
            
            # Prepare binary using the script
            if ./scripts/prepare-binary.sh "$VERSION" "$target"; then
                print_success "Binary prepared for $target"
                successful_targets+=("$target")
            else
                print_error "Binary preparation failed for $target"
                failed_targets+=("$target")
            fi
        else
            print_error "Build failed for $target"
            failed_targets+=("$target")
        fi
    done
    
    # Summary
    print_header "Test Results"
    
    if [ ${#successful_targets[@]} -gt 0 ]; then
        print_success "Successful targets:"
        for target in "${successful_targets[@]}"; do
            echo "  - $target"
        done
    fi
    
    if [ ${#failed_targets[@]} -gt 0 ]; then
        print_error "Failed targets:"
        for target in "${failed_targets[@]}"; do
            echo "  - $target"
        done
    fi
    
    # Show generated files
    if [ -d "dist" ] && [ "$(ls -A dist)" ]; then
        print_header "Generated Files"
        ls -la dist/
    fi
    
    # Exit with error if any targets failed
    if [ ${#failed_targets[@]} -gt 0 ]; then
        print_error "Some targets failed. Release workflow may have issues."
        exit 1
    else
        print_success "All tested targets built successfully!"
        print_status "$GREEN" "Release workflow should work correctly."
    fi
}

# Show usage if --help is passed
if [[ "${1:-}" == "--help" ]] || [[ "${1:-}" == "-h" ]]; then
    echo "Usage: $0"
    echo
    echo "Test cross-compilation and release process locally."
    echo
    echo "This script:"
    echo "  1. Tests building for all release targets"
    echo "  2. Validates binary functionality where possible"
    echo "  3. Generates release-style binaries in dist/"
    echo "  4. Creates checksums for verification"
    echo
    echo "Prerequisites:"
    echo "  - Rust toolchain with rustup"
    echo "  - cross (installed automatically if needed)"
    echo
    echo "Note: Some targets may be skipped based on host platform."
    exit 0
fi

main "$@"
