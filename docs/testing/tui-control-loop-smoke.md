# TUI Control Loop Smoke

This smoke checklist validates the control-loop behavior added to Harper's chat and plan runtime:

- classify intent
- decide task mode
- inspect or plan when needed
- execute tools
- emit feedback and retry/replan state
- surface loop state in the TUI

## Fast pre-check without the TUI

Before doing the manual TUI pass, use `harper-batch` to verify the same core chat flow in a shell:

```bash
cargo run -p harper-ui --bin harper-batch -- --strategy deterministic --prompt "where is execution strategy used in this repo"
```

```bash
cargo run -p harper-ui --bin harper-batch -- --strategy deterministic \
  --prompt "run the git status" \
  --prompt "run that"
```

What `harper-batch` shows:

- `ROUTING`
  - selected strategy
  - task mode such as `direct_deterministic` or `respond_only`
- `DETERMINISTIC INTENT`
  - the routed deterministic intent when one exists
- `NORMALIZED COMMAND`
  - command normalization for run-command paths
- `CLARIFICATION`
  - follow-up clarification when Harper refuses an ambiguous action
- `ACTIVITY`
  - runtime stage transitions
- `ASSISTANT`
  - final response text

Use this first when debugging natural-language behavior. If the shell harness is wrong, the TUI will also be wrong because both use the same `ChatService` flow.

## Preconditions

- Launch Harper inside this repository.
- Use a session with the standard TUI header and plan panel visible.
- If needed, ensure the plan panel is visible from the chat screen.

## 1. Grounded deterministic request

Set strategy:

- `/strategy grounded`

Prompt:

- `where is execution strategy used in this repo`

Expected behavior:

- Harper should take the deterministic inspection path quickly.
- The header activity should briefly show stages like:
  - `executing deterministic action`
  - `summarizing result`
- Harper should respond with grounded repo information rather than generic prose.

## 2. Planned authoring request

Set strategy:

- `/strategy grounded`

Prompt:

- `refactor the planner flow in this repo and then update the tui followup rendering`

Expected behavior:

- Harper should not jump straight to editing.
- The header activity should show stages like:
  - `planning task`
  - `inspecting repo context`
- The plan panel should persist loop information such as:
  - `loop: plan`
  - `loop: inspect`
- If a plan exists, the plan panel should continue to show loop state after the transient header activity clears.

## 3. Plain response request

Set strategy:

- `/strategy auto`

Prompt:

- `what is the difference between grounded and deterministic`

Expected behavior:

- Harper should respond directly.
- The request should not be forced into a plan or repo inspection path.
- The header activity should align with a response-only flow.

## 4. Retry / replan path

Prompt shape:

- use any planned task that causes a failing validation or blocked command
- or trigger planner follow-up actions through the existing retry / replan flow

Expected behavior:

- The plan panel should persist outcome lines such as:
  - `last outcome: retry suggested`
  - `last outcome: replan required`
- The plan panel should also show the latest feedback summary line below the outcome.

## 5. Follow-up history visibility

Prompt shape:

- complete or replace a follow-up during a planned task

Expected behavior:

- The plan panel should show:
  - the active follow-up when present
  - `recent followups`
  - recent archived follow-up entries

## Notes

This smoke does not verify every tool path. It is focused on the visible control loop:

- decision gate
- runtime loop state
- TUI loop rendering
