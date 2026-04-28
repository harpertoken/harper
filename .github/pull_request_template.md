## Changes

- Use bullet points.
- Prefer concrete reviewer-facing changes.
- Use nested bullets when one change needs examples, endpoints, commands, or sub-points.
- Call out behavior changes, storage changes, migrations, config changes, and docs updates explicitly.

Example shape:
- Added browser and TUI auth flows using PKCE:
  - `/auth/login/{provider}`
  - `/auth/callback`
  - `/auth/me`
- Scoped session load and delete operations to the authenticated user.
- Updated CLI behavior:
  - `cargo harper` starts the TUI and local server by default.
  - `--no-server` is the explicit opt-out.

## Validation

- List the exact commands or manual checks you used.
- Keep each item concrete and reproducible.
- Include any important manual verification steps when behavior depends on the local environment.

Examples:
- `cargo check -p harper-core`
- `cargo test -p harper-ui`
- manual verification of TUI sign-in and restart persistence
