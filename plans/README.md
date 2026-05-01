# Harper Plans

This directory contains ready-to-use planner payloads for Harper's `update_plan` flow.

## Files

- `master-plan.json` - top-level end-to-end roadmap
- `milestone-1-live-execution.json`
- `milestone-2-plan-job-coordination.json`
- `milestone-3-cross-interface.json`
- `milestone-4-planner-maturity.json`
- `agents-md-improvement.json` - completed record for the scoped `AGENTS.md` improvement workstream
- `routing-improvement.json` - partially completed plan for the remaining routing work
- `distribution-and-self-update.json` - active plan for release artifacts, self-update, and remaining install-path polish
- `../PLANNER_NEXT_STEPS.md` - completed planner/runtime polish record for the last focused workstream

## Notes

- All files use the same shape: `explanation` plus ordered `items`
- Most plan files are intended to be copied into Harper's `update_plan` tool flow
- Some files now record completed or partially completed workstreams rather than a fully pending seed
- `PLANNER_NEXT_STEPS.md` and `agents-md-improvement.json` are completion-oriented records
