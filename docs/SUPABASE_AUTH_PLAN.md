# Supabase Auth Plan

This repository currently supports provider API keys, not Harper user accounts.

This plan adds real user sign-in using Supabase Auth, with GitHub and Google first, and Apple after the callback/session flow is stable.

## Phase 1: Shared auth model

- Add Supabase auth config to `config/default.toml`
- Add core user/session/auth-claims types in `lib/harper-core/src/core/auth.rs`
- Re-export those types so server, TUI, and future clients share one model

## Phase 2: Server-side auth verification

- Add HTTP auth middleware to `lib/harper-core/src/server/mod.rs`
- Accept Supabase bearer JWTs on authenticated routes
- Verify JWT claims using the configured Supabase JWT secret
- Attach authenticated user context to request handlers

## Phase 3: Auth routes

- Add `GET /auth/login/:provider`
- Add `GET /auth/callback`
- Use Supabase PKCE-style redirect flow for browser-based sign-in
- Support `github` and `google` first
- Add `apple` after the web callback flow is stable

## Phase 4: Session ownership

- Add `user_id` ownership to persisted Harper sessions
- Scope session list, session fetch, exports, and review endpoints by authenticated user
- Keep local/offline mode available when auth is disabled

## Phase 5: TUI login flow

- Add `harper auth login --provider github|google|apple --user`
- Launch browser to the Harper auth route
- Complete callback through a local HTTP route or configured redirect
- Store Harper refresh/session tokens securely

## Phase 6: Website and product UX

- Add sign-in buttons on the website
- Add account/session explanation to privacy and troubleshooting docs
- Make it clear which flows require auth and which stay local-only

## Recommended rollout

1. Shared auth model
2. Server-side JWT verification
3. GitHub + Google web login
4. Session ownership
5. TUI login
6. Apple login

## Notes

- Supabase is a good fit for web and API auth.
- GitHub and Google are the practical first providers.
- Apple support is possible, but should not be the first provider wired for a terminal-first product.
- Keep auth optional so Harper can still run in a purely local mode.
