# Script Integration Guide

This document explains how the testing scripts integrate with the existing documentation and development workflow.

## Overview

The refactoring introduced several testing scripts that work together with existing documentation to provide comprehensive testing and debugging capabilities.

## Scripts and Their Documentation Integration

### Core Testing Scripts

#### `scripts/compare-baseline.sh`
- **Purpose**: Validates detection output against baseline snapshots
- **CI Integration**: Used in `.github/workflows/ci.yml`
- **Documentation References**:
  - `docs/testing.md` - Testing guidelines and baseline testing section
  - `docs/architecture.md` - Detector architecture and testing infrastructure
  - `.devcontainer/README.md` - CI environment setup

#### `scripts/test-ci-environment.sh`
- **Purpose**: Tests in CI-like environment
- **Usage**: Development and debugging
- **Documentation References**:
  - `docs/testing.md` - Testing guidelines
  - `.devcontainer/README.md` - CI environment setup
  - `docs/debugging-ci.md` - Detailed debugging guide

#### `scripts/compare-environments.sh`
- **Purpose**: Compares local vs CI output
- **Usage**: Debugging environment differences
- **Documentation References**:
  - `docs/testing.md` - Testing guidelines
  - `docs/debugging-ci.md` - Detailed debugging guide
  - `.devcontainer/README.md` - CI environment setup

#### `scripts/run-in-docker.sh`
- **Purpose**: Docker-based CI simulation
- **Usage**: Alternative to devcontainer for non-VS Code users
- **Documentation References**:
  - `docs/testing.md` - Testing guidelines
  - `.devcontainer/README.md` - VS Code devcontainer setup
  - `docs/debugging-ci.md` - Detailed debugging guide

## Documentation Updates

### `docs/testing.md`
- Added **Baseline Tests** section explaining snapshot testing
- Added references to testing scripts
- Updated testing guidelines to include baseline scenarios
- Added `insta` crate to test tools list

### `docs/architecture.md`
- Added **Detector Architecture** section explaining the new trait-based system
- Added **Testing Infrastructure** section documenting baseline testing
- Updated extensibility guidelines to include baseline scenarios
- Added cross-references to testing documentation

### `README.md`
- Added **Development** section with testing instructions
- Added references to testing scripts and documentation
- Updated prerequisites to include Docker for CI testing

### `.devcontainer/README.md`
- Already well-integrated with testing scripts
- Provides VS Code devcontainer setup for CI environment testing
- References all relevant testing scripts

## Cross-References

### Scripts → Documentation
Each script includes header comments referencing relevant documentation:
```bash
# This script is part of the testing infrastructure. See:
# - docs/testing.md for testing guidelines
# - docs/architecture.md for detector architecture
# - .devcontainer/README.md for CI environment setup
```

### Documentation → Scripts
Documentation files reference scripts in appropriate contexts:
- Testing guidelines reference baseline validation scripts
- Architecture docs reference testing infrastructure
- README references development tools

### CI Integration
- `.github/workflows/ci.yml` uses `scripts/compare-baseline.sh`
- CI workflow validates baseline snapshots on every PR
- Ensures detection stability across environments

## Development Workflow

### For New Contributors
1. Read `README.md` for quick start
2. Review `docs/architecture.md` for system understanding
3. Use `docs/testing.md` for testing guidelines
4. Use `.devcontainer/README.md` for CI environment setup

### For Debugging Issues
1. Use `scripts/test-ci-environment.sh` to reproduce CI issues
2. Use `scripts/compare-environments.sh` to compare local vs CI output
3. Use `scripts/run-in-docker.sh` for Docker-based testing
4. Reference `docs/debugging-ci.md` for detailed troubleshooting

### For Adding New Detection
1. Follow guidelines in `docs/architecture.md`
2. Add unit tests following `docs/testing.md`
3. Add baseline scenarios in `tests/snapshots/`
4. Update baseline validation in CI

## Benefits of Integration

1. **Consistent Documentation**: All scripts reference relevant documentation
2. **Clear Workflows**: Documentation provides clear paths for different use cases
3. **CI Integration**: Baseline testing ensures detection stability
4. **Debugging Support**: Multiple tools for troubleshooting environment issues
5. **Developer Experience**: Clear guidance for new contributors and debugging

## Maintenance

When updating scripts or documentation:
1. Update cross-references to maintain consistency
2. Test script functionality after documentation changes
3. Update CI workflow if baseline testing changes
4. Ensure all documentation references are current

This integration ensures that the testing infrastructure is well-documented, discoverable, and maintainable for the long term.
