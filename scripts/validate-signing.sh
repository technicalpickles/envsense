#!/usr/bin/env bash
# Validates that release signing is working correctly

set -euo pipefail

VERSION="${1:-latest}"
REPO="technicalpickles/envsense"

echo "Validating signing for version: $VERSION"

# Check if cosign is installed
if ! command -v cosign &> /dev/null; then
    echo "❌ cosign is not installed. Please install it first:"
    echo "   brew install cosign"
    echo "   or download from: https://github.com/sigstore/cosign/releases"
    exit 1
fi

# Check if jq is installed
if ! command -v jq &> /dev/null; then
    echo "❌ jq is not installed. Please install it first:"
    echo "   brew install jq"
    exit 1
fi

# Download release assets to temp directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "Working in temporary directory: $TEMP_DIR"

# Get release info and download assets
if [ "$VERSION" = "latest" ]; then
    RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
else
    RELEASE_URL="https://api.github.com/repos/$REPO/releases/tags/v$VERSION"
fi

echo "Fetching release information from: $RELEASE_URL"

# Download all assets
echo "Downloading release assets..."
curl -s "$RELEASE_URL" | jq -r '.assets[].browser_download_url' | while read -r url; do
    echo "  Downloading: $(basename "$url")"
    curl -sLO "$url"
done

# Verify signatures
echo
echo "Verifying signatures..."
SUCCESS=true

for binary in envsense-*; do
    if [[ "$binary" != *.sha256 && "$binary" != *.sig ]]; then
        sig_file="${binary}.sig"
        if [ -f "$sig_file" ]; then
            echo "  Verifying: $binary"
            if cosign verify-blob \
                --signature "$sig_file" \
                --certificate-identity-regexp "https://github.com/$REPO" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$binary" > /dev/null 2>&1; then
                echo "  ✅ $binary signature valid"
            else
                echo "  ❌ $binary signature verification failed"
                SUCCESS=false
            fi
        else
            echo "  ❌ Missing signature for: $binary"
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
