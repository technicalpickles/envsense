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

# Test if a target can be built
test_target() {
    local target=$1
    local build_type=${2:-false}
    
    print_status "$BLUE" "Testing target: $target"
    
    if [ "$build_type" = "true" ]; then
        if ! command -v cross >/dev/null 2>&1; then
            print_warning "cross not installed, installing..."
            cargo install cross --git https://github.com/cross-rs/cross
        fi
        
        if cross build --release --target "$target"; then
            print_success "Cross-compilation successful for $target"
            return 0
        else
            print_error "Cross-compilation failed for $target"
            return 1
        fi
    elif [ "$build_type" = "universal" ]; then
        # Build universal binary for macOS
        print_status "$BLUE" "Building universal binary for macOS..."
        
        # Check if both targets are installed
        for arch_target in "x86_64-apple-darwin" "aarch64-apple-darwin"; do
            if ! rustup target list --installed | grep -q "$arch_target"; then
                print_status "$YELLOW" "Installing target $arch_target..."
                rustup target add "$arch_target"
            fi
        done
        
        # Build for both architectures
        if cargo build --release --target x86_64-apple-darwin && \
           cargo build --release --target aarch64-apple-darwin; then
            
            # Check if lipo is available (macOS only)
            if ! command -v lipo >/dev/null 2>&1; then
                print_error "lipo not available - universal binaries only work on macOS"
                return 1
            fi
            
            # Create universal binary
            mkdir -p "target/universal-apple-darwin/release"
            if lipo -create \
                "target/x86_64-apple-darwin/release/envsense" \
                "target/aarch64-apple-darwin/release/envsense" \
                -output "target/universal-apple-darwin/release/envsense"; then
                
                print_success "Universal binary created successfully"
                
                # Verify the universal binary
                print_status "$BLUE" "Verifying universal binary..."
                lipo -info "target/universal-apple-darwin/release/envsense"
                return 0
            else
                print_error "Failed to create universal binary"
                return 1
            fi
        else
            print_error "Failed to build one or both architectures for universal binary"
            return 1
        fi
    else
        # Check if target is installed
        if ! rustup target list --installed | grep -q "$target"; then
            print_status "$YELLOW" "Installing target $target..."
            rustup target add "$target"
        fi
        
        if cargo build --release --target "$target"; then
            print_success "Compilation successful for $target"
            return 0
        else
            print_error "Compilation failed for $target"
            return 1
        fi
    fi
}

# Test binary functionality
test_binary() {
    local target=$1
    local binary_path="target/$target/release/envsense"
    
    # Add .exe for Windows targets
    if [[ "$target" == *"windows"* ]]; then
        binary_path="${binary_path}.exe"
    fi
    
    if [ ! -f "$binary_path" ]; then
        print_error "Binary not found: $binary_path"
        return 1
    fi
    
    print_status "$BLUE" "Testing binary: $binary_path"
    
    # Test help command
    if "$binary_path" --help >/dev/null 2>&1; then
        print_success "Help command works"
    else
        print_error "Help command failed"
        return 1
    fi
    
    # Test info command
    if "$binary_path" info --json >/dev/null 2>&1; then
        print_success "Info command works"
    else
        print_error "Info command failed"
        return 1
    fi
    
    # Test check command
    if "$binary_path" check --list >/dev/null 2>&1; then
        print_success "Check command works"
    else
        print_error "Check command failed"
        return 1
    fi
    
    return 0
}

# Generate binary name like the release workflow
generate_binary_name() {
    local version=$1
    local target=$2
    
    if [[ "$target" == *"windows"* ]]; then
        echo "envsense-v${version}-${target}.exe"
    else
        echo "envsense-v${version}-${target}"
    fi
}

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
        
        if test_target "$target" "$build_type"; then
            successful_targets+=("$target")
            
            # Test the binary if it's for the current platform
            if [[ "$OSTYPE" == "linux-gnu"* ]] && [[ "$target" == "x86_64-unknown-linux-gnu" ]]; then
                test_binary "$target"
            elif [[ "$OSTYPE" == "darwin"* ]] && [[ "$target" == *"apple-darwin" ]]; then
                # Test if it's the right architecture or universal
                if [[ "$target" == "universal-apple-darwin" ]]; then
                    test_binary "$target"
                elif [[ "$(uname -m)" == "x86_64" ]] && [[ "$target" == "x86_64-apple-darwin" ]]; then
                    test_binary "$target"
                elif [[ "$(uname -m)" == "arm64" ]] && [[ "$target" == "aarch64-apple-darwin" ]]; then
                    test_binary "$target"
                fi
            fi
            
            # Copy to dist with release naming
            binary_name=$(generate_binary_name "$VERSION" "$target")
            source_path="target/$target/release/envsense"
            if [[ "$target" == *"windows"* ]]; then
                source_path="${source_path}.exe"
            fi
            
            if [ -f "$source_path" ]; then
                cp "$source_path" "dist/$binary_name"
                chmod +x "dist/$binary_name" 2>/dev/null || true
                print_success "Copied to dist/$binary_name"
                
                # Generate checksum
                if command -v sha256sum >/dev/null 2>&1; then
                    (cd dist && sha256sum "$binary_name" > "${binary_name}.sha256")
                elif command -v shasum >/dev/null 2>&1; then
                    (cd dist && shasum -a 256 "$binary_name" > "${binary_name}.sha256")
                fi
            fi
        else
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
