# Contributing to Harper

Thank you for considering contributing to Harper! We welcome all contributions, from bug reports to new features and documentation improvements.

## Table of Contents
- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Making Changes](#making-changes)
- [Pull Request Process](#pull-request-process)
- [Code Style](#code-style)
- [Testing](#testing)
- [Issue Reporting](#issue-reporting)
- [License](#license)

## Code of Conduct

This project and everyone participating in it is governed by our [Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to uphold this code.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally:
   ```bash
   git clone https://github.com/bniladridas/harper.git
   cd harper
   ```
3. Add the upstream remote:
   ```bash
   git remote add upstream https://github.com/bniladridas/harper.git
   ```
4. Create a new branch for your changes:
   ```bash
   git checkout -b feature/your-feature-name
   ```

## Development Setup

1. Install Rust (if not already installed):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Install development dependencies:
   ```bash
   rustup component add rustfmt clippy
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Configure your environment:
   ```bash
   cp env.example .env
   # Edit .env with your API keys
   ```
5. Run tests:
   ```bash
   cargo test
   ```

## Making Changes

1. Make your changes following the code style guidelines
2. Add tests for new functionality
3. Run the test suite:
   ```bash
   cargo test && cargo clippy -- -D warnings
   ```
4. Format your code:
   ```bash
   cargo fmt --all
   ```
5. Commit your changes with a descriptive message:
   ```
   git commit -m "Add: Brief description of changes
   
   More detailed explanation if needed.
   Fixes #123"
   ```

## Pull Request Process

1. Update your fork with the latest changes from upstream:
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```
2. Push your changes to your fork:
   ```bash
   git push -u origin your-branch-name
   ```
3. Open a Pull Request against the `main` branch
4. Ensure all CI checks pass
5. Address any code review feedback

## Code Style

- Follow the Rust API Guidelines: https://rust-lang.github.io/api-guidelines/
- Use `cargo fmt` to format your code
- Run `cargo clippy` and fix all warnings
- Document all public APIs with Rustdoc comments
- Keep commits focused and atomic
- Write meaningful commit messages following the Conventional Commits specification

## Testing

- Write unit tests for new functionality
- Add integration tests for critical paths
- Run all tests before submitting a PR:
  ```bash
  cargo test
  cargo test -- --ignored  # Run ignored tests
  ```

## Issue Reporting

When reporting issues, please include:
- Harper version (`cargo run -- --version`)
- Your operating system and version
- Steps to reproduce the issue
- Expected vs actual behavior
- Any relevant error messages or logs

## License

By contributing to Harper, you agree that your contributions will be licensed under the [Apache License 2.0](LICENSE).
- Keep commits atomic and descriptive

## Adding AI Providers

1. Add provider to `ApiProvider` enum
2. Update `ApiConfig` struct
3. Implement API logic in `call_llm()`
4. Add menu configuration
5. Update documentation and env.example

## Pull Requests

- One feature per PR
- Include tests
- Update README for user-facing features
- Describe changes and rationale

## Questions

- Bugs/features: Open an issue
- Contributing questions: Start a discussion
- Check existing issues first

## Code of Conduct

Be respectful, inclusive, and constructive.