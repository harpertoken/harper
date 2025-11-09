# Workspace Reset Guide

This guide provides commands to completely reset your Harper workspace to a clean state.

## Reset Commands

<details>
<summary>Revert All Changes</summary>

```bash
git reset --hard HEAD
git clean -fd
```
</details>

<details>
<summary>Remove All Untracked Files</summary>

```bash
git clean -fx
```
</details>

<details>
<summary>Update Submodules</summary>

```bash
git submodule update --init --recursive
```
</details>

<details>
<summary>Clean Build Artifacts</summary>

```bash
cargo clean
```
</details>

<details>
<summary>Rebuild Project</summary>

```bash
cargo build
```
</details>

## Important Notes

- `git clean -fd` removes untracked files and directories
- `git clean -fx` also removes ignored files (use with caution)
- ⚠️ **Warning**: These commands permanently delete uncommitted changes
- Always commit or stash important changes before running reset commands

For more information, see the main [README](README.md).
