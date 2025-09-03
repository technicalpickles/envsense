# 🚀 Add Automated Release Workflow with Universal macOS Binaries

## Overview

This PR implements a comprehensive automated release workflow that builds and
publishes cross-platform binaries whenever the version changes in `Cargo.toml`.
The implementation includes **universal macOS binaries** that work on both Intel
and Apple Silicon Macs.

## ✨ Key Features

### 🎯 **Universal macOS Binary Support**

- **Single binary for all Macs**: Uses `lipo` to create universal binaries
  containing both x86_64 and aarch64 code
- **Better user experience**: Users don't need to know their Mac's architecture
- **Future-proof**: Works seamlessly across Apple's architecture transitions

### 🔄 **Automated Release Process**

- **Trigger**: Monitors `Cargo.toml` version changes on main branch
- **Cross-platform builds**: Linux (x64/ARM64), macOS (Intel/Apple
  Silicon/Universal), Windows (x64)
- **Automatic releases**: Creates GitHub releases with binaries, checksums, and
  release notes
- **Zero manual intervention**: Just bump the version and push to main

### 🧩 **Modular Architecture**

- **Clean GitHub Actions workflow**: Simple YAML that delegates to external
  scripts
- **Testable components**: Each script can be run and tested independently
- **Easy maintenance**: Logic separated from workflow configuration

## 📦 Release Artifacts

Each release includes:

| Binary                                           | Platform  | Architecture     |
| ------------------------------------------------ | --------- | ---------------- |
| `envsense-v{version}-x86_64-unknown-linux-gnu`   | Linux     | x64              |
| `envsense-v{version}-aarch64-unknown-linux-gnu`  | Linux     | ARM64            |
| `envsense-v{version}-x86_64-apple-darwin`        | macOS     | Intel            |
| `envsense-v{version}-aarch64-apple-darwin`       | macOS     | Apple Silicon    |
| **`envsense-v{version}-universal-apple-darwin`** | **macOS** | **Universal** ⭐ |
| `envsense-v{version}-x86_64-pc-windows-msvc.exe` | Windows   | x64              |

Plus SHA256 checksums for all binaries.

## 🛠 Implementation Details

### GitHub Actions Workflow (`.github/workflows/release.yml`)

```yaml
# Triggers on Cargo.toml version changes
on:
  push:
    branches: [main]
    paths: ["Cargo.toml"]
# Three-stage process:
# 1. check-version: Detect version changes
# 2. build: Cross-platform compilation matrix
# 3. release: Create GitHub release with binaries
```

### Modular Scripts

#### 1. **`scripts/check-version-change.sh`**

- Compares current vs previous `Cargo.toml` version
- Outputs GitHub Actions variables for downstream jobs
- Works locally and in CI

#### 2. **`scripts/build-target.sh`**

```bash
# Usage: ./build-target.sh <target> [build_type]
# Supports: normal, cross, universal
./build-target.sh universal-apple-darwin universal
```

- Handles different build types (normal, cross-compilation, universal)
- Universal binary creation using `lipo`
- Automatic dependency installation (cross, targets)

#### 3. **`scripts/prepare-binary.sh`**

```bash
# Usage: ./prepare-binary.sh <version> <target>
./prepare-binary.sh 0.1.0 universal-apple-darwin
```

- Binary naming and copying
- SHA256 checksum generation
- Smoke testing (help, info, check commands)
- Executable permissions

#### 4. **`scripts/create-release.sh`**

- Changelog extraction for specific versions
- Release notes generation
- GitHub release preparation

#### 5. **`scripts/test-release.sh`**

- Comprehensive local testing
- Cross-platform build validation
- Binary functionality verification

## 🧪 Testing & Validation

### Local Testing

```bash
# Test all supported targets
./scripts/test-release.sh

# Test specific components
./scripts/build-target.sh universal-apple-darwin universal
./scripts/prepare-binary.sh 0.1.0 universal-apple-darwin
```

### Validation Results

- ✅ **Universal binary creation**: Verified with `lipo -info`
- ✅ **Cross-compilation**: All targets build successfully (where supported)
- ✅ **Binary functionality**: All smoke tests pass
- ✅ **Workflow syntax**: Passes `actionlint` validation
- ✅ **Local compatibility**: Scripts work on macOS, Linux, Windows

## 📚 Documentation Updates

### Updated Files

- **`README.md`**: Added installation instructions with universal binary
- **`docs/development.md`**: Complete release process documentation

