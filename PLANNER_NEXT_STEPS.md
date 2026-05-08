# Harper Native Shell

This file tracks the native shell interaction model.

The goal is to give Harper a first-class command surface for Harper concepts. Native shell commands make common Harper actions explicit, scriptable, and consistent across the TUI and batch mode.

This is not a replacement for `bash`, `zsh`, PowerShell, or Harper's existing controlled command runner. Harper should continue to delegate operating-system commands through the existing safe execution path.

## Product Direction

When users run Harper, the input surface should behave like a Harper command shell:

```text
harper> ask "inspect the update flow"
harper> plan show
harper> plan done 2
harper> session list
harper> session open 3
harper> history show 3
harper> auth status
harper> update check
harper> config show
harper> run cargo test -p harper-core
```

The shell should support Harper-native commands directly and reserve arbitrary process execution for an explicit `run` command.

## Scope

The native shell covers Harper's own concepts first:

- chat and model interaction
- planner state
- sessions and history
- configuration
- authentication
- updates
- controlled command execution
- diagnostics/status

The same command parser is reused from the TUI and batch mode.

## Commands

The first typed command set is:

```text
ask "question or instruction"
plan show
plan list
plan add "Inspect current planner state"
plan done 1
plan start 2
plan block 3 "Waiting on CI"
plan clear
session list
session ls
session open 3
session show 3
history show
history list
history show 3
auth status
auth login
auth login github
auth logout
update check
update apply
config show
config set key value
status
run cargo test -p harper-core
help
```

These commands map to existing Harper services wherever possible. Avoid adding parallel state or duplicate business logic.

## Implementation

The first shell slice is implemented:

- shared typed parser and dispatcher
- TUI command routing
- batch command routing
- explicit `run ...` delegation through the existing safe command runner
- planner commands for show, add, start, done, block, and clear
- service commands for session list, session show, session open, history show, auth status, auth logout in the TUI, config show, config set for execution policy fields, status, help, update check, and explicit update apply guidance
- safe aliases for `plan list`, `plan ls`, `session ls`, and `history list`
- focused parser and planner storage tests
- documented release boundary: Harper-native commands are not a Unix-compatible shell

## Non-Goals

- Do not implement a Unix-compatible shell.
- Do not support pipes, redirects, aliases, glob expansion, job control, or process substitution.
- Do not make arbitrary process execution implicit.
- Do not bypass Harper's existing command safety and approval model.
- Do not replace `update_plan`; the model should still use it for multi-step work.
- Do not create duplicate storage models for sessions, plans, config, or auth.

## Completed `update_plan` Shape

```json
{
  "explanation": "Implemented the first Harper-native shell slice.",
  "items": [
    { "step": "Define native shell grammar", "status": "completed" },
    { "step": "Add typed command parser", "status": "completed" },
    { "step": "Add command dispatcher", "status": "completed" },
    { "step": "Integrate shell with TUI input", "status": "completed" },
    { "step": "Integrate shell with batch mode", "status": "completed" },
    { "step": "Add planner shell actions", "status": "completed" },
    { "step": "Add core service actions", "status": "completed" },
    { "step": "Add native shell tests", "status": "completed" }
  ]
}
```

## Release Note Boundary

Add release notes only when this slice ships in a release. The release note should describe the feature as Harper-native command handling, not a complete Unix shell.
