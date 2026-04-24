# Backend Collaborative Server

Axum, Tokio, Yrs 기반의 실시간 협업 편집 백엔드 부트스트랩 프로젝트입니다. 문서 단위 협업 서버를 빠르게 시작할 수 있도록 HTTP API, WebSocket 동기화 경계, room registry, snapshot 저장 추상화, 역할/운영 규칙을 함께 제공합니다.

현재 운영 범위는 단일 프로세스 기준입니다. 다중 프로세스 분산 전략은 문서화되어 있지만, 외부 snapshot store와 owner coordination 저장소가 준비되기 전까지는 한 `doc_id`를 하나의 프로세스만 소유해야 합니다.

## 문서 바로가기

- [Agent Rules](./docs/agent-rules.md)
- [Setup](./docs/setup.md)
- [Architecture](./docs/architecture.md)
- [API](./docs/api.md)
- [Roles](./docs/roles.md)
- [Conventions](./docs/conventions.md)
- [Checklist](./docs/checklist.md)

## 핵심 기능

- `GET /api/health` 헬스체크
- `GET /api/documents` active room과 persisted snapshot을 합친 문서 목록 조회
- `POST /api/documents` 문서 생성 및 room 초기화
- `GET /api/documents/:id` 문서 상세 조회
- `DELETE /api/documents/:id` 문서 삭제
- `GET /ws/:doc_id` 문서별 협업 WebSocket 진입점
- 관리용 `API_TOKEN`과 문서별 `access_token` 기반 접근 제어
- `DashMap` 기반 room registry와 idle room eviction
- `yrs-axum` 기반 Yrs broadcast group 연결
- `SnapshotStore` trait 기반 `memory` / `file` snapshot store 지원
- 앱 재시작 시 snapshot hydrate 및 on-demand room restore
- awareness payload 검증과 표준 JSON 계약 유지

## 모듈 의존성

### 주요 라이브러리

- 웹 서버 / 런타임: `axum`, `tokio`, `tower-http`
- 실시간 협업 / CRDT: `yrs`, `yrs-axum`
- 동시성 상태 관리: `dashmap`
- 직렬화 / 식별자 / 시간: `serde`, `serde_json`, `uuid`, `chrono`
- 설정 / 로깅: `dotenvy`, `tracing`, `tracing-subscriber`
- 에러 처리: `anyhow`, `thiserror`
- 테스트: `axum-test`

### 내부 모듈 책임

- `src/main.rs`: 환경변수 로딩, tracing 초기화, 서버 부팅
- `src/app.rs`: 앱 조립, CORS, trace layer, 라우트 연결
- `src/config.rs`: 환경변수 파싱과 기본값/검증
- `src/state.rs`: 전역 `AppState`, room registry, 허용 origin 보관
- `src/errors.rs`: 공통 에러 타입과 HTTP 응답 변환
- `src/auth.rs`: Bearer 토큰 파싱과 인증 경계
- `src/routes`: health/documents REST endpoint
- `src/collab`: room registry, Yrs protocol, WebSocket 협업 경계
- `src/models`: document/access/awareness 직렬화 모델
- `src/storage`: snapshot store trait과 memory/file adapter

## 역할 분담

- `A` PM / Integration: 범위 정의, 일정 관리, 프런트-백엔드 계약 조율, 통합 우선순위 결정
- `B` Frontend Editor / UI Owner: 편집기 UI, provider 연결, 문서 진입 흐름, 사용자 상호작용 설계
- `C` Backend Realtime / API Owner: HTTP API, room registry, WebSocket 협업, CRDT 서버 구조 유지
- `D` QA / Docs / DevOps Owner: 테스트 실행, 문서 최신화, 실행 절차 검증, 릴리스/운영 준비

## 개발 계획

README에는 `3단계 플랜`으로 개발 계획을 정리합니다.

### 1주차: 기반 정리

