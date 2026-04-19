# Backend Collaborative Server

Axum, Tokio, Yrs 기반으로 시작하는 협업 편집 백엔드 부트스트랩 프로젝트입니다.

## 프로젝트 개요

문서 단위의 실시간 협업 서버를 Rust로 안전하게 시작할 수 있도록 최소 실행 구조를 제공합니다. 현재 단계에서는 HTTP 헬스체크, 문서 생성/조회/삭제 API, 문서별 WebSocket 진입점, 관리용 API 토큰과 문서별 access token 기반 접근 제어, in-memory room registry, 그리고 memory/file/sqlite snapshot 저장 추상화를 포함합니다.

## 해결하려는 문제

협업 편집 시스템은 HTTP API, WebSocket 세션, 문서별 상태 관리, CRDT 동기화 경계를 초기에 잘 나누지 않으면 빠르게 복잡해집니다. 이 레포는 그 복잡도를 초기에 제어하기 위해 compile-safe한 기본 골격과 문서화를 함께 제공합니다.

## 핵심 기능

- `GET /api/health` 헬스체크
- `GET /api/documents` active room과 persisted snapshot을 합친 문서 목록 조회
- `POST /api/documents` 문서 생성 및 room 초기화
- `GET /api/documents/:id` 기존 문서 상세 조회
- `DELETE /api/documents/:id` 문서 및 room 제거
- `GET /ws/:doc_id` 문서별 협업 WebSocket 진입점
- 관리용 API 토큰과 문서별 access token 기반 인증/접근 제어
- `DashMap` 기반 room registry와 idle room eviction
- `yrs-axum` 기반 broadcast group 연결
- `SnapshotStore` trait 및 memory/file/sqlite adapter
- `RoomLocator` 경계와 config-driven `local`/`static`/`file` ownership resolver
- `RoomCoordinator` 경계와 config-driven `noop`/`logging`/`file` session lifecycle hook

## 기술 스택

- Rust
- Axum
- Tokio
- Yrs
- yrs-axum
- DashMap
- Tracing / tracing-subscriber

## 로컬 실행 방법

```bash
cp .env.example .env
cargo run
```

기본 실행 주소는 `127.0.0.1:4000`입니다. 기본 `FRONTEND_ORIGIN`은 `http://localhost:3000`으로 설정되어 있어 로컬 프런트엔드 개발 서버와 포트가 겹치지 않습니다.

## 검증 흐름

```bash
./scripts/verify.sh core
./scripts/preflight.sh publish
./scripts/verify.sh websocket
```

- `./scripts/preflight.sh commit`는 `.git` 메타데이터 쓰기 가능 여부를 먼저 확인해 commit/stage 차단을 조기에 드러낸다.
- `./scripts/preflight.sh publish`는 여기에 `github.com` DNS 확인을 더해 push 가능성을 사전에 확인한다.
- `./scripts/preflight.sh websocket`는 socket bind가 필요한 WebSocket 검증 레인이 현재 러너에서 실행 가능한지 probe test로 확인한다.
- `./scripts/verify.sh core`는 `cargo fmt --check`, `cargo check --locked`, 그리고 socket bind가 필요 없는 테스트만 실행한다. commit/push 가능 여부와는 분리돼 있어 sandbox 환경에서도 core 검증을 막지 않는다.
- `./scripts/verify.sh websocket`는 socket bind가 필요한 WebSocket/삭제 통합 테스트만 분리 실행한다.
- socket-required 테스트를 새로 추가하면 `scripts/verify.sh`의 core skip 목록과 websocket lane을 함께 갱신한다.

## API/WS 개요

- HTTP base path: `/api`
- Health: `GET /api/health`
- Documents: `GET /api/documents`, `POST /api/documents`, `GET /api/documents/:id`, `DELETE /api/documents/:id`
- Collaboration WebSocket: `GET /ws/:doc_id`

`GET /api/documents`와 `POST /api/documents`는 `Authorization: Bearer <API_TOKEN>` 헤더가 필요합니다. `POST /api/documents` 응답에는 해당 문서 전용 `access_token`이 포함되며, 이후 `GET /api/documents/:id`, `DELETE /api/documents/:id`, `GET /ws/:doc_id`는 모두 `Authorization: Bearer <access_token>` 헤더가 필요합니다. 존재하지 않는 문서 ID로 상세 조회나 WebSocket 연결을 시도하면 `404`를 반환합니다. 활성 협업 WebSocket 세션이 남아 있는 문서를 삭제하려 하면 `409 conflict`를 반환합니다. WebSocket 핸드셰이크의 `Origin` 헤더는 `FRONTEND_ORIGIN`과 정확히 일치해야 합니다.

