# Changelog

## Unreleased
- Added automatic detection of terminal traits including color depth,
  TTY status, and hyperlink support.
- Added CI environment detection powered by `ci_info` with new traits,
  `envsense check ci`, and enhanced `info` output.

## [0.2.0] - 2024-01-XX
### Breaking Changes
- **Schema Version**: Bumped from 0.1.0 to 0.2.0
- **Removed**: `rules_version` field from JSON output (was always empty)
- **Updated**: All snapshot tests and documentation

### Technical
- Removed unused `rules_version` field that was intended for a rules system that was never implemented
- Updated schema contract and documentation to reflect the simplified structure
- All tests updated and passing
