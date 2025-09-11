# LLD Removal Plan

**Date**: 2025-09-11 **Status**: Plan Ready for Implementation **Risk Level**:
LOW

## Overview

This document provides a step-by-step plan to remove LLD from the envsense build
and CI processes, based on the analysis in `analysis.md`.

## Implementation Strategy

### Phase 1: CI Workflow Cleanup (Immediate)

Remove LLD installation and configuration from CI workflows.

#### 1.1 Update `.github/workflows/ci.yml`

**Current state**: Both `lint` and `test` jobs install and configure LLD

**Target state**: Standard Rust toolchain without LLD

**Changes needed**:

```diff
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

-      # Add LLD with fallback
-      - name: Install LLD
-        run: |
-          sudo apt update
-          sudo apt install -y clang lld || echo "LLD installation failed, using default linker"
-
-      - name: Configure LLD
-        run: |
-          if command -v ld.lld >/dev/null 2>&1; then
-            echo 'RUSTFLAGS=-C link-arg=-fuse-ld=lld' >> "$GITHUB_ENV"
-            echo "Using LLD linker"
-          else
-            echo "LLD not available, using default linker"
-          fi

      - name: Check Rust formatting
        run: cargo fmt --all -- --check
```

```diff
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

-      # Platform-specific LLD setup
-      - name: Install LLD (Linux)
-        if: matrix.os == 'ubuntu-latest'
-        run: |
-          sudo apt update
-          sudo apt install -y clang lld || echo "LLD installation failed"
-
-      - name: Install LLD (macOS)
-        if: matrix.os == 'macos-latest'
-        run: |
-          brew install llvm lld || echo "LLVM/LLD installation failed"
-          echo "CC=clang" >> "$GITHUB_ENV"
-          echo "CXX=clang++" >> "$GITHUB_ENV"
-
-      - name: Configure LLD
-        run: |
-          if command -v ld.lld >/dev/null 2>&1; then
-            echo 'RUSTFLAGS=-C link-arg=-fuse-ld=lld' >> "$GITHUB_ENV"
-            echo "Using LLD linker"
-          else
-            echo "LLD not available, using default linker"
-          fi

      - name: Run tests
        run: cargo test --all --locked
      - name: Validate baselines
        run: scripts/compare-baseline.sh
```

### Phase 2: Cargo Configuration Simplification

#### 2.1 Update `.cargo/config.toml`

**Current state**: Sets `linker = "clang"` for all targets  
**Target state**: Use standard Rust linker selection

**Option A - Complete removal (Recommended)**:

```diff
- [target.aarch64-apple-darwin]
- linker = "clang"
-
- [target.x86_64-apple-darwin]
- linker = "clang"
-
- [target.x86_64-unknown-linux-gnu]
- linker = "clang"
```

**Option B - Keep clang linker without LLD**: Keep the existing config as-is.
The `linker = "clang"` setting without LLD flags will use clang as the linker
driver but with the system's default linker backend.

**Recommendation**: Option A (complete removal) for maximum simplicity, unless
there's a specific need for clang as the linker driver.

### Phase 3: Cleanup Supporting Files

#### 3.1 Remove Unused Scripts

- `scripts/install_lld.sh` - No longer needed

#### 3.2 Archive Documentation

Move current LLD removal documentation to archive:

```bash
mkdir -p docs/archive/planning/lld-removal/
mv docs/planning/remove-lld/* docs/archive/planning/lld-removal/
```

Update `docs/planning/README.md` to reflect LLD removal.

### Phase 4: Validation and Testing

#### 4.1 Local Testing

Before implementing, test locally:

```bash
# Test without LLD configuration
unset RUSTFLAGS
cargo clean
time cargo build --release

# Test with current configuration for comparison
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
cargo clean
time cargo build --release
```

#### 4.2 CI Testing

1. Create PR with CI changes only
2. Verify all jobs pass with standard linker
3. Monitor build times (expect minimal increase)
4. Confirm all tests pass

#### 4.3 Release Testing

Verify that release builds continue to work:

```bash
./scripts/build-target.sh x86_64-unknown-linux-gnu
./scripts/build-target.sh universal-apple-darwin universal
```

## Implementation Schedule

### Immediate (Phase 1)

- [ ] Update `.github/workflows/ci.yml` to remove LLD
- [ ] Test CI changes in PR
- [ ] Merge if all tests pass

### Next (Phase 2)

- [ ] Decide on `.cargo/config.toml` approach (A or B)
- [ ] Update configuration
- [ ] Test local builds

### Cleanup (Phase 3)

- [ ] Remove `scripts/install_lld.sh`
- [ ] Archive documentation
- [ ] Update planning docs

### Validation (Phase 4)

- [ ] Measure actual time savings in CI
- [ ] Confirm build consistency
- [ ] Update team on changes

## Expected Outcomes

### Time Savings

- **CI lint job**: 40-60 seconds faster
- **CI test job (Linux)**: 40-60 seconds faster
- **CI test job (macOS)**: 60-90 seconds faster
- **Total per CI run**: 140-210 seconds saved

### Complexity Reduction

- Fewer CI steps to maintain
- Single, consistent build approach
- No platform-specific installation logic
- Reduced fallback/error handling

### Build Consistency

- CI and release builds use identical linking
- Eliminates potential linking inconsistencies
- Simpler debugging when issues arise

## Rollback Plan

If LLD removal causes unexpected issues:

### Quick Rollback (Emergency)

Revert the CI workflow changes:

```bash
git revert <commit-hash>
```

### Partial Rollback

Re-add LLD only to problematic platform:

```yaml
- name: Install LLD (Linux only)
  if: matrix.os == 'ubuntu-latest'
  run: sudo apt install -y clang lld
```

### Full Restoration

- Restore all removed files from git history
- Re-implement full LLD configuration
- Update documentation to reflect restoration

## Risk Mitigation

### Low-Risk Nature

- **Standard approach**: Using default Rust linker is well-tested
- **Small change**: Only removes optimization, doesn't change core functionality
- **Easy rollback**: All changes are reversible
- **No user impact**: Internal build process change only

### Monitoring Plan

- Watch CI build times before/after
- Monitor for any linking errors
- Track developer feedback on local build times
- Verify release binary functionality

## Success Criteria

- [ ] CI workflows run 40-210 seconds faster per run
- [ ] All tests continue to pass
- [ ] Local development builds work normally
- [ ] Release builds work normally
- [ ] No linking errors or warnings
- [ ] Simplified configuration is easier to understand/maintain

## Documentation Updates

After successful removal:

1. Update `README.md` development section if it references LLD
2. Update `docs/development.md` if it includes LLD setup
3. Add note to `AGENTS.md` about LLD removal if relevant
4. Archive LLD planning documentation

This plan provides a low-risk path to remove unnecessary build complexity while
improving CI performance.