## 폴더 구조 요약

```text
.
|-- AGENTS.md
|-- README.md
|-- .env.example
|-- docs/
|-- scripts/
|-- src/
|   |-- app.rs
|   |-- collab/
|   |-- config.rs
|   |-- errors.rs
|   |-- models/
|   |-- routes/
|   |-- state.rs
|   |-- lib.rs
|   `-- main.rs
`-- tests/
```

## 환경변수 요약

- `HOST`: 서버 바인드 호스트
- `PORT`: 서버 바인드 포트
- `FRONTEND_ORIGIN`: CORS 허용 origin
- `RUST_LOG`: tracing 필터 설정
- `API_TOKEN`: 문서 생성/목록 조회용 관리 토큰
- `SNAPSHOT_STORE`: `memory`, `file`, 또는 `sqlite`
- `SNAPSHOT_DIR`: `SNAPSHOT_STORE=file`일 때 snapshot JSON 파일을 저장할 디렉터리
- `SNAPSHOT_SQLITE_PATH`: `SNAPSHOT_STORE=sqlite`일 때 snapshot SQLite DB 파일 경로
- `ROOM_LOCATOR`: `local`, `static`, 또는 `file`
- `ROOM_COORDINATOR`: `noop`, `logging`, 또는 `file`
- `ROOM_COORDINATOR_STATE_DIR`: `ROOM_COORDINATOR=file`일 때 active room state JSON 파일을 저장하는 디렉터리이며, `ROOM_LOCATOR=file`은 같은 디렉터리를 읽는다
- `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS`: `ROOM_COORDINATOR=file`일 때 lease heartbeat 갱신 간격(초)
- `ROOM_COORDINATOR_LEASE_TTL_SECS`: `ROOM_COORDINATOR=file`일 때 lease 만료 TTL(초)
- `NODE_ID`: 현재 collaboration node 식별자
- `ROOM_OWNER_HINTS_PATH`: `ROOM_LOCATOR=static`일 때 문서별 owner 힌트 JSON 파일 경로

## 현재 범위

- 단일 프로세스 room 관리
- room snapshot 저장/복구 및 idle eviction 정책
- 문서별 WebSocket 협업 세션 진입
- API/앱 상태/설정/에러 모듈 분리
- 테스트 가능한 앱 빌더 제공
- 기본 in-memory snapshot store와 로컬 file/sqlite snapshot store 지원
- config-driven room locator local/static/file 모드와 room coordinator dry-run logging/file state 모드 지원

## 비범위

- 데이터베이스 연동
- 외부 영속 저장소 구현
- 문서 수정용 REST API
- 멀티 노드 분산 동기화

현재 문서화된 분산 전략 검토 결과도 실제 운영 범위는 여전히 단일 프로세스다. `SNAPSHOT_STORE=sqlite`로 DB-backed snapshot 내구성을 올릴 수는 있지만, room owner coordination 저장소가 아직 file rehearsal 경계에 머물러 있으므로 한 `doc_id`를 하나의 프로세스만 소유해야 한다.

현재 `blocked` 상태는 둘로 나눠 관리한다. 멀티 프로세스 room 분산 지원은 roadmap 차원의 blocked 항목이고, 로컬 commit/push/test 실패는 실행 환경 차원의 blocked 항목으로 별도 취급한다.

`ROOM_LOCATOR=static`은 외부 coordinator를 대체하지 않는다. 대신 운영자가 문서별 owner 힌트를 선언해 현재 노드 비소유 문서를 조기에 거절하고, 응답 JSON의 `owner.node_id` / optional `owner.base_url`로 upstream 라우팅 결정을 돕는 용도다. 힌트에 없는 문서는 현재 노드 소유로 간주한다.

`ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`의 active room lease state를 읽어 현재 노드 비소유 문서를 거절한다. 이 모드는 `FileRoomCoordinator`가 같은 디렉터리에 남긴 state를 소비하는 best-effort resolver이며, 현재 포맷에는 `base_url`이 없으므로 conflict 응답에는 `owner.node_id`만 포함된다. stale owner 판단은 file mtime이 아니라 persisted `expires_at`만 기준으로 한다.

## 향후 확장 방향

- provider awareness payload 연동
- 외부 저장소 adapter 추가
- provider / frontend editor 연동 계약 고도화
- 아래 `Lease / Heartbeat Coordination Contract`에 맞는 authoritative coordination adapter 구현
- shared filesystem state 대신 외부 coordination 저장소와 shared snapshot store를 연결해 실제 owner handoff 활성화

