# Changelog

## [0.3.0] - 2024-XX-XX

### Breaking Changes

- **Schema Version**: Bumped from 0.2.0 to 0.3.0
- **Removed**: Legacy `facet:` and `trait:` syntax
- **Updated**: Contexts now returned as array instead of object
- **Changed**: All field access now uses dot notation (`agent.id`,
  `terminal.interactive`)

### Added

- **New dot notation syntax**: `agent.id=cursor`, `terminal.interactive`
- **Migration tools**: `envsense migrate` command for upgrading from v0.2.0
- **Comprehensive migration guide**: Step-by-step upgrade instructions
- **Improved CLI interface**: Cleaner, more intuitive predicate syntax

### Removed

- **Legacy schema structures**: `Contexts`, `Facets`, `Traits` objects
- **Legacy parser and evaluation functions**: All `facet:` and `trait:` handling
- **Backward compatibility layers**: Direct migration to new schema required

### Migration

See [Migration Guide](docs/migration-guide.md) for comprehensive upgrade
instructions.

**Quick Reference**:

- `facet:agent_id=cursor` → `agent.id=cursor`
- `trait:is_interactive` → `terminal.interactive`
- `facet:ci_id=github` → `ci.id=github`

---

## Unreleased

- Added automatic detection of terminal traits including color depth, TTY
  status, and hyperlink support.
- Added CI environment detection powered by `ci_info` with new traits,
  `envsense check ci`, and enhanced `info` output.

## [0.2.0] - 2024-01-XX

### Breaking Changes

- **Schema Version**: Bumped from 0.1.0 to 0.2.0
- **Removed**: `rules_version` field from JSON output (was always empty)
- **Updated**: All snapshot tests and documentation

### Technical

- Removed unused `rules_version` field that was intended for a rules system that
  was never implemented
- Updated schema contract and documentation to reflect the simplified structure
- All tests updated and passing
