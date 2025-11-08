# Changelog

## [Unreleased]

### Added
- Dev container configuration for consistent development environment
- Pre-commit and commit-msg git hooks for code quality enforcement
- Clippy linting in pre-commit hook

### Changed
- Updated Rust toolchain and MSRV to 1.85.0 for ICU and cargo-audit compatibility
- Enhanced CI with updated MSRV testing

### Fixed
- Compilation error in chat_service.rs due to incorrect slice mutation
- Clippy warning for manual div_ceil implementation in performance tests
- Removed unused imports and comments from source files

## [0.1.5] - 2025-09-20
- Version flag, test suites, documentation updates

## [0.1.4] - 2025-09-15
- Minor fixes

## [0.1.3] - 2025-09-14
- CI fixes, security updates, new features

## [0.1.2] - 2025-09-03
- Security scanning, CI enhancements

## [0.1.1] - 2025-09-03
- Caching, tests, security improvements

## [0.1.0] - 2025-08-26
- Initial release with AI providers, CLI, sessions
