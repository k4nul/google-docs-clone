# External Provisioning Review

This review packet records what must be decided outside the local collaborative
editor validation phase. It is documentation evidence only. It does not create
accounts, deploy services, replace repository owners, publish data, or introduce
real credentials.

## Local Validation Boundary

The current repository is validated as a local collaborative editor:

- Frontend development uses `Front-End/.env.example` values for a local backend:
  `VITE_API_BASE_URL=http://localhost:4000/api`,
  `VITE_API_TOKEN=dev-admin-token`, and `VITE_WS_URL=ws://localhost:4000`.
- Backend development starts from `Back-End/.env.example` with
  `HOST=127.0.0.1`, `PORT=4000`, `FRONTEND_ORIGIN=*`,
  `API_TOKEN=dev-admin-token`, `SNAPSHOT_STORE=file`,
  `ROOM_LOCATOR=local`, and `ROOM_COORDINATOR=noop`.
- The active ownership file is root `.github/CODEOWNERS`, which still maps all
  repository paths to the baseline `@System-Docs-H` owner.
- The active CI entry point is root `.github/workflows/ci.yml`. It runs the
  frontend build, lint, test, and typecheck gates, plus backend
  `./scripts/verify.sh core` and `./scripts/verify.sh websocket`.

These defaults are local-development defaults. They are not production hosting,
external storage, external coordination, or owner-account provisioning.

## Current Phase Boundary

This packet is now the evidence file for the active
`external-account-provisioning-review` phase. It does not authorize account
creation, deployment, secret creation, public data publication, or a phase move
to `maintenance-only`.

The phase gate in `docs/instructions/phase-gates.json` still requires the local
validation command to pass:

```bash
cd Front-End && npm run deps:ci && npm run build && npm run lint && npm run test && npm run typecheck && cd ../Back-End && ./scripts/verify.sh core && ./scripts/verify.sh websocket
```

The root CI workflow covers the frontend gates, backend core lane, and backend
WebSocket lane. The backend WebSocket lane also remains explicit in the phase
command because it exercises socket binding and collaboration session behavior.

The phase command is intentionally stricter than the root CI frontend install
step because generated maintenance worktrees start from a clean checkout.
`npm run deps:ci` installs from `Front-End/package-lock.json` before TypeScript,
Vite, ESLint, Vitest, or typecheck run, matching
`docs/instructions/phase-gates.json` and `docs/management/VALIDATION.json`.

The remaining transition blocker is owner approval. The
`external-owner-approval-recorded` gate is pending until an owner explicitly
approves the external account, hosting, storage, publish, secret, and
public-data plan. Until that approval is recorded, keep the current phase at
`external-account-provisioning-review`.

## Reviewed External Needs

| Area | Current state | Required owner decision before action |
| --- | --- | --- |
| GitHub ownership | `.github/CODEOWNERS` uses `@System-Docs-H` for every active path. | Provision or identify dedicated GitHub users or teams, then replace the baseline owner with real handles. |
| Production hosting | No production host is selected in repository docs or config. | Choose hosting, public URL, HTTPS termination, allowed frontend origins, and rollback process. |
| Admin API token | `dev-admin-token` is documented for loopback development only. | Generate a deployment-specific admin token and store it outside the repository. |
| Document credentials | Document access tokens are issued by `POST /api/documents` and stored in browser `localStorage`. | Decide sharing, rotation, revocation, and support procedures before public use. |
| Snapshot durability | Local default is `SNAPSHOT_STORE=file`; `s3` and `managed` are supported config surfaces. | Choose whether production uses file, S3-compatible object storage, or a managed snapshot service. Provision buckets or services only after approval. |
| Room ownership | Local default is `ROOM_LOCATOR=local` and `ROOM_COORDINATOR=noop`; file, sqlite, and managed coordination modes are documented. | Decide whether deployment is single-node or multi-node. If multi-node, select and provision an authoritative coordination plane. |
| Secrets | `.env.example` contains placeholder local values and commented examples. | Store real tokens, S3 keys, managed-service tokens, and citadeldb passphrases in an approved secret store. |
| Public data | No public dataset or sample production documents are selected. | Approve any demo data, redaction rules, retention rules, and backup handling before publishing. |

## Automation Boundary

Automation may keep the docs and local validation commands current. It must not:

- create GitHub users or teams.
- replace `@System-Docs-H` with unprovisioned handles.
- create hosting, S3 buckets, managed snapshot services, or managed
  coordination services.
- commit real secrets or production `.env` files.
- deploy the frontend or backend.
- publish real document data.

## Owner Review Inputs

The active external provisioning review can start from these owner-review
inputs:

1. GitHub owner handles or teams that should replace `@System-Docs-H`.
2. Hosting target, public origin list, TLS boundary, and rollback owner.
3. Snapshot durability choice: local file, S3-compatible storage, or managed
   snapshot service.
4. Room coordination choice: single-node local ownership or an external
   coordination plane.
5. Secret storage location for admin API token, document-service secrets, S3
   credentials, managed-service tokens, and citadeldb passphrase if used.
6. Data publication and retention policy for demos, backups, and support access.

## Owner Review Checklist

The `external-account-provisioning-review` phase should record explicit owner
decisions before any external action:

- Replace the baseline CODEOWNERS handle only after the dedicated GitHub users
  or teams exist.
- Choose the hosting target, public origin allowlist, TLS boundary, rollback
  owner, and deployment approval path.
- Choose snapshot durability and room coordination surfaces before provisioning
  buckets, databases, or managed services.
- Choose the secret store before generating deployment tokens or service
  credentials.
- Approve public demo data, retention rules, backup handling, and support-access
  policy before publishing any document data.

The local-validation external review packet is complete because these external
needs are listed and bounded. Actual approval remains the open item for the
active `external-account-provisioning-review` phase, and provisioning must wait
until that approval is recorded.
