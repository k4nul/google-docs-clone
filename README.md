# Google Docs Clone

React 기반 collaborative editor 프론트엔드와 Rust 기반 실시간 협업 백엔드를 함께 둔 Google Docs 스타일 문서 편집기 프로젝트입니다. 프론트엔드는 Tiptap/Yjs로 편집 UI와 협업 provider를 구성하고, 백엔드는 Axum/Tokio/Yrs로 문서 API, WebSocket 협업 세션, snapshot 저장 경계를 제공합니다.

## 핵심 기능

- 문서 목록 화면과 `/docs/:docId` 편집기 라우트
- Tiptap 기반 rich text editor
- Yjs/Yrs binary sync protocol 기반 실시간 공동 편집
- collaborator awareness/caret 상태 전송 구조
- DOCX import/export와 HTML sanitize 유틸리티
- 문서 생성, 조회, 삭제 REST API
- 문서별 `GET /ws/:doc_id` WebSocket 협업 endpoint
- 파일, 메모리, SQLite, S3, managed service 기반 snapshot store
- optional `full-snapshot-stores` feature를 통한 다수의 embedded KV snapshot adapter 검증

## 기술 스택

| 영역 | 주요 기술 |
| --- | --- |
| Front-End | React 19, TypeScript, Vite, React Router, Tiptap, Yjs, Mammoth, DOMPurify, Vitest |
| Back-End | Rust 2024, Axum, Tokio, Yrs, yrs-axum, DashMap, tower-http, tracing |
| Realtime | Yjs/Yrs sync protocol over binary WebSocket frame |
| Persistence | `SnapshotStore` trait, 기본 `memory`/`file`/`sqlite`/`s3`/`managed`, 확장 adapter inventory |

## 요구 사항

- Node.js 20.19+, 22.12+, or 24+
- npm
- Rust toolchain과 Cargo

## 폴더 구조

```text
.
|-- Front-End/   # React + TypeScript + Vite collaborative editor
`-- Back-End/    # Axum + Tokio + Yrs collaborative server
```

| 경로 | 설명 |
| --- | --- |
| `Front-End/src/app`, `Front-End/src/pages` | 앱 진입점, 라우팅, route-level page |
| `Front-End/src/features/editor` | Tiptap editor shell, toolbar, extension 조합 |
| `Front-End/src/lib/collab` | Y.Doc와 binary WebSocket provider lifecycle |
| `Front-End/src/lib/api` | 백엔드 REST API helper |
| `Front-End/src/lib/import` | DOCX import 및 sanitize utility |
| `Front-End/src/lib/export` | editor HTML을 DOCX blob으로 변환하는 export utility |
| `Back-End/src/routes` | `/api/health`, `/api/documents` REST route |
| `Back-End/src/collab` | room registry, WebSocket, Yrs protocol boundary |
| `Back-End/src/storage` | snapshot store trait과 adapter 구현 |
| `Back-End/vendor` | snapshot adapter 검증을 위한 vendored/patched embedded KV crate |
| `Back-End/docs`, `Front-End/docs` | 각 영역의 setup, architecture, conventions, checklist 문서 |

## 빠른 실행

### 1. 백엔드 실행

```bash
cd Back-End
cp .env.example .env
cargo run
```

기본 서버 주소는 `http://127.0.0.1:4000`입니다. 기본 `FRONTEND_ORIGIN=*`라 로컬 개발에서는 Vite dev server origin을 별도 등록하지 않아도 됩니다. 기본 `SNAPSHOT_STORE=file`은 `Back-End/data/snapshots` 아래에 문서 snapshot을 저장합니다.
`HOST=0.0.0.0`처럼 loopback 밖으로 바인드할 때는 백엔드가 시작 시 `dev-admin-token`, `FRONTEND_ORIGIN=*`, `SNAPSHOT_STORE=citadeldb`의 기본 passphrase 조합을 거부합니다. 이 경우 `API_TOKEN`을 새 값으로 바꾸고, `FRONTEND_ORIGIN`을 실제 프런트엔드 origin 목록으로 제한하며, citadeldb를 쓰면 `SNAPSHOT_CITADELDB_PASSPHRASE`도 바꿉니다.