- README / `/docs` / 역할 분담 / 작업 규칙 정리
- 로컬 실행 흐름, 환경변수, 인증 토큰 사용법 팀 공통 기준 확정
- 프런트엔드와 연결하는 API / WebSocket 계약 재확인
- snapshot store 동작과 room lifecycle 기준 점검
- 완료 기준: 팀원이 동일한 절차로 `cargo check`, `cargo run`, `cargo test`까지 재현 가능

### 2주차: 통합 안정화

- frontend editor와 백엔드 WebSocket 연결 검증
- awareness payload, 문서 access token, `Origin` 검증 흐름 점검
- file snapshot 복구, idle eviction, delete conflict(`409`) 시나리오 확인
- 회귀 테스트와 smoke test 보강
- 완료 기준: 문서 생성, 조회, 협업 연결, 세션 종료 후 복구 흐름이 테스트와 수동 검증에서 모두 일관되게 동작

### 3주차: 마감 정리

- 남은 버그 수정과 문서 마감
- 배포/운영 전 체크리스트 정리
- PR 리뷰 반영 및 머지 준비
- 역할별 handoff 결과 취합
- 완료 기준: `main` 반영 후보 변경이 문서와 테스트를 포함해 정리되고, 데모 또는 내부 검토가 가능한 상태

### 마감 시점 비범위

- 데이터베이스 연동
- 외부 영속 저장소의 실제 운영 구현
- 문서 수정용 별도 REST API
- 멀티 노드 분산 동기화의 실제 활성화

## API / 협업 흐름 요약

- HTTP base path: `/api`
- Health: `GET /api/health`
- Documents: `GET /api/documents`, `POST /api/documents`, `GET /api/documents/:id`, `DELETE /api/documents/:id`
- Collaboration WebSocket: `GET /ws/:doc_id`

`GET /api/documents`와 `POST /api/documents`는 `Authorization: Bearer <API_TOKEN>` 헤더가 필요합니다. `POST /api/documents` 응답의 `credentials.access_token`은 해당 문서 전용 토큰이며, 이후 `GET /api/documents/:id`, `DELETE /api/documents/:id`, `GET /ws/:doc_id`에서 사용합니다.

WebSocket 연결의 `Origin` 헤더는 `FRONTEND_ORIGIN`과 정확히 일치해야 합니다. active room이 없으면 snapshot store에서 room을 복구하고, 마지막 WebSocket 세션이 종료되면 최신 snapshot을 저장한 뒤 idle room을 메모리에서 제거합니다.

## 환경변수

| 변수 | 설명 | 기본값 |
| --- | --- | --- |
| `HOST` | 서버 바인드 호스트 | `127.0.0.1` |
| `PORT` | 서버 포트 | `4000` |
| `FRONTEND_ORIGIN` | CORS 및 WebSocket `Origin` 허용값 | `http://localhost:3000` |
| `RUST_LOG` | tracing 필터 설정 | `backend=debug,tower_http=info` |
| `API_TOKEN` | 문서 생성/목록 조회용 관리 토큰 | `dev-admin-token` |
| `SNAPSHOT_STORE` | snapshot store 종류 | `memory` |
| `SNAPSHOT_DIR` | `SNAPSHOT_STORE=file`일 때 snapshot 저장 디렉터리 | `./data/snapshots` |

## 실행 방법

### 1. 개발 환경 준비

```bash
cp .env.example .env
cargo check
```

### 2. 서버 실행

```bash
cargo run
```

기본 실행 주소는 `127.0.0.1:4000`입니다. 기본 `FRONTEND_ORIGIN`은 `http://localhost:3000`이므로 프런트엔드 개발 서버를 별도 포트에서 띄우는 흐름을 바로 재현할 수 있습니다.

### 3. 기본 확인 순서

1. `GET /api/health`로 서버 기동 확인
2. `Authorization: Bearer <API_TOKEN>`으로 `POST /api/documents` 호출
3. 응답의 `credentials.access_token` 확보
4. 같은 토큰으로 문서 상세 조회 / 삭제 / WebSocket 연결 수행
5. WebSocket 접속 시 `Origin: <FRONTEND_ORIGIN>` 헤더 포함

### 4. 검증 명령어

```bash
cargo fmt --check
cargo check
cargo test
```

