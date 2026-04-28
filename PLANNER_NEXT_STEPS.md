# Harper Planner Next Steps

This file captures the remaining planner/runtime work after the current planning, job, follow-up, retry, scoped `AGENTS.md`, and cross-process delivery changes.

## Count

There are **3 real next steps** left without changing direction.

## Next Steps

1. **Live command output polish**
   - Refine the existing command output panel and active runtime display.
   - Add clearer truncation controls, richer status summaries, and better failure surfacing.

2. **Legacy fallback cleanup**
   - Reduce the remaining heuristic fallback paths where explicit command intent now exists.
   - Prefer explicit retry/sandbox intent end to end over command-shape inference.

3. **Planner follow-up history polish**
   - Extend the current single follow-up state into richer visible history if needed.
   - Make retry/checkpoint review easier without losing the current lightweight UI.

## Recommended Order

1. Live command output polish
2. Legacy fallback cleanup
3. Planner follow-up history polish

## Suggested Milestones

### Milestone 1: Runtime polish
- Live command output polish

### Milestone 2: Fallback cleanup
- Reduce heuristic retry/sandbox fallback paths
- Keep explicit intent paths as the primary contract

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

### Seed 2: Fallback cleanup

```json
{
  "explanation": "Reduce legacy inference paths now that explicit retry and sandbox intent are available in the core tool contract.",
  "items": [
    { "step": "Find heuristic-only paths", "status": "pending" },
    { "step": "Prefer explicit intent", "status": "pending" },
    { "step": "Keep compatibility fallback", "status": "pending" }
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
  "explanation": "Finish the remaining planner/runtime polish after core orchestration, scoped AGENTS resolution, retry handling, and cross-process delivery are in place.",
  "items": [
    { "step": "Polish live command output", "status": "pending" },
    { "step": "Reduce legacy fallback paths", "status": "pending" },
    { "step": "Improve follow-up history UX", "status": "pending" }
  ]
}
```
