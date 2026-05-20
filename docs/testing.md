# Testing

루트에는 통합 test runner가 없다. 각 영역의 검증은 해당 디렉터리에서 실행한다.

## Frontend

처음 설치하거나 lockfile 기준으로 깨끗하게 재설치해야 할 때는 `npm ci`를 사용한다. 로컬 개발 중 기존 `node_modules`를 유지해도 되는 경우에는 `npm install` 뒤 동일한 gate를 실행한다.

```bash
cd Front-End
npm ci
npm run build
npm run lint
npm run test
npm run typecheck
```

| Command | Evidence | Purpose |
| --- | --- | --- |
| `npm run build` | `Front-End/package.json` | TypeScript build와 Vite production build |
| `npm run lint` | `Front-End/package.json` | ESLint, `--max-warnings 0` |
| `npm run test` | `Front-End/package.json`, `Front-End/vite.config.ts` | Vitest with jsdom and `src/test/setup.ts` |
| `npm run typecheck` | `Front-End/package.json` | `tsc -b --pretty false` |

`Front-End/.github/workflows/ci.yml` mirrors these frontend gates, but it is stored under the frontend subdirectory. From the current repository root there is no root `.github/workflows/ci.yml` dispatcher.

## Backend

Use the scripted lanes first because they separate socket-free checks from WebSocket tests that need local socket binding.

```bash
cd Back-End
./scripts/verify.sh core
./scripts/preflight.sh publish
./scripts/verify.sh websocket
cargo check --features full-snapshot-stores
```

| Command | Purpose |
| --- | --- |
| `./scripts/verify.sh core` | Runs `cargo fmt --check`, `cargo check --locked`, and socket-free tests |
| `./scripts/preflight.sh publish` | Checks `.git` metadata write access and `github.com` DNS for publish readiness |
| `./scripts/verify.sh websocket` | Probes socket binding, then runs WebSocket/delete/managed/S3 lanes |
| `cargo check --features full-snapshot-stores` | Compile-checks the full snapshot adapter inventory |

For full adapter regression instead of compile-only inventory, run:

```bash
cd Back-End
cargo test --features full-snapshot-stores
```

## When To Run More

- API, WebSocket, room ownership, or snapshot persistence changes: run backend `core`, `websocket`, and the relevant `full-snapshot-stores` check or test.
- Frontend route, editor, provider, import/export, or env handling changes: run all frontend gates.
- Cross-stack contract changes: run frontend gates, backend `core`, backend `websocket`, and manually exercise create/open/edit with the backend and Vite dev servers running.
- Docs-only changes: run `git diff --check`; run full gates only when the documentation change reveals a command or contract that needs live verification.