### Installation Example

```bash
# macOS Universal (recommended)
curl -L https://github.com/your-org/envsense/releases/latest/download/envsense-v0.1.0-universal-apple-darwin -o envsense
chmod +x envsense
```

## 🚀 How to Use

### For Releases

1. **Update version** in `Cargo.toml`:

   ```toml
   version = "0.2.0" # Bump version
   ```

2. **Commit and push**:

   ```bash
   git add Cargo.toml
   git commit -m "Release v0.2.0"
   git push origin main
   ```

3. **Automatic process**:
   - GitHub Actions detects version change
   - Builds cross-platform binaries
   - Creates release with universal macOS binary
   - Users can download and use immediately

### For Users

- **One download works everywhere**: Universal macOS binary runs on any Mac
- **Consistent naming**: Predictable binary names across platforms
- **Verified integrity**: SHA256 checksums for all downloads

## 🔧 Technical Implementation

### Universal Binary Creation

```bash
# Build both architectures
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Combine with lipo
lipo -create \
  target/x86_64-apple-darwin/release/envsense \
  target/aarch64-apple-darwin/release/envsense \
  -output target/universal-apple-darwin/release/envsense

# Verify: "Architectures in the fat file: x86_64 arm64"
lipo -info target/universal-apple-darwin/release/envsense
```

### Cross-Platform Build Matrix

- **Linux builds**: Use LLD linker for faster compilation
- **ARM64 Linux**: Cross-compilation with `cross` tool
- **macOS builds**: Native compilation with LLD optimization
- **Windows builds**: MSVC toolchain
- **Universal macOS**: Combines Intel + Apple Silicon binaries

## 🎯 Benefits

### For Users

- **Simplified installation**: One binary works on all Macs
- **No architecture confusion**: Universal binary handles detection
- **Fast downloads**: Optimized binaries with LLD linker
- **Verified integrity**: SHA256 checksums for security

### For Maintainers

- **Zero-touch releases**: Automated from version bump to publication
- **Easy testing**: All scripts work locally
- **Clear separation**: Logic in scripts, not complex YAML
- **Comprehensive validation**: Multiple test layers

### For the Project

- **Professional distribution**: Multi-platform release artifacts
- **Better adoption**: Easy installation reduces friction
- **Future-ready**: Supports new architectures automatically
- **Maintainable**: Modular design for easy updates

## 🔍 Files Changed

### New Files

- `.github/workflows/release.yml` - Main release workflow
- `scripts/check-version-change.sh` - Version change detection
- `scripts/build-target.sh` - Target-specific building
- `scripts/prepare-binary.sh` - Binary preparation and testing
- `scripts/create-release.sh` - Release notes extraction
- `scripts/test-release.sh` - Comprehensive local testing

### Updated Files

- `README.md` - Installation instructions with universal binary
- `docs/development.md` - Release process documentation

## ⚡ Performance Optimizations

- **LLD Linker**: 2-3x faster linking on supported platforms
- **Parallel builds**: Matrix strategy builds all targets simultaneously
- **Rust caching**: `Swatinem/rust-cache` for faster CI builds
- **Optimized binaries**: Release builds with full optimizations

## 🛡 Security Considerations

- **Minimal permissions**: `contents: write` only for releases
- **Checksum verification**: SHA256 for all binaries
- **Automated process**: Reduces human error in releases
- **Audit trail**: Full GitHub Actions logs for each release

## 🎉 Ready for Production

This implementation is **production-ready** and thoroughly tested:

- ✅ **Local validation**: All scripts tested independently
- ✅ **Workflow validation**: Passes actionlint without warnings
- ✅ **Cross-platform support**: Builds verified on multiple platforms
- ✅ **Universal binary**: Creates perfect fat binaries for macOS
- ✅ **Documentation**: Complete usage and maintenance docs

The next version bump will automatically trigger the first release! 🚀

---

## Testing Instructions

To test this PR:

1. **Local testing**:

   ```bash
   ./scripts/test-release.sh
   ```

2. **Manual workflow test** (after merge):
   - Bump version in `Cargo.toml`
   - Push to main
   - Watch GitHub Actions create the release

3. **Binary verification**:

   ```bash
   # Download universal binary
   curl -L <release-url> -o envsense
   chmod +x envsense

   # Verify it works
   ./envsense --help
   ./envsense info --json

   # Verify universal architecture (macOS only)
   lipo -info ./envsense
   # Should show: x86_64 arm64
   ```