## Snapshot Restore / Eviction Policy

- 문서 생성 시 초기 snapshot을 저장하고 active room을 메모리에 등록한다.
- `GET /api/documents`는 active room이 없어도 snapshot store에 남아 있는 문서를 카탈로그로 반환한다.
- `GET /api/documents/:id`와 `GET /ws/:doc_id`는 먼저 `RoomLocator`로 현재 노드 ownership을 확인한 뒤, active room이 없으면 snapshot store에서 room을 on-demand로 복구한다.
- WebSocket 세션이 종료될 때마다 room의 active session 수를 감소시키고, 마지막 세션이 닫히면 최신 snapshot을 저장한 뒤 room을 메모리에서 제거한다.
- 문서가 삭제된 경우에는 snapshot과 active room을 함께 제거한다. 활성 WebSocket 세션이 남아 있으면 삭제를 거절하고 `409 conflict`를 반환한다.
- `SNAPSHOT_STORE=file`일 때 손상된 snapshot 파일은 startup hydrate와 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛴다. 해당 문서를 직접 복구하려고 로드하면 여전히 corrupt snapshot 오류로 취급한다.
- `SNAPSHOT_STORE=file` 저장은 같은 디렉터리의 임시 파일 작성 후 `rename`으로 마무리해, 저장 도중 프로세스가 중단돼도 마지막 정상 snapshot을 바로 덮어쓰지 않도록 한다.
- interrupted save가 남긴 `.tmp` 파일은 `FileSnapshotStore` 초기화 시점에 정리되고, catalog/hydrate는 계속 `.json` snapshot만 복구 대상으로 취급한다.
- 문서 삭제 시 `FileSnapshotStore`는 본 snapshot과 같은 문서 ID를 가진 stale `.tmp` 파일도 함께 정리한다.
- `SNAPSHOT_STORE=file`이면 snapshot과 문서 토큰이 `SNAPSHOT_DIR/<doc_id>.json`에 저장되고, 앱 시작 시 해당 디렉터리에서 문서를 hydrate한다.
- `SNAPSHOT_STORE=sqlite`이면 snapshot과 문서 토큰이 `SNAPSHOT_SQLITE_PATH` SQLite DB의 `snapshots` 테이블에 저장되고, 앱 시작 시 DB catalog에서 문서를 hydrate한다.
- `SqliteSnapshotStore`는 row-level upsert로 기존 snapshot을 교체하며, 잘못된 timestamp나 손상된 row는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- 기본 `LocalRoomLocator`는 모든 문서를 현재 프로세스 소유로 해석한다.
- `StaticRoomLocator`는 `ROOM_OWNER_HINTS_PATH`의 문서별 owner 힌트를 읽고, 현재 `NODE_ID`와 다른 owner를 가진 문서에 대해 `409 conflict`와 owner 힌트를 반환한다.
- `FileRoomLocator`는 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`을 읽고, 현재 `NODE_ID`와 다른 node가 active owner로 기록돼 있으며 `expires_at`이 아직 지나지 않았으면 `409 conflict`와 `owner.node_id`를 반환한다.
- `ROOM_COORDINATOR=noop`은 아무 side effect 없이 통과하고, `ROOM_COORDINATOR=logging`은 `NODE_ID`와 `doc_id` 기준 lifecycle log만 남긴다.
- `ROOM_COORDINATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`에 canonical lease state (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 atomic write로 남기고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 시 compare-and-release 방식으로 정리한다.
- `ROOM_LOCATOR=file`과 `ROOM_COORDINATOR=file`은 같은 `ROOM_COORDINATOR_STATE_DIR`를 공유해야 하며, 멀티 노드에서 쓰려면 각 노드가 같은 디렉터리를 읽고 쓸 수 있어야 한다.
- WebSocket 첫 세션 시작과 마지막 세션 종료 시점에 `RoomCoordinator` hook이 호출되도록 런타임 경계가 이미 연결돼 있다.
- 현재 file-backed lease state는 shared filesystem 위에서만 동작하는 best-effort 구현이다. crash 뒤에는 `expires_at` 경과 후에만 stale로 간주되며, `SNAPSHOT_STORE=sqlite`를 써도 실제 authoritative handoff는 여전히 외부 coordination backend가 준비된 뒤에만 활성화해야 한다.

## Lease / Heartbeat Coordination Contract

- authoritative coordination store는 최소 `get`, `acquire`, `renew`, `release` 네 동작을 제공해야 한다.
- owner record는 최소 `doc_id`, `node_id`, optional `base_url`, `lease_id`, `acquired_at`, `renewed_at`, `expires_at`, `epoch`를 저장해야 한다.
- `owner.base_url`을 노출하는 경우 현재 `StaticRoomLocator`와 같은 규칙을 따라 path/query 없는 origin-only absolute `http://` 또는 `https://` URL만 허용하고, 응답에는 canonical origin (`scheme://authority`)으로 실어야 한다.
- `lease_id`는 compare-and-swap 기준값이다. `renew`와 `release`는 현재 holder의 `lease_id`와 `node_id`가 모두 일치할 때만 성공해야 한다.
- `epoch`는 lease 재획득마다 증가하는 fencing token이다. snapshot write, redirect metadata, future async side effect는 이 값을 함께 기록해 stale owner가 늦게 도착한 작업을 덮어쓰지 못하게 해야 한다.
- `acquire`는 active lease가 없거나 `expires_at <= now`인 경우에만 새 owner를 기록해야 한다.
- `renew`는 첫 WebSocket 세션 시작 직후 background heartbeat loop에서 주기적으로 실행해야 하며, room이 active인 동안 `expires_at`을 앞으로 민다.
- `release`는 마지막 세션 종료 후 snapshot 저장이 성공한 뒤에만 호출해야 한다. snapshot 저장 실패 시 lease를 즉시 반환하지 말고 TTL 만료까지 기존 owner를 유지해야 한다.
- locator는 `expires_at`이 지나기 전까지는 non-local owner를 authoritative하게 취급하고, 만료 뒤에만 stale owner로 간주해야 한다. 단순 파일 mtime이나 로컬 clock drift만으로 조기 handoff를 결정하지 않는다.
- 권장 기본값은 `heartbeat_interval=10s`, `lease_ttl=30s`, `stale_after_missed_heartbeats=2`다. 즉, owner는 TTL의 절반보다 짧은 간격으로 renew를 시도하고, 다른 노드는 마지막 `expires_at`이 지난 뒤에만 ownership takeover를 시도한다.
- crash 복구 경로는 `owner crash -> renew 중단 -> expires_at 경과 -> 새 owner acquire -> snapshot restore -> room activate` 순서를 따른다. awareness는 재게시 허용 범위로 두고 내구성 복구 대상에는 포함하지 않는다.
- 현재 저장소의 `FileRoomCoordinator`/`FileRoomLocator`는 이 계약의 file-backed 준비 구현을 제공한다. canonical lease record, compare-and-release, background heartbeat renew, `expires_at` 기반 stale 판정은 로컬/shared filesystem 경계에서 검증할 수 있지만, external backend의 authoritative CAS 보장과 shared snapshot store가 없으므로 여전히 best-effort rehearsal mode로만 사용해야 한다.

