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

# Harper Changelog

Hey there! This is where we keep track of all the changes and improvements to Harper. Think of it as our development diary - what we built, what we fixed, and what we're planning next.

## Table of Contents

- [Unreleased](#unreleased)
- [0.3.4](#034---2025-12-24)
- [0.3.3](#033---2025-12-20)
- [0.3.2](#032---2025-12-16)
- [0.3.1](#031---2025-12-16)
- [0.3.0](#030---2025-12-15)
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
- Enhanced plugin ecosystem with more tools and integrations
- Support for multiple languages
- Better analytics on your chat sessions
- Maybe cloud deployment options

---

## [Unreleased]

This release fixes critical TUI stability issues and enhances file reference functionality with proper tab completion support.

### What We Fixed

* **File Reference Processing**: Fixed `@file` syntax to properly convert to `[READ_FILE file]` commands for AI processing, enabling seamless file reading capabilities.
* **Tab Completion Cycling**: Resolved tab completion that only showed first result - now properly cycles through all matching files and directories.
* **Hidden File Access**: Added support for `@.` syntax to access hidden files and directories (`.cargo`, `.git`, etc.) while keeping them filtered from regular completion for cleaner UX.
* **TUI Stability**: Fixed critical panic in chat interface when accessing empty message lists by adding proper bounds checking and enum field management.
* **Memory Safety**: Resolved field sharing issues between completion indexing and scroll offset that caused crashes.

### What Changed

* Enhanced file reference preprocessing in `ChatService` to handle `@file` syntax correctly.
* Completely rewrote tab completion logic in `events.rs` with intelligent cycling and prefix tracking.
* Added hidden file filtering with explicit access via `@.` syntax.
* Expanded `AppState::Chat` enum with separate fields for completion tracking and scrolling.
* Added test coverage for file reference preprocessing.

### Technical Improvements

* **Robust Tab Completion**: Smart detection of completion vs cycling states prevents UI freezes.
* **Path Resolution**: Enhanced path handling for edge cases like `.` and hidden files.
* **User Experience**: Clean separation of regular vs hidden files with intuitive access patterns.
* **Code Quality**: Added unit test for file reference preprocessing.

### Performance & Code Quality Enhancements

* **Todo Clear Optimization**: Modified `clear_todos()` to return deleted row count directly from SQL execution, eliminating unnecessary SELECT query and reducing database round trips.
* **Todo Remove Optimization**: Replaced load_all + index lookup with direct SQL LIMIT/OFFSET query to avoid race conditions and improve performance by fetching only the specific todo.
* **Database Query Simplification**: Replaced manual iteration with `collect()` in `load_todos()` for more idiomatic Rust code and better performance.
* **Tool Execution Unification**: Combined duplicate `execute_sync_tool` and `execute_sync_tool_with_conn` functions into single generic implementation, eliminating code duplication.
* **String Building Optimization**: Replaced manual loop with iterator `map().collect().join()` pattern for todo list formatting, improving performance and code clarity.

---

## [0.3.4] - 2025-12-24

This release expands Harper's tool ecosystem with advanced integrations for software development workflows, including GitHub operations, code analysis, database queries, API testing, and image processing.

### What We Built

* **GitHub Integration Tools**: Added `create_issue` and `create_pr` functions for direct GitHub repository management via bracket commands `[GITHUB_ISSUE title body]` and `[GITHUB_PR title body branch]`.
* **Code Analysis Tool**: Implemented `analyze_code` for basic complexity metrics including line counts, function/struct/enum counts, and estimated complexity scores via `[CODE_ANALYZE path]`.
* **Database Query Tool**: Added `run_query` for safe, read-only SELECT operations on SQLite databases using `[DB_QUERY db_path query]`.
* **API Testing Tool**: Introduced `test_api` for HTTP request testing with configurable methods, headers, and bodies via `[API_TEST method url headers body]`.
* **Image Processing Tools**: Added `get_image_info` for image metadata and `resize_image` for dimension changes using the image crate.

### What Changed

* Extended tool system in `src/tools/` with new modules: `github.rs`, `code_analysis.rs`, `db.rs`, `api.rs`, `image.rs`.
* Added new bracket command prefixes and constants in `src/core/constants.rs`.
* Integrated async API testing with reqwest and image processing with the image crate.
* Enhanced user approval workflows for all new tools with descriptive prompts.
* Updated ToolService to handle async and sync tool executions appropriately.
* Refactored ToolService to accept database connection parameter for better integration with SQLite operations.

### Technical Improvements

* **Extensibility**: Modular tool architecture allows easy addition of new capabilities.
* **Security**: All tools include user consent checks and safe execution patterns.
* **Performance**: Efficient implementations with minimal dependencies.
* **User Experience**: Consistent command syntax and clear feedback for all operations.
* **CI/CD Cleanup**: Renamed GitHub workflows and actions to lowercase minimal names (ci, build, draft, pr, release) and made job names concise (build, e2e, publish, audit, coverage).
* **Testing**: Added unit tests for tool parsing functions to ensure reliability of new agent capabilities.

In short, **0.3.4 transforms Harper into a comprehensive development assistant with integrated tools for the full software engineering lifecycle**.

---

## [0.3.3] - 2025-12-20

This release improves the chat user experience and fixes CI issues related to security auditing.

### What We Built

* **Chat Message Scrolling**: Implemented scroll functionality in the TUI to navigate through chat history using next/previous controls, replacing TODO placeholders with working scroll offset logic.

### What Changed

* Simplified scroll offset handling in `TuiApp` by removing redundant empty message checks, as the bounds calculation handles it correctly.
* Updated `draw_chat` to display messages starting from the current scroll offset.
* Temporarily disabled cargo-audit and cargo-deny in CI due to CVSS 4.0 parsing issues in the advisory database.
* Updated cargo-audit version to 0.21.0 for better compatibility.
* Made validate.sh executable.

### Technical Improvements

* **User Experience**: Chat interface now supports scrolling through message history for better navigation in long conversations.
* **Code Quality**: Simplified logic reduces complexity and potential bugs in scroll handling.
* **CI Stability**: Disabled problematic security audit steps to prevent pipeline failures until tools support CVSS 4.0.

In short, **0.3.3 enhances chat usability and ensures reliable CI operation**.

---

## [0.3.2] - 2025-12-16

This release introduces syntax highlighting for code blocks in chat messages and establishes a plugin architecture for future extensibility.

### What We Built

* **Syntax Highlighting Plugin**: Added support for syntax highlighting code blocks in chat using `syntect`, with automatic language detection for Rust, Python, JavaScript, and more.
* **Plugin Architecture**: Refactored codebase to support plugins in `src/plugins/`, making it easier to add new features like code formatting or linting tools.
* **Enhanced UI Parsing**: Improved message parsing in TUI to handle code blocks with proper highlighting and fallback for unclosed blocks.

### What Changed

* Moved syntax highlighting logic to `src/plugins/syntax/` for better organization.
* Updated UI widgets to integrate with the plugin system.
* Added comprehensive unit and integration tests for syntax highlighting functionality.

### Technical Improvements

* **Modularity**: Established plugin structure for cleaner feature separation and easier maintenance.
* **Code Quality**: Added tests covering parsing, highlighting, and edge cases; all pre-commit checks pass.
* **User Experience**: Chat messages with code blocks (```language\ncode\n```) now display with proper syntax coloring in the TUI.

In short, **0.3.2 makes code discussions in Harper more readable and sets up the foundation for a rich plugin ecosystem**.

---



## [0.3.1] - 2025-12-16

This release significantly enhances Harper's capabilities by integrating advanced features from leading AI CLIs and ensures comprehensive licensing compliance across the entire codebase.

### What We Built

* **TUI Themes**: Added configurable color schemes (default, dark, light) for better user experience.
* **Custom Commands**: Implemented user-defined slash commands via config, extending Harper's command system with extensible functionality.
* **Exec Policy**: Added command execution control with allowed/blocked lists for safer shell operations.
* **Enhanced Configuration**: Expanded config options for tools permissions, custom commands, and execution policies to provide granular control.
* **Comprehensive Licensing**: Added SPDX-compliant Apache 2.0 license headers to all source files, documentation, configs, and scripts for full license compliance.

### What Changed

* Updated configuration structure with new sections for UI themes, tools, exec_policy, and custom_commands.
* Enhanced shell execution with policy-based approval system (auto-allow for whitelisted commands, prompt for others).
* Improved TUI with theme-aware colors and styles for better visual consistency.
* Added extensible command system allowing users to define custom slash commands.
* Standardized license headers across all file types (Rust, Markdown, YAML, TOML, scripts) using appropriate comment formats.

### Technical Improvements

* **Security**: Implemented exec policy to prevent unauthorized command execution, with configurable allow/block lists.
* **User Experience**: Added theme support and custom commands for more personalized and powerful interactions.
* **Code Quality**: Added comprehensive pre-commit hooks and ensured all code passes linting and formatting.
* **Compliance**: Full SPDX licensing ensures legal compliance and proper attribution across the entire codebase.
* **Configurability**: Enhanced TOML-based configuration system for advanced customization options.

In short, **0.3.1 makes Harper more secure, customizable, and professionally licensed while integrating cutting-edge features from leading AI CLIs**.

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
