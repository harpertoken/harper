# Routing Improvement Plan

This plan treats routing as a product capability, not a growing pile of one-off heuristics.

## Goal

Make Harper's routing more reliable, more explainable, and more meaningfully differentiated across `auto`, `grounded`, and `deterministic`.

## Problems In The Current State

- Deterministic routing is still rule-based and phrase-sensitive
- `grounded` only recently became meaningfully different from `auto`
- Overlapping intents still rely on code-order precedence instead of explicit ranking
- Multi-intent prompts are not decomposed into bounded deterministic steps
- Error, compiler, and stack-trace prompts are only handled indirectly through generic codebase search phrasing
- Symbol extraction works for common cases but still lacks full ambiguity handling

## Desired Behavior

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

## Recommended Order

1. Strengthen grounded deterministic routing
2. Expand deterministic inspection/search coverage
3. Add ranked intent competition
4. Add compiler/error/trace-aware routing
5. Add bounded multi-intent decomposition
6. Add routing observability and regression coverage

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

See `plans/routing-improvement.json`.
