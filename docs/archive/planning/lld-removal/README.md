# LLD Removal Planning

**Created**: 2025-09-11  
**Status**: Analysis Complete, Ready for Implementation  
**Decision**: Remove LLD from build and CI processes

## Overview

This directory contains the complete analysis and plan for removing LLD (LLVM
linker) from the envsense project's build and CI processes. The decision is
based on thorough analysis showing that LLD installation overhead significantly
outweighs linking performance benefits for this project.

## Key Findings

- **Installation overhead**: 40-210 seconds per CI run
- **Linking time savings**: 1-3 seconds per build
- **Net impact**: 37-207 seconds LOST per CI run
- **Complexity cost**: Multiple configuration approaches, platform-specific
  logic
- **Inconsistency**: CI uses LLD, release builds don't

## Files

### [`analysis.md`](analysis.md)

Comprehensive analysis of current LLD usage, time costs, and architectural
issues.

**Key conclusions**:

- LLD adds more time than it saves
- Configuration is inconsistent across CI/release
- Project size doesn't justify LLD complexity
- Standard Rust linking is sufficient

### [`removal-plan.md`](removal-plan.md)

Step-by-step implementation plan for removing LLD.

**Phases**:

1. **CI Workflow Cleanup**: Remove LLD installation from GitHub Actions
2. **Cargo Configuration**: Simplify `.cargo/config.toml`
3. **Cleanup**: Remove unused scripts and documentation
4. **Validation**: Test and measure improvements

### [`measure-impact.sh`](measure-impact.sh)

Measurement script to quantify actual time costs and validate analysis.

**Usage**:

```bash
./docs/planning/remove-lld/measure-impact.sh
```

**Measures**:

- LLD installation time by platform
- Build time comparison (with/without LLD)
- Incremental linking time differences

## Decision Rationale

### Cost Analysis

| Factor          | Current (with LLD)    | After Removal            |
| --------------- | --------------------- | ------------------------ |
| CI time per run | +40-210s installation | Standard build time      |
| Linking time    | 2-3s faster           | 2-3s slower (negligible) |
| **Net impact**  | **37-207s slower**    | **37-207s faster**       |
| Complexity      | High (dual config)    | Low (standard)           |
| Consistency     | Inconsistent          | Consistent               |

### Risk Assessment: **LOW**

- **Performance impact**: Minimal (1-3s per build)
- **Functionality risk**: None (standard linking is reliable)
- **Rollback difficulty**: Easy (can re-add LLD if needed)

## Implementation Status

- [x] **Analysis Complete**: Comprehensive review of LLD usage and impact
- [x] **Plan Complete**: Detailed step-by-step removal plan
- [x] **Measurement Tool**: Script to validate analysis
- [ ] **Implementation**: Execute removal plan
- [ ] **Validation**: Measure actual improvements
- [ ] **Documentation**: Archive planning docs

## Expected Benefits Post-Removal

### Performance

- **40-210 seconds faster CI** per workflow run
- **Consistent build times** across CI and release
- **Simplified caching** (no platform-specific dependencies)

### Maintainability

- **Single configuration approach**
- **No platform-specific installation logic**
- **Fewer CI failure points**
- **Easier debugging** (standard toolchain)

### Consistency

- **CI and release builds identical**
- **No configuration drift between environments**
- **Predictable build behavior**

## Related Context

### Background

LLD was adopted in 2024 with the goal of improving build performance. The
initial planning documentation is in `docs/archive/planning/lld-adoption/` and
indicates "100% Complete ✅" status.

### Why Removal Now?

1. **Usage analysis** shows installation overhead exceeds benefits
2. **Architectural inconsistencies** between CI and release builds
3. **Project characteristics** don't match LLD's sweet spot
4. **Maintenance burden** outweighs performance gains

### LLD Sweet Spot (Not This Project)

LLD provides most benefit for:

- Large projects with many native dependencies
- Complex linking scenarios
- Development workflows with frequent rebuilds
- Projects where linking is >10% of build time

### This Project's Reality

- Small CLI with focused dependencies
- Linking represents <5% of build time
- CI runs don't benefit from iteration speed
- Standard Rust linking is fast and reliable

## Next Steps

1. **Run measurement script** to validate analysis: `./measure-impact.sh`
2. **Follow removal plan** in `removal-plan.md`
3. **Monitor improvements** after implementation
4. **Archive documentation** when complete

This removal will make the envsense build process faster, simpler, and more
maintainable.