### 5. file snapshot 복구 확인

프로세스 재시작 뒤에도 문서를 유지하려면 아래처럼 설정합니다.

```bash
SNAPSHOT_STORE=file
SNAPSHOT_DIR=./data/snapshots
```

이 상태에서 문서를 생성한 뒤 서버를 재시작하면, 앱 시작 시 snapshot catalog를 읽어 room registry를 hydrate합니다.

## 커밋 규칙

- 커밋 메시지 형식은 반드시 `type(scope): subject`
- 허용 `type`: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `build`, `ci`, `rename`, `remove`
- 권장 `scope`: `api`, `sync`, `yrs`, `auth`, `db`, `websocket`, `storage`, `config`, `docs`, `repo`
- `subject`는 현재형, 소문자 시작, 마침표 없음, 변경 내용을 직접 설명
- 한 커밋에는 한 가지 목적만 담고 리팩토링과 동작 변경을 섞지 않음
- 스키마/API/환경변수/WebSocket 계약 변경 시 관련 문서와 테스트를 같은 작업 안에서 함께 갱신
- 마무리 전에 가능하면 `cargo fmt --check`, `cargo check`, `cargo test`를 실행
- 불확실한 구현은 추측으로 밀어 넣지 말고 `TODO` 또는 blocked 상태로 명시

예시:

```text
feat(websocket): add document room restore on connect
fix(storage): guard corrupt snapshot catalog entries
docs(repo): refresh readme for collaboration workflow
```

## PR 규칙

- `main`에 직접 push하지 않고 기능 브랜치에서 작업 후 PR로 병합
- PR은 하나의 목적만 다루고, 리팩토링과 기능 변경을 섞지 않음
- 제목은 가능하면 커밋 형식과 같은 `type(scope): subject`를 사용
- 본문에는 변경 배경, 핵심 변경점, 영향 범위, 테스트 결과를 포함
- API / WS / 환경변수 변경이 있으면 `README.md`와 관련 `/docs` 문서를 함께 갱신
- 프런트 계약이 바뀌면 `B`, 백엔드 핵심 흐름은 `C`, 문서/검증/운영은 `D`, 범위 조정은 `A`와 리뷰 포인트를 공유
- merge 전 최소한 `cargo fmt --check`, `cargo check`, `cargo test` 결과를 남기는 것을 권장

## 브랜치 규칙

- 모든 작업 브랜치는 `main`에서 분기
- 브랜치 하나당 작업 목적 하나만 담당
- 브랜치 이름은 아래 형식을 권장

```text
<type>/<scope>-<short-kebab-description>
```

예시:

```text
feat/websocket-document-sync
fix/storage-file-snapshot-catalog
docs/repo-readme-refresh
```

- 장기 브랜치보다 짧고 작은 브랜치를 선호
- PR 생성 전 최신 `main` 기준으로 충돌을 정리
- 실험성 작업은 `wip/` 접두사를 사용해도 되지만, merge 전에는 목적이 드러나는 이름으로 정리

## 디렉터리 구조

```text
.
|-- AGENTS.md
|-- README.md
|-- .env.example
|-- Cargo.toml
|-- docs/
|-- src/
|   |-- app.rs
|   |-- auth.rs
|   |-- collab/
|   |-- config.rs
|   |-- errors.rs
|   |-- lib.rs
|   |-- main.rs
|   |-- models/
|   |-- routes/
|   |-- state.rs
|   `-- storage/
`-- tests/
```

## 참고 메모

- snapshot store가 `memory`일 때는 프로세스 재시작 후 문서 상태가 유지되지 않습니다.
- `GET /api/documents/:id`와 `GET /ws/:doc_id`는 active room이 없어도 snapshot store에 문서가 있으면 on-demand restore를 시도합니다.
- 활성 WebSocket 세션이 남아 있는 문서를 삭제하려 하면 `409 conflict`를 반환합니다.
- awareness payload는 `user`, optional `selection`, `client` 구조를 따르며, 잘못된 JSON shape나 필드 값은 협업 상태에 반영되기 전에 거절됩니다.
