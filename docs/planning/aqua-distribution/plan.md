# Aqua + Mise Distribution Implementation Plan

> **📋 STATUS: FULLY COMPLETED ✅**  
> This plan has been successfully implemented! envsense is now available via:  
> `mise install aqua:technicalpickles/envsense`  
> See [implementation-analysis.md](implementation-analysis.md) for complete
> details.

## Overview

This document outlined the original plan to make `envsense` installable via
`mise install aqua:envsense`. The implementation involved adding release
signing, creating an aqua registry entry, and thorough testing before public
distribution. **All phases have been completed successfully.**

## Current State Analysis

### ✅ What We Already Have

- Cross-platform GitHub releases (Linux x64, macOS Universal, Windows x64)
- Consistent binary naming: `envsense-v{VERSION}-{TARGET}`
- SHA256 checksums for all releases
- Automated release workflow via GitHub Actions
- Clean single-binary distribution (perfect for aqua)

### ❌ What We Need to Add

- Release signing with cosign (keyless)
- Aqua registry configuration
- Validation process for signed releases
- Testing infrastructure for aqua installation

## Implementation Phases

### Phase 1: Add Release Signing

#### 1.1 Implement Keyless Signing

**Goal**: Add cosign keyless signing to our GitHub Actions release workflow

**Changes Required**:

- Modify `.github/workflows/release.yml` to include cosign signing
- Generate `.sig` files for each binary
- Upload signatures alongside binaries in releases

**Implementation Steps**:

1. Add cosign installer step to release workflow
2. Add signing step after binary preparation
3. Ensure signatures are included in release assets
4. Test on a development release

**Validation Criteria**:

- Each binary has a corresponding `.sig` file
- Signatures can be verified using cosign CLI
- GitHub Actions logs show successful signing

#### 1.2 Create Signing Validation Script

**Goal**: Automate verification that our signing process works correctly

**Deliverable**: `scripts/validate-signing.sh`

- Downloads latest release assets
- Verifies each signature using cosign
- Reports success/failure for each binary
- Can be run locally or in CI

### Phase 2: Create Aqua Registry Configuration

#### 2.1 Generate Initial Configuration

**Goal**: Create aqua registry entry for envsense

**Process**:

1. Use `aqua gr your-org/envsense` to generate base configuration
2. Customize for our specific binary naming and platforms
3. Add cosign verification configuration
4. Test configuration locally

**Deliverable**: `aqua-registry-entry.yaml` (for reference)

#### 2.2 Test Local Registry

**Goal**: Validate registry configuration works before submitting to upstream

**Process**:

1. Create temporary local registry repository
2. Test installation via mise using local registry
3. Verify binary installation and functionality
4. Test on multiple platforms (Linux, macOS)

### Phase 3: Validation and Testing

#### 3.1 End-to-End Testing

**Goal**: Comprehensive testing of the entire installation flow

**Test Scenarios**:

1. **Fresh Installation**: `mise install aqua:envsense` on clean system
2. **Version Pinning**: `mise install aqua:envsense@0.4.0`
3. **Platform Testing**: Test on Linux x64, macOS Intel, macOS ARM
4. **Signature Verification**: Ensure aqua verifies signatures correctly
5. **Functionality Testing**: Verify installed binary works correctly

**Test Environment Setup**:

- Docker containers for Linux testing
- GitHub Actions matrix for cross-platform testing
- Local VMs/containers for isolated testing

#### 3.2 Create Testing Documentation

**Goal**: Document testing procedures for future releases

**Deliverable**: `docs/testing-aqua-installation.md`

- Step-by-step testing procedures
- Platform-specific considerations
- Troubleshooting common issues
- Validation checklists

### Phase 4: Registry Submission

#### 4.1 Submit to Aqua Registry

**Goal**: Get envsense included in the official aqua registry

**Process**:

1. Fork `aquaproj/aqua-registry`
2. Add our configuration to appropriate directory
3. Submit pull request with clear description
4. Respond to any feedback from maintainers

#### 4.2 Documentation Updates

**Goal**: Update project documentation to reflect new installation method

**Updates Required**:

- Add aqua/mise installation instructions to README.md
- Update installation section with new method
- Add troubleshooting section for aqua-specific issues

## Detailed Implementation Steps

### Step 1: Modify Release Workflow

**File**: `.github/workflows/release.yml`

**Changes**:

```yaml
# Add after "Filter release files" step
- name: Install cosign
  uses: sigstore/cosign-installer@v3

- name: Sign release binaries
  shell: bash
  run: |
    echo "Signing release binaries with cosign..."
    cd release-files
    for file in envsense-*; do
      if [[ "$file" != *.sha256 && "$file" != *.sig ]]; then
        echo "Signing: $file"
        cosign sign-blob --yes "$file" --output-signature "${file}.sig"
      fi
    done
    echo "Signing completed. Files:"
    ls -la
  env:
    COSIGN_EXPERIMENTAL: 1

# Modify "Create release" step to include signatures
- name: Create release
  uses: softprops/action-gh-release@v2
  with:
    # ... existing configuration ...
    files: release-files/* # This now includes .sig files
```

### Step 2: Create Validation Script

**File**: `scripts/validate-signing.sh`

