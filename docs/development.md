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

1. Bump `SCHEMA_VERSION` in `src/schema.rs`
2. Update tests to expect new version
3. Run `cargo insta accept` to update snapshots
4. Verify all tests pass

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

See `docs/testing.md` for detailed testing guidelines.
