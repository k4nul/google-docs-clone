# Maintenance

This document summarizes the repository-level maintenance gates for the combined `Front-End/` and `Back-End/` workspace.

## Active Automation Boundaries

- The active GitHub Actions entry point is the root `.github/workflows/ci.yml`. It runs frontend gates from `Front-End/`, backend core verification from `Back-End/`, and the backend WebSocket verification lane from `Back-End/`.
- The retired nested frontend `.github/` files have been removed so the root workflow and CODEOWNERS file are the only repository automation entry points.
- The active ownership file is root `.github/CODEOWNERS`. It maps the current A/B/C/D role paths to the `@System-Docs-H` baseline owner until dedicated GitHub users or teams are provisioned.

## Progress Gates

The maintainer progress dashboard scores this project from the checklist and
command criteria in `docs/management/VALIDATION.json`. The checklist criteria
read checkbox completion in:

- `Front-End/docs/checklist.md`
- `Back-End/docs/checklist.md`

The command criteria mirror the active phase gate: the frontend first runs
`npm run deps:ci` from `Front-End/`, then build, lint, test, and typecheck must
pass; backend `core` plus `websocket` verification lanes must also pass before
local validation can transition. Root CI covers the same frontend build, lint,
test, typecheck, backend core, and backend WebSocket lanes, while the maintainer
phase command keeps `npm run deps:ci` explicit so clean temporary worktrees
install from the lockfile before validation. Keep implementation completion
items as checked only when local source, docs, and validation evidence support
them. External account provisioning can be recorded as a non-checkbox follow-up
when it does not block local implementation or verification.

When the progress dashboard reads `100%/complete` while the phase shows
`external-account-provisioning-review->maintenance-only`, the remaining
movement is owner approval for the external provisioning plan, followed by a
phase-transition run. A docs-only run should only keep the review evidence and
validation instructions accurate; it must not force the phase forward, provision
external accounts, create secrets, deploy hosting, or replace CODEOWNERS.

## Local Validation Closeout Evidence

- 2026-08-12: The full local-validation transition command passed in a clean
  worktree: frontend dependency install, build, lint, tests, and typecheck,
  followed by backend `./scripts/verify.sh core` and
  `./scripts/verify.sh websocket`. This refreshes local validation evidence
  only; it does not record external owner approval or authorize a phase move.
- 2026-06-12: `Front-End/docs/checklist.md` and `Back-End/docs/checklist.md` contain no unchecked local implementation checklist items for the collaborative editor, REST API, WebSocket sync, document credentials, or snapshot-store boundary.
- 2026-06-12: External provisioning needs for local-validation exit are listed in `docs/external-provisioning-review.md`. Known follow-ups are dedicated GitHub users or teams, production hosting, snapshot durability choice, room coordination choice, secret storage, and public-data policy. Those follow-ups are not required for local source validation and remain reserved for owner review in the active external provisioning phase.
- 2026-06-16: The active phase is `external-account-provisioning-review`. The local validation evidence remains useful, but the transition to `maintenance-only` is blocked until `external-owner-approval-recorded` is no longer pending in `docs/instructions/phase-gates.json`.
- Automation must not create external accounts, add secrets, deploy hosting, publish public data, or replace CODEOWNERS with unprovisioned handles. Those actions remain reserved for explicit owner approval during `external-account-provisioning-review`.

## Validation Lanes

For documentation-only changes, run:

```bash
git diff --check
```

For phase readiness, run the full validation command from the repository root:

```bash
cd Front-End && npm run deps:ci && npm run build && npm run lint && npm run test && npm run typecheck && cd ../Back-End && ./scripts/verify.sh core && ./scripts/verify.sh websocket
```

The current phase remains `external-account-provisioning-review` until that
chain exits zero and owner approval is recorded. A WebSocket lane failure such
as `document_detail_restores_latest_sqlite_snapshot_after_managed_owner_handoff`
is still a validation blocker for managed coordination plus shared SQLite
snapshot handoff; it is not an external provisioning task. Reproduce that
single filter from `Back-End/` with:

```bash
cargo test --locked --test health document_detail_restores_latest_sqlite_snapshot_after_managed_owner_handoff
```

For frontend behavior, API client, route, UI, or env changes, run:

```bash
cd Front-End
npm run deps:ci
npm run build
npm run lint
npm run test
npm run typecheck
```

For backend behavior, API, WebSocket, storage, or verification-script changes, run:

```bash
cd Back-End
./scripts/verify.sh core
./scripts/preflight.sh websocket
./scripts/verify.sh websocket
```

Run `cargo check --features full-snapshot-stores` only when the full snapshot adapter inventory or related documentation changes.

## Known External Follow-Up

Dedicated GitHub users or teams can replace the current `@System-Docs-H` baseline owner in root `.github/CODEOWNERS` when those accounts exist. The full follow-up list is maintained in `docs/external-provisioning-review.md`; account provisioning, hosting, external storage, secret creation, and public data publication are separate from the local product implementation and validation gates.