### 2. 프론트엔드 실행

```bash
cd Front-End
cp .env.example .env.local
npm install
npm run dev
```

`Front-End/.env.example`의 기본값은 로컬 백엔드에 맞춰져 있습니다.
`VITE_API_TOKEN`은 브라우저 번들에 포함되므로 `dev-admin-token`은 local loopback 개발 전용입니다.

```bash
VITE_API_BASE_URL=http://localhost:4000/api
VITE_API_TOKEN=dev-admin-token
VITE_WS_URL=ws://localhost:4000
```

프론트엔드는 `VITE_API_TOKEN`으로 문서 목록/생성 API를 호출하고, 문서 생성 응답의 `credentials.access_token`을 브라우저 `localStorage`에 저장합니다. 편집기 상세 조회, 제목 변경, 삭제, realtime WebSocket 연결은 이 문서별 credential이 있어야 열립니다.

### 3. 브라우저에서 확인

1. Vite가 출력한 로컬 URL로 접속합니다.
2. 홈 화면에서 문서 목록을 확인하거나 `New document`를 클릭합니다.
3. 프론트엔드가 `POST /api/documents`로 UUID 문서를 만들고 `/docs/<uuid>`로 이동합니다.
4. 편집기는 저장된 문서 credential으로 `GET /api/documents/:id` 메타데이터를 확인한 뒤 `ws://localhost:4000/ws/<uuid>?access_token=<token>`으로 WebSocket을 열어 협업 동기화를 시작합니다.

문서 목록을 불러올 수 없을 때 홈 화면은 사용자용 unavailable 상태와 재시도 버튼을 표시합니다. 실제 백엔드 협업 WebSocket은 백엔드가 생성한 UUID 문서에서만 정상 연결됩니다.

## API / WebSocket 계약

HTTP base path는 `/api`입니다.

| Method | Path | 설명 |
| --- | --- | --- |
| `GET` | `/api/health` | 서버 상태 확인 |
| `GET` | `/api/documents` | active room과 persisted snapshot 문서 목록 조회. body preview는 `hide_preview=false`일 때만 포함 |
| `POST` | `/api/documents` | 새 문서 생성 |
| `GET` | `/api/documents/:id` | UUID 문서 상세 조회 |
| `PATCH` | `/api/documents/:id` | UUID 문서 제목 또는 preview visibility 변경 |
| `DELETE` | `/api/documents/:id` | 문서 삭제. active WebSocket 세션이 있으면 `409 conflict` |
| `GET` | `/ws/:doc_id` | Yjs/Yrs binary sync WebSocket endpoint |

문서 목록/생성은 `Authorization: Bearer <API_TOKEN>`을 사용하고, 상세/제목 변경/preview visibility 변경/삭제는 `Authorization: Bearer <document-access-token>`을 사용합니다. 문서 목록은 저장된 Yrs `content` update에서 plain-text `preview`를 만들어 반환하며, `hide_preview=true` 문서는 body-derived preview를 보내지 않고 `preview_hidden=true`만 반환합니다. 브라우저 WebSocket은 임의 헤더를 보낼 수 없으므로 문서 credential을 `access_token` query parameter로 전달합니다. WebSocket payload는 JSON이 아니라 Yrs v1 binary message입니다. 프론트엔드 provider는 `Sync`, `Awareness`, `AwarenessQuery` 메시지를 인코딩/디코딩하고, 백엔드는 같은 `doc_id` room에 연결된 클라이언트에게 update를 broadcast합니다.

## 외부 프로비저닝 경계

루트 `.github/CODEOWNERS`는 현재 `@System-Docs-H` baseline owner를 사용합니다. dedicated GitHub users/teams가 준비되고 owner review가 끝나면 이 baseline을 실제 팀/사용자 핸들로 교체할 수 있습니다.

