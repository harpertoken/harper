# Changelog

All notable changes to this project are documented here.  
Follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **Privacy Policy**: Comprehensive privacy policy document (PRIVACY.md) with detailed data collection, usage, and user rights information
- **API Response Caching**: Intelligent caching system for API responses to improve performance and reduce redundant requests
- **Named Constants**: Replaced magic numbers with named constants throughout the codebase for better maintainability
- **Service Layer Architecture**: Extracted business logic into dedicated services (ChatService, SessionService) for better separation of concerns
- **Enhanced Configuration Validation**: Robust validation for all configuration values with meaningful error messages
- **Comprehensive Error Handling**: Custom `HarperError` enum with specific error variants and improved error propagation
- **Expanded Test Coverage**: Added extensive unit tests covering core functionality, configuration validation, and edge cases (21 total tests)
- **Performance Optimizations**: Implemented caching and optimized string operations for better performance
- **Security Enhancements**: Removed debug print statements from cryptographic utilities and improved security practices

### Fixed
- **Compilation Errors**: Resolved all compilation errors and warnings throughout the codebase
- **Import Conflicts**: Fixed import resolution issues between different modules and external crates
- **Memory Safety**: Improved error handling to prevent potential memory safety issues
- **Code Quality**: Fixed Clippy warnings and improved code formatting consistency

### Changed
- **Code Organization**: Refactored monolithic functions into smaller, focused, and testable components
- **Error Messages**: Enhanced error messages with more context and actionable information
- **Documentation**: Added comprehensive documentation for all public functions and modules
- **Dependencies**: Cleaned up unused dependencies and updated to latest compatible versions
- **README**: Updated with privacy policy information and improved structure

### Security
- **Cryptographic Improvements**: Enhanced cryptographic utilities with better error handling and security practices
- **Data Protection**: Improved handling of sensitive data like API keys and conversation history
- **Input Validation**: Added comprehensive validation for user inputs and configuration values

## [0.1.1] - 2025-08-26
Feature-rich release with improved functionality, code quality, and documentation.

### Added
- Model Context Protocol (MCP) integration with configurable options
- Advanced configuration system with environment and file support
- Comprehensive test suite with unit and integration tests
- Cryptographic utilities including AES-GCM, SHA-256 hashing, key generation, and nonce management
- Persistent session export and management improvements
- Enhanced CLI interactivity and tool commands
- Updated CONTRIBUTING.md with detailed contribution guidelines

### Fixed
- Clippy warnings across the codebase
- Potential index out-of-bounds issue in provider selection
- Error handling improvements in cryptographic utilities

### Changed
- Improved code documentation and formatting
- Enhanced error messages for better debugging

## [0.1.0] - 2025-08-21
Initial release of Harper AI Agent.

### Added
- Multi-provider AI agent support (OpenAI, Sambanova, Google Gemini)
- Shell command execution and web search capabilities
- Session management with SQLite
- Interactive CLI with colored output
- Session export functionality
- Persistent benchmark results
- GitHub Actions CI/CD workflow
- Automated release process via GitHub Releases
- Basic project structure and documentation
