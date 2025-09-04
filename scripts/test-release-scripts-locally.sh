#!/usr/bin/env bash

# test-release-scripts-locally.sh
#
# Purpose: Test all release scripts locally before pushing
# Usage: ./test-release-scripts-locally.sh
#
# This script validates that all release scripts work correctly
# on the current platform without requiring GitHub Actions.

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

# Detect current platform
detect_platform() {
    case "$OSTYPE" in
        linux-gnu*)
            echo "x86_64-unknown-linux-gnu"
            ;;
        darwin*)
            if [[ "$(uname -m)" == "arm64" ]]; then
                echo "aarch64-apple-darwin"
            else
                echo "x86_64-apple-darwin"
            fi
            ;;
        msys*|cygwin*)
            echo "x86_64-pc-windows-msvc"
            ;;
        *)
            echo "unknown"
            ;;
    esac
}

# Main test function
main() {
    print_header "Release Scripts Local Test"
    
    # Check prerequisites
    if ! command -v cargo >/dev/null 2>&1; then
        print_error "cargo not found. Please install Rust."
        exit 1
    fi
    
    local platform
    platform=$(detect_platform)
    print_status "$GREEN" "Detected platform: $platform"
    
    local test_version="0.1.0-test-$(date +%s)"
    print_status "$GREEN" "Using test version: $test_version"
    
    # Test 1: Version change detection
    print_header "Testing Version Change Detection"
    if ./scripts/check-version-change.sh; then
        print_success "Version change detection script works"
    else
        print_error "Version change detection script failed"
        return 1
    fi
    
    # Test 2: Build script for current platform
    print_header "Testing Build Script"
    local build_type="normal"
    
    # Use universal build for macOS if available
    if [[ "$platform" == *"apple-darwin" ]] && command -v lipo >/dev/null 2>&1; then
        platform="universal-apple-darwin"
        build_type="universal"
        print_status "$BLUE" "Using universal binary for macOS"
    fi
    
    # Note: Cross-compilation testing is only available in CI on Linux
    # Local testing focuses on the current platform
    
    if ./scripts/build-target.sh "$platform" "$build_type"; then
        print_success "Build script works for $platform"
    else
        print_error "Build script failed for $platform"
        return 1
    fi
    
    # Test 3: Binary preparation
    print_header "Testing Binary Preparation"
    if ./scripts/prepare-binary.sh "$test_version" "$platform"; then
        print_success "Binary preparation script works"
    else
        print_error "Binary preparation script failed"
        return 1
    fi
    
    # Test 4: Verify prepared binary
    print_header "Testing Prepared Binary"
    local binary_name
    if [[ "$platform" == *"windows"* ]]; then
        binary_name="envsense-${test_version}-${platform}.exe"
    else
        binary_name="envsense-${test_version}-${platform}"
    fi
    
    local binary_path="dist/$binary_name"
    if [ -f "$binary_path" ]; then
        print_status "$BLUE" "Testing binary: $binary_path"
        
        # Test basic functionality (no --version flag available)
        if "./$binary_path" --help >/dev/null 2>&1; then
            print_success "Binary help command works"
        else
            print_error "Binary help command failed"
            return 1
        fi
        
        if "./$binary_path" info --json >/dev/null 2>&1; then
            print_success "Binary info command works"
        else
            print_error "Binary info command failed"
            return 1
        fi
        
        # Check checksum file
        if [ -f "${binary_path}.sha256" ]; then
            print_success "Checksum file created"
        else
            print_warning "Checksum file not found"
        fi
        
    else
        print_error "Prepared binary not found: $binary_path"
        print_status "$BLUE" "Available files in dist/:"
        ls -la dist/ || echo "dist/ directory not found"
        return 1
    fi
    
    # Test 5: Release notes creation
    print_header "Testing Release Notes Creation"
    if ./scripts/create-release.sh "$test_version"; then
        print_success "Release notes creation script works"
        
        if [ -f "changelog_excerpt.md" ]; then
            print_success "Changelog excerpt created"
            print_status "$BLUE" "Changelog content:"
            cat changelog_excerpt.md
        else
            print_warning "Changelog excerpt not created"
        fi
    else
        print_error "Release notes creation script failed"
        return 1
    fi
    
    # Test 6: Universal binary verification (macOS only)
    if [[ "$platform" == "universal-apple-darwin" ]] && command -v lipo >/dev/null 2>&1; then
        print_header "Testing Universal Binary"
        local universal_binary="target/universal-apple-darwin/release/envsense"
        
        if [ -f "$universal_binary" ]; then
            print_status "$BLUE" "Verifying universal binary architecture:"
            if lipo -info "$universal_binary"; then
                print_success "Universal binary verification passed"
            else
                print_error "Universal binary verification failed"
                return 1
            fi
        else
            print_error "Universal binary not found: $universal_binary"
            return 1
        fi
    fi
    
    # Summary
    print_header "Test Summary"
    print_success "All release scripts work correctly on $platform!"
    
    if [ -d "dist" ]; then
        print_status "$BLUE" "Generated test files:"
        ls -la dist/
    fi
    
    # Cleanup option (non-interactive by default)
    if [[ "${INTERACTIVE:-false}" == "true" ]]; then
        echo
        read -p "Clean up test files? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf dist/
            rm -f changelog_excerpt.md
            print_success "Test files cleaned up"
        else
            print_status "$BLUE" "Test files preserved for inspection"
        fi
    else
        print_status "$BLUE" "Test files preserved for inspection (use --interactive for cleanup prompt)"
    fi
    
    print_success "Release scripts validation completed successfully!"
}

# Parse command line arguments
INTERACTIVE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Test all release scripts locally on the current platform."
            echo
            echo "Options:"
            echo "  --interactive    Enable interactive cleanup prompt"
            echo "  --help, -h       Show this help message"
            echo
            echo "This script:"
            echo "  1. Tests version change detection"
            echo "  2. Tests building for the current platform"
            echo "  3. Tests binary preparation and validation"
            echo "  4. Tests release notes creation"
            echo "  5. Verifies universal binary creation (macOS only)"
            echo
            echo "Prerequisites:"
            echo "  - Rust toolchain with cargo"
            echo "  - Platform-specific build tools"
            echo "  - lipo (for macOS universal binaries)"
            echo
            echo "Environment variables:"
            echo "  INTERACTIVE=true    Same as --interactive flag"
            exit 0
            ;;
        --interactive)
            INTERACTIVE=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

main
