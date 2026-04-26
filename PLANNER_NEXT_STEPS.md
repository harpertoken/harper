# Harper Planner Next Steps

This file captures the next implementation steps after the current planning/runtime work.

## Count

There are **10 solid next steps** that can be implemented without changing direction.

## Next Steps

1. **Live command output panel**
   - Stream stdout/stderr into the TUI while a command is running.
   - Show the active command, partial output, and final exit state.

2. **Background job model**
   - Introduce first-class job IDs, job records, and job lifecycle state.
   - Separate long-running jobs from simple synchronous tool calls.

3. **Job list UI**
   - Add a panel or overlay showing running, completed, blocked, and failed jobs.
   - Allow selecting a job to inspect its output and status history.

4. **Step-to-job linkage**
   - Explicitly attach a plan step to a spawned/running job.
   - Keep the step `in_progress` while the linked job is active.

5. **Automatic completion from job finish**
   - Mark a linked step completed when the job succeeds and the match is high-confidence.
   - Move the next pending step to `in_progress` when appropriate.

6. **Failure-aware plan transitions**
   - Mark runtime as `failed` or `blocked` when a job exits non-zero or approval is denied.
   - Nudge the model to revise the plan instead of continuing blindly.

7. **HTTP API plan/job events**
   - Expose live plan/job state through API endpoints or server-sent events.
   - Keep web clients in sync with the same runtime state as the TUI.

8. **Plan editing commands in UI**
   - Add direct UI actions for promoting, completing, or clearing steps.
   - Make plans editable even when the model is not actively driving the session.

9. **Scoped AGENTS.md resolution**
   - Replace the current single-file append behavior with scoped lookup and merge logic.
   - Use nested `AGENTS.md` rules per directory tree instead of only repo-root text.

10. **Planner-quality orchestration**
    - Add stronger multi-step execution policies around sequencing, retries, checkpoints, and summaries.
    - Make the model behave more like a planner/executor instead of only a reactive chat loop.

## Recommended Order

If we implement these in sequence, this is the best order:

1. Live command output panel
2. Background job model
3. Job list UI
4. Step-to-job linkage
5. Automatic completion from job finish
6. Failure-aware plan transitions
7. HTTP API plan/job events
8. Plan editing commands in UI
9. Scoped `AGENTS.md` resolution
10. Planner-quality orchestration

## Suggested Milestones

### Milestone 1: Live execution visibility
- Live command output panel
- Background job model
- Job list UI

### Milestone 2: Plan/job coordination
- Step-to-job linkage
- Automatic completion from job finish
- Failure-aware plan transitions

### Milestone 3: Cross-interface support
- HTTP API plan/job events
- Plan editing commands in UI

### Milestone 4: Planner maturity
- Scoped `AGENTS.md` resolution
- Planner-quality orchestration

## Harper `update_plan` Seeds

These are ready-to-use plan payloads in Harper-style structure.

### Seed 1: Live execution visibility

```json
{
  "explanation": "Build the first real live execution UX so running work is visible while a turn is still active.",
  "items": [
    { "step": "Design command event model", "status": "pending" },
    { "step": "Stream command output chunks", "status": "pending" },
    { "step": "Add live output panel", "status": "pending" },
    { "step": "Show job status summary", "status": "pending" }
  ]
}
```

### Seed 2: Background jobs

```json
{
  "explanation": "Introduce a first-class job model so long-running commands are tracked independently from the chat turn.",
  "items": [
    { "step": "Define job record schema", "status": "pending" },
    { "step": "Persist job lifecycle state", "status": "pending" },
    { "step": "Wire worker job messages", "status": "pending" },
    { "step": "Add job list interactions", "status": "pending" }
  ]
}
```

### Seed 3: Plan and job coordination

```json
{
  "explanation": "Tie plan progress to actual runtime state so steps reflect real execution instead of only model intent.",
  "items": [
    { "step": "Link steps to jobs", "status": "pending" },
    { "step": "Advance steps on success", "status": "pending" },
    { "step": "Handle blocked and failed runs", "status": "pending" },
    { "step": "Refine plan transition rules", "status": "pending" }
  ]
}
```

### Seed 4: Cross-interface support

```json
{
  "explanation": "Expose plan and job runtime state consistently across TUI and HTTP clients.",
  "items": [
    { "step": "Add HTTP plan runtime endpoints", "status": "pending" },
    { "step": "Stream API-side runtime events", "status": "pending" },
    { "step": "Add UI plan editing actions", "status": "pending" },
    { "step": "Keep clients in sync", "status": "pending" }
  ]
}
```

### Seed 5: Planner maturity

```json
{
  "explanation": "Move from basic reactive execution toward a planner-quality agent with scoped repo instructions and stronger orchestration.",
  "items": [
    { "step": "Resolve scoped AGENTS rules", "status": "pending" },
    { "step": "Add planner checkpoints", "status": "pending" },
    { "step": "Improve retry and recovery", "status": "pending" },
    { "step": "Summarize execution state cleanly", "status": "pending" }
  ]
}
```

## Single Master Plan

If you want one top-level plan instead of milestone plans, use this:

```json
{
  "explanation": "Finish Harper's planner/runtime system from live execution visibility through planner-quality orchestration.",
  "items": [
    { "step": "Add live output panel", "status": "pending" },
    { "step": "Introduce job model", "status": "pending" },
    { "step": "Build job list UI", "status": "pending" },
    { "step": "Link steps to jobs", "status": "pending" },
    { "step": "Auto-advance on job finish", "status": "pending" },
    { "step": "Handle blocked and failed states", "status": "pending" },
    { "step": "Expose runtime over HTTP", "status": "pending" },
    { "step": "Add UI plan editing", "status": "pending" },
    { "step": "Resolve scoped AGENTS rules", "status": "pending" },
    { "step": "Improve planner orchestration", "status": "pending" }
  ]
}
```
