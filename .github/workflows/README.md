# GitHub Workflows

This directory contains all automation that runs in GitHub Actions for the Harper repository. Use this document as a quick reference when you need to understand what a workflow does, why it exists, or which job to update.

## Quick Reference

| Workflow | File | What it does | Key trigger(s) |
| --- | --- | --- | --- |
| Apply Rulesets | `apply-rulesets.yml` | Applies branch ruleset definitions from `.github/rulesets/*.json` to GitHub. Edit `main-branch-protection.json` to change rules; the workflow syncs them on push. | Push to `main` touching rulesets, manual dispatch |
| Auto Merge | `auto-merge.yml` | Three jobs via `libnudget/auto-merge@v1`: `auto-merge` enables GitHub's built-in auto-merge (waits for `CI (ubuntu-latest)` + 1 review); `auto-merge-now` merges immediately via `BYPASS_TOKEN` bypassing ruleset checks; `cancel-auto-merge` disables queued auto-merge when the `auto-merge` label is removed. | `labeled`/`unlabeled`/PR events |
| Bazel CI | `build-bazel.yml` | Builds `:harper_bin` with Bazel on Linux and macOS, plus a scoped Windows smoke build/test for `harper-core` (including lockfile repinning fallback). | Push/PR to `main`, Bazel branches |
| Bazel Smoke | `bazel-smoke.yml` | Daily `bazel test //...` to catch dependency drift outside PRs. | Daily cron, manual dispatch |
| Rust Benchmarks | `benchmarks.yml` | Runs `cargo bench` nightly and stores results as artifacts. | Daily cron, manual dispatch |
| Integration Tests | `integration.yml` | Executes `cargo test -- --include-ignored` against real services (requires secrets). | PRs touching app code, manual dispatch (with environment input) |
| Package Test | `package-test.yml` | Builds and packages release artifacts for Linux x86_64, Linux aarch64, macOS x86_64, macOS aarch64, and Windows x86_64 to smoke-test the updater contract. | Tag push (`v*`), manual dispatch |
| Post Auto Merge CI | `post-auto-merge-ci.yml` | Re-runs fmt/clippy/tests on `main` after Auto Merge completes; also locks the merged PR via `gh pr lock`. | Completion of Auto Merge workflow |
| Normalize PR Description | `normalize-pr-description.yml` | Rewrites `## Summary`/`## Testing` bullet-style PR bodies into a single paragraph with backtick-wrapped technical terms via `libnudget/prune@v1`. Skips forks and dependabot. | PR opened/edited/ready_for_review |
| Build | `build.yml` | Builds Docker image and runs e2e tests; publishes to GHCR/Docker Hub on merge to `main`. | Push/PR to `main` touching docker files, workflow dispatch |
| CI | `ci.yml` | Runs on `main`/`develop`: validation checks, clippy, docs, unit+integration tests, release build, security audit, e2e tests. Also runs a weekly audit and coverage upload. | Push/PR to `main`/`develop`, weekly cron |
| CLA | `cla.yml` | Enforces the Contributor License Agreement via cla-bot. | PR opened/synchronized |
| CodeQL | `codeql.yml` | Performs CodeQL static analysis on Rust (security scanning). | Push/PR to `main`, weekly cron |
| Dependency Review | `dependency-review.yml` | Uses GitHub's dependency-review action on PRs. | PR events |
| Docs | `docs.yml` | Builds mkdocs documentation and deploys to GitHub Pages. | Push/PR touching docs |
| Fix PR Title | `fix-pr-title.yml` | Rewrites PR titles into `[scope] message` format via `libnudget/title@v1`. | PR events on `main` |
| Label Sync | `label-sync.yml` | Syncs repository labels from `config/labeler.yml` via custom script. | Weekly cron, manual dispatch |
| Lock Merged PRs | `lock-merged-prs.yml` | Locks PRs after merge (via `gh pr lock`). Auto Merge path is handled by `post-auto-merge-ci.yml`. | PR closed (merged) |
| Nightly | `nightly.yml` | Uses `libnudget/rust-nightly` reusable workflow: runs tests, builds release, creates prerelease with tag `nightly-{sha}`. Benchmarks disabled for faster runs. | Daily cron (midnight UTC), manual dispatch |
| PR Checks | `pr-checks.yml` | Validates PR metadata and auto-labels via `config/labeler.yml`. | PR workflow_call, push to `main` |
| Release | `release.yml` | Creates release PRs or direct tags for harper-core, harper-ui, harper-firmware, harper-mcp-server, harper-sandbox via `libnudget/release@v1.0.0`, then publishes packaged Harper binaries, checksums, detached signatures, and `release-manifest.json` on tag pushes. This workflow requires `HARPER_UPDATE_SIGNING_KEY_PEM_B64` to sign updater artifacts and checks that the secret matches the repo-shipped public key before publishing. | Push to `main` touching lib dirs, tag push (`v*`), PR merged, manual dispatch |
| Rust Auto-Fix Bot | `rust-auto-fix.yml` | Applies automated `cargo fmt`/`clippy --fix` patches via `libnudget/rust-fix@v1` when `/rust-fix` comment is confirmed with `/confirm`. | Issue comment on PRs |
| Cancel Runs Bot | `cancel-runs.yml` | Cancels in-progress runs when `/cancel-runs` is commented on PRs via `libnudget/cancel@v1`. Also triggers on `cancel-runs` label. | Issue comment on PRs, `cancel-runs` label |
| Update Rust lockfiles | `update-lockfiles.yml` | Runs `cargo update` + `CARGO_BAZEL_REPIN=true bazel build :harper_bin` (repins `cargo-bazel-lock.json`), opens PR. | Weekly cron (Sunday midnight UTC), manual dispatch |
| Deploy Website | `website.yml` | Builds and deploys the website bundle to GitHub Pages. | Push to `main` touching `website/**`, manual dispatch |

