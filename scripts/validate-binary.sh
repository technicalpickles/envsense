#!/usr/bin/env bash

# validate-binary.sh
#
# Purpose: Validate binary preparation and functionality
# Usage: ./validate-binary.sh <version> <target>
#
# Special targets:
#   - "linux": Validates both x86_64 and aarch64 Linux binaries
#   - Other targets: Validates single binary for that target

set -euo pipefail

VERSION="$1"
TARGET="$2"

# Function to validate a single binary
validate_single_binary() {
    local version="$1"
    local target="$2"
    
    echo "Preparing binary for $target..."
    if ! ./scripts/prepare-binary.sh "$version" "$target"; then
        echo "Failed to prepare binary for $target"
        return 1
    fi
    
    # Use the same naming logic as prepare-binary.sh
    case "$target" in
      "universal-apple-darwin")
        BINARY="dist/envsense-${version}-universal-apple-darwin"
        ;;
      *)
        BINARY="dist/envsense-${version}-${target}"
        ;;
    esac
    
    echo "Testing binary: $BINARY"
    
    # Test binary functionality
    if [ -f "$BINARY" ]; then
        # Test that binary runs and shows help
        "./$BINARY" --help > /dev/null
        
        # Test basic functionality  
        "./$BINARY" info --json | head -5 > /dev/null
        
        echo "✓ Binary functionality test passed for $target"
        return 0
    else
        echo "✗ Binary not found: $BINARY"
        ls -la dist/
        return 1
    fi
}

echo "Testing binary preparation script..."

# Handle special "linux" target for cross-compilation validation
if [[ "$TARGET" == "linux" ]]; then
    echo "Validating Linux cross-compiled binaries for version $VERSION..."
    
    # Define Linux targets
    LINUX_TARGETS=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")
    
    # Validate each Linux target
    for target in "${LINUX_TARGETS[@]}"; do
        if ! validate_single_binary "$VERSION" "$target"; then
            echo "Validation failed for $target"
            exit 1
        fi
    done
    
    echo "All Linux binary validations passed!"
else
    # Single target validation
    if ! validate_single_binary "$VERSION" "$TARGET"; then
        echo "Validation failed for $TARGET"
        exit 1
    fi
    
    echo "Binary validation passed!"
fi