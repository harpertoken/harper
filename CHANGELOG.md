# Changelog

[![Release](https://img.shields.io/github/v/release/harpertoken/harper)](https://github.com/harpertoken/harper/releases)
[![All Releases](https://img.shields.io/github/downloads/harpertoken/harper/total)](https://github.com/harpertoken/harper/releases)

All notable changes to Harper will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Table of Contents

- [Unreleased](#unreleased)
- [0.1.8-beta.1](#018-beta1---2025-11-09)
- [0.1.8-beta](#018-beta---2025-11-09)
- [0.1.7](#017---2025-11-09)
- [0.1.6](#016---2025-11-08)
- [0.1.5](#015---2025-09-20)
- [0.1.4](#014---2025-09-15)
- [0.1.3](#013---2025-09-14)
- [0.1.2](#012---2025-09-03)
- [0.1.1](#011---2025-09-03)
- [0.1.0](#010---2025-08-26)

---

## [Unreleased]

### Planned Features
- Web interface for chat sessions
- Plugin system for custom tools
- Multi-language support
- Advanced session analytics
- Cloud deployment options

---

## [0.1.8-beta.1] - 2025-11-09

### Fixed
- **Release Workflow**: Enhanced release process with GitHub CLI integration for deleting existing releases
- **Release Workflow**: Added comprehensive logging and validation to release automation
- **Release Workflow**: Removed invalid overwrite configuration from release creation
- **Release Workflow**: Improved handling of immutable releases by deleting and recreating them
- **API Integration**: Fixed error handling in release notes script
- **Testing**: Corrected database path quoting in test configurations
- **Testing**: Fixed environment variable handling in binary execution tests
- **Documentation**: Enhanced documentation and fixed git hooks

### Changed
- **Dependencies**: Updated Cargo.lock with latest dependency versions

---

## [0.1.8-beta] - 2025-11-09

### Fixed
- **Release Workflow**: Handle immutable releases by deleting and recreating them to allow asset overwrites

---

## [0.1.7] - 2025-11-09

### Changed
- **Documentation**: Complete rewrite of all documentation files with professional structure and comprehensive guides
- **Quality Assurance**: Enhanced pre-commit hooks and commit message validation

### Fixed
- **Testing**: Resolved compilation error in integration tests

---

## [0.1.6] - 2025-11-08

### Added
- **Docker Support**: Automated container image publishing to releases
- **Documentation**: Comprehensive README and contributing guides
- **Security**: Enhanced security policy and vulnerability reporting

### Changed
- **Development Environment**: Improved dev container configuration
- **Code Quality**: Pre-commit and commit-msg git hooks for automated quality checks
- **Linting**: Enhanced Clippy configuration in CI pipeline
- **Toolchain**: Updated Rust MSRV to 1.82.0 for better compatibility
- **CI/CD**: Improved continuous integration with comprehensive testing

### Fixed
- **Compilation**: Fixed slice mutation error in chat service
- **Performance**: Resolved manual `div_ceil` implementation warnings
- **Code Quality**: Removed unused imports and outdated comments
- **Testing**: Fixed integration test environment variable handling

### Security
- **Audit**: Completed security audit of dependencies
- **Encryption**: Verified AES-GCM-256 implementation integrity
- **Input Validation**: Enhanced validation for all user inputs

---

## [0.1.5] - 2025-09-20

### Added
- **Version Command**: `--version` flag for version information
- **Test Suites**: Comprehensive test suite with `./harpertest` script
- **Documentation**: Initial documentation and usage examples

### Changed
- **Error Handling**: Improved error messages and user feedback
- **Performance**: Optimized database queries and response times

### Fixed
- **Memory Usage**: Reduced memory footprint for large conversations
- **Stability**: Fixed occasional crashes during long sessions

---

## [0.1.4] - 2025-09-15

### Fixed
- **Database**: Connection stability improvements
- **UI**: Minor interface rendering fixes
- **Dependencies**: Updated vulnerable dependencies

---

## [0.1.3] - 2025-09-14

### Added
- **CI/CD Pipeline**: GitHub Actions for automated testing and releases
- **Security Scanning**: CodeQL integration for vulnerability detection
- **Multi-Provider Support**: Enhanced AI provider integration

### Changed
- **Architecture**: Improved modular design for better maintainability
- **Error Handling**: More robust error recovery mechanisms

### Fixed
- **Build Process**: Resolved compilation issues on different platforms
- **API Integration**: Fixed authentication issues with AI providers

---

## [0.1.2] - 2025-09-03

### Added
- **Security Features**: DevSkim security analysis integration
- **CI Enhancements**: Expanded test coverage and automated checks

### Changed
- **Performance**: Optimized API response handling
- **Reliability**: Improved connection stability

### Security
- **Vulnerability Fixes**: Addressed reported security issues
- **Code Review**: Enhanced security-focused code review process

---

## [0.1.1] - 2025-09-03

### Added
- **Caching**: Response caching for improved performance
- **Testing**: Expanded unit and integration test coverage

### Changed
- **User Experience**: Improved CLI interface and feedback
- **Configuration**: More flexible configuration options

### Fixed
- **Stability**: Resolved application crashes under certain conditions
- **Compatibility**: Fixed issues with different operating systems

---

## [0.1.0] - 2025-08-26

### Added
- **Initial Release**: Core Harper functionality
- **AI Providers**: Support for OpenAI, Sambanova, and Gemini
- **CLI Interface**: Interactive command-line interface
- **Session Management**: Persistent conversation storage
- **SQLite Database**: Local data storage with encryption
- **Command Execution**: Safe system command execution
- **Web Search**: Integrated search capabilities

### Features
- Multi-provider AI integration with automatic model selection
- Encrypted local storage for conversation history
- Interactive menu-driven interface
- Session export and management
- Cross-platform compatibility (Linux, macOS, Windows)

---

## Version History Legend

- **Added**: New features or functionality
- **Changed**: Modifications to existing features
- **Deprecated**: Features marked for future removal
- **Removed**: Features completely removed
- **Fixed**: Bug fixes and patches
- **Security**: Security-related changes and fixes

## Contributing to Changelog

When contributing changes that should be documented:

1. **Add entries** to the "Unreleased" section above
2. **Categorize properly** using Added, Changed, Fixed, etc.
3. **Be descriptive** but concise in change descriptions
4. **Reference issues** when applicable (e.g., `#123`)

The changelog is automatically updated during the release process.

---

**For the latest updates, see the [GitHub Releases](https://github.com/harpertoken/harper/releases) page.**
