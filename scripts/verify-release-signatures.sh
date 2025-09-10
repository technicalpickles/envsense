#!/usr/bin/env bash
# Verify release signatures immediately after creation

set -euo pipefail

RELEASE_DIR="${1:-release-files}"
REPO="${2:-$GITHUB_REPOSITORY}"
WORKFLOW="${3:-}"

if [ ! -d "$RELEASE_DIR" ]; then
    echo "❌ Release directory $RELEASE_DIR does not exist"
    exit 1
fi

if [ -z "$REPO" ]; then
    echo "❌ Repository not specified. Set GITHUB_REPOSITORY or pass as second argument."
    exit 1
fi

# Auto-detect workflow if not specified
if [ -z "$WORKFLOW" ]; then
    if [ -n "${GITHUB_WORKFLOW:-}" ]; then
        # Convert workflow display name to filename
        case "$GITHUB_WORKFLOW" in
            "Release") WORKFLOW="release.yml" ;;
            "Test Signing Process") WORKFLOW="test-signing.yml" ;;
            *) WORKFLOW="$GITHUB_WORKFLOW" ;;
        esac
        echo "🔍 Auto-detected workflow: $GITHUB_WORKFLOW -> $WORKFLOW"
    else
        WORKFLOW="release.yml"  # Default fallback
        echo "🔍 Using default workflow: $WORKFLOW"
    fi
fi

echo "🔍 Verifying signatures immediately after creation..."
echo "🔍 Repository: $REPO"
echo "🔍 Workflow: $WORKFLOW"
cd "$RELEASE_DIR"

# Check if cosign is available
if ! command -v cosign &> /dev/null; then
    echo "❌ cosign is not available"
    exit 1
fi

VERIFIED_COUNT=0
FAILED_COUNT=0

for file in envsense-*; do
    if [[ "$file" != *.sha256 && "$file" != *.sig && "$file" != *.bundle ]]; then
        echo "  🔍 Verifying signature for: $file"
        
        # Try bundle verification first, then fall back to signature verification
        if [ -f "${file}.bundle" ]; then
            echo "    Trying bundle verification..."
            echo "    Bundle command: cosign verify-blob --bundle ${file}.bundle $file"
            
            # Determine the branch reference for bundle verification
            BRANCH_REF="refs/heads/main"
            if [ -n "${GITHUB_HEAD_REF:-}" ]; then
                # This is a pull request, use the PR branch
                BRANCH_REF="refs/heads/$GITHUB_HEAD_REF"
                echo "    Detected PR branch for bundle: $GITHUB_HEAD_REF"
            elif [ -n "${GITHUB_REF:-}" ]; then
                # Use the current ref
                BRANCH_REF="$GITHUB_REF"
                echo "    Using current ref for bundle: $GITHUB_REF"
            fi
            
            # Try bundle verification with certificate identity verification
            # For keyless signing, we still need to verify the certificate identity even with bundles
            CERT_IDENTITY="https://github.com/$REPO/.github/workflows/$WORKFLOW@$BRANCH_REF"
            echo "    Bundle certificate identity: $CERT_IDENTITY"
            
            if BUNDLE_OUTPUT=$(cosign verify-blob \
                --bundle "${file}.bundle" \
                --certificate-identity "$CERT_IDENTITY" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$file" 2>&1); then
                echo "    ✅ Bundle signature verified for: $file"
                echo "    Bundle output: $BUNDLE_OUTPUT"
                VERIFIED_COUNT=$((VERIFIED_COUNT + 1))
                continue
            else
                echo "    ⚠️  Bundle verification failed:"
                echo "    Bundle error: $BUNDLE_OUTPUT"
                echo "    Trying signature verification instead..."
            fi
        fi
        
        if [ -f "${file}.sig" ]; then
            echo "    Trying signature verification..."
            # Try multiple certificate identity formats that GitHub Actions might use
            VERIFICATION_SUCCESS=false
            
            # Determine the branch reference
            BRANCH_REF="refs/heads/main"
            if [ -n "${GITHUB_HEAD_REF:-}" ]; then
                # This is a pull request, use the PR branch
                BRANCH_REF="refs/heads/$GITHUB_HEAD_REF"
                echo "    Detected PR branch: $GITHUB_HEAD_REF"
            elif [ -n "${GITHUB_REF:-}" ]; then
                # Use the current ref
                BRANCH_REF="$GITHUB_REF"
                echo "    Using current ref: $GITHUB_REF"
            fi
            
            # Format 1: Standard workflow path
            CERT_IDENTITY="https://github.com/$REPO/.github/workflows/$WORKFLOW@$BRANCH_REF"
            echo "    Trying certificate identity: $CERT_IDENTITY"
            if cosign verify-blob \
                --signature "${file}.sig" \
                --certificate-identity "$CERT_IDENTITY" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$file" 2>&1; then
                VERIFICATION_SUCCESS=true
                echo "    ✅ Verification successful with standard workflow path"
            # Format 2: Try with regexp for more flexibility
            elif cosign verify-blob \
                --signature "${file}.sig" \
                --certificate-identity-regexp "https://github.com/$REPO/.*" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$file" > /dev/null 2>&1; then
                VERIFICATION_SUCCESS=true
            # Format 3: Try without specific workflow path
            elif cosign verify-blob \
                --signature "${file}.sig" \
                --certificate-identity-regexp ".*$REPO.*" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$file" > /dev/null 2>&1; then
                VERIFICATION_SUCCESS=true
            fi
            
            if [ "$VERIFICATION_SUCCESS" = true ]; then
                echo "    ✅ Signature verified for: $file"
                VERIFIED_COUNT=$((VERIFIED_COUNT + 1))
            else
                echo "    ❌ Signature verification failed for: $file"
                FAILED_COUNT=$((FAILED_COUNT + 1))
            fi
        else
            echo "    ❌ No signature or bundle found for: $file"
            FAILED_COUNT=$((FAILED_COUNT + 1))
        fi
    fi
done

echo
echo "📊 Verification Summary:"
echo "  ✅ Verified: $VERIFIED_COUNT files"
echo "  ❌ Failed: $FAILED_COUNT files"

if [ $FAILED_COUNT -gt 0 ]; then
    echo "💥 Some signatures failed verification!"
    exit 1
elif [ $VERIFIED_COUNT -eq 0 ]; then
    echo "⚠️  No signatures were verified. This might indicate an issue."
    exit 1
else
    echo "🎉 All signatures verified successfully!"
fi
