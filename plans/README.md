# Harper Plans

This directory contains ready-to-use planner payloads for Harper's `update_plan` flow.

## Files

- `master-plan.json` - top-level end-to-end roadmap
- `milestone-1-live-execution.json`
- `milestone-2-plan-job-coordination.json`
- `milestone-3-cross-interface.json`
- `milestone-4-planner-maturity.json`
- `agents-md-improvement.json` - dedicated plan for improving `AGENTS.md` handling
- `routing-improvement.json` - dedicated plan for strengthening deterministic and grounded routing
- `../PLANNER_NEXT_STEPS.md` - completed planner/runtime polish record for the last focused workstream

## Notes

- All files use the same shape: `explanation` plus ordered `items`
- Every item starts as `pending`
- These plans are intended to be copied into Harper's `update_plan` tool flow
- `PLANNER_NEXT_STEPS.md` is the exception: it now records a completed workstream rather than an active plan seed
