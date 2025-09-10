#!/usr/bin/env bash
# Validates that release signing is working correctly

set -euo pipefail

VERSION="${1:-latest}"
REPO="technicalpickles/envsense"

# If no version specified, get current version from Cargo.toml if available
if [ "$VERSION" = "latest" ] && [ -f "Cargo.toml" ]; then
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    if [ -n "$CURRENT_VERSION" ]; then
        VERSION="$CURRENT_VERSION"
        echo "Using current version from Cargo.toml: $VERSION"
    fi
fi

echo "Validating signing for version: $VERSION"

# Check if cosign is installed
if ! command -v cosign &> /dev/null; then
    echo "❌ cosign is not installed. Please install it first:"
    echo "   brew install cosign"
    echo "   or download from: https://github.com/sigstore/cosign/releases"
    exit 1
fi

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed. Please install it first:"
    echo "   brew install gh"
    exit 1
fi

# Download release assets to temp directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "Working in temporary directory: $TEMP_DIR"

# Download release assets using GitHub CLI
echo "Downloading release assets for version: $VERSION"
if [ "$VERSION" = "latest" ]; then
    # Get the latest release tag first
    RELEASE_TAG=$(gh release list --repo "$REPO" --limit 1 --json tagName --jq '.[0].tagName')
    echo "Latest release tag: $RELEASE_TAG"
    gh release download "$RELEASE_TAG" --repo "$REPO" --pattern "envsense-*"
else
    # Download specific version
    gh release download "$VERSION" --repo "$REPO" --pattern "envsense-*"
fi

echo "Downloaded files:"
ls -la

# Verify signatures
echo
echo "Verifying signatures..."
SUCCESS=true

for binary in envsense-*; do
    if [[ "$binary" != *.sha256 && "$binary" != *.sig && "$binary" != *.bundle ]]; then
        bundle_file="${binary}.bundle"
        sig_file="${binary}.sig"
        
        echo "  Verifying: $binary"
        
        # Try bundle verification first (more reliable)
        if [ -f "$bundle_file" ]; then
            echo "    Trying bundle verification..."
            if cosign verify-blob --bundle "$bundle_file" "$binary" > /dev/null 2>&1; then
                echo "  ✅ $binary bundle signature valid"
                continue
            else
                echo "    Bundle verification failed, trying signature..."
            fi
        fi
        
        # Fall back to signature verification
        if [ -f "$sig_file" ]; then
            echo "    Trying signature verification..."
            # Try different cosign verification approaches
            if cosign verify-blob \
                --signature "$sig_file" \
                --certificate-identity "https://github.com/$REPO/.github/workflows/release.yml@refs/heads/main" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$binary" > /dev/null 2>&1; then
                echo "  ✅ $binary signature valid (exact identity match)"
            elif cosign verify-blob \
                --signature "$sig_file" \
                --certificate-identity-regexp "https://github.com/$REPO" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$binary" > /dev/null 2>&1; then
                echo "  ✅ $binary signature valid (regexp match)"
            elif cosign verify-blob \
                --signature "$sig_file" \
                --certificate-identity-regexp ".*$REPO.*" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$binary" > /dev/null 2>&1; then
                echo "  ✅ $binary signature valid (loose regexp match)"
            else
                echo "  ❌ $binary signature verification failed"
                echo "  🔍 Debugging info:"
                echo "      Repository: $REPO"
                echo "      Signature file: $sig_file"
                echo "      Bundle file: $bundle_file (exists: $([ -f "$bundle_file" ] && echo "yes" || echo "no"))"
                echo "      Binary: $binary"
                # Try verbose verification for debugging
                echo "  🔍 Attempting verbose verification:"
                cosign verify-blob \
                    --signature "$sig_file" \
                    --certificate-identity-regexp ".*$REPO.*" \
                    --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                    "$binary" || true
                SUCCESS=false
            fi
        else
            echo "  ❌ Missing signature and bundle for: $binary"
            SUCCESS=false
        fi
    fi
done

echo
if [ "$SUCCESS" = true ]; then
    echo "🎉 All signatures validated successfully!"
    EXIT_CODE=0
else
    echo "💥 Some signature validations failed!"
    EXIT_CODE=1
fi

# Cleanup
cd - > /dev/null
rm -rf "$TEMP_DIR"

exit $EXIT_CODE
