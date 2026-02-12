# Codex Short Signals

If the user says "check" or "check status", run:
- `git status --short`
- `git diff --stat`

If the user says "diff", show `git diff` (tracked files only).
If the user says "whole diff" or "full diff", show `git diff` and include untracked new files by catting them.

If the request is ambiguous, default to the smallest reasonable action instead of asking follow-ups.
