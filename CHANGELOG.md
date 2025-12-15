# Harper Changelog

Hey there! This is where we keep track of all the changes and improvements to Harper. Think of it as our development diary - what we built, what we fixed, and what we're planning next.

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

## [0.3.0] - 2025-12-15

This release enhances Harper's user experience and agent capabilities by integrating advanced features from Mistral Vibe and improving code quality.

### What We Built

* **Advanced Autocompletion**: Added intelligent file path completion using `@` and slash command completion using `/` in the TUI, with Tab cycling through suggestions.
* **Agent Guidelines Integration**: AGENTS.md is now loaded into the system prompt, providing comprehensive Rust coding standards that the AI follows for better code generation.
* **Enhanced Documentation**: Expanded CONTRIBUTING.md with detailed Rust-specific guidelines for contributors, and established AGENTS.md as the single source of truth for coding standards.
* **Improved Code Quality**: Refactored autocompletion logic using `std::path::Path` for robust path handling, eliminated unsafe unwrap calls, and improved type safety throughout.

### What Changed

* Moved AGENTS.md to project root for better accessibility and consistency.
* Strengthened type safety by making session_id non-optional in Chat state.
* Deduplicated documentation to avoid maintenance issues while keeping clear separation of concerns.
* Enhanced pre-commit hooks and validation scripts for better code quality assurance.

### Technical Improvements

* **Path Completion**: Replaced manual string splitting with idiomatic `std::path::Path` operations for better edge case handling.
* **Type Safety**: Eliminated potential panics by enforcing compile-time guarantees for required fields.
* **Documentation Architecture**: Established clear roles - CONTRIBUTING.md for contributors, AGENTS.md for agent behavior.

In short, **0.3.0 makes Harper smarter, safer, and easier to contribute to**.

---

## [0.2.1] - 2025-12-15

This release builds on the architectural foundation laid in 0.2.0 and focuses on user-facing experience and interaction. Harper now feels like a complete application rather than a collection of CLI flows.

### What We Built

* **Full Terminal UI**: Replaced the basic text-driven interface with a structured, Ratatui-based terminal UI.
* **Real Chat Experience**: Live, streaming conversations with Gemini that feel natural and continuous.
* **Session Management**: Browse, load, and resume previous conversations with proper persistence.
* **Tool Integration in UI**: Tool execution (shell, git, file operations) is now visible and controllable from the interface.
* **Web Search Toggle**: Added an interactive toggle to enable or disable web search during conversations.
* **UI-Oriented Architecture**: Introduced clearer separation between rendering, events, and application state.

### What Changed

* Connected the refactored backend (tools, models, storage) to a real interactive frontend.
* Added version tracking via a VERSION file.
* Cleaned up warnings and UI-related rendering issues introduced during integration.

### What Works Now

* Keyboard-driven navigation and menus
* Live chat with streaming responses
* Session browser and history management
* Status indicators and shortcuts for power users

In short, **0.2.1 turns the refactored core into a usable product**.

---

## [0.2.0] - 2025-12-14

This release focused on restructuring Harper's internals, stabilizing CI, and introducing MCP-aligned tool-use workflows. Most changes are architectural and infrastructural, laying the groundwork for future user-facing improvements.

### Architecture & Core

* Refactored and restructured the codebase to support a modular, MCP-style architecture.
* Implemented a structured tool-use workflow.
* Added robust argument parsing with shared parsing utilities.
* Introduced basic in-memory todo management for agent state.
* Moved model configuration into dedicated files.
* Updated Gemini to the `2.5-flash` model.

### CI & Infrastructure

* Added composite GitHub Actions for CI reuse and clarity.
* Standardized Rust toolchain handling across CI, Docker, and local builds.
* Switched to direct Rust setup where appropriate.
* Added missing system dependencies to CI jobs.
* Restored and debugged validation scripts.
* Updated CodeQL action to v4.

### Fixes & Stability

* Fixed CI failures related to cargo test targets and e2e configuration.
* Corrected Dockerfile test target names and Rust versions.
* Downgraded `turul-mcp-client` to `0.1.1` to restore compatibility.
* Updated dependencies affected by yanked crates (`futures`, `crossbeam`).
* Improved shell command security validation.
* Handled metadata errors more gracefully.
* Addressed clippy warnings, dead code warnings, and formatting issues.

### Maintenance

* Updated `Cargo.lock`.
* Addressed review feedback and cleanup items.
* Improved consistency across actions, workflows, and configs.

In short, **0.2.0 is the foundation release** that made 0.2.1 possible.

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

We try to keep this changelog honest and helpful. Each entry explains what we actually did, why we did it, and how it affects users. No corporate jargon - just real talk about the development process.

When we make changes that users should know about, we add them to the "Unreleased" section above. The changelog gets automatically updated when we create a new release.

---

**Want the latest updates? Check the [GitHub Releases](https://github.com/harpertoken/harper/releases) page for downloadable versions.**
