# GitHub Workflows

This directory contains all automation that runs in GitHub Actions for the Harper repository. Use this document as a quick reference when you need to understand what a workflow does, why it exists, or which job to update.

## Quick Reference

| Workflow | File | What it does | Key trigger(s) |
| --- | --- | --- | --- |
| Apply Rulesets | `apply-rulesets.yml` | Applies branch ruleset definitions from `.github/rulesets/*.json` to GitHub. Edit `main-branch-protection.json` to change rules; the workflow syncs them on push. | Push to `main` touching rulesets, manual dispatch |
| Auto Merge | `auto-merge.yml` | Three jobs: `auto-merge` enables GitHub's built-in auto-merge (waits for `CI (ubuntu-latest)` + 1 review); `auto-merge-now` merges immediately via `BYPASS_TOKEN` bypassing ruleset checks; `cancel-auto-merge` disables queued auto-merge when the `auto-merge` label is removed. | `labeled`/`unlabeled`/PR events |
| Bazel CI | `build-bazel.yml` | Builds `:harper_bin` with Bazel (including lockfile repinning fallback). | Push/PR to `main`, Bazel branches |
| Bazel Smoke | `bazel-smoke.yml` | Daily fetch + `bazel test //...` to catch dependency drift outside PRs. | Daily cron, manual dispatch |
| Rust Benchmarks | `benchmarks.yml` | Runs `cargo bench` nightly and stores results as artifacts. | Daily cron, manual dispatch |
| Integration Tests | `integration.yml` | Executes `cargo test -- --ignored` against real services (requires secrets). | PRs touching app code, manual dispatch |
| Package Test | `package-test.yml` | Builds release binaries and packages them for smoke testing. | Tag push (`v*`), manual dispatch |
| Post Auto Merge CI | `post-auto-merge-ci.yml` | Re-runs fmt/clippy/tests on `main` after Auto Merge completes; also locks the merged PR via `gh pr lock`. | Completion of Auto Merge workflow |
| Normalize PR Description | `normalize-pr-description.yml` | Rewrites `## Summary`/`## Testing` bullet-style PR bodies into a single paragraph with backtick-wrapped technical terms. Skips forks and dependabot. | PR opened/edited/ready_for_review |
| Build | `build.yml` | Runs the canonical `cargo fmt`, `cargo clippy`, and test matrix. | Push/PR to `main` |
| CI | `ci.yml` | Lightweight checks (lint, formatting, unit tests) for fast feedback. | Push/PR to `main` |
| CLA | `cla.yml` | Enforces the Contributor License Agreement via comment status. | PR opened/synchronized |
| CodeQL | `codeql.yml` | Performs CodeQL static analysis on Rust (security scanning). | Push/PR to `main`, weekly cron |
| Dependency Review | `dependency-review.yml` | Uses GitHub's dependency-review action on PRs. | PR events |
| Docs | `docs.yml` | Builds documentation/mdbook (fails PR if docs break). | Push/PR touching docs |
| Fix Changelog | `fix-changelog.yml` | Ensures changelog formatting stays consistent. | PRs touching changelog |
| Fix PR Title | `fix-pr-title.yml` | Rewrites PR titles into `[scope] message` format and lowercases the description part. | PR events |
| Label Sync | `label-sync.yml` | Syncs repository labels from `config/labeler.yml`. | Manual/cron |
| Lock Merged PRs | `lock-merged-prs.yml` | Locks PRs after manual merge. Auto Merge path is handled by `post-auto-merge-ci.yml`. | PR closed |
| Nightly | `nightly.yml` | Runs the nightly job (currently the same matrix as `ci.yml` but scheduled). | Nightly cron |
| PR Checks | `pr-checks.yml` | Aggregates status checks required for merge (references other workflows). | PR workflow_call |
| Release | `release.yml` | Builds release artifacts (binaries, packages). | Manual dispatch / tag push |
| Release Please | `release-please.yml` | Uses release-please to cut versions and changelog PRs. | Push to `main` |
| Rust Auto-Fix Bot | `rust-auto-fix.yml` | Applies automated `cargo fmt`/`clippy --fix` patches via bot PRs. | Issue comment / workflow_dispatch |
| Update Rust lockfiles | `update-lockfiles.yml` | Weekly `cargo update` + `bazel sync --only=crates`, opens PR. | Weekly cron + manual dispatch |
| Deploy Website | `website.yml` | Builds and deploys the docs/website bundle to GitHub Pages. | Push to `main` / workflow_dispatch |

> **Tip:** Run `rg -n '^name:' .github/workflows` to see the canonical name shown in the Actions UI.

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

1. **Prefer reusable actions that are actively maintained.** We pin Bazel jobs to `bazel-contrib/setup-bazel@0.15.0` because the old `bazelbuild/setup-bazelisk` repository is archived and GitHub 404s the newer `bazelbuild/setup-bazel` path.
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
