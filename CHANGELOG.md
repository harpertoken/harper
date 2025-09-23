# Changelog

## [Unreleased]

### Added
- Comprehensive Docker support with multi-stage builds, docker-compose, and CI validation
- Non-root user execution in Docker for improved security
- Optimized Docker build caching for faster development builds
- Cross-platform Docker volume mount examples in documentation

### Changed
- Updated minimum Rust version to 1.82.0+ across all configurations
- Pinned Rust toolchain to 1.82.0 for consistent development environment
- Fixed database path in Docker for proper data persistence
- Pinned cargo-deny to 0.17.0 for CI compatibility

### Fixed
- Docker build failures due to outdated Rust version

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
