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

같은 검증은 활성 CI entry point인 루트 `.github/workflows/ci.yml`에서 `Front-End/` 기준으로 실행된다.

- `build`
- `lint`
- `test`
- `typecheck`

## Environment Variables

`.env.example`를 기준으로 `.env.local`을 구성한다.

- `VITE_API_BASE_URL`: REST API base URL. 없으면 `<current-origin>/api`를 사용한다.
- `VITE_API_TOKEN`: local backend의 `API_TOKEN`과 같은 값. 문서 목록과 문서 생성을 호출할 때 `Authorization: Bearer <token>`으로 사용한다. Vite 환경변수는 브라우저 번들에 포함되므로 `dev-admin-token`은 local loopback 개발 전용으로만 사용한다.
- `VITE_WS_URL`: Yjs websocket origin/base host. 없으면 `ws(s)://<current-host>/ws`를 사용한다. provider는 이 값에서 `/ws/:docId?access_token=<document-token>` endpoint를 구성한다.

문서 생성 응답의 `credentials.access_token`은 브라우저 `localStorage`에 문서별로 저장된다. 편집기 상세 조회와 제목 변경은 이 문서 credential을 `Authorization` 헤더로 보내고, 브라우저 WebSocket은 `/ws/:docId?access_token=<document-token>` query parameter를 사용한다.

예시:

```bash
# optional, only needed when the backend is not served through the same origin
VITE_API_BASE_URL=http://localhost:4000/api
VITE_API_TOKEN=dev-admin-token
VITE_WS_URL=ws://localhost:4000
```

현재 origin 기반 기본값을 쓰면 `localhost`, DDNS, 새 도메인, HTTPS 전환 시 프론트 환경변수를 바꾸지 않아도 된다.
Vite dev server에서는 환경변수를 생략했을 때의 `/api`와 `/ws` current-origin 요청이 `vite.config.ts`의 proxy를 통해 `127.0.0.1:4000` 백엔드로 전달된다. `.env.example`의 direct backend URL을 쓰는 방식도 같은 로컬 계약을 명시적으로 고정한다.
