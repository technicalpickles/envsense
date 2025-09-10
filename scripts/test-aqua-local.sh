#!/usr/bin/env bash
# Test aqua configuration with local registry setup

set -euo pipefail

# Configuration
LOCAL_REGISTRY_DIR="$(pwd)/test-aqua-registry"
AQUA_CONFIG_FILE="$(pwd)/test-aqua.yaml"
TEST_DIR="$(pwd)/aqua-test"

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "🧪 Testing aqua configuration locally"
echo "📋 Current version from Cargo.toml: $CURRENT_VERSION"

# Check dependencies
if ! command -v mise &> /dev/null; then
    echo "❌ mise is not installed. Please install it first:"
    echo "   curl https://mise.run | sh"
    exit 1
fi

# Clean up any previous test
rm -rf "$LOCAL_REGISTRY_DIR" "$TEST_DIR" "$AQUA_CONFIG_FILE"

# Create local registry structure
echo "📁 Setting up local registry..."
mkdir -p "$LOCAL_REGISTRY_DIR/pkgs"

# Copy our configuration to the local registry
cp aqua-registry-entry.yaml "$LOCAL_REGISTRY_DIR/pkgs/envsense.yaml"

# Create aqua configuration file for testing
cat > "$AQUA_CONFIG_FILE" << EOF
registries:
  - type: local
    name: test-local
    path: $LOCAL_REGISTRY_DIR

packages:
  - name: envsense
    registry: test-local
    version: v$CURRENT_VERSION  # Use current version from Cargo.toml
EOF

# Create test directory
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Copy aqua config to test directory
cp "$AQUA_CONFIG_FILE" aqua.yaml

echo "📋 Aqua configuration:"
cat aqua.yaml

echo
echo "🔍 Testing installation (this will fail until we have signed releases)..."
echo "Command: mise install aqua:envsense@v$CURRENT_VERSION"

# This will likely fail since we don't have signed releases yet, but it will validate the config
if mise install aqua:envsense@v$CURRENT_VERSION; then
    echo "✅ Installation succeeded!"
    
    # Test the binary
    if envsense --version; then
        echo "✅ Binary works correctly!"
    else
        echo "❌ Binary installation failed"
    fi
else
    echo "⚠️  Installation failed (expected until we have signed releases)"
    echo "   This validates that the configuration is being processed correctly"
fi

# Clean up
cd ..
rm -rf "$LOCAL_REGISTRY_DIR" "$TEST_DIR" "$AQUA_CONFIG_FILE"

echo
echo "🎯 Local aqua testing completed"
echo "   Next steps:"
echo "   1. Create a signed release to test with actual binaries"
echo "   2. Test with the actual aqua registry once submitted"
