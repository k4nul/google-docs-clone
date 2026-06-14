# Maintenance

This document summarizes the repository-level maintenance gates for the combined `Front-End/` and `Back-End/` workspace.

## Active Automation Boundaries

- The active GitHub Actions entry point is the root `.github/workflows/ci.yml`. It runs frontend gates from `Front-End/` and backend core verification from `Back-End/`.
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
local validation can transition. Keep implementation completion items as checked
only when local source, docs, and validation evidence support them. External
account provisioning can be recorded as a non-checkbox follow-up when it does
not block local implementation or verification.

## Local Validation Closeout Evidence

- 2026-06-12: `Front-End/docs/checklist.md` and `Back-End/docs/checklist.md` contain no unchecked local implementation checklist items for the collaborative editor, REST API, WebSocket sync, document credentials, or snapshot-store boundary.
- 2026-06-12: External provisioning needs for local-validation exit are listed in `docs/external-provisioning-review.md`. Known follow-ups are dedicated GitHub users or teams, production hosting, snapshot durability choice, room coordination choice, secret storage, and public-data policy. Those follow-ups are not required for local source validation and remain reserved for owner review in the next phase.
- Automation must not create external accounts, add secrets, deploy hosting, publish public data, or replace CODEOWNERS with unprovisioned handles. Those actions remain reserved for the `external-account-provisioning-review` phase.

## Validation Lanes

For documentation-only changes, run:

```bash
git diff --check
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
