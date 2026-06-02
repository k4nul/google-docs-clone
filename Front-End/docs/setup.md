# Setup

## Requirements

- Node.js 20.19+, 22.12+, or 24+
- npm

## Install

```bash
npm install
```

## Run

```bash
npm run dev
```

## Automation / PowerShell

- 기본 예시는 `npm run <task>` 기준으로 작성한다.
- Windows PowerShell에서 실행 정책 때문에 `npm.ps1`이 차단되면 `npm.cmd run <task>`를 사용한다.

예시:

```powershell
npm.cmd run dev
```

기본 라우트:

- `/` backend 문서 목록과 사용자용 unavailable 상태
- `/docs/:docId` backend detail lookup 이후 collaborative editor shell

## Build

```bash
npm run build
```

## Lint

```bash
npm run lint
```

## Test

```bash
npm run test
```

## Type Check

```bash
npm run typecheck
```

## CI Quality Gates

같은 검증은 루트 `.github/workflows/ci.yml`에서 `Front-End/` 기준으로 실행된다. `Front-End/.github/workflows/ci.yml`은 package-local mirror로 남아 있다.

- `build`
- `lint`
- `test`
- `typecheck`

## Environment Variables

`.env.example`를 기준으로 `.env.local`을 구성한다.

- `VITE_API_BASE_URL`: REST API base URL. 없으면 `<current-origin>/api`를 사용한다.
- `VITE_WS_URL`: Yjs websocket origin/base host. 없으면 `ws(s)://<current-host>/ws`를 사용한다. provider는 이 값에서 `/ws/:docId` endpoint를 구성한다.
- `VITE_API_TOKEN`: optional legacy token. 현재 local backend contract에서는 문서 생성, 조회, WebSocket 연결에 필요하지 않다.

예시:

```bash
# optional, only needed when the backend is not served through the same origin
VITE_API_BASE_URL=http://localhost:4000/api
VITE_WS_URL=ws://localhost:4000
# optional legacy compatibility only
# VITE_API_TOKEN=dev-admin-token
```

현재 origin 기반 기본값을 쓰면 `localhost`, DDNS, 새 도메인, HTTPS 전환 시 프론트 환경변수를 바꾸지 않아도 된다.
Vite dev server에서는 환경변수를 생략했을 때의 `/api`와 `/ws` current-origin 요청이 `vite.config.ts`의 proxy를 통해 `127.0.0.1:4000` 백엔드로 전달된다. `.env.example`의 direct backend URL을 쓰는 방식도 같은 로컬 계약을 명시적으로 고정한다.
