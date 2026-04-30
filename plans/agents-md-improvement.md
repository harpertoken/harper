# AGENTS.md Improvement Plan

This plan treated `AGENTS.md` support as a product feature, not just prompt text.

## Status

This workstream is now effectively complete for the originally scoped goals.

## What Is Implemented

- Directory-scope resolution from repo root to target path
- Nested override behavior with deeper-path precedence
- Path-aware filtering for touched files and targeted tool calls
- Resolved `AGENTS.md` state surfaced into session/UI state
- Visible AGENTS source/status in the TUI

## Original Goal

1. Find applicable `AGENTS.md` files from repo root down to the target path
2. Merge them in order
3. Let deeper files override broader files
4. Apply instructions only where their scope is valid
5. Surface resolved instructions to the planner and UI

## Remaining Gaps

- Persisting a startup preference for AGENTS context on/off in user config
- Broader docs coverage for AGENTS UI behavior and debug flow
- Additional regression coverage only if new AGENTS behavior is introduced

## Completed Plan Payload

See `plans/agents-md-improvement.json`, which now records the completed state of this workstream rather than an active pending seed.
