# Testing Aqua Installation

This document provides comprehensive testing procedures for validating
`envsense` installation via aqua/mise.

## Overview

The testing process validates:

- Release signing with cosign
- Aqua registry configuration
- Cross-platform installation
- Binary functionality after installation

## Prerequisites

### Required Tools

```bash
# Install mise (includes aqua support)
curl https://mise.run | sh

# Install cosign for signature verification
brew install cosign

# Install jq for JSON processing
brew install jq
```

### Test Environment Setup

```bash
# Clone the repository
git clone https://github.com/technicalpickles/envsense.git
cd envsense

# Ensure scripts are executable
chmod +x scripts/*.sh
```

## Testing Procedures

### 1. Signature Validation Testing

Test that release signatures are valid and verifiable:

```bash
# Test signature validation for latest release
./scripts/validate-signing.sh

# Test specific version
./scripts/validate-signing.sh 0.3.0
```

**Expected Results:**

- ✅ All binaries have corresponding `.sig` files
- ✅ All signatures verify successfully with cosign
- ✅ Script reports success for all platforms

**Troubleshooting:**

- If cosign is missing: `brew install cosign`
- If jq is missing: `brew install jq`
- If signatures fail: Check GitHub Actions logs for signing errors

### 2. Local Registry Testing

Test aqua configuration before submitting to upstream registry:

```bash
# Test local aqua registry setup
./scripts/test-aqua-local.sh
```

**Expected Results:**

- ✅ Local registry structure created correctly
- ✅ Aqua configuration parsed without errors
- ⚠️ Installation may fail (expected until signed releases exist)

### 3. End-to-End Installation Testing

Once signed releases are available, test full installation flow:

```bash
# Test installation via mise
mise install aqua:envsense

# Test specific version
mise install aqua:envsense@0.3.0

# Test binary functionality
envsense --version
envsense info
envsense check
```

**Expected Results:**

- ✅ Installation completes without errors
- ✅ Binary is executable and functional
- ✅ All core commands work correctly

## Platform-Specific Testing

### Linux (x86_64)

```bash
# Using Docker for isolated testing
docker run --rm -it ubuntu:latest bash -c "
  curl https://mise.run | sh
  source ~/.bashrc
  mise install aqua:envsense
  envsense --version
"
```

### macOS (Universal)

```bash
# Test on macOS (should work on both Intel and Apple Silicon)
mise install aqua:envsense
envsense --version

# Verify architecture support
file $(which envsense)
```

### Cross-Platform Matrix Testing

Create test matrix for comprehensive validation:

| Platform | Architecture  | Test Status | Notes              |
| -------- | ------------- | ----------- | ------------------ |
| Linux    | x86_64        | ⏳ Pending  | Docker testing     |
| macOS    | Universal     | ⏳ Pending  | Native testing     |
| macOS    | Intel         | ⏳ Pending  | Rosetta validation |
| macOS    | Apple Silicon | ⏳ Pending  | Native ARM testing |

## Validation Checklists

### Pre-Release Checklist

- [ ] GitHub Actions workflow includes cosign signing
- [ ] All target platforms have signing enabled
- [ ] Validation script passes for test releases
- [ ] Aqua configuration is syntactically correct

### Post-Release Checklist

- [ ] All release assets include `.sig` files
- [ ] Signature validation passes for all platforms
- [ ] Local aqua registry testing succeeds
- [ ] Installation works via `mise install aqua:envsense`
- [ ] Installed binary is functional

### Registry Submission Checklist

- [ ] Aqua configuration tested locally
- [ ] All supported platforms verified
- [ ] Documentation updated with installation instructions
- [ ] Registry PR submitted with clear description

## Troubleshooting Common Issues

### Signature Verification Failures

**Problem**: `cosign verify-blob` fails

**Solutions**:

1. Check certificate identity matches repository URL
2. Verify OIDC issuer is GitHub Actions
3. Ensure signature file exists and is not corrupted

### Installation Failures

**Problem**: `mise install aqua:envsense` fails

**Solutions**:
1. Check network connectivity
2. Verify release assets exist on GitHub
3. Validate aqua registry configuration syntax
4. Check mise/aqua version compatibility

### Binary Not Found

**Problem**: `envsense` command not found after installation **Solutions**:

1. Check mise shims: `mise which envsense`
2. Verify PATH includes mise shims directory
3. Reload shell configuration: `source ~/.bashrc`

### Platform-Specific Issues

**macOS Gatekeeper**:

- Binary may be quarantined by macOS security
- Solution: `xattr -d com.apple.quarantine $(which envsense)`

**Linux Permissions**:

- Binary may not have execute permissions
- Solution: `chmod +x $(which envsense)`

## Continuous Testing

### Automated Testing Setup

Add aqua installation testing to CI/CD:

```yaml
# .github/workflows/test-aqua.yml
name: Test Aqua Installation
on:
  release:
    types: [published]

jobs:
  test-installation:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - name: Install mise
        run: curl https://mise.run | sh

      - name: Test installation
        run: |
          source ~/.bashrc
          mise install aqua:envsense
          envsense --version
```

### Monitoring and Alerts

- Monitor GitHub releases for missing signatures
- Set up alerts for installation failures
- Track user feedback from aqua/mise communities

## Performance Benchmarks

### Installation Time

Target metrics:

- Installation time: < 30 seconds
- Binary size: < 10MB per platform
- Signature verification: < 5 seconds

### Measurement Commands

```bash
# Time installation
time mise install aqua:envsense

# Check binary size
ls -lh $(which envsense)

# Time signature verification
time ./scripts/validate-signing.sh
```

## Documentation Updates

After successful testing, update project documentation:

1. **README.md**: Add aqua/mise installation instructions
2. **Installation section**: Include new installation method
3. **Troubleshooting**: Add aqua-specific issues and solutions

## Future Considerations

### Version Management

- Test version pinning: `mise install aqua:envsense@0.3.0`
- Test version updates: `mise install aqua:envsense@latest`
- Validate semantic versioning compatibility

### Registry Maintenance

- Monitor aqua registry updates for breaking changes
- Test configuration with new aqua/mise versions
- Maintain compatibility with registry schema changes

---

**Last Updated**: 2025-01-10 **Tested Versions**: envsense v0.3.0, mise latest,
aqua latest
