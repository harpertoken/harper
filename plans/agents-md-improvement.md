# AGENTS.md Improvement Plan

This plan treats `AGENTS.md` support as a product feature, not just prompt text.

## Goal

Make Harper resolve, merge, and apply `AGENTS.md` files in a scoped way similar to stronger coding agents.

## Problems In The Current State

- Only the repo-root `AGENTS.md` is effectively appended into prompt context
- There is no directory-scope resolution
- There is no nested override behavior
- There is no path-based filtering for touched files
- There is no explicit planner/runtime visibility into which `AGENTS.md` files are active

## Desired Behavior

1. Find applicable `AGENTS.md` files from repo root down to the target path
2. Merge them in order
3. Let deeper files override broader files
4. Apply instructions only where their scope is valid
5. Surface resolved instructions to the planner and UI

## Implementation Outline

1. Define exact scope semantics
2. Build a resolver for parent-directory lookup
3. Add path-aware instruction filtering
4. Add precedence merging for nested files
5. Feed resolved instructions into prompt/planner state
6. Surface active `AGENTS.md` sources in the UI
7. Add tests for precedence, scope, and filtering

## Planner Payload

See `plans/agents-md-improvement.json`.
