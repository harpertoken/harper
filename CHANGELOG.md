# Changelog

## [Unreleased]

### Added
- Privacy policy document
- API response caching system
- Named constants for magic numbers
- Service layer architecture (ChatService, SessionService)
- Configuration validation with error messages
- Custom HarperError enum with error variants
- Unit tests (21 total) for core functionality
- Performance optimizations with caching
- Security enhancements in cryptographic utilities
- CodeQL security analysis workflow
- Clean script output (removed emojis)

### Fixed
- Compilation errors and warnings
- Import resolution conflicts
- Memory safety issues
- Clippy warnings and formatting
- Security vulnerabilities in dependencies

### Changed
- Refactored monolithic functions into smaller components
- Enhanced error messages with context
- Added documentation for public APIs
- Cleaned up and updated dependencies
- Updated README structure
- Cleaned up test scripts

### Security
- Enhanced cryptographic utilities
- Improved sensitive data handling
- Added input validation

## [0.1.1] - 2025-08-26

### Added
- MCP integration with configuration
- Advanced configuration system
- Test suite with unit and integration tests
- Cryptographic utilities (AES-GCM, SHA-256, key generation, nonce management)
- Session export and management improvements
- Enhanced CLI interactivity
- Updated contribution guidelines

### Fixed
- Clippy warnings
- Index out-of-bounds in provider selection
- Error handling in cryptographic utilities

### Changed
- Improved documentation and formatting
- Enhanced error messages

## [0.1.0] - 2025-08-21

### Added
- Multi-provider AI support (OpenAI, Sambanova, Gemini)
- Shell command execution and web search
- SQLite session management
- Interactive CLI with colored output
- Session export functionality
- GitHub Actions CI/CD workflow
- Automated release process
- Basic project structure
