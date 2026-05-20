# Troubleshooting

## Backend List Is Unavailable

Symptoms:

- Home page shows `Showing local samples`.
- Runtime wiring shows `API base` but document list has a backend error.

Checks:

```bash
cd Back-End
cp .env.example .env
cargo run
```

Then open `http://127.0.0.1:4000/api/health`. The expected response has `"status": "ok"` and `"service": "backend"`.

If the frontend is served from Vite, `Front-End/vite.config.ts` proxies `/api` to `http://127.0.0.1:4000`. If `VITE_API_BASE_URL` is set, verify it points to `http://localhost:4000/api` or another reachable backend API base.

## WebSocket Stays Disconnected

The frontend should connect only after `GET /api/documents/:id` succeeds. A local sample ID such as `launch-plan` is not a backend UUID document and will not open a real collaboration room.

Use this flow:

1. Start the backend on port `4000`.
2. Start the frontend dev server.
3. Click `Create backend editor` from the home page.
4. Confirm the editor route uses a UUID.
5. Confirm the provider endpoint is `ws://localhost:4000/ws/<uuid>` or the equivalent proxied `/ws/<uuid>` URL.

If the backend rejects the WebSocket handshake, check `FRONTEND_ORIGIN`. The default `*` allows local development origins. A restricted value must match the browser origin exactly, or be part of the comma-separated allowlist.

## Delete Returns Conflict

`DELETE /api/documents/:id` returns `409 conflict` while an active collaboration WebSocket session is attached to that document. Close editor tabs for the document, wait for the session to close, then retry the delete request.

## Snapshot Data Does Not Persist

The default backend `.env.example` uses:

```bash
SNAPSHOT_STORE=file
SNAPSHOT_DIR=./data/snapshots
```

With this setting, documents should survive process restarts. If persistence is not needed for a one-off run, set `SNAPSHOT_STORE=memory`. If an extended store is selected, confirm its matching `SNAPSHOT_*_PATH` or service configuration is present in `Back-End/.env.example` and that the adapter is available in the current feature set.

## Backend Validation Is Blocked

Use the scripted lanes to distinguish environment blockers from test failures.

```bash
cd Back-End
./scripts/verify.sh core
./scripts/preflight.sh websocket
./scripts/verify.sh websocket
```

- If `preflight.sh websocket` reports that the runner cannot bind socket addresses, the WebSocket lane is blocked by the environment.
- If `verify.sh core` fails, inspect the preceding `cargo fmt`, `cargo check`, or socket-free test output.
- If `preflight.sh publish` fails DNS or `.git` write checks, that blocks publish readiness, not necessarily local build correctness.

## Frontend Validation Is Blocked

Run commands from `Front-End/` because the package manifest and lockfile live there.

```bash
cd Front-End
npm ci
npm run build
npm run lint
npm run test
npm run typecheck
```

Vitest uses jsdom and `Front-End/src/test/setup.ts` from `Front-End/vite.config.ts`. If `npm` is blocked by PowerShell execution policy on Windows, use `npm.cmd run <task>`.
