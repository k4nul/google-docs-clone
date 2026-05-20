# Onboarding

이 문서는 루트 기준으로 프론트엔드와 백엔드를 함께 띄우는 최소 온보딩 절차를 정리한다. 세부 영역 문서는 `Front-End/docs/`와 `Back-End/docs/`를 기준으로 확인한다.

## Repository Shape

```text
.
|-- Front-End/   # React, Vite, Tiptap, Yjs collaborative editor
|-- Back-End/    # Axum, Tokio, Yrs collaborative server
`-- docs/        # cross-stack onboarding, testing, troubleshooting
```

- 프론트엔드는 문서 목록, `/docs/:docId` 편집기, DOCX/JSON import/export, Yjs binary WebSocket provider를 담당한다.
- 백엔드는 `/api/documents` REST API, `/ws/:doc_id` WebSocket, Yrs room registry, snapshot persistence를 담당한다.
- 백엔드 기본 snapshot store는 `file`이며 `Back-End/data/snapshots` 아래에 문서를 복구 가능한 snapshot으로 저장한다.
- 로컬 개발 계약에서는 문서 API와 WebSocket 연결에 `Authorization` 헤더가 필요하지 않다.

## Requirements

- Node.js 20+
- npm
- Rust toolchain and Cargo

## First Run

터미널 1에서 백엔드를 먼저 실행한다.

```bash
cd Back-End
cp .env.example .env
cargo run
```

기본 백엔드 주소는 `http://127.0.0.1:4000`이다. `.env.example`의 기본값은 `FRONTEND_ORIGIN=*`, `SNAPSHOT_STORE=file`, `SNAPSHOT_DIR=./data/snapshots`다.

터미널 2에서 프론트엔드를 실행한다.

```bash
cd Front-End
cp .env.example .env.local
npm install
npm run dev
```

`Front-End/.env.example`은 로컬 백엔드에 맞춰져 있다.

```bash
VITE_API_BASE_URL=http://localhost:4000/api
VITE_WS_URL=ws://localhost:4000
```

`VITE_WS_URL`은 WebSocket origin/base host다. 실제 room endpoint는 provider가 `/ws/:docId`로 구성하므로 로컬 UUID 문서는 `ws://localhost:4000/ws/<uuid>`에 연결된다.

## Local Editing Flow

1. 브라우저에서 Vite dev server URL을 연다.
2. 홈 화면은 `GET /api/documents`로 백엔드 문서 목록을 불러온다.
3. 백엔드 목록을 불러올 수 없으면 local sample 문서가 표시된다.
4. `Create backend editor`를 클릭하면 프론트엔드가 `POST /api/documents`를 호출한다.
5. 백엔드가 UUID 문서를 만들고 프론트엔드는 `/docs/<uuid>`로 이동한다.
6. 편집기 화면은 `GET /api/documents/:id`가 성공한 뒤 `ws://localhost:4000/ws/<uuid>`로 실시간 협업 연결을 연다.

## Documentation Map

| 문서 | 용도 |
| --- | --- |
| `README.md` | 프로젝트 개요와 빠른 실행 |
| `docs/testing.md` | 루트 기준 검증 명령과 CI 상태 |
| `docs/troubleshooting.md` | 로컬 API/WS 연동과 validation 문제 해결 |
| `Front-End/README.md` | 프론트엔드 기능, 개발 규칙, 실행 방법 |
| `Front-End/docs/setup.md` | 프론트엔드 설치, env, 검증 |
| `Front-End/docs/architecture.md` | 프론트엔드 모듈과 realtime flow |
| `Back-End/README.md` | 백엔드 API, WebSocket, snapshot store 개요 |
| `Back-End/docs/setup.md` | 백엔드 실행, env, store 선택 |
| `Back-End/docs/api.md` | REST/WebSocket 계약과 에러 응답 |
