# AGENTS

This repository contains the `envsense` Rust CLI and library for detecting runtime environments.

## Development workflow
- Format Rust code with `cargo fmt --all` before committing.
- Run the full test suite with `cargo test`.
- Keep `Cargo.lock` up to date and commit it when dependencies change.
- GitHub Actions runs `cargo test --all --locked` on pushes and pull requests.

## Schema considerations
- The JSON schema defined in Phase 0 is the contract for all consumers.
- Keep contexts, facets, traits, and evidence distinct.
- Changing the schema requires updating the `version` field and accompanying tests.

## CLI behavior
- The CLI uses `clap` and should match examples in `README.md`.
- `--check` exits with status 0 on success and 1 on failure.

