# AGENTS Guidelines

This repository hosts a macOS time-tracking application written in **Rust**. The `SPECS.md` file describes the planned features. The `screenshots` folder contains UI references and should not be modified unless explicitly requested.

## Development
- Use idiomatic Rust and keep macOS compatibility in mind.
- All code should be formatted with `cargo fmt`. Pull requests should run `cargo fmt -- --check` and fail if formatting is not clean.
- Lint with `cargo clippy -- -D warnings` to ensure no warnings remain.
- Include unit tests where possible and run `cargo test` before committing.
- Avoid committing build artifacts or dependency lockfiles other than `Cargo.lock`.

## Commit Messages
- Use short, descriptive commit summaries in the form `area: summary`.

## Pull Request Instructions
When submitting a PR, describe the changes and include the results of the format, lint, and test commands in the Testing section. If commands fail due to environment issues, note it in the PR message.

