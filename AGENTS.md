# AGENTS

This repository contains the `envsense` Rust CLI and library for detecting runtime environments.

## Development workflow
- Format Rust code with `cargo fmt --all` before committing (enforced by lefthook).
- Run the full test suite with `cargo test --all`.
- Keep `Cargo.lock` up to date and commit it when dependencies change.
- GitHub Actions runs `cargo test --all --locked` on pushes and pull requests.
- Use `cargo run --` instead of `envsense` when calling in development.

## Schema considerations
- The JSON schema defined in `src/schema.rs` is the contract for all consumers.
- Keep contexts, facets, traits, and evidence distinct.
- Changing the schema requires updating the `SCHEMA_VERSION` field and accompanying tests.
- Current schema version: `0.2.0` (CI detection moved from nested facets to flat traits).

## CLI behavior
- The CLI uses `clap` and should match examples in `README.md`.
- `--check` exits with status 0 on success and 1 on failure.

