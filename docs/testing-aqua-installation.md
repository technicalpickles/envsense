# Testing Aqua Installation

This document describes how to test the aqua registry configuration for envsense
before submitting to the official registry.

## Overview

The aqua registry configuration has been successfully implemented and tested.
Users will be able to install envsense using:

```bash
mise install aqua:envsense
```

## Local Testing

### Prerequisites

- `mise` installed and configured
- `aqua` available via mise (`mise install aqua`)

### Running Tests

Use the provided test script:

```bash
./scripts/test-aqua-local.sh
```

This script:

1. Creates a temporary test environment
2. Sets up local registry configuration
3. Installs envsense via aqua
4. Validates the installation works correctly
5. Tests basic functionality

### Manual Testing

If you need to test manually:

1. **Create test directory:**

   ```bash
   mkdir -p tmp/manual-test && cd tmp/manual-test
   ```

2. **Copy registry configuration:**

   ```bash
   cp ../aqua-registry-entry.yaml registry.yaml
   ```

3. **Create aqua.yaml:**

   ```yaml
   ---
   registries:
     - type: local
       name: envsense-local
       path: registry.yaml
   packages:
     - name: technicalpickles/envsense@0.3.4
       registry: envsense-local
   ```

4. **Create policy file:**

   ```yaml
   ---
   registries:
     - name: envsense-local
       type: local
       path: registry.yaml
   packages:
     - name: technicalpickles/envsense
       registry: envsense-local
   ```

5. **Install and test:**
   ```bash
   AQUA_POLICY_CONFIG=aqua-policy.yaml mise exec aqua -- aqua install
   AQUA_POLICY_CONFIG=aqua-policy.yaml mise exec aqua -- envsense info
   ```

## Configuration Details

### Registry Configuration

The registry configuration (`aqua-registry-entry.yaml`) includes:

- **Proper binary naming**: Uses `envsense-{{.Version}}-{{.Arch}}-{{.OS}}`
  format (no 'v' prefix)
- **Cross-platform support**: Linux x64 and macOS Universal binaries
- **Checksum verification**: SHA256 checksums for all binaries
- **Cosign verification**: Keyless signing verification with GitHub OIDC
- **Version constraints**: Uses `version_overrides` for flexible version
  matching

### Key Features

1. **Security**: Cosign signature verification for all binaries
2. **Platform Support**:
   - Linux x64 (`x86_64-unknown-linux-gnu`)
   - macOS Universal (`universal-apple-darwin`)
3. **Policy Support**: Requires explicit policy configuration for security
4. **Checksum Validation**: Automatic SHA256 checksum verification

### Policy Requirements

Aqua requires explicit policy configuration to allow package installation. Users
need:

```yaml
# aqua-policy.yaml
registries:
  - type: standard
    ref: semver(">= 3.0.0")
packages:
  - name: technicalpickles/envsense
  - registry: standard
```

Or set `AQUA_POLICY_CONFIG` environment variable.

## Troubleshooting

### Common Issues

1. **"Package isn't allowed" error**:
   - Ensure proper policy configuration
   - Set `AQUA_POLICY_CONFIG` environment variable
   - Add package to policy file

2. **Asset not found**:
   - Verify binary naming convention matches releases
   - Check version exists in GitHub releases
   - Validate asset templates in registry configuration

3. **Checksum verification fails**:
   - Ensure SHA256 files exist for all binaries
   - Verify checksum file naming matches asset naming

4. **Cosign verification fails**:
   - Verify signatures exist (`.sig` files)
   - Check certificate identity configuration
   - Ensure OIDC issuer is correct

## Next Steps

1. **Submit to Official Registry**: Create PR to `aquaproj/aqua-registry`
2. **Update Documentation**: Add installation instructions to README
3. **Monitor Usage**: Track installation success and user feedback

## Testing Results

✅ **Basic Installation**: Works correctly with local registry  
✅ **Binary Functionality**: All envsense commands work as expected  
✅ **Cross-platform**: Tested on macOS (Universal binary)  
✅ **Policy Configuration**: Security policies work correctly  
✅ **Checksum Verification**: SHA256 validation successful  
🔄 **Cosign Verification**: Configuration ready (requires testing with official
registry)

The configuration is ready for submission to the official aqua registry.
