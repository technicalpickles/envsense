# GitHub Actions Release Workflow Implementation Plan

## Overview

This document outlines the implementation plan for adding a CI step that builds
and uploads compiled binaries to GitHub releases when the version changes on the
main branch.

## Current State Analysis

### Project Structure

- **Language**: Rust with Cargo workspace
- **Version Management**:
  - Version defined in `Cargo.toml` (currently `0.1.0`)
  - Schema version in `src/schema/mod.rs` (currently `0.3.0`)
  - No existing git tags or releases
- **Current CI**:
  - Runs on `ubuntu-latest` and `macos-latest`
  - Uses LLD linker optimization
  - Includes formatting, linting, and testing

### Versioning Strategy

The project uses two version numbers:

1. **Package version** in `Cargo.toml` - for the actual release
2. **Schema version** in `src/schema/mod.rs` - for API compatibility

For releases, we should use the **package version** from `Cargo.toml`.

## Implementation Strategy

### 1. Version Change Detection

**Option A: Git Tag-Based (Recommended)**

- Manually create and push git tags when ready to release
- Workflow triggers on `v*` tags (e.g., `v0.1.0`)
- Pros: Explicit control, follows standard practices
- Cons: Manual step required

**Option B: Cargo.toml Monitoring**

- Monitor changes to version field in `Cargo.toml`
- Automatically create tags and releases
- Pros: Fully automated
- Cons: Every version bump triggers release (including development)

**Recommendation**: Use Option A (git tag-based) for better control over when
releases are created.

### 2. Cross-Compilation Targets

Based on the current CI matrix and common CLI tool distribution:

**Primary Targets**:

- `x86_64-unknown-linux-gnu` (Linux x64)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows x64)

**Secondary Targets** (future consideration):

- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-unknown-linux-musl` (Linux x64 static)

### 3. Workflow Design

**Structure**:

```
release.yml
├── trigger: on push tags v*
├── job: build-matrix
│   ├── strategy: matrix of OS/target combinations
│   ├── steps: checkout, setup rust, build, upload artifacts
└── job: create-release
    ├── needs: build-matrix
    ├── steps: download artifacts, create release, upload binaries
```

### 4. Binary Naming Convention

Format: `envsense-{version}-{target}` Examples:

- `envsense-v0.1.0-x86_64-unknown-linux-gnu`
- `envsense-v0.1.0-x86_64-apple-darwin`
- `envsense-v0.1.0-x86_64-pc-windows-msvc.exe`

### 5. Release Notes

**Sources**:

- Extract from `CHANGELOG.md` for the specific version
- Include build information (targets, Rust version)
- Link to documentation

## Implementation Details

### Workflow File Structure

```yaml
name: Release
on:
  push:
    tags: ["v*"]

permissions:
  contents: write

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.os }}
    steps:
      # Build steps with cross-compilation

  release:
    needs: build
    runs-on: ubuntu-latest
    steps:
      # Create release and upload binaries
```

### Key Considerations

**Security**:

- Use `GITHUB_TOKEN` with minimal required permissions
- Consider binary signing for future versions
- Validate that only authorized users can create release tags

**Performance**:

- Leverage existing LLD linker setup from current CI
- Use `Swatinem/rust-cache@v2` for dependency caching
- Parallel builds across target matrix

**Reliability**:

- Fail fast if any target build fails
- Include checksums for binary verification
- Test binaries before upload (basic smoke test)

**Maintenance**:

- Keep workflow in sync with main CI configuration
- Document the release process in project docs
- Consider automation for changelog updates

## Testing Strategy

**Before Implementation**:

1. Test cross-compilation locally for all targets
2. Verify binary functionality on each platform
3. Test workflow with a fork/branch first

**After Implementation**:

1. Create a test tag to verify workflow
2. Validate binary downloads and execution
3. Check release notes formatting

## Migration Path

**Phase 1**: Implement basic workflow

- Single target (current platform)
- Manual testing and validation

**Phase 2**: Add cross-compilation

- Full target matrix
- Automated testing

**Phase 3**: Enhancements

- Binary signing
- Automated changelog extraction
- Performance optimizations

## Files to Create/Modify

1. **New**: `.github/workflows/release.yml` - Main release workflow
2. **Update**: `README.md` - Document installation from releases
3. **Update**: `docs/development.md` - Document release process
4. **Optional**: `scripts/test-release.sh` - Local testing script

## Success Criteria

- [ ] Workflow triggers only on version tags
- [ ] Builds successfully for all target platforms
- [ ] Creates GitHub release with proper metadata
- [ ] Uploads binaries with consistent naming
- [ ] Binaries are functional on target platforms
- [ ] Process is documented and maintainable

## Risks and Mitigations

**Risk**: Cross-compilation failures

- **Mitigation**: Test locally first, use proven GitHub Actions

**Risk**: Large binary sizes

- **Mitigation**: Use release builds with optimizations, consider compression

**Risk**: Workflow complexity

- **Mitigation**: Start simple, iterate, maintain good documentation

**Risk**: Security concerns

- **Mitigation**: Use minimal permissions, consider signing, audit workflow

## Next Steps

1. Implement basic release workflow for current platform
2. Test with a development tag
3. Add cross-compilation targets incrementally
4. Document the process
5. Create first official release

## References

- [GitHub Actions: Publishing Rust binaries](https://docs.github.com/en/actions/publishing-packages)
- [Rust Cross-compilation Guide](https://rust-lang.github.io/rustup/cross-compilation.html)
- [cargo-dist](https://opensource.axo.dev/cargo-dist/) - Alternative tool for
  consideration