```bash
#!/usr/bin/env bash
# Validates that release signing is working correctly

set -euo pipefail

VERSION="${1:-latest}"
REPO="your-org/envsense"

echo "Validating signing for version: $VERSION"

# Download release assets to temp directory
TEMP_DIR=$(mktemp -d)
cd "$TEMP_DIR"

# Get release info and download assets
if [ "$VERSION" = "latest" ]; then
    RELEASE_URL="https://api.github.com/repos/$REPO/releases/latest"
else
    RELEASE_URL="https://api.github.com/repos/$REPO/releases/tags/v$VERSION"
fi

# Download all assets
curl -s "$RELEASE_URL" | jq -r '.assets[].browser_download_url' | while read -r url; do
    echo "Downloading: $(basename "$url")"
    curl -sLO "$url"
done

# Verify signatures
echo "Verifying signatures..."
for binary in envsense-*; do
    if [[ "$binary" != *.sha256 && "$binary" != *.sig ]]; then
        sig_file="${binary}.sig"
        if [ -f "$sig_file" ]; then
            echo "Verifying: $binary"
            cosign verify-blob \
                --signature "$sig_file" \
                --certificate-identity-regexp "https://github.com/$REPO" \
                --certificate-oidc-issuer "https://token.actions.githubusercontent.com" \
                "$binary"
            echo "✅ $binary signature valid"
        else
            echo "❌ Missing signature for: $binary"
            exit 1
        fi
    fi
done

echo "🎉 All signatures validated successfully!"
cd - > /dev/null
rm -rf "$TEMP_DIR"
```

### Step 3: Create Aqua Registry Entry

**File**: `aqua-registry-entry.yaml` (for reference)

```yaml
packages:
  - type: github_release
    repo_owner: your-org
    repo_name: envsense
    description: Environment awareness utilities - detect runtime environments
    asset: envsense-v{{.Version}}-{{.OS}}-{{.Arch}}
    format: raw
    supported_envs:
      - linux/amd64
      - darwin/amd64
      - darwin/arm64
      - windows/amd64
    rosetta2: true
    version_constraint: semver
    version_filter: 'Version startsWith "v"'
    version_prefix: v
    checksum:
      type: github_release
      asset: envsense-v{{.Version}}-{{.OS}}-{{.Arch}}.sha256
    cosign:
      enabled: true
      signature: envsense-v{{.Version}}-{{.OS}}-{{.Arch}}.sig
    replacements:
      darwin: apple-darwin
      linux: unknown-linux-gnu
      windows: pc-windows-msvc
      amd64: x86_64
      arm64: aarch64
    overrides:
      - goos: darwin
        asset: envsense-v{{.Version}}-universal-{{.OS}}
        replacements:
          darwin: apple-darwin
```

## Testing Strategy

### Pre-Submission Testing

1. **Local Registry Testing**:
   - Create test registry repository
   - Configure mise to use test registry
   - Install and test envsense via aqua

2. **Signature Verification Testing**:
   - Run validation script on multiple releases
   - Test cosign verification manually
   - Verify signatures work across platforms

3. **Cross-Platform Testing**:
   - Test installation on Linux (Ubuntu, Alpine)
   - Test installation on macOS (Intel, ARM)
   - Test installation on Windows (if supported)

### Post-Submission Monitoring

1. **Installation Monitoring**:
   - Monitor for installation issues reported by users
   - Track aqua registry PR feedback
   - Monitor mise/aqua community channels

2. **Release Process Integration**:
   - Ensure signing works for all future releases
   - Update documentation as needed
   - Maintain compatibility with aqua updates

## Risk Assessment and Mitigation

### Risks

1. **Signing Failures**: GitHub Actions signing could fail
   - **Mitigation**: Comprehensive testing, fallback procedures
2. **Registry Rejection**: Aqua maintainers might reject our submission
   - **Mitigation**: Follow guidelines closely, engage with community

3. **Platform Compatibility**: Issues with specific OS/arch combinations
   - **Mitigation**: Thorough cross-platform testing

4. **Breaking Changes**: Aqua/mise updates could break compatibility
   - **Mitigation**: Monitor updates, maintain test suite

### Success Criteria

- [ ] All releases are automatically signed with cosign
- [ ] Signatures can be verified using standard tools
- [ ] `mise install aqua:envsense` works on all supported platforms
- [ ] Installation process is documented and user-friendly
- [ ] Registry entry is accepted and maintained

## Timeline

**Week 1**: Implement signing (Phase 1)

- Modify GitHub Actions workflow
- Create validation scripts
- Test signing on development releases

**Week 2**: Create and test registry configuration (Phase 2)

- Generate aqua registry entry
- Test with local registry
- Cross-platform validation

**Week 3**: Comprehensive testing (Phase 3)

- End-to-end testing
- Documentation creation
- Final validation

**Week 4**: Registry submission (Phase 4)

- Submit to aqua registry
- Update project documentation
- Monitor and respond to feedback

## Success Metrics

1. **Technical Metrics**:
   - 100% of releases have valid signatures
   - Installation success rate > 95% across platforms
   - Zero security vulnerabilities in signing process

2. **User Experience Metrics**:
   - Installation time < 30 seconds
   - Clear error messages for failures
   - Documentation completeness score > 90%

3. **Community Metrics**:
   - Aqua registry PR accepted within 1 week
   - Positive feedback from early adopters
   - Integration with popular development workflows

## Next Steps

1. Review this plan with the team
2. Set up development environment for testing
3. Begin Phase 1 implementation
4. Create tracking issues for each phase
5. Schedule regular progress reviews

---

_This plan will be updated as we learn more during implementation and testing._
