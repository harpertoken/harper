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

# Harper AI Agent Guidelines

## User Intent Recognition

Map user requests to the correct tool:

| User Says | Action | Tool |
|----------|--------|------|
| "check/view/read/look at <file>" | Read file content | `read_file` |
| "update/fix/change/edit <file>" | Modify file | `search_replace` |
| "create/new/make <file>" | Write new file | `write_file` |
| "delete/remove <file>" | Remove file (ask first) | `run_command` with `rm` |
| "run/execute <command>" | Run shell command | `run_command` |
| "search/find <pattern>" | Search code | `run_command` with `grep` |
| "list/show <stuff>" | List items | `run_command` or list tool |
| "commit/push changes" | Git operations | `git_*` tools |
| "understand how <x> works" | Investigate code | `codebase_investigator` |
| "what changed?" | Show diffs | `git_diff` or `list_changed_files` |

## File Operations

### Allowed
- Read/write files within current project directory
- Run commands in project scope
- Search code with grep/find

### Forbidden (always ask for confirmation)
- Delete operations: "use git rm instead"
- System file modifications
- Files outside project workspace
- Sensitive files: `/etc/passwd`, `~/.ssh/*`, `.env`, secrets

### Validation
- Reject paths with `..` (directory traversal)
- Reject paths outside current working directory
- Limit file reads to 1MB

## Command Execution

### Require Confirmation
- Any destructive command (rm, mv of critical files)
- Commands that modify git history
- Commands that could break the build
- Network operations

### Safety Rules
- Always show the command before execution
- Explain what the command will do
- Never hide commands from the user

## Codebase Conventions

When working with this Rust codebase:

- **Conventional commits**: `type: message` (feat, fix, docs, refactor, etc.)
- **Error handling**: Use `Result`/`Option`, avoid `unwrap()`
- **PR format**: `[scope] description`
- **Max commit message**: 72 chars first line

## Response Style

- Be concise and direct
- Explain before acting
- Confirm destructive actions
- Show command output when relevant
- Ask for clarification if intent is unclear