## Static Room Owner Hints

`ROOM_LOCATOR=static`일 때 `ROOM_OWNER_HINTS_PATH`는 아래 구조의 JSON 파일을 가리킨다.

```json
{
  "documents": {
    "00000000-0000-0000-0000-000000000000": {
      "node_id": "node-b",
      "base_url": "http://127.0.0.1:5001"
    }
  }
}
```

- `documents`에 없는 문서는 현재 노드 소유로 간주한다.
- `node_id`는 비어 있으면 안 된다.
- `node_id`와 `base_url`은 trim 후 저장된다.
- `base_url`은 선택값이며, 있으면 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 한다.
- 유효한 `base_url`은 canonical origin (`scheme://authority`) 형태로 non-local owner conflict 응답의 `owner.base_url`에 실린다.

## Awareness Metadata Contract

Non-null awareness payloads are validated against `AwarenessState` on the WebSocket collaboration path. Malformed JSON and invalid field values are rejected before shared room awareness state is mutated.

WebSocket 연결 이후 클라이언트가 게시하는 Yrs awareness state는 아래 JSON 구조를 표준으로 사용한다.

```json
{
  "user": {
    "id": "user-7",
    "name": "Kim",
    "color": "#1f6feb"
  },
  "selection": {
    "anchor": 3,
    "head": 11
  },
  "client": {
    "id": "session-3",
    "kind": "editor"
  }
}
```

- `user.id`, `user.name`, `client.id`, `client.kind`는 비어 있으면 안 된다.
- `user.color`는 `#RRGGBB` 형식의 6자리 hex color를 사용한다.
- `selection`은 선택 사항이며, 커서/선택 범위를 공유하지 않을 때는 생략할 수 있다.
- 서버는 이 구조를 canonical contract로 문서화하고, 현재 단계에서는 awareness payload를 그대로 중계한다.
