#!/usr/bin/env bash
# Sign release binaries with cosign

set -euo pipefail

RELEASE_DIR="${1:-release-files}"

if [ ! -d "$RELEASE_DIR" ]; then
    echo "❌ Release directory $RELEASE_DIR does not exist"
    exit 1
fi

echo "🔐 Signing release binaries with cosign..."
cd "$RELEASE_DIR"

# Check if cosign is available
if ! command -v cosign &> /dev/null; then
    echo "❌ cosign is not available"
    exit 1
fi

SIGNED_COUNT=0

for file in envsense-*; do
    if [[ "$file" != *.sha256 && "$file" != *.sig && "$file" != *.bundle ]]; then
        echo "  🔏 Signing: $file"
        
        # Create both signature and bundle formats for compatibility
        echo "    Creating signature file: ${file}.sig"
        cosign sign-blob --yes "$file" --output-signature "${file}.sig"
        
        echo "    Creating bundle file: ${file}.bundle"
        cosign sign-blob --yes "$file" --bundle "${file}.bundle"
        
        SIGNED_COUNT=$((SIGNED_COUNT + 1))
    fi
done

echo
echo "✅ Signing completed. Signed $SIGNED_COUNT files."
echo "📁 Release files:"
ls -la

if [ $SIGNED_COUNT -eq 0 ]; then
    echo "⚠️  No files were signed. This might indicate an issue."
    exit 1
fi
