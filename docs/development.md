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

1. Bump `SCHEMA_VERSION` in `src/schema/main.rs` (currently 0.3.0)
2. Update tests to expect new version
3. Run `cargo insta accept` to update snapshots
4. Verify all tests pass
5. **Note**: Schema version 0.3.0 removed legacy `facet:` and `trait:` syntax

**Important**: Most releases do NOT require schema version changes. The schema
version (0.3.0) is separate from the crate version. Only bump the schema version
when making breaking changes to the JSON output structure.

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
on the main branch. Releases are created via pull requests for review and
safety.

### Creating a Release

See the [Release Skill](.claude/skills/release/SKILL.md) for comprehensive
step-by-step instructions.

**Quick Summary:**

1. **Create release branch**:

   ```bash
   git checkout -b release-v0.6.0
   ```

2. **Update version** in `Cargo.toml` (root workspace, line ~10):

   ```toml
   version = "0.6.0" # New version
   ```

3. **Commit and push**:

   ```bash
   git add Cargo.toml
   git commit -m "Bump version to 0.6.0"
   git push origin release-v0.6.0
   ```

4. **Create and merge PR**:

   ```bash
   gh pr create --title "Release v0.6.0" --body "Bump version from 0.5.0 to 0.6.0 for minor release."
   gh pr merge --squash
   ```

5. **Automated release process**:
   - CI workflow runs first (tests, linting, formatting)
   - Release workflow detects the version change
   - Builds binaries for supported platforms:
     - **Linux x64** (`x86_64-unknown-linux-gnu`)
     - **Linux ARM64** (`aarch64-unknown-linux-gnu`)
     - **macOS Universal** (`universal-apple-darwin`) - supports both Intel and
       Apple Silicon
   - Signs binaries with cosign (keyless signing)
   - Creates a GitHub release with binaries, checksums, and signatures
   - Automatically creates and pushes a git tag

### Binary Naming Convention

Released binaries follow the pattern: `envsense-{version}-{target}` (following
conventions used by popular CLI tools like ripgrep).

Examples:

- `envsense-0.6.0-x86_64-unknown-linux-gnu` (Linux x64)
- `envsense-0.6.0-aarch64-unknown-linux-gnu` (Linux ARM64)
- `envsense-0.6.0-universal-apple-darwin` (macOS Universal - supports both Intel
  and Apple Silicon)

**Note**: The "v" prefix is not used in artifact names to align with common CLI
tool conventions. The universal macOS binary is the only macOS build provided,
eliminating the need for separate Intel and Apple Silicon binaries.

#### Universal macOS Binary Approach

The project uses a single universal macOS binary instead of separate Intel and
Apple Silicon builds for several reasons:

- **User Experience**: Users don't need to determine their architecture - one
  binary works on all modern Macs
- **Simplified Distribution**: Reduces the number of release artifacts and
  potential confusion
- **Industry Standard**: Follows the approach used by popular CLI tools like
  ripgrep, fd, and others
- **Maintenance**: Reduces CI complexity and build time while maintaining full
  compatibility

The universal binary is created using Apple's `lipo` tool, which combines
separate Intel and Apple Silicon binaries into a single file that automatically
runs the appropriate architecture.

### Release Notes

The release workflow automatically:

- Uses the PR description as the release notes
- Generates additional release notes from commit history
- Provides SHA256 checksums for all binaries
- Creates cryptographic signatures (.sig, .bundle) for all binaries

**Note**: There is currently no CHANGELOG.md file. Release notes are generated
from PR descriptions and git commit history.

### Testing Releases

Before making a release, test the build process locally:

```bash
# Test universal binary creation (macOS only)
./scripts/build-target.sh universal-apple-darwin universal

# Verify universal binary contains both architectures
lipo -info target/universal-apple-darwin/release/envsense

# Test the binary functionality
./target/universal-apple-darwin/release/envsense --help
./target/universal-apple-darwin/release/envsense info --json

# Test binary preparation (includes validation and checksums)
./scripts/prepare-binary.sh 0.2.2-test universal-apple-darwin

# Use the comprehensive test script
./scripts/test-release.sh
```

**Cross-platform testing**: Linux builds are tested in CI. For local Linux
testing on macOS, use Docker:

```bash
./scripts/dev-docker.sh
# Inside container:
cargo build --release --target x86_64-unknown-linux-gnu
```

See `docs/testing.md` for detailed testing guidelines.
