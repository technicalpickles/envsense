#!/usr/bin/env bash
# Debug signature verification to understand certificate identity issues

set -euo pipefail

REPO="${1:-technicalpickles/envsense}"
VERSION="${2:-0.3.3}"

echo "🔍 Debug Signature Verification"
echo "Repository: $REPO"
echo "Version: $VERSION"

# Download the release assets to debug
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

echo "Downloading release assets..."
gh release download "$VERSION" --repo "$REPO" --pattern "envsense-$VERSION-*"

echo "Files downloaded:"
ls -la

# Try to get certificate information from the signature
for file in envsense-*; do
    if [[ "$file" != *.sha256 && "$file" != *.sig && "$file" != *.bundle ]]; then
        echo
        echo "🔍 Debugging: $file"
        
        if [ -f "${file}.bundle" ]; then
            echo "Bundle file exists, trying to inspect it..."
            # Try to get information from the bundle
            echo "Bundle content preview:"
            head -c 500 "${file}.bundle" | strings | head -10 || echo "Could not extract readable strings"
        fi
        
        if [ -f "${file}.sig" ]; then
            echo "Signature file exists"
            echo "Signature content (base64):"
            cat "${file}.sig"
            
            echo
            echo "Trying verification with verbose output..."
            
            # Try verification with different identity formats and show errors
            echo "Attempt 1: Standard workflow path"
            cosign verify-blob \
                --signature "${file}.sig" \
                --certificate-identity "https://github.com/$REPO/.github/workflows/release.yml@refs/heads/main" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$file" 2>&1 || echo "Failed with standard path"
            
            echo
            echo "Attempt 2: Regexp pattern"
            cosign verify-blob \
                --signature "${file}.sig" \
                --certificate-identity-regexp "https://github.com/$REPO/.*" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$file" 2>&1 || echo "Failed with regexp pattern"
            
            echo
            echo "Attempt 3: Loose regexp"
            cosign verify-blob \
                --signature "${file}.sig" \
                --certificate-identity-regexp ".*$REPO.*" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$file" 2>&1 || echo "Failed with loose regexp"
        fi
        
        # Only debug first file to avoid too much output
        break
    fi
done

echo
echo "Cleaning up..."
cd - > /dev/null
rm -rf "$TEMP_DIR"
