<!--
Copyright 2025 harpertoken

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
-->

# Contributing to Harper

[![GitHub Issues](https://img.shields.io/github/issues/harpertoken/harper)](https://github.com/harpertoken/harper/issues)
[![GitHub Pull Requests](https://img.shields.io/github/issues-pr/harpertoken/harper)](https://github.com/harpertoken/harper/pulls)

We welcome contributions from the community! This guide will help you get started with contributing to Harper.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Development Workflow](#development-workflow)
- [Code Style & Guidelines](#code-style--guidelines)
- [Testing](#testing)
- [Submitting Changes](#submitting-changes)
- [Reporting Issues](#reporting-issues)
- [Adding New Features](#adding-new-features)
- [License](#license)

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust**: 1.82.0 or later ([install Rust](https://rustup.rs/))
- **Git**: For version control
- **SQLite3**: For database functionality
- **API Keys**: For testing AI provider integrations (optional)

### Quick Setup

1. **Fork and clone** the repository:
   ```bash
   git clone https://github.com/harpertoken/harper.git
   cd harper
   ```

2. **Set up development environment**:
   ```bash
   # Copy environment configuration
   cp config/env.example .env

   # Build the project
   cargo build

   # Run tests
   cargo test
   ```

3. **Optional: Set up git hooks** for commit validation:
   ```bash
   cp scripts/commit-msg .git/hooks/commit-msg && chmod +x .git/hooks/commit-msg
   cp scripts/pre-commit .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit
   ```

## Development Setup

### Environment Configuration

Create a `.env` file for local development:

```bash
# Copy the example configuration
cp config/env.example .env

# Edit with your API keys (optional for basic development)
# OPENAI_API_KEY=your_key_here
# SAMBASTUDIO_API_KEY=your_key_here
# GEMINI_API_KEY=your_key_here
```

### Building and Running

Before submitting any changes, it is crucial to validate them by running the
full preflight check. This command will build the repository, run all tests,
check for type errors, and lint the code.

To run the full suite of checks, execute the following command:

```bash
make all
```

This single command ensures that your changes meet all the quality gates of the
project. While you can run the individual steps (`make build`, `make test`, `make lint`,
`make fmt`) separately, it is highly recommended to use `make all` to
ensure a comprehensive validation.

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run the application
cargo run --release

# Run with specific features
cargo run --features additional_features

# Development mode with auto-reload (requires cargo-watch)
make dev
```

### Development Tools

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Generate documentation
cargo doc --open

# Check for security issues
cargo audit
```

## Git Repo

The main branch for this project is called "main".

## Development Workflow

### 1. Choose an Issue

- Check [GitHub Issues](https://github.com/harpertoken/harper/issues) for open tasks
- Look for issues labeled `good first issue` or `help wanted`
- Comment on the issue to indicate you're working on it

### 2. Create a Branch

```bash
# Create and switch to a new branch
git checkout -b feature/your-feature-name
# or
git checkout -b fix/issue-number-description
```

### 3. Make Changes

- Write clear, focused commits
- Follow the [Conventional Commits](https://conventionalcommits.org/) format
- Test your changes thoroughly

### 4. Run Quality Checks

```bash
# Run all checks
cargo test
cargo clippy
cargo fmt -- --check

# Run the full test suite
./harpertest
```

### 5. Update Documentation

- Update relevant documentation for any new features
- Ensure examples and guides reflect your changes
- Update the changelog if needed

## Code Style & Guidelines

### Rust Standards

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for consistent formatting
- Resolve all `cargo clippy` warnings
- Write idiomatic Rust code

### Code Quality

- **Documentation**: Document all public APIs with clear examples
- **Error Handling**: Use appropriate error types and provide meaningful messages
- **Performance**: Consider performance implications of changes
- **Security**: Follow secure coding practices

### Rust Coding Guidelines

For detailed Rust coding guidelines that the AI agent follows, see [AGENTS.md](../AGENTS.md#rust-coding-guidelines). These include best practices for structs/enums, iterators, error handling, testing, and more.

### Documentation Guidelines

When contributing to the codebase, follow these documentation guidelines:

- **Role:** You are an expert technical writer for contributors to Harper. Produce professional, accurate, and consistent documentation.
- **Technical Accuracy:** Do not invent facts, commands, code, API names, or output. All technical information must be based on code in the repository.
- **Style Authority:** Follow Rust documentation conventions and the project's established style.
- **Proactive User Consideration:** The user experience should be primary. Fill knowledge gaps while keeping documentation concise and accessible.

### Comments Policy

Only write high-value comments. Avoid excessive commenting; let the code be self-documenting where possible.

## General Requirements

- If there is something you do not understand or is ambiguous, seek confirmation or clarification before making changes.
- Use hyphens instead of underscores in command-line flags (e.g., `--my-flag` instead of `--my_flag`).
- Always refer to Harper as `Harper`, never `the Harper`.

### Commit Messages

Use conventional commit format:

```bash
# Format: type(scope): description
feat: add new AI provider support
fix(ui): resolve button alignment issue
docs: update installation instructions
test: add integration tests for chat service
```

### Pull Request Guidelines

- **Title**: Clear, descriptive title following conventional commit format
- **Description**: Detailed explanation of changes and rationale
- **Tests**: Include tests for new functionality
- **Documentation**: Update docs for user-facing changes
- **Breaking Changes**: Clearly mark and document breaking changes

## Testing

Harper uses Rust's built-in testing framework. When writing tests, aim to follow existing patterns. Key conventions include:

### Test Structure and Framework

- **Framework**: All tests are written using Rust's `#[test]` attribute and standard testing utilities.
- **File Location**: Test modules are co-located with the source files they test, using `#[cfg(test)]` modules.
- **Configuration**: Test behavior is configured via Cargo.toml and `#[test]` attributes.
- **Setup/Teardown**: Use `#[test]` functions for isolated tests. For shared setup, consider helper functions or test fixtures.

### Commonly Mocked Dependencies

- **External APIs**: Mock HTTP clients and API responses using libraries like `mockito` or `wiremock`.
- **File System**: Use `tempfile` for temporary files and directories.
- **Database**: Use in-memory SQLite or test-specific database instances.
- **Time**: Use `std::time::Instant` or libraries like `tokio::time` for async timing.

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with verbose output
cargo test -- --nocapture

# Run integration tests
cargo test --test integration_test

# Run the comprehensive test suite
./harpertest
```

### Test Coverage

Harper maintains comprehensive test coverage including:

- **Unit Tests**: Core functionality testing
- **Integration Tests**: End-to-end workflow validation
- **Security Tests**: Input validation and encryption
- **Performance Tests**: Benchmarking and optimization

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_functionality() {
        // Arrange
        let input = "test input";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, expected_output);
    }

    #[tokio::test]
    async fn test_async_functionality() {
        // For async tests
        let result = async_function().await;
        assert!(result.is_ok());
    }
}
```

### Asynchronous Testing

- Use `#[tokio::test]` for async tests when using Tokio.
- For timers, use `tokio::time::pause()` and `tokio::time::advance()` in tests.
- Test promise rejections with `assert!(result.is_err())` or pattern matching on `Result`.

### General Guidance

- When adding tests, first examine existing tests to understand and conform to established conventions.
- Pay close attention to the test setup at the top of existing test files; they reveal critical dependencies and how they are managed in a test environment.
- Use descriptive test names that explain what is being tested and the expected outcome.
- Prefer table-driven tests for testing multiple inputs/outputs.
- Mock external dependencies to ensure tests are fast and reliable.

## Submitting Changes

### Pull Request Process

1. **Ensure tests pass**: All CI checks must pass
2. **Update documentation**: Include relevant documentation changes
3. **Squash commits**: Combine related commits into logical units
4. **Write clear description**: Explain what and why, not just how

### Review Process

- Maintainers will review your PR
- Address any requested changes
- Once approved, your PR will be merged
- Your contribution will be acknowledged in the changelog

## Reporting Issues

### Bug Reports

When reporting bugs, please include:

- **Version**: `harper --version`
- **Operating System**: OS name and version
- **Steps to Reproduce**: Clear, numbered steps
- **Expected Behavior**: What should happen
- **Actual Behavior**: What actually happens
- **Error Logs**: Relevant error messages or logs
- **Configuration**: Any relevant configuration details

### Feature Requests

For feature requests, please:

- Clearly describe the proposed feature
- Explain the use case and benefits
- Consider alternative implementations
- Reference similar features in other tools

## Adding New Features

### Adding AI Providers

To add support for a new AI provider:

1. **Update the API Provider enum**:
   ```rust
   #[derive(Debug, Clone, PartialEq)]
   pub enum ApiProvider {
       OpenAI,
       Sambanova,
       Gemini,
       NewProvider, // Add your new provider
   }
   ```

2. **Extend the configuration**:
   - Add provider to `ApiConfig`
   - Update validation logic
   - Add environment variable handling

3. **Implement the provider**:
   - Add provider-specific logic in `providers/`
   - Update the LLM calling function
   - Handle provider-specific parameters

4. **Update documentation**:
   - Add to README provider table
   - Update configuration examples
   - Document any special requirements

### Adding CLI Commands

1. Add new command to the menu system
2. Implement the command logic
3. Update help text and documentation
4. Add tests for the new functionality

### Database Schema Changes

1. Update the schema migration logic
2. Ensure backwards compatibility
3. Update any affected queries
4. Test with existing data

### Comments Policy

Only write high-value comments if at all. Avoid talking to the user through comments.

### General Requirements

- If there is something you do not understand or is ambiguous, seek confirmation or clarification from the user before making changes based on assumptions.
- Use hyphens instead of underscores in flag names (e.g., `--my-flag` instead of `--my_flag`).
- Always refer to Harper as `Harper`, never `the Harper`.

## Community

- **Discussions**: [GitHub Discussions](https://github.com/harpertoken/harper/discussions)
- **Discord**: Join our community chat (link TBD)
- **Newsletter**: Subscribe for updates (link TBD)

## License

By contributing to Harper, you agree that your contributions will be licensed under the Apache 2.0 License. See [LICENSE](LICENSE) for details.

---

Thank you for contributing to Harper! Your efforts help make AI more accessible and secure for everyone.
