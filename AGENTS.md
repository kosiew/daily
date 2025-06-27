# AGENTS Guidelines

This repository hosts a macOS time-tracking application written in **Rust**. The `SPECS.md` file describes the planned features. The `screenshots` folder contains UI references and should not be modified unless explicitly requested.

## Task Execution Protocol

Follow the development plan outlined in `PLAN.md` in sequential order:

1. **Project Setup Phase** - Initialize Rust project with macOS dependencies
2. **Core Tracking Phase** - Implement sampling-based time tracking
3. **UI Development Phase** - Build status bar app and interfaces
4. **Data Management Phase** - Implement timesheet viewing and export
5. **Automation Phase** - Add productivity features and integrations
6. **Security Phase** - Implement privacy and encryption features
7. **Testing Phase** - Comprehensive testing and release preparation

### Implementation Guidelines
- Update `PLAN.md` with progress status after completing each major milestone
- Mark completed tasks with ✅ and in-progress tasks with 🔄
- Add timestamps and notes for completed items
- Create logical incremental commits following the plan structure
- Test each phase thoroughly before moving to the next

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

