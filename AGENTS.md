# Harper Agent Rules

These rules apply to the whole repository unless a deeper `AGENTS.md` overrides them.

## Priority

Follow instructions in this order:
1. system / developer / user instructions
2. nearest applicable `AGENTS.md`
3. broader parent `AGENTS.md`

Do not invent capabilities that Harper does not implement.

## Working Style

- Be direct and technical.
- Explain the next concrete action before running commands or changing files.
- Prefer small, reversible changes over broad refactors.
- Fix root causes, not surface symptoms.
- Do not touch unrelated files just because they are nearby.

## Repository Expectations

- This is a Rust workspace with `harper-core` and `harper-ui`.
- Keep changes minimal and consistent with existing code style.
- Prefer extending existing services, models, and state flows over adding parallel systems.
- When a feature is visible in both core and TUI, keep data flow explicit: storage -> service -> worker/event -> UI state -> widget.

## AGENTS.md Handling

When changing AGENTS support itself:

- Treat `AGENTS.md` as scoped repository policy, not generic prompt text.
- Preserve ancestor-based resolution and deeper-path precedence.
- Prefer structured state and explicit propagation over prompt-only behavior.
- When a tool call targets files, use the actual target paths for rule resolution.
- Keep UI/API visibility aligned with the resolved source set shown to the model.

Override syntax supported by Harper's structured merge layer:

- `Delete: Ask first` defines or overrides the `Delete` rule
- `replace: Delete: Use git rm instead` explicitly replaces the `Delete` rule
- `remove: Delete` removes the inherited `Delete` rule
- `! Delete` is shorthand for removing the inherited `Delete` rule

## Tool Mapping

Map user intent to Harper capabilities precisely:

- inspect/read file -> `read_file`
- edit existing file -> `search_replace`
- create file -> `write_file`
- run command -> `run_command`
- inspect changes -> `git_diff`, `git_status`, `list_changed_files`
- investigate code flow -> `codebase_investigator`
- update execution plan -> `update_plan`

Do not claim a dedicated tool exists if Harper only supports the behavior through another path.

## File Safety

- Stay inside the repository unless the user explicitly asks otherwise.
- Reject traversal-style paths like `..` when implementing file tools.
- Treat secrets, `.env` files, SSH material, and system paths as sensitive.
- Do not delete files unless the user explicitly asks for it.
- Prefer `git rm` over raw `rm` when the intent is tracked deletion.

## Command Safety

- Show the exact command before running it.
- Request approval for destructive, networked, or history-rewriting commands.
- Do not hide shell behavior behind vague summaries.
- Prefer focused validation commands first, then broader test/build commands.

## Coding Rules

- Use `Result` / `Option` cleanly; avoid `unwrap()` in production paths.
- Keep public state models serializable when they cross storage, API, or UI boundaries.
- Reuse existing types before adding new ones.
- Avoid one-off helpers when the logic belongs in an existing service.
- Add tests near the behavior you changed when the codebase already has adjacent tests.

## Validation

- Run the narrowest useful tests first.
- If a compile or test failure is caused by your change, fix it before moving on.
- Do not “solve” unrelated failing tests unless the user asked for that.

## Response Rules

- Report what changed, where it changed, and what you validated.
- Be explicit about remaining limitations.
- If a next step is obvious, suggest one concrete next step.
