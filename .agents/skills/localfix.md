---
name: localfix
description: Fix local Bazel and Cargo-Bazel issues by cleaning caches and repinning dependencies
---

# Local Fix Skill

This skill provides commands to fix local Bazel and Cargo-Bazel issues by cleaning caches and repinning dependencies.

## Usage

Run the following commands to fix common local build issues:

```bash
bazel shutdown
rm -rf ~/.cache/bazel ~/.bazel
rm -f cargo-bazel-lock.json
CARGO_BAZEL_REPIN=true bazel build :harper_bin
```

## Description

- `bazel shutdown`: Shuts down the Bazel server
- `rm -rf ~/.cache/bazel ~/.bazel`: Removes Bazel cache directories
- `rm -f cargo-bazel-lock.json`: Removes the Cargo-Bazel lock file
- `CARGO_BAZEL_REPIN=true bazel build :harper_bin`: Repins Cargo dependencies and builds the main binary

## Important Notes

- **Time Warning**: Initial dependency repinning takes 10-15 minutes (recompiles ~200 Rust crates)
- **Subsequent Builds**: Run `bazel build :harper_bin` (no CARGO_BAZEL_REPIN) for fast ~30-60 second builds
- **Expected Behavior**: Build may show compilation progress for several minutes before completing

This sequence clears corrupted caches and forces dependency repinning, resolving issues with stale or inconsistent build artifacts.
