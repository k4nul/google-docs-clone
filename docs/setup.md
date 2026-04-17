# Setup

## Requirements

- Node.js 20+
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

- `/` 문서 목록 placeholder
- `/docs/:docId` collaborative editor shell

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

같은 검증은 `.github/workflows/ci.yml`에서 자동 실행한다.

- `build`
- `lint`
- `test`
- `typecheck`

## Environment Variables

`.env.example`를 기준으로 `.env.local`을 구성한다.

- `VITE_API_BASE_URL`: REST API base URL
- `VITE_WS_URL`: Yjs websocket provider base URL

예시:

```bash
VITE_API_BASE_URL=http://localhost:4000/api
VITE_WS_URL=ws://localhost:1234
```

`VITE_WS_URL`이 비어 있으면 editor는 local-only Yjs mode로 렌더링된다.
