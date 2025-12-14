# Harper Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Table of Contents

- [Unreleased](#unreleased)
- [0.2.1](#021---2025-12-15)
- [0.2.0](#020---2025-12-14)
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

## What's Coming Next

Still working on some cool stuff:
- A web interface so you can chat from your browser
- Plugin system for custom tools and integrations
- Support for multiple languages
- Better analytics on your chat sessions
- Maybe cloud deployment options

---

## [0.2.1] - 2025-12-15

### Added
- Full Terminal UI: Replaced the basic text-driven interface with a structured, Ratatui-based terminal UI
- Real Chat Experience: Live, streaming conversations with Gemini that feel natural and continuous
- Session Management: Browse, load, and resume previous conversations with proper persistence
- Tool Integration in UI: Tool execution (shell, git, file operations) is now visible and controllable from the interface
- Web Search Toggle: Added an interactive toggle to enable or disable web search during conversations
- UI-Oriented Architecture: Introduced clearer separation between rendering, events, and application state
- Keyboard-driven navigation and menus
- Live chat with streaming responses
- Session browser and history management
- Status indicators and shortcuts for power users

### Changed
- Connected the refactored backend (tools, models, storage) to a real interactive frontend
- Added version tracking via a VERSION file
- Cleaned up warnings and UI-related rendering issues introduced during integration

---

## [0.2.0] - 2025-12-14

### Added
- Modular, MCP-style architecture support
- Structured tool-use workflow
- Robust argument parsing with shared parsing utilities
- Basic in-memory todo management for agent state
- Dedicated model configuration files
- Composite GitHub Actions for CI reuse and clarity

### Changed
- Refactored and restructured the codebase
- Standardized Rust toolchain handling across CI, Docker, and local builds
- Updated Gemini to the `2.5-flash` model
- Switched to direct Rust setup where appropriate
- Updated CodeQL action to v4

### Fixed
- CI failures related to cargo test targets and e2e configuration
- Dockerfile test target names and Rust versions
- Compatibility issues by downgrading `turul-mcp-client` to `0.1.1`
- Updated dependencies affected by yanked crates (`futures`, `crossbeam`)
- Improved shell command security validation
- Metadata error handling
- Clippy warnings, dead code warnings, and formatting issues

---

## [0.1.8-beta.1] - 2025-11-09

Quick bug fixes and improvements to the release process. Made the automated releases more reliable by fixing some issues with GitHub's release API. Also cleaned up some testing configurations and improved the documentation.

---

## [0.1.8-beta] - 2025-11-09

Fixed a problem with the release workflow where it couldn't overwrite existing releases. Now it properly handles immutable releases by deleting and recreating them when needed.

---

## [0.1.7] - 2025-11-09

Spent a lot of time rewriting all the documentation from scratch. Made it much more professional and comprehensive. Also strengthened the code quality checks with better pre-commit hooks. Fixed a compilation issue in the integration tests that was causing CI failures.

---

## [0.1.6] - 2025-11-08

Big infrastructure improvements! Added automated Docker image publishing to releases, which makes deployment much easier. Completely rewrote the documentation with proper guides for everything. Enhanced security with better policies and vulnerability reporting.

Also improved the development environment with better dev containers, added comprehensive pre-commit hooks, upgraded the Rust toolchain, and made the CI/CD pipeline much more robust. Fixed some compilation warnings and cleaned up the codebase.

Did a full security audit and verified all the encryption implementations are solid.

---

## [0.1.5] - 2025-09-20

Added a `--version` command so users can check which version they have. Built a comprehensive test suite that you can run with `./harpertest`. Started writing proper documentation and usage examples.

Improved error messages to be more helpful, optimized database performance, reduced memory usage for long conversations, and fixed some crashes that happened occasionally.

---

## [0.1.4] - 2025-09-15

Small but important fixes: improved database connection stability, fixed some UI rendering issues, and updated dependencies to patch security vulnerabilities.

---

## [0.1.3] - 2025-09-14

Set up the full CI/CD pipeline with GitHub Actions for automated testing and releases. Added CodeQL for security scanning. Enhanced support for multiple AI providers.

Improved the overall architecture to be more modular and maintainable. Made error handling much more robust. Fixed compilation issues on different platforms and resolved some API authentication problems.

---

## [0.1.2] - 2025-09-03

Added DevSkim for automated security analysis and expanded the CI test coverage. Optimized API response handling and improved connection stability. Fixed some security vulnerabilities that were reported.

---

## [0.1.1] - 2025-09-03

Added response caching to improve performance. Expanded the test coverage significantly. Improved the CLI interface and made configuration more flexible. Fixed application crashes and compatibility issues across different operating systems.

---

## [0.1.0] - 2025-08-26

The first release! Harper was born with support for multiple AI providers (OpenAI, Sambanova, Gemini), an interactive CLI interface, persistent conversation storage with encrypted SQLite database, safe command execution, and web search capabilities.

It was a working AI assistant with cross-platform support, but the interface was pretty basic. This was the foundation that everything else built on top of.

---

## How We Write These Updates

We adhere to the [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) format for structured, scannable entries. Each version includes Added, Changed, Fixed, and Removed sections as applicable.

When we make changes that users should know about, we add them to the "Unreleased" section above. The changelog gets automatically updated when we create a new release.

---

**Want the latest updates? Check the [GitHub Releases](https://github.com/harpertoken/harper/releases) page for downloadable versions.**
