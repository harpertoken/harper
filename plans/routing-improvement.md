# Routing Improvement Plan

This plan treats routing as a product capability, not a growing pile of one-off heuristics.

## Status

This workstream is partially complete. The core strategy split, broader deterministic coverage, follow-up command resolution, shell-visible routing diagnostics, and graceful no-backend fallback are now in place.

## Problems In The Current State

- Deterministic routing is still rule-based and phrase-sensitive in edge cases
- Overlapping intents still rely too much on precedence and heuristics instead of explicit ranking
- Multi-intent prompts are not decomposed into bounded deterministic steps
- Error, compiler, and stack-trace prompts are still not first-class routing inputs
- Search quality is still ranking-based, not true semantic xref
- Symbol extraction is stronger, but ambiguity handling is still incomplete

## What Is Already Done

1. `grounded` now prefers deterministic routing when the request is clearly routable
2. Deterministic coverage for common repo inspection, follow-up command resolution, and codebase tracing prompts is broader
3. Shell-visible routing observability exists through `harper-batch` and `debug_turn_summary(...)`
4. Strategy behavior is meaningfully differentiated across `deterministic`, `grounded`, `auto`, and `model`
5. No-backend model failures now degrade cleanly when deterministic fallback exists, and return clear assistant messaging when it does not

## Desired Remaining Behavior

1. Let `grounded` prefer deterministic routing when the request is clearly routable
2. Expand deterministic coverage for common repo inspection and codebase tracing prompts
3. Rank overlapping intents instead of relying only on first-match order
4. Surface ambiguity explicitly when more than one deterministic interpretation is plausible
5. Decompose safe multi-intent prompts into bounded deterministic steps
6. Recognize compiler, panic, and stack-trace style prompts as first-class routing inputs
7. Keep routing behavior observable enough to debug and tune

## Implementation Outline

1. Strengthen `grounded` deterministic-first behavior
2. Expand deterministic inspection/search phrasing
3. Add ranked intent competition for overlapping matches
4. Add ambiguity-aware fallback behavior
5. Add bounded multi-intent decomposition
6. Add compiler/error/trace-specific extraction
7. Add routing observability and regression coverage

## Recommended Remaining Order

1. Add ranked intent competition
2. Add compiler/error/trace-aware routing
3. Add bounded multi-intent decomposition
4. Expand routing observability and regression coverage

## Validation Focus

- Deterministic intent unit tests in `lib/harper-core/src/agent/intent.rs`
- Strategy-behavior tests in `lib/harper-core/src/agent/chat.rs`
- Prompt regressions for:
  - changed-files vs overview
  - read-file vs search
  - symbol extraction vs context symbols
  - compiler/error/trace prompts
- Narrow validation first:
  - `cargo test -p harper-core route -- --nocapture`
  - `cargo check -p harper-core`

## Planner Payload

See `plans/routing-improvement.json`, which now marks completed phases and leaves only the remaining routing work pending.
