#!/usr/bin/env bash
# Test aqua installation locally using our registry configuration
# This script validates that the aqua registry configuration works correctly

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEST_DIR="$PROJECT_ROOT/tmp/aqua-installation-test"

echo "🧪 Testing aqua installation for envsense"
echo "Project root: $PROJECT_ROOT"
echo "Test directory: $TEST_DIR"

# Clean up any previous test
rm -rf "$TEST_DIR"
mkdir -p "$TEST_DIR"
cd "$TEST_DIR"

# Create registry configuration
echo "📝 Creating test registry configuration..."
cp "$PROJECT_ROOT/aqua-registry-entry.yaml" registry.yaml

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/version = "\(.*\)"/\1/')
echo "📋 Testing with version: $CURRENT_VERSION"

# Create aqua configuration
cat > aqua.yaml << EOF
---
registries:
- type: local
  name: envsense-local
  path: registry.yaml
packages:
- name: technicalpickles/envsense@$CURRENT_VERSION
  registry: envsense-local
EOF

# Create policy configuration
cat > aqua-policy.yaml << 'EOF'
---
registries:
  - name: envsense-local
    type: local
    path: registry.yaml
packages:
  - name: technicalpickles/envsense
    registry: envsense-local
EOF

echo "📦 Installing envsense via aqua..."
AQUA_POLICY_CONFIG=aqua-policy.yaml mise exec aqua -- aqua install

echo "🔍 Testing installed binary..."
AQUA_POLICY_CONFIG=aqua-policy.yaml mise exec aqua -- envsense --help

echo "🎯 Testing envsense functionality..."
AQUA_POLICY_CONFIG=aqua-policy.yaml mise exec aqua -- envsense info

echo "✅ Aqua installation test completed successfully!"
echo ""
echo "📍 Binary location:"
AQUA_POLICY_CONFIG=aqua-policy.yaml mise exec aqua -- aqua which envsense

echo ""
echo "🚀 Ready for submission to aqua registry!"
echo "Next steps:"
echo "1. Submit aqua-registry-entry.yaml to https://github.com/aquaproj/aqua-registry"
echo "2. Update project documentation with installation instructions"