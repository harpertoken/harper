# Distribution and Self-Update Plan

This plan treats install and update behavior as a product capability, not a release afterthought.

## Current State

- Harper now has:
  - `harper version`
  - `harper self-update --check`
  - `harper self-update`
- Direct installs can now:
  - fetch a release manifest
  - compare versions
  - download artifacts
  - verify checksums
  - replace the installed binary
- Release packaging now emits:
  - checksummed archives
  - detached signatures
  - `release-manifest.json`
  - Linux x86_64
  - Linux aarch64
  - Windows x86_64
  - macOS x86_64
  - macOS aarch64 targets
- Install source can now be:
  - inferred from the executable path
  - persisted under `~/.harper/update/install-source.json`

## Desired Behavior

1. Publish canonical release artifacts for supported platforms
2. Keep package-manager distribution aligned with those artifacts
3. Add a Harper-native `self-update` command for direct installs
4. Detect or record install source so managed installs are not mutated incorrectly
5. Give users a clean version and update workflow:
   - `harper version`
   - `harper self-update --check`
   - `harper self-update`

## Implementation Outline

Completed:
1. Define release artifact layout and manifest schema
2. Publish versioned binaries plus checksums
3. Add install-source metadata or inference
4. Implement `self-update --check`
5. Implement direct-install binary replacement with checksum verification
6. Delegate package-managed installs to the correct external command
7. Add detached signature verification for published artifacts
8. Add TUI update visibility and on-demand refresh paths

Remaining:
9. Add install-time metadata capture beyond runtime inference
10. Add key-rotation and signing-ops documentation for maintainers

## Recommended Order

1. Add install-time metadata capture if package managers can stamp it cleanly
2. Document key rotation and signing-secret maintenance
3. Keep release signing and manifest publishing aligned

## Validation Focus

- Manifest parsing and platform selection tests
- Version comparison and update-available tests
- Direct-install update flow tests with checksum and signature verification
- Package-manager detection tests
- Install-source metadata persistence tests
- Manual release-path verification for:
  - direct binary install
  - Homebrew-managed install
- Narrow validation first:
  - `cargo test -p harper-core`
  - `cargo check`

## Planner Payload

See `plans/distribution-and-self-update.json` for the matching `update_plan` seed.
