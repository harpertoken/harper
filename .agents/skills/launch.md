<!--
Copyright 2026 harpertoken

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

name: launch
description: Build and launch Harper with checks
entrypoint: |
  # Build and Launch Harper

  See AGENTS.md for full guidelines on file operations, security, and coding conventions.

  ## Pre-flight Checks
  - Run: `cargo check`
  - Run clippy: `cargo clippy -- -D warnings`
  - Check fmt: `cargo fmt -- --check`
  - Run tests: `cargo test`

  ## Build
  - Release: `cargo build --release`
  - Or Bazel: `bazel build //...`

  ## Launch
  - Run binary with appropriate flags

  ## Core Guidelines (see AGENTS.md for details)
  - Read only within project scope
  - Require user consent for writes
  - Never delete; use git operations
  - Use Result/Option over unwrap()
  - Prefer structs/enums over inheritance
  - Use iterator methods (.map, .filter, .fold)