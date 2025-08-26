# Contributing to Harper

Harper welcomes contributions of all types, including bug reports, new features, and documentation improvements. Contributors are expected to follow project guidelines and maintain code quality.

## Table of Contents

* [Code of Conduct](#code-of-conduct)
* [Getting Started](#getting-started)
* [Development Setup](#development-setup)
* [Making Changes](#making-changes)
* [Pull Request Process](#pull-request-process)
* [Code Style](#code-style)
* [Testing](#testing)
* [Issue Reporting](#issue-reporting)
* [Adding AI Providers](#adding-ai-providers)
* [Pull Requests](#pull-requests)
* [License](#license)
* [Questions](#questions)

---

## Code of Conduct

This project and all participants are governed by the [Code of Conduct](CODE_OF_CONDUCT.md). Contributors are expected to adhere to its principles.

---

## Getting Started

1. Fork the repository on GitHub.
2. Clone the fork locally:

   ```bash
   git clone https://github.com/bniladridas/harper.git
   cd harper
   ```
3. Add the upstream remote:

   ```bash
   git remote add upstream https://github.com/bniladridas/harper.git
   ```
4. Create a new branch for changes:

   ```bash
   git checkout -b feature/your-feature-name
   ```

---

## Development Setup

1. Install Rust (if not already installed):

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Add development components:

   ```bash
   rustup component add rustfmt clippy
   ```
3. Build the project:

   ```bash
   cargo build
   ```
4. Configure the environment:

   ```bash
   cp env.example .env
   # Edit .env with API keys
   ```
5. Run initial tests:

   ```bash
   cargo test
   ```

---

## Making Changes

1. Implement changes following the project’s **Code Style** guidelines.
2. Add tests for new functionality.
3. Run the full test suite, Clippy, and formatting checks:

   ```bash
   cargo test --all-features --workspace --verbose
   cargo clippy --all-targets --all-features --workspace -- -D warnings
   cargo fmt --all -- --check
   ```
4. **Check passes confirmation:**

> The code is expected to pass both `cargo fmt --check` and `cargo clippy -- -D warnings` without any issues. Once these checks pass, the changes can be committed and pushed, and the CI should pass successfully.

5. Format the code for consistency:

   ```bash
   cargo fmt --all
   ```
6. Commit changes with a descriptive message:

   ```bash
   git add .
   git commit -m "Add: Brief description of changes

   More detailed explanation if needed.
   Fixes #<issue-number>"
   ```
7. Push the branch:

   ```bash
   git push -u origin your-branch-name
   ```

---

## Pull Request Process

1. Update the fork with upstream changes:

   ```bash
   git fetch upstream
   git rebase upstream/main
   ```
2. Push the branch to the fork.
3. Open a Pull Request against the `main` branch.
4. Ensure all CI checks pass.
5. Address any code review feedback before merging.

---

## Code Style

* Follow Rust API Guidelines: [https://rust-lang.github.io/api-guidelines/](https://rust-lang.github.io/api-guidelines/)
* Use `cargo fmt` to format code
* Run `cargo clippy -- -D warnings` and resolve all warnings
* Document all public APIs with Rustdoc comments
* Keep commits atomic and descriptive
* Follow Conventional Commits specification

---

## Testing

* Write unit tests for new functionality
* Add integration tests for critical paths
* Run all tests before submitting a PR:

  ```bash
  cargo test
  cargo test -- --ignored  # Run ignored tests
  ```

---

## Issue Reporting

When reporting issues, include:

* Harper version: `cargo run -- --version`
* Operating system and version
* Steps to reproduce the issue
* Expected vs actual behavior
* Relevant logs or error messages

---

## Adding AI Providers

1. Add provider to the `ApiProvider` enum.
2. Update the `ApiConfig` struct.
3. Implement API logic in `call_llm()`.
4. Add menu configuration if required.
5. Update documentation and `env.example`.

---

## Pull Requests

* One feature per PR
* Include tests for new functionality
* Update README for user-facing features
* Describe changes and rationale clearly

---

## License

Contributions are licensed under the [Apache License 2.0](LICENSE).

---

## Questions

* For bugs or features: open an issue
* For contribution questions: start a discussion
* Check existing issues before asking