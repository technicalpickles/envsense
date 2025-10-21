---
name: release
description: |
  Use this skill when the user asks to create a release, cut a release, publish a new version, or perform release-related tasks for the envsense project.

  This skill provides comprehensive guidance for the automated release process, including:
  - Version bumping in Cargo.toml
  - Pre-release verification (tests, formatting, linting)
  - Monitoring the automated CI/CD pipeline
  - Multi-platform binary builds (Linux x64/ARM64, macOS Universal)
  - Cryptographic signing with cosign
  - Release verification and troubleshooting
  - Schema version considerations

  The envsense project uses an automated workflow triggered by version changes on the main branch. This skill walks through the entire process from start to finish.
---

# Release Skill

This skill guides you through creating a new release for the envsense project.

## Release Process Overview

The envsense project uses an **automated release workflow** that is triggered
when:

1. A version change is detected in `Cargo.toml` on the `main` branch
2. CI tests pass successfully
3. No git tag exists yet for that version

## Step-by-Step Release Instructions

Follow these steps to create a new release via pull request:

### 1. Verify Current State

Before starting a release, check:

- All tests pass locally: `cargo test --all`
- Code is formatted: `cargo fmt --all -- --check`
- Linting passes: `cargo clippy --all --locked -- -D warnings`
- Baseline validation: `./scripts/compare-baseline.sh`
- You're on the `main` branch and up to date

### 2. Determine Version Number

Follow semantic versioning:

- **Patch** (0.5.x): Bug fixes, minor changes, no breaking changes
- **Minor** (0.x.0): New features, functionality additions, backward compatible
- **Major** (x.0.0): Breaking changes (when 1.0+ is reached)

Check current version:

```bash
grep '^version = ' Cargo.toml | head -1
```

Check existing tags:

```bash
git tag --sort=-version:refname | head -10
```

### 3. Update Version in Cargo.toml

Edit the version field in `Cargo.toml`:

```toml
version = "0.5.1" # Update to new version
```

**Important**: This is a workspace project. Only update the root `Cargo.toml`
version field (line ~10).

### 4. Create Release Branch and PR

Create a release branch with your changes:

```bash
git checkout -b release-v0.5.1
git add Cargo.toml
git commit -m "Release v0.5.1"
git push origin release-v0.5.1
```

### 5. Create Pull Request

Create a PR with a structured description of the release. Use this template:

```markdown
Release v0.5.1

Brief description of what this release includes (new features, bug fixes,
improvements).

## Changes

- Feature/fix description with link to PR #123
- Another change description with link to PR #124
- Additional improvement with link to PR #125

## Test Results

All XXX tests passing.

## Breaking Changes

None (or list any breaking changes if applicable)
```

**Example from recent release:**

```markdown
Release v0.5.0

Minor release adding Amp agent detection support.

## Changes

- Add Amp agent detection via `AGENT=amp` environment variable (#56)

All 329 tests passing.
```

### 6. Merge PR to Main

Once the PR is reviewed and approved:

```bash
gh pr merge --squash  # Or merge via GitHub UI
```

### 7. Monitor Automated Release

The automated workflow will:

1. **CI Workflow** runs first (`.github/workflows/ci.yml`):
   - Runs linting (formatting, clippy)
   - Runs prettier checks
   - Runs tests on Ubuntu and macOS
   - Validates baselines

2. **Release Workflow** triggers after CI succeeds
   (`.github/workflows/release.yml`):
   - Checks if version changed (using `scripts/check-version-change.sh`)
   - Builds binaries for multiple platforms:
     - Linux x64 (`x86_64-unknown-linux-gnu`)
     - Linux ARM64 (`aarch64-unknown-linux-gnu`)
     - macOS Universal (`universal-apple-darwin` - Intel + Apple Silicon)
   - Signs binaries with cosign (keyless signing)
   - Creates GitHub release with:
     - Release notes (from PR description and auto-generated from commits)
     - Binary artifacts
     - SHA256 checksums
     - Cryptographic signatures (.sig files)
   - Creates and pushes git tag (format: `{version}`, e.g., `0.5.1`)

3. **Validate Release Workflow** runs after release is published
   (`.github/workflows/validate-release.yml`):
   - Validates all binary signatures using cosign
   - Tests local aqua configuration
   - Reports next steps (aqua registry submission)

### 8. Monitor Workflow Progress

Check workflow status:

```bash
# View recent workflow runs
gh run list --limit 5

# Watch a specific workflow run
gh run watch

# View workflow logs if something fails
gh run view --log
```

Or visit: https://github.com/technicalpickles/envsense/actions

### 9. Verify Release

Once complete, verify the release:

```bash
# Check the new tag was created
git fetch --tags
git tag --sort=-version:refname | head -5

# View the GitHub release
gh release view 0.5.1  # Use your version number

# Download and test a binary
gh release download 0.5.1 --pattern "*x86_64-unknown-linux-gnu*"
```

### 10. Post-Release Tasks (Optional)

After a successful release:

1. **Update aqua registry** (if needed):
   - Submit PR to https://github.com/aquaproj/aqua-registry
   - Follow validation workflow output for guidance

2. **Announce release** (if significant):
   - Update project documentation
   - Post to relevant channels/communities

## Binary Naming Convention

Released binaries follow this pattern:

```
envsense-{version}-{target}
```

Examples:

- `envsense-0.5.1-x86_64-unknown-linux-gnu` (Linux x64)
- `envsense-0.5.1-aarch64-unknown-linux-gnu` (Linux ARM64)
- `envsense-0.5.1-universal-apple-darwin` (macOS Universal)

Each binary includes:

- `.sha256` checksum file
- `.sig` cryptographic signature file
- `.bundle` signature bundle file

## Testing Releases Locally

Before pushing a release, you can test the build process:

```bash
# Run comprehensive release tests
./scripts/test-release.sh

# Test specific target build
./scripts/build-target.sh x86_64-unknown-linux-gnu normal

# Test universal macOS build (macOS only)
./scripts/build-target.sh universal-apple-darwin universal

# Verify universal binary (macOS only)
lipo -info target/universal-apple-darwin/release/envsense

# Test binary preparation and validation
./scripts/prepare-binary.sh 0.5.1-test x86_64-unknown-linux-gnu
./scripts/validate-binary.sh dist/envsense-0.5.1-test-x86_64-unknown-linux-gnu
```

## Troubleshooting

### Release Workflow Didn't Trigger

Check:

- CI workflow completed successfully
- Version in Cargo.toml changed from last tagged release
- No existing tag with that version exists
- Push was to `main` branch

### Build Failed

Check:

- All tests pass locally: `cargo test --all`
- Code compiles for all targets
- Review workflow logs: `gh run view --log`

### Signature Validation Failed

The validate-release workflow might fail if:

- Cosign signatures weren't created properly
- Network issues downloading release assets
- This is usually non-critical; the release itself succeeded

### Need to Retry a Release

If you need to fix and retry:

1. Delete the GitHub release: `gh release delete 0.5.1`
2. Delete the git tag locally and remotely:
   ```bash
   git tag -d 0.5.1
   git push origin :refs/tags/0.5.1
   ```
3. Fix issues in a new commit
4. Restart from step 5 (commit and push)

## Schema Version Considerations

**Important**: The JSON schema version (currently 0.3.0) is separate from the
crate version.

When making **breaking schema changes**:

1. Update `SCHEMA_VERSION` in `src/schema/main.rs`
2. Update all snapshots: `cargo insta accept`
3. Document changes in migration guide: `docs/migration-guide.md`
4. Reference `CONTRACT.md` for compatibility guarantees

Schema changes require:

- Field renames: Add `#[serde(alias = "old_name")]`
- Field removals: Bump schema version
- New fields: Can be added freely (non-breaking)

## Release Checklist

Use this checklist when performing a release:

- [ ] All tests pass locally (`cargo test --all`)
- [ ] Code is formatted (`cargo fmt --all -- --check`)
- [ ] Clippy passes (`cargo clippy --all --locked -- -D warnings`)
- [ ] Prettier passes (`npm run format:check`)
- [ ] Version updated in `Cargo.toml`
- [ ] Release branch created
- [ ] Pull request created with proper description and linked PRs
- [ ] PR reviewed and approved
- [ ] PR merged to main branch
- [ ] CI workflow completed successfully
- [ ] Release workflow completed successfully
- [ ] Git tag created and pushed
- [ ] GitHub release published
- [ ] Binaries available and signed
- [ ] Validation workflow completed (optional)
- [ ] Release tested (download and run binary)

## Key Files and Scripts

- `Cargo.toml` - Version definition
- `.github/workflows/release.yml` - Main release workflow
- `.github/workflows/ci.yml` - CI prerequisite workflow
- `.github/workflows/validate-release.yml` - Post-release validation
- `scripts/check-version-change.sh` - Detects version changes
- `scripts/build-target.sh` - Builds for specific targets
- `scripts/prepare-binary.sh` - Prepares release artifacts
- `scripts/sign-release-binaries.sh` - Signs binaries with cosign
- `scripts/create-release.sh` - Extracts changelog content
- `CONTRACT.md` - Schema stability guarantees
- `docs/development.md` - Development and release documentation

## Support

For issues with the release process:

1. Check GitHub Actions logs
2. Review recent successful releases for comparison
3. Consult `docs/development.md` for detailed documentation
4. Open an issue if you discover a bug in the release workflow