Production hosting, S3/managed external services, public data publishing, and real secrets are outside the local collaborative-editor validation phase. They require explicit owner review in the `external-account-provisioning-review` phase and must not be provisioned or committed by automation.

The local-validation external review packet is recorded in [External Provisioning Review](docs/external-provisioning-review.md). It lists the GitHub owner, hosting, snapshot durability, room coordination, secret storage, and public-data decisions that remain for the next phase without provisioning any external resource.

## 검증 명령

프론트엔드:

```bash
cd Front-End
npm run build
npm run lint
npm run test
npm run typecheck
```

백엔드:

```bash
cd Back-End
./scripts/verify.sh core
./scripts/verify.sh websocket
cargo check --features full-snapshot-stores
```

기본 백엔드 빌드는 compile fan-out을 줄이기 위해 `memory`, `file`, `sqlite`, `s3`, `managed` snapshot backend만 바로 컴파일합니다. 전체 adapter inventory를 점검할 때만 `--features full-snapshot-stores`를 사용합니다.
`./scripts/preflight.sh publish`는 `.git` metadata 쓰기와 `github.com` DNS를 확인하는 publish readiness 점검이며, local-validation phase transition command에는 포함되지 않습니다.

## 문서 바로가기

| 문서 | 내용 |
| --- | --- |
| [Product Direction](docs/product-direction.md) | 2026년 5월 제품 방향성 지시사항과 2026-06-05 구현 완료 상태 |
| [Onboarding](docs/onboarding.md) | 루트 기준 프론트엔드/백엔드 로컬 실행 흐름 |
| [Testing](docs/testing.md) | 프론트엔드, 백엔드 검증 명령과 CI 상태 |
| [Troubleshooting](docs/troubleshooting.md) | 로컬 API/WS 연동과 validation 문제 해결 |
| [Maintenance](docs/maintenance.md) | 활성 CI/CODEOWNERS 경계, progress gate, 검증 lane |
| [External Provisioning Review](docs/external-provisioning-review.md) | local validation exit을 위한 외부 계정/호스팅/스토리지/secret 경계와 다음 phase owner decision |
| [Phase Gates](docs/instructions/phase-gates.json) | current phase, next phase, transition command, required gate evidence |
| [Management Index](docs/management/INDEX.json) | machine-readable project management docs map |
| [Front-End README](Front-End/README.md) | 프론트엔드 기능, 실행 방법, 디버깅 체크리스트 |
| [Front-End Architecture](Front-End/docs/architecture.md) | 프론트엔드 모듈 구조와 realtime flow |
| [Front-End Setup](Front-End/docs/setup.md) | 프론트엔드 설치, 실행, 환경변수 |
| [Back-End README](Back-End/README.md) | 백엔드 기능, API/WS 개요, snapshot store 운영 정보 |
| [Back-End Architecture](Back-End/docs/architecture.md) | 백엔드 모듈 구조, room registry, persistence 경계 |
| [Back-End API](Back-End/docs/api.md) | REST API, WebSocket, 에러 응답 계약 |
| [Back-End Setup](Back-End/docs/setup.md) | 백엔드 빌드, 실행, 환경변수, store 선택 가이드 |

## 개발 규칙 요약

- 커밋 메시지는 `type(scope): subject` 형식을 사용합니다.
- 프론트엔드 scope는 `ui`, `editor`, `auth`, `api`, `state`, `router`, `styles`, `docs`, `repo`를 기준으로 합니다.
- 백엔드 scope는 `api`, `sync`, `yrs`, `auth`, `db`, `websocket`, `storage`, `config`, `docs`, `repo`를 기준으로 합니다.
- API, WebSocket, 환경변수, snapshot store 계약이 바뀌면 코드와 함께 관련 `docs/` 문서를 갱신합니다.
- 직접 `main`에 누적 작업을 밀어 넣기보다 목적이 분명한 작은 브랜치와 PR 단위를 선호합니다.
