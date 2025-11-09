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

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run the application
cargo run --release

# Run with specific features
cargo run --features additional_features
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
}
```

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

## Community

- **Discussions**: [GitHub Discussions](https://github.com/harpertoken/harper/discussions)
- **Discord**: Join our community chat (link TBD)
- **Newsletter**: Subscribe for updates (link TBD)

## License

By contributing to Harper, you agree that your contributions will be licensed under the Apache 2.0 License. See [LICENSE](LICENSE) for details.

---

Thank you for contributing to Harper! Your efforts help make AI more accessible and secure for everyone.