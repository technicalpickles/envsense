# Release Signing Validation Strategy

## Overview

This document outlines our strategy for validating that release signing is
working correctly before we publish to the aqua registry. The goal is to ensure
that every release is properly signed and verifiable before users attempt to
install via `mise install aqua:envsense`.

## Validation Layers

### Layer 1: Build-Time Validation

**When**: During GitHub Actions release workflow **Purpose**: Catch signing
failures immediately

#### Automated Checks in CI

1. **Signature Generation Verification**:

   ```yaml
   - name: Verify signatures were created
     run: |
       cd release-files
       for binary in envsense-*; do
         if [[ "$binary" != *.sha256 && "$binary" != *.sig ]]; then
           sig_file="${binary}.sig"
           if [ ! -f "$sig_file" ]; then
             echo "❌ Missing signature for: $binary"
             exit 1
           fi
           echo "✅ Signature exists for: $binary"
         fi
       done
   ```

2. **Immediate Signature Verification**:
   ```yaml
   - name: Verify signatures are valid
     run: |
       cd release-files
       for binary in envsense-*; do
         if [[ "$binary" != *.sha256 && "$binary" != *.sig ]]; then
           echo "Verifying signature for: $binary"
           cosign verify-blob \
             --signature "${binary}.sig" \
             --certificate-identity-regexp "https://github.com/${{ github.repository }}" \
             --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
             "$binary"
         fi
       done
   ```

### Layer 2: Post-Release Validation

**When**: After release is published **Purpose**: Ensure end-users can verify
signatures

#### Automated Post-Release Testing

Create a separate workflow that runs after releases:

**File**: `.github/workflows/validate-release-signing.yml`

```yaml
name: Validate Release Signing

on:
  release:
    types: [published]
  workflow_dispatch:
    inputs:
      version:
        description: "Version to validate (e.g., v0.4.0)"
        required: true

jobs:
  validate-signing:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install cosign
        uses: sigstore/cosign-installer@v3

      - name: Validate release signatures
        run:
          ./scripts/validate-signing.sh ${{ github.event.inputs.version ||
          github.event.release.tag_name }}
```

### Layer 3: Manual Validation Checklist

**When**: Before submitting to aqua registry **Purpose**: Human verification of
the complete process

#### Pre-Submission Checklist

**Release Artifacts Validation**:

- [ ] All expected binaries are present in the release
- [ ] Each binary has a corresponding `.sig` file
- [ ] Each binary has a corresponding `.sha256` file
- [ ] File naming follows expected pattern: `envsense-v{VERSION}-{TARGET}`

**Signature Verification**:

- [ ] Run `scripts/validate-signing.sh` successfully
- [ ] Manually verify at least one signature using cosign CLI
- [ ] Verify signature metadata shows correct GitHub repository
- [ ] Verify signature timestamp is reasonable

**Cross-Platform Testing**:

- [ ] Download and verify signatures on Linux
- [ ] Download and verify signatures on macOS
- [ ] Test signature verification in Docker container

#### Manual Verification Commands

```bash
# 1. Download a release binary and signature
curl -LO https://github.com/your-org/envsense/releases/latest/download/envsense-v0.4.0-x86_64-unknown-linux-gnu
curl -LO https://github.com/your-org/envsense/releases/latest/download/envsense-v0.4.0-x86_64-unknown-linux-gnu.sig

# 2. Verify the signature
cosign verify-blob \
  --signature envsense-v0.4.0-x86_64-unknown-linux-gnu.sig \
  --certificate-identity-regexp "https://github.com/your-org/envsense" \
  --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
  envsense-v0.4.0-x86_64-unknown-linux-gnu

# 3. Verify the binary works
chmod +x envsense-v0.4.0-x86_64-unknown-linux-gnu
./envsense-v0.4.0-x86_64-unknown-linux-gnu --version
```

### Layer 4: Aqua Integration Testing

**When**: Before and after registry submission **Purpose**: Ensure aqua can
properly verify our signatures

#### Local Aqua Registry Testing

1. **Create Test Registry**:

   ```bash
   # Create temporary registry repository
   mkdir test-aqua-registry
   cd test-aqua-registry
   git init

   # Add our registry configuration
   cp ../aqua-registry-entry.yaml registry.yaml
   git add registry.yaml
   git commit -m "Add envsense test registry"
   ```

2. **Test Installation with Local Registry**:

   ```bash
   # Configure aqua to use local registry
   cat > aqua.yaml << EOF
   registries:
     - name: test-envsense
       type: local
       path: ./test-aqua-registry/registry.yaml
   packages:
     - name: your-org/envsense@v0.4.0
       registry: test-envsense
   EOF

   # Test installation
   aqua install
   ```

3. **Verify Aqua Signature Verification**:

   ```bash
   # Enable aqua debug logging to see signature verification
   AQUA_LOG_LEVEL=debug aqua install

   # Look for cosign verification messages in output
   ```

## Validation Scripts

### Primary Validation Script

**File**: `scripts/validate-signing.sh`