> **Tip:** Run `rg -n '^name:' .github/workflows` to see the canonical name shown in the Actions UI.

 > **Naming convention:** Workflow `name:` fields describe *what* the workflow does (`CI`, `Build`, `Rust Benchmarks`). Platform/architecture granularity (`ubuntu-latest`, `macos-latest`, `windows-latest`, `aarch64`, `x86_64`) belongs in job or matrix names inside the workflow e.g. `CI (ubuntu-latest)` not in the top-level workflow name.

## Labels

| Label | Color | Purpose |
| --- | --- | --- |
| `auto-merge` | green | Enables GitHub's built-in auto-merge; waits for `CI (ubuntu-latest)` and 1 approving review before squash-merging. Remove the label to cancel a queued merge. |
| `auto-merge-now` | red | Merges immediately via `BYPASS_TOKEN`, bypassing ruleset status-check and review requirements. Use with caution. |

## Branch Ruleset

The `main` branch is protected by the **Main Branch Protection** ruleset (ID `11608827`). Its definition lives in `.github/rulesets/main-branch-protection.json` and is applied automatically by `apply-rulesets.yml` on every push that touches it. To change a rule, edit the JSON and merge to `main`.

Current rules:
- No deletion or force-push to `main`
- Squash or rebase merge only
- 1 approving review required
- `CI (ubuntu-latest)` must pass
- Bypass: OrganizationAdmin (always)

## Editing Guidelines

1. **Prefer reusable actions that are actively maintained.** We pin Bazel jobs to `bazel-contrib/setup-bazel@0.19.0` because the old `bazelbuild/setup-bazelisk` repository is archived and GitHub 404s the newer `bazelbuild/setup-bazel` path.
2. **Document non-obvious behavior.** If a workflow has unusual permissions, secrets, or environment requirements (for example, the lockfile job needing Bazel cache access), add comments in the YAML.
3. **Test locally when possible.** For shell steps that don't rely on GitHub-specific context, run them via `act` or a local script before committing.
4. **Respect required checks.** `pr-checks.yml` gates merges—update its `workflow_run` dependencies whenever you add/remove a workflow that should block PRs.
5. **Keep triggers minimal.** Avoid running heavy jobs on every push; scope `paths:` or branch filters when applicable.

## Troubleshooting

- **Action not found:** Verify the action path and version exist (e.g., `bazel-contrib/setup-bazel@0.15.0`). GitHub's error usually means the tag or repository is missing.
- **Cache warnings:** Archived actions (such as the old Bazel setup) may emit 400s from the cache API. Migrating to an actively maintained action usually resolves this.
- **`auto-merge-now` fails with ruleset violation:** Ensure `BYPASS_TOKEN` secret is set to a PAT from an org admin account with `repo` scope. The `GITHUB_TOKEN` cannot bypass rulesets.
- **Sandbox permissions (Codex/CI reproductions):** Some local sandbox sessions can mark `.git/refs/heads` with macOS provenance flags, blocking branch creation. If you see "Operation not permitted" writing inside `.git`, create/push branches from a fresh session or your host machine; the issue is environmental, not workflow-related.

Feel free to expand this file with additional details (matrix descriptions, secrets used, etc.) as workflows evolve.

## Last Updated
2026-05-01
