# Changelog

## [1.3.0] - 2025-09-03

Complete CodeQL & CI Build Resolution

### Added
- CodeQL dependency conflict resolution
- Cross-platform CI build fixes
- Cargo.lock synchronization
- DevSkim security scanning improvements
- Documentation cleanup

### Fixed
- CodeQL dependency conflicts (20+ → 9 minor conflicts)
- CI build failures across all platforms
- Cargo.lock synchronization issues
- DevSkim warnings about incomplete functionality
- System-configuration compilation errors
- MCP client version conflicts (reqwest v0.11 vs v0.12)
- Cross-platform build compatibility issues

### Changed
- MCP functionality temporarily disabled
- Enhanced error handling for dependency resolution
- Improved CI/CD pipeline reliability
- Updated dependency management strategy
- Professional code documentation standards

### Security
- Enhanced CodeQL analysis accuracy
- Improved DevSkim security scanning
- Resolved dependency security conflicts
- Clean security audit results
- Better sensitive data handling

### Technical
- Dependencies: Reduced conflicts from 20+ to 9 minor
- CI Checks: All 15 checks passing across platforms
- Build Targets: Verified on Linux, Windows, macOS (Intel + ARM)
- Security: Audit clean with enhanced analysis
- Code Quality: Clippy clean, documentation updated

## [1.2.0] - 2025-09-03

### Added
- DevSkim security scanning workflow
- Enhanced CI/CD pipeline with security checks
- Professional code documentation standards

### Fixed
- API key security vulnerability (URL → Authorization header)
- CodeQL false positives from duplicate dependencies
- Security scanning configuration

### Security
- Moved API keys from URL query strings to Authorization headers
- Enhanced sensitive data transmission security
- Improved security analysis workflows

## [1.1.0] - 2025-09-03

### Added
- Privacy policy document
- API response caching system
- Named constants for magic numbers
- Service layer architecture (ChatService, SessionService)
- Configuration validation
- Custom HarperError enum
- Unit tests (21 total)
- Performance optimizations with caching
- Security enhancements in cryptographic utilities
- CodeQL security analysis workflow

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
