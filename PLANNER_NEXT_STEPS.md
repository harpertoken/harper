# Harper Planner Next Steps

This file captures the remaining planner/runtime work after the current planning, job, follow-up, retry, scoped `AGENTS.md`, cross-process delivery, deterministic repo routing, structured codebase helper changes, and persisted authoring-manager workflow.

## Count

There are **3 real next steps** left without changing direction.

## Next Steps

1. **Live command output polish**
   - Refine the existing command output panel and active runtime display.
   - Add clearer truncation controls, richer status summaries, and better failure surfacing.

2. **Semantic authoring depth**
   - Extend the current authoring manager beyond the lightweight semantic graph.
   - Improve Rust symbol/type/trait resolution and richer multi-file edit planning.

3. **Planner follow-up history polish**
   - Extend the current single follow-up state into richer visible history if needed.
   - Make retry/checkpoint review easier without losing the current lightweight UI.

## Recommended Order

1. Live command output polish
2. Semantic authoring depth
3. Planner follow-up history polish

## Suggested Milestones

### Milestone 1: Runtime polish
- Live command output polish

### Milestone 2: Semantic authoring depth
- Improve compiler-backed candidate ownership
- Deepen symbol/reference-aware repo understanding
- Improve multi-file authoring decomposition quality

### Milestone 3: Follow-up history
- Add richer checkpoint/retry review history
- Keep active follow-up state easy to understand

## Harper `update_plan` Seeds

### Seed 1: Runtime polish

```json
{
  "explanation": "Refine planner runtime visibility now that plan/job/follow-up state and cross-process delivery are in place.",
  "items": [
    { "step": "Clarify active command status", "status": "pending" },
    { "step": "Improve output truncation", "status": "pending" },
    { "step": "Surface failures more clearly", "status": "pending" }
  ]
}
```

### Seed 2: Semantic authoring depth

```json
{
  "explanation": "Deepen the authoring helper so repo changes rely on grounded semantic context instead of shallow candidate matching.",
  "items": [
    { "step": "Improve trait and type links", "status": "pending" },
    { "step": "Strengthen multi-file decomposition", "status": "pending" },
    { "step": "Keep authoring reasoning grounded", "status": "pending" }
  ]
}
```

### Seed 3: Follow-up history

```json
{
  "explanation": "Improve visibility of completed checkpoints and recovery actions after the active follow-up model is already in place.",
  "items": [
    { "step": "Design follow-up history view", "status": "pending" },
    { "step": "Store checkpoint history", "status": "pending" },
    { "step": "Render retry/checkpoint review", "status": "pending" }
  ]
}
```

## Single Master Plan

```json
{
  "explanation": "Finish the remaining planner/runtime polish after core orchestration, scoped AGENTS resolution, retry handling, codebase helpers, and authoring-manager workflow are in place.",
  "items": [
    { "step": "Polish live command output", "status": "pending" },
    { "step": "Deepen semantic authoring context", "status": "pending" },
    { "step": "Improve follow-up history UX", "status": "pending" }
  ]
}
```
