# Maintenance

This document summarizes the repository-level maintenance gates for the combined `Front-End/` and `Back-End/` workspace.

## Active Automation Boundaries

- The active GitHub Actions entry point is the root `.github/workflows/ci.yml`. It runs frontend gates from `Front-End/` and backend core verification from `Back-End/`.
- `Front-End/.github/workflows/ci.yml` and `Front-End/.github/CODEOWNERS` are legacy nested frontend files. GitHub does not use nested `.github/` files as active workflow or owner files for this root repository.
- The active ownership file is root `.github/CODEOWNERS`. It maps the current A/B/C/D role paths to the `@System-Docs-H` baseline owner until dedicated GitHub users or teams are provisioned.

## Progress Gates

The maintainer progress dashboard scores this project from checkbox completion in:

- `Front-End/docs/checklist.md`
- `Back-End/docs/checklist.md`

Keep implementation completion items as checked only when local source, docs, and validation evidence support them. External account provisioning can be recorded as a non-checkbox follow-up when it does not block local implementation or verification.

## Local Validation Closeout Evidence

- 2026-06-12: `Front-End/docs/checklist.md` and `Back-End/docs/checklist.md` contain no unchecked local implementation checklist items for the collaborative editor, REST API, WebSocket sync, document credentials, or snapshot-store boundary.
- 2026-06-12: External provisioning review for local-validation exit found one known follow-up: replace the root `.github/CODEOWNERS` `@System-Docs-H` baseline with dedicated GitHub users or teams after those accounts exist. That follow-up is not required for local source validation.
- Automation must not create external accounts, add secrets, deploy hosting, publish public data, or replace CODEOWNERS with unprovisioned handles. Those actions remain reserved for the `external-account-provisioning-review` phase.

## Validation Lanes

For documentation-only changes, run:

```bash
git diff --check
```

For frontend behavior, API client, route, UI, or env changes, run:

```bash
cd Front-End
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

Dedicated GitHub users or teams can replace the current `@System-Docs-H` baseline owner in root `.github/CODEOWNERS` when those accounts exist. That account provisioning is separate from the local product implementation and validation gates.