```bash
#!/usr/bin/env bash
# Comprehensive validation of release signing

set -euo pipefail

VERSION="${1:-latest}"
REPO="${GITHUB_REPOSITORY:-your-org/envsense}"
TEMP_DIR=$(mktemp -d)
FAILED_VALIDATIONS=()

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

echo "🔍 Validating signing for $REPO version: $VERSION"
echo "Working directory: $TEMP_DIR"
cd "$TEMP_DIR"

# Function to validate a single binary
validate_binary() {
    local binary="$1"
    local sig_file="${binary}.sig"

    echo "  Validating: $binary"

    # Check signature file exists
    if [ ! -f "$sig_file" ]; then
        echo "    ❌ Missing signature file: $sig_file"
        FAILED_VALIDATIONS+=("$binary: missing signature")
        return 1
    fi

    # Verify signature
    if cosign verify-blob \
        --signature "$sig_file" \
        --certificate-identity-regexp "https://github.com/$REPO" \
        --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
        "$binary" > /dev/null 2>&1; then
        echo "    ✅ Signature valid"
        return 0
    else
        echo "    ❌ Signature verification failed"
        FAILED_VALIDATIONS+=("$binary: signature verification failed")
        return 1
    fi
}

# Get release info
if [ "$VERSION" = "latest" ]; then
    RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
else
    RELEASE_URL="https://api.github.com/repos/$REPO/releases/tags/$VERSION"
fi

echo "📥 Downloading release assets..."
curl -s "$RELEASE_URL" | jq -r '.assets[].browser_download_url' | while read -r url; do
    filename=$(basename "$url")
    echo "  Downloading: $filename"
    curl -sLO "$url"
done

echo ""
echo "🔐 Validating signatures..."

# Validate each binary
for binary in envsense-*; do
    if [[ "$binary" != *.sha256 && "$binary" != *.sig ]]; then
        validate_binary "$binary" || true
    fi
done

echo ""
if [ ${#FAILED_VALIDATIONS[@]} -eq 0 ]; then
    echo "🎉 All signatures validated successfully!"
    exit 0
else
    echo "❌ Validation failures:"
    for failure in "${FAILED_VALIDATIONS[@]}"; do
        echo "  - $failure"
    done
    exit 1
fi
```

### Aqua Integration Test Script

**File**: `scripts/test-aqua-integration.sh`

```bash
#!/usr/bin/env bash
# Test envsense installation via aqua

set -euo pipefail

VERSION="${1:-latest}"
TEMP_DIR=$(mktemp -d)

cleanup() {
    rm -rf "$TEMP_DIR"
}
trap cleanup EXIT

cd "$TEMP_DIR"

echo "🧪 Testing aqua integration for version: $VERSION"

# Create test aqua configuration
cat > aqua.yaml << EOF
registries:
  - type: github_content
    repo_owner: your-org
    repo_name: envsense-aqua-registry
    ref: main
    path: registry.yaml

packages:
  - name: your-org/envsense@$VERSION
EOF

echo "📦 Installing envsense via aqua..."
if aqua install; then
    echo "✅ Installation successful"
else
    echo "❌ Installation failed"
    exit 1
fi

echo "🔍 Testing installed binary..."
if ./bin/envsense --version; then
    echo "✅ Binary works correctly"
else
    echo "❌ Binary execution failed"
    exit 1
fi

echo "🎉 Aqua integration test passed!"
```

## Continuous Monitoring

### Release Health Dashboard

Create a simple dashboard to monitor signing health:

**File**: `scripts/signing-health-check.sh`

```bash
#!/usr/bin/env bash
# Check signing health across recent releases

REPO="your-org/envsense"
RELEASES_TO_CHECK=5

echo "🏥 Signing Health Check for $REPO"
echo "Checking last $RELEASES_TO_CHECK releases..."
echo ""

# Get recent releases
curl -s "https://api.github.com/repos/$REPO/releases?per_page=$RELEASES_TO_CHECK" | \
jq -r '.[].tag_name' | while read -r version; do
    echo "Checking $version..."
    if ./scripts/validate-signing.sh "$version" > /dev/null 2>&1; then
        echo "  ✅ $version - All signatures valid"
    else
        echo "  ❌ $version - Signature issues detected"
    fi
done
```

### Automated Alerts

Set up GitHub Actions to alert on signing failures:

```yaml
# Add to existing workflows
- name: Notify on signing failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: failure
    text: "🚨 Release signing failed for ${{ github.event.release.tag_name }}"
  env:
    SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK }}
```

## Troubleshooting Guide

### Common Issues and Solutions

1. **"certificate identity does not match"**:
   - Cause: Repository name mismatch in verification
   - Solution: Ensure `--certificate-identity-regexp` matches exact repo URL

2. **"no matching signatures"**:
   - Cause: Signature file not found or corrupted
   - Solution: Check signature file exists and was uploaded correctly

3. **"certificate not valid yet"**:
   - Cause: Clock skew or certificate timing issues
   - Solution: Check system time, retry verification

4. **"OIDC issuer mismatch"**:
   - Cause: Wrong OIDC issuer specified
   - Solution: Use `https://token.actions.githubusercontent.com`

### Validation Failure Response Plan

1. **Immediate Actions**:
   - Stop any registry submissions
   - Investigate root cause
   - Fix signing process
   - Re-run validation

2. **Communication**:
   - Update team on status
   - Document lessons learned
   - Update validation procedures

3. **Recovery**:
   - Create new signed release if needed
   - Update documentation
   - Resume registry submission process

## Success Criteria

Before submitting to aqua registry, all of these must pass:

- [ ] `scripts/validate-signing.sh latest` passes
- [ ] Manual verification checklist completed
- [ ] Cross-platform testing completed
- [ ] Local aqua registry testing successful
- [ ] All validation scripts pass in CI
- [ ] No signing-related issues in last 3 releases

This comprehensive validation strategy ensures that our signing implementation
is robust and reliable before we expose it to the broader aqua/mise community.
