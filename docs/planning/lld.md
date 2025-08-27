# LLD Adoption Strategy for envsense

## Context

This document outlines the practical approach to adopting LLD (LLVM's fast linker) for the envsense project, considering the recent CI instability and current project state.

## Current Project State

### Recent CI Issues
- Experienced rustup download failures in GitHub Actions
- Updated CI workflow to use modern tooling (`dtolnay/rust-toolchain@stable` + `Swatinem/rust-cache@v2`)
- Project uses lefthook, mise, and modern development practices

### Project Characteristics
- **CLI tool**: Fast linking valuable for development iteration
- **Library + macros**: Multiple crates benefit from faster linking
- **Cross-platform**: macOS and Linux support needed
- **Solo development**: Simpler rollout but still needs robust setup

## Practical Adoption Strategy

### Phase 1: Local Development Validation (Week 1)

#### 1.1 Install and Test Locally

**macOS Setup:**
```bash
# Install LLVM (includes LLD)
brew install llvm

# Verify installation
which ld.lld

# Test with current project
cargo clean
cargo build --release
```

**Linux Setup:**
```bash
# Ubuntu/Debian
sudo apt install clang lld

# Verify installation
which ld.lld
```

#### 1.2 Create Performance Benchmark

```bash
# Create benchmark script
cat > scratch/lld-benchmark.sh << 'EOF'
#!/usr/bin/env bash
echo "=== LLD Performance Benchmark ==="

echo "Baseline build (default linker):"
time cargo clean && cargo build --release

echo "LLD build:"
export RUSTFLAGS="-C link-arg=-fuse-ld=lld"
time cargo clean && cargo build --release

echo "Benchmark complete"
EOF

chmod +x scratch/lld-benchmark.sh
```

#### 1.3 Configure Cargo for LLD

Create `~/.cargo/config.toml`:
```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.aarch64-apple-darwin]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

[target.x86_64-apple-darwin]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=lld"]
```

### Phase 2: Gradual CI Integration (Week 2)

#### 2.1 Start with Single Job

Add LLD to the `lint` job first (lowest risk):

```yaml
lint:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
      with:
        components: rustfmt, clippy
    - uses: Swatinem/rust-cache@v2
    
    # Add LLD with fallback
    - name: Install LLD
      run: |
        sudo apt update
        sudo apt install -y clang lld || echo "LLD installation failed, using default linker"
    
    - name: Configure LLD
      run: |
        if command -v ld.lld >/dev/null 2>&1; then
          echo 'RUSTFLAGS="-C link-arg=-fuse-ld=lld"' >> $GITHUB_ENV
          echo "Using LLD linker"
        else
          echo "LLD not available, using default linker"
        fi
    
    - name: Check formatting
      run: cargo fmt --all -- --check
    - name: Run clippy
      run: cargo clippy --all --locked -- -D warnings
```

#### 2.2 Monitor and Validate

- Watch for any build failures
- Compare build times with previous runs
- Ensure all tests still pass

#### 2.3 Expand to Other Jobs

Once lint job is stable, add to test jobs:

```yaml
test:
  strategy:
    matrix:
      os: [ubuntu-latest, macos-latest]
  runs-on: ${{ matrix.os }}
  steps:
    - uses: actions/checkout@v4
    - uses: dtolnay/rust-toolchain@stable
    - uses: Swatinem/rust-cache@v2
    
    # Platform-specific LLD setup
    - name: Install LLD (Linux)
      if: matrix.os == 'ubuntu-latest'
      run: |
        sudo apt update
        sudo apt install -y clang lld || echo "LLD installation failed"
    
    - name: Install LLD (macOS)
      if: matrix.os == 'macos-latest'
      run: |
        brew install llvm || echo "LLVM installation failed"
        echo "CC=clang" >> $GITHUB_ENV
        echo "CXX=clang++" >> $GITHUB_ENV
    
    - name: Configure LLD
      run: |
        if command -v ld.lld >/dev/null 2>&1; then
          echo 'RUSTFLAGS="-C link-arg=-fuse-ld=lld"' >> $GITHUB_ENV
          echo "Using LLD linker"
        else
          echo "LLD not available, using default linker"
        fi
    
    - name: Run tests
      run: cargo test --all --locked
```

### Phase 3: Optimization and Monitoring (Week 3)

#### 3.1 Performance Tracking

Monitor these metrics:
- **CI build times**: Compare before/after LLD adoption
- **Local build times**: Track development iteration speed
- **Build success rate**: Ensure no increase in failures

#### 3.2 Configuration Optimization

Consider these Cargo.toml optimizations for LLD:

```toml
[profile.release]
# Optimize for LLD
lto = true
codegen-units = 1

[profile.dev]
# Faster debug builds with LLD
codegen-units = 16
```

#### 3.3 Troubleshooting Common Issues

**Issue**: Build failures with LLD
**Solution**: Fallback to default linker temporarily

**Issue**: Incompatible crates
**Solution**: Document and create workarounds

**Issue**: Performance regression
**Solution**: Revert and investigate

### Phase 4: Documentation and Team Rollout (Week 4)

#### 4.1 Update Project Documentation

**README.md additions:**
```markdown
## Performance Optimization

This project uses LLD (LLVM's fast linker) for improved build performance.

### Local Setup
```bash
# macOS
brew install llvm

# Linux
sudo apt install clang lld
```

### Configuration
Create `~/.cargo/config.toml` with LLD settings (see docs/planning/lld.md)
```

**AGENTS.md updates:**
- Add LLD setup to development workflow
- Include troubleshooting steps

#### 4.2 Create Troubleshooting Guide

```markdown
# LLD Troubleshooting

## Common Issues

### Build Failures
If builds fail with LLD, temporarily disable:
```bash
unset RUSTFLAGS
cargo build
```

### Performance Issues
If LLD is slower than expected:
1. Check LLD version: `ld.lld --version`
2. Verify configuration in `~/.cargo/config.toml`
3. Consider reverting to default linker

### CI Issues
If CI fails with LLD:
1. Check GitHub Actions logs for LLD installation errors
2. Verify platform-specific setup
3. Use fallback configuration
```

## Success Criteria

### Performance Targets
- **Local builds**: 2-3x faster linking on Linux, 1.5-2x on macOS
- **CI builds**: 10-20% reduction in total build time
- **Development iteration**: Noticeably faster `cargo build` times

### Stability Targets
- **Build success rate**: No increase in failures
- **Test pass rate**: All existing tests continue to pass
- **CLI functionality**: No regression in tool behavior

### Adoption Targets
- **Local development**: LLD configured and working
- **CI integration**: All jobs using LLD successfully
- **Documentation**: Complete setup and troubleshooting guides

## Risk Mitigation

### Fallback Strategy
- **Graceful degradation**: CI continues with default linker if LLD fails
- **Easy rollback**: Simple environment variable change to disable LLD
- **Monitoring**: Track build times and failure rates

### Compatibility Testing
- **Test all build types**: debug, release, test
- **Verify all platforms**: macOS, Linux
- **Check all crates**: main library, macros, CLI

## Timeline Summary

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1 | Local validation | LLD working locally, performance benchmarks |
| 2 | CI integration | LLD in CI with fallback, monitoring setup |
| 3 | Optimization | Performance tuning, issue resolution |
| 4 | Documentation | Complete docs, team rollout |

## Resources

- [LLD Documentation](https://lld.llvm.org/)
- [Rust Linker Configuration](https://doc.rust-lang.org/rustc/codegen-options/index.html#linker)
- [LLVM Releases](https://releases.llvm.org/)
- [GitHub Actions LLD Example](https://github.com/actions-rs/toolchain/issues/123)
