# TUI Routing Smoke Test

Use this checklist to verify Harper’s repo-aware routing, grounded tool use, and TUI rendering after changes to chat routing, codebase helpers, file tools, or transcript rendering.

## Prerequisites

- Build from the current workspace.
- Restart Harper before testing.
- Run the prompts in a fresh TUI session when possible.

## Important Note

- Restart Harper before testing.
- Otherwise you may still be exercising an older running process instead of the current build.

## Prompts

### Repo identity

- `can you check which repo we are working on`
- `which branch am i on`
- `which directory are we in`

### Header and strategy UI

- Type `/`
- Press `Tab` to cycle slash completions
- `/strategy`
- `/strategy auto`
- `/strategy grounded`
- `/strategy deterministic`
- `/strategy model`
- Open `Settings -> Execution Policy`
- Change visible header widgets
- Save settings
- Restart Harper

### Direct file read

- `Read Cargo.toml and tell me the package name.`

### Codebase overview

- `tell me the codebase`
- `give me a quick overview of this repo`
- `what are we working on here`

### Open-ended authoring

- `refactor the planner flow in this repo`
- `change the retry rendering behavior in the tui`
- `modify the subsystem that handles followup retry display`

### Codebase search / render path

- `Find where retry metadata is rendered in this repo.`
- `where does the planner followup retry metadata get rendered`
- `what file renders the retry count in the tui`

### Git / change inspection

- `Run git status and summarize it.`
- `show git diff`
- `check the code changes`

### Simple command routing

- `run the clear command`
- `run pwd`
- `run ls`
- `run harper`
- `run fmt`
- `run check`
- `run tests`

### Simple file creation

- `create a hello world txt file`
- `create a python hello joy file`

### Explicit file creation

- `create hello.rs with fn main() { println!("Hi"); }`
- `create notes.txt with hello from harper`
- `create script.py with print("Hello from Harper")`

### Explicit file modification

- `modify notes.txt to hello from harper`

### Follow-up creation

Use this exact sequence:

1. `hey can you create a python hello joy file`
2. `please create the file`

## Expected Outcomes

- Repo/branch prompts bypass generic prose.
- The chat header shows the configured widgets only.
- The chat header shows the current working directory when `cwd` is enabled.
- `/strategy` reports the current execution strategy and switching it updates the live chat session.
- Header widget changes made in `Settings -> Execution Policy` persist to `config/local.toml`.
- `Cargo.toml` prompt goes through a real file read.
- Codebase prompts do not answer with generic grep advice.
- Open-ended authoring prompts inspect first and should not jump straight to a bad edit target.
- `git status` runs `git status`, not `git status and summarize it`.
- Create/modify prompts route to real write operations.
- Created file responses show fenced content with syntax highlighting.
- Diff/code outputs render with syntax-aware formatting when detectable.

## Pass / Fail Checklist

- [ ] Repo identity prompts return grounded repo/branch data.
- [ ] Current working directory prompt returns the concrete workspace path.
- [ ] Typing `/` opens slash completion suggestions.
- [ ] `Tab` cycles slash completions.
- [ ] `/strategy` shows the current strategy and switching it changes the live session.
- [ ] Header widget changes persist after save and restart.
- [ ] Chat header only shows the widgets currently enabled.
- [ ] Direct file read uses the real workspace file.
- [ ] Codebase overview uses grounded workspace context.
- [ ] Codebase search returns relevant repo files, not generic prose.
- [ ] Open-ended authoring builds context/plan before first edit.
- [ ] Git status/diff route to the correct underlying tool/command.
- [ ] `clear`, `pwd`, and `ls` route as commands, not literal prose.
- [ ] `run harper`, `run fmt`, `run check`, and `run tests` route as execution-layer commands.
- [ ] Simple file creation writes real files in the workspace.
- [ ] Explicit file creation writes the requested file/content.
- [ ] Explicit file modification overwrites the requested file content.
- [ ] Follow-up `please create the file` reuses the prior filename/content.
- [ ] Created code/text renders back as fenced, highlighted output.

## When To Run This

Run this checklist after changes to:

- `lib/harper-core/src/agent/intent.rs`
- `lib/harper-core/src/agent/chat.rs`
- `lib/harper-core/src/tools/filesystem.rs`
- `lib/harper-core/src/tools/codebase_investigator.rs`
- `lib/harper-core/src/tools/shell.rs`
- `lib/harper-core/src/tools/plan.rs`
- `lib/harper-core/src/core/plan.rs`
- `lib/harper-core/src/runtime/config.rs`
- `lib/harper-ui/src/interfaces/ui/app.rs`
- `lib/harper-ui/src/interfaces/ui/events.rs`
- `lib/harper-ui/src/interfaces/ui/settings.rs`
- `lib/harper-ui/src/interfaces/ui/tui.rs`
- `lib/harper-ui/src/interfaces/ui/widgets.rs`
- `lib/harper-ui/src/plugins/syntax/mod.rs`
