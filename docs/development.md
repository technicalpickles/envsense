# Development

## Prerequisites

- Rust 1.70+
- `cargo-insta` for snapshot testing: `cargo install cargo-insta`

## Testing

```bash
# Run all tests
cargo test --all

# Run snapshot tests
cargo test --test info_snapshots

# Update snapshots after schema changes
cargo insta accept

# Test specific components
cargo test --package envsense
cargo test --package envsense-macros
```

## Schema Changes

When making breaking schema changes (like removing fields):

1. Bump `SCHEMA_VERSION` in `src/schema.rs` (currently 0.3.0)
2. Update tests to expect new version
3. Run `cargo insta accept` to update snapshots
4. Verify all tests pass
5. **Note**: Schema version 0.3.0 removed legacy `facet:` and `trait:` syntax

## Development Workflow

```bash
# Format code (enforced by pre-commit hooks)
cargo fmt --all

# Lint and fix issues
cargo clippy --all --fix -D warnings

# Run full test suite
cargo test --all

# Build release version
cargo build --release
```

## Release Process

The project uses automated releases triggered by version changes in `Cargo.toml`
on the main branch.

### Creating a Release

1. **Update the version** in `Cargo.toml`:

   ```bash
   # Edit Cargo.toml and change version field
   version = "0.2.0"  # New version
   ```

2. **Update CHANGELOG.md** (if it exists):

   ```markdown
   ## [0.2.0] - 2024-01-15

   ### Added

   - New feature descriptions

   ### Changed

   - Breaking changes

   ### Fixed

   - Bug fixes
   ```

3. **Commit and push to main**:

   ```bash
   git add Cargo.toml CHANGELOG.md
   git commit -m "Release v0.2.0"
   git push origin main
   ```

4. **Automated release process**:
   - GitHub Actions detects the version change
   - Builds binaries for multiple platforms:
     - Linux x64 (`x86_64-unknown-linux-gnu`)
     - Linux ARM64 (`aarch64-unknown-linux-gnu`)
     - macOS Intel (`x86_64-apple-darwin`)
     - macOS Apple Silicon (`aarch64-apple-darwin`)
     - macOS Universal (`universal-apple-darwin`) - single binary for both Intel
       and Apple Silicon
     - Windows x64 (`x86_64-pc-windows-msvc`)
   - Creates a GitHub release with binaries
   - Automatically creates and pushes a git tag

### Binary Naming Convention

Released binaries follow the pattern: `envsense-v{version}-{target}`

Examples:

- `envsense-v0.2.0-x86_64-unknown-linux-gnu`
- `envsense-v0.2.0-universal-apple-darwin` (recommended for macOS)
- `envsense-v0.2.0-aarch64-apple-darwin`
- `envsense-v0.2.0-x86_64-pc-windows-msvc.exe`

### Release Notes

The release workflow automatically:

- Extracts changelog content for the specific version (if available)
- Includes build information and supported platforms
- Generates additional release notes from commit history
- Provides SHA256 checksums for all binaries

### Testing Releases

Before making a release, test the build process locally:

```bash
# Test cross-compilation for different targets
cargo build --release --target x86_64-unknown-linux-gnu
cargo build --release --target x86_64-apple-darwin

# Test universal binary creation (macOS only)
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
mkdir -p target/universal-apple-darwin/release
lipo -create \
  target/x86_64-apple-darwin/release/envsense \
  target/aarch64-apple-darwin/release/envsense \
  -output target/universal-apple-darwin/release/envsense

# Verify universal binary
lipo -info target/universal-apple-darwin/release/envsense

# Test the binary
./target/release/envsense --help
./target/release/envsense info --json

# Use the test script for comprehensive testing
./scripts/test-release.sh
```

See `docs/testing.md` for detailed testing guidelines.
