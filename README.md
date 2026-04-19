# Backend Collaborative Server

Axum, Tokio, Yrs 기반으로 시작하는 협업 편집 백엔드 부트스트랩 프로젝트입니다.

## 프로젝트 개요

문서 단위의 실시간 협업 서버를 Rust로 안전하게 시작할 수 있도록 최소 실행 구조를 제공합니다. 현재 단계에서는 HTTP 헬스체크, 문서 생성/조회/삭제 API, 문서별 WebSocket 진입점, 관리용 API 토큰과 문서별 access token 기반 접근 제어, in-memory room registry, 그리고 memory/file/sqlite/heed/jammdb/fjall/persy/native_db/parity_db/pickledb/microkv/redb/sled/rustbreak/yedb/btree_store/s3/managed snapshot 저장 추상화를 포함합니다.

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
- `SnapshotStore` trait 및 memory/file/sqlite/heed/jammdb/fjall/persy/native_db/parity_db/pickledb/microkv/redb/sled/rustbreak/yedb/btree_store/s3/managed adapter
- `RoomLocator` 경계와 config-driven `local`/`static`/`file`/`sqlite`/`managed` ownership resolver
- `RoomCoordinator` 경계와 config-driven `noop`/`logging`/`file`/`sqlite`/`managed` session lifecycle hook

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

non-local owner 때문에 `409 conflict`가 반환될 때는 기존 JSON body와 함께 ingress/proxy 레이어가 바로 사용할 수 있도록 `x-collab-owner-node-id` 헤더가 추가됩니다. `owner.base_url`이 있으면 canonical owner origin을 담은 `x-collab-owner-base-url`, 현재 요청 path/query를 owner origin에 붙인 `x-collab-redirect-location`, 그리고 표준 `Location` 헤더도 함께 실립니다.

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
- `SNAPSHOT_STORE`: `memory`, `file`, `sqlite`, `heed`, `jammdb`, `fjall`, `persy`, `native_db`, `parity_db`, `pickledb`, `microkv`, `redb`, `sled`, `rustbreak`, `yedb`, `btree_store`, `s3`, 또는 `managed`
- `SNAPSHOT_DIR`: `SNAPSHOT_STORE=file`일 때 snapshot JSON 파일을 저장할 디렉터리
- `SNAPSHOT_SQLITE_PATH`: `SNAPSHOT_STORE=sqlite`일 때 snapshot SQLite DB 파일 경로
- `SNAPSHOT_HEED_PATH`: `SNAPSHOT_STORE=heed`일 때 snapshot heed DB 디렉터리 경로
- `SNAPSHOT_JAMMDB_PATH`: `SNAPSHOT_STORE=jammdb`일 때 snapshot jammdb 파일 경로
- `SNAPSHOT_FJALL_PATH`: `SNAPSHOT_STORE=fjall`일 때 snapshot fjall DB 디렉터리 경로
- `SNAPSHOT_PERSY_PATH`: `SNAPSHOT_STORE=persy`일 때 snapshot persy 파일 경로
- `SNAPSHOT_NATIVE_DB_PATH`: `SNAPSHOT_STORE=native_db`일 때 snapshot native_db 파일 경로
- `SNAPSHOT_PARITY_DB_PATH`: `SNAPSHOT_STORE=parity_db`일 때 snapshot parity-db 디렉터리 경로
- `SNAPSHOT_PICKLEDB_PATH`: `SNAPSHOT_STORE=pickledb`일 때 snapshot PickleDB 파일 경로
- `SNAPSHOT_MICROKV_PATH`: `SNAPSHOT_STORE=microkv`일 때 snapshot MicroKV base path. 실제 데이터 파일은 `<path>.kv`로 생성된다
- `SNAPSHOT_REDB_PATH`: `SNAPSHOT_STORE=redb`일 때 snapshot redb 파일 경로
- `SNAPSHOT_SLED_PATH`: `SNAPSHOT_STORE=sled`일 때 snapshot sled DB 디렉터리 경로
- `SNAPSHOT_RUSTBREAK_PATH`: `SNAPSHOT_STORE=rustbreak`일 때 snapshot rustbreak 단일 파일 경로
- `SNAPSHOT_YEDB_PATH`: `SNAPSHOT_STORE=yedb`일 때 snapshot yedb DB 디렉터리 경로
- `SNAPSHOT_BTREE_STORE_PATH`: `SNAPSHOT_STORE=btree_store`일 때 snapshot btree-store 단일 파일 경로
- `SNAPSHOT_S3_ENDPOINT`: `SNAPSHOT_STORE=s3`일 때 S3-compatible object storage endpoint
- `SNAPSHOT_S3_REGION`: S3 signing region
- `SNAPSHOT_S3_BUCKET`: snapshot object를 저장할 bucket 이름
- `SNAPSHOT_S3_PREFIX`: snapshot object key prefix. 기본값은 `snapshots/`
- `SNAPSHOT_S3_ACCESS_KEY_ID`: S3 access key id
- `SNAPSHOT_S3_SECRET_ACCESS_KEY`: S3 secret access key
- `SNAPSHOT_S3_SESSION_TOKEN`: optional session token
- `SNAPSHOT_S3_TIMEOUT_SECS`: S3 object storage HTTP timeout(초)
- `SNAPSHOT_S3_PATH_STYLE`: path-style addressing 사용 여부. 기본값은 `true`
- `SNAPSHOT_MANAGED_BASE_URL`: `SNAPSHOT_STORE=managed`일 때 external snapshot service base URL
- `SNAPSHOT_MANAGED_AUTH_TOKEN`: managed snapshot service에 보낼 optional Bearer 토큰
- `SNAPSHOT_MANAGED_TIMEOUT_SECS`: managed snapshot service HTTP timeout(초)
- `ROOM_LOCATOR`: `local`, `static`, `file`, `sqlite`, 또는 `managed`
- `ROOM_COORDINATOR`: `noop`, `logging`, `file`, `sqlite`, 또는 `managed`
- `ROOM_COORDINATOR_STATE_DIR`: `ROOM_COORDINATOR=file`일 때 active room state JSON 파일을 저장하는 디렉터리이며, `ROOM_LOCATOR=file`은 같은 디렉터리를 읽는다
- `ROOM_COORDINATOR_SQLITE_PATH`: `ROOM_COORDINATOR=sqlite`일 때 lease row를 저장하는 SQLite DB 파일 경로이며, `ROOM_LOCATOR=sqlite`는 같은 DB를 읽는다
- `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS`: `ROOM_COORDINATOR=file|sqlite|managed`일 때 lease heartbeat 갱신 간격(초)
- `ROOM_COORDINATOR_LEASE_TTL_SECS`: `ROOM_COORDINATOR=file|sqlite|managed`일 때 lease 만료 TTL(초)
- `ROOM_COORDINATION_MANAGED_BASE_URL`: `ROOM_LOCATOR=managed` 또는 `ROOM_COORDINATOR=managed`일 때 외부 lease service base URL
- `ROOM_COORDINATION_MANAGED_AUTH_TOKEN`: managed coordination service에 보낼 optional Bearer 토큰
- `ROOM_COORDINATION_MANAGED_TIMEOUT_SECS`: managed coordination service HTTP timeout(초)
- `NODE_ID`: 현재 collaboration node 식별자
- `NODE_BASE_URL`: 현재 collaboration node를 다른 노드에 안내할 때 사용할 canonical origin-only base URL. `ROOM_COORDINATOR=file|sqlite|managed` state와 conflict 응답의 `owner.base_url`에 반영된다.
- `ROOM_OWNER_HINTS_PATH`: `ROOM_LOCATOR=static`일 때 문서별 owner 힌트 JSON 파일 경로

## 현재 범위

- 단일 프로세스 room 관리
- room snapshot 저장/복구 및 idle eviction 정책
- 문서별 WebSocket 협업 세션 진입
- API/앱 상태/설정/에러 모듈 분리
- 테스트 가능한 앱 빌더 제공
- 기본 in-memory snapshot store와 로컬 file/sqlite/heed/jammdb/fjall/persy/native_db/parity_db/pickledb/microkv/redb/sled/rustbreak/yedb/btree_store, S3-compatible object storage, external managed snapshot store 지원
- config-driven room locator local/static/file/sqlite/managed 모드와 room coordinator dry-run logging/file/sqlite/managed state 모드 지원

## 비범위

- 데이터베이스 연동
- 문서 수정용 REST API
- 추가 vendor-specific database durability backend

현재 기본값은 여전히 단일 프로세스다. 다만 `SNAPSHOT_STORE=sqlite`와 `ROOM_LOCATOR=sqlite` / `ROOM_COORDINATOR=sqlite`를 같은 shared SQLite DB 경로에 맞추면, lock-capable storage 위에서는 lease compare-and-swap과 snapshot 내구성을 함께 가져갈 수 있다. `SNAPSHOT_STORE=heed`, `SNAPSHOT_STORE=jammdb`, `SNAPSHOT_STORE=fjall`, `SNAPSHOT_STORE=persy`, `SNAPSHOT_STORE=native_db`, `SNAPSHOT_STORE=parity_db`, `SNAPSHOT_STORE=pickledb`, `SNAPSHOT_STORE=microkv`, `SNAPSHOT_STORE=redb`, `SNAPSHOT_STORE=sled`, `SNAPSHOT_STORE=rustbreak`, `SNAPSHOT_STORE=yedb`, `SNAPSHOT_STORE=btree_store`는 같은 `SnapshotStore` 경계를 vendor-specific embedded database durability로 확장해 로컬 durable restart 복구를 제공한다. `SNAPSHOT_STORE=s3`는 object key 단위 durability를 제공하고, `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 external lease service에 연결하고 `SNAPSHOT_STORE=managed`를 external snapshot service에 연결하면 ownership coordination plane과 snapshot durability plane을 shared SQLite 밖으로도 분리할 수 있다. 현재 저장소는 managed coordination + managed snapshot durability 조합까지 실제 multi-host handoff 회귀 테스트로 검증한다.

현재 `blocked` 상태는 실행 환경 차원의 commit/push/test 제한을 별도 관리하는 정도로 축소됐다. 반면 vendor-specific embedded database durability backend인 heed/jammdb/fjall/persy/native_db/parity_db/pickledb/microkv/redb/sled/rustbreak/yedb/btree_store, S3-compatible object storage durability backend, shared SQLite를 넘어서는 external durability backend 자체, managed-managed owner handoff rehearsal은 이제 회귀 테스트로 검증됐다.

`ROOM_LOCATOR=static`은 외부 coordinator를 대체하지 않는다. 대신 운영자가 문서별 owner 힌트를 선언해 현재 노드 비소유 문서를 조기에 거절하고, 응답 JSON의 `owner.node_id` / optional `owner.base_url` 및 대응 헤더로 upstream 라우팅 결정을 돕는 용도다. 힌트에 없는 문서는 현재 노드 소유로 간주한다.

`ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`의 active room lease state를 읽어 현재 노드 비소유 문서를 거절한다. 이 모드는 `FileRoomCoordinator`가 같은 디렉터리에 남긴 state를 소비하는 best-effort resolver이며, `NODE_BASE_URL`이 설정돼 있으면 conflict 응답 body와 redirect/proxy 헤더 모두에 canonical `owner.base_url`을 실어 upstream 라우팅 결정을 도울 수 있다. stale owner 판단은 file mtime이 아니라 persisted `expires_at`만 기준으로 한다.

`ROOM_LOCATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블에서 active lease row를 읽어 현재 노드 비소유 문서를 거절한다. 이 모드는 `SqliteRoomCoordinator`가 같은 DB에 기록한 lease를 그대로 소비하며, stale owner 판단도 persisted `expires_at`만 기준으로 수행한다. `NODE_BASE_URL`이 설정돼 있으면 conflict 응답 body와 redirect/proxy 헤더 모두에 canonical `owner.base_url`을 실어 실제 ingress redirect/proxy 결정을 도울 수 있다.

`ROOM_LOCATOR=managed`는 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service에서 `GET /v1/leases/:doc_id`를 조회해 현재 노드 비소유 문서를 거절한다. 이 모드는 `ManagedRoomCoordinator`가 같은 service에 기록한 canonical lease record를 그대로 소비하며, stale owner 판단도 persisted `expires_at`만 기준으로 수행한다. `NODE_BASE_URL`이 설정돼 있으면 conflict 응답 body와 redirect/proxy 헤더 모두에 canonical `owner.base_url`을 실어 실제 ingress redirect/proxy 결정을 도울 수 있다.

## 향후 확장 방향

- provider awareness payload 연동
- 외부 저장소 adapter 추가
- provider / frontend editor 연동 계약 고도화
- 추가 vendor-specific database durability backend

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
- `SNAPSHOT_STORE=file`이면 snapshot과 문서 토큰이 `SNAPSHOT_DIR/<doc_id>.json`에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 해당 디렉터리에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=sqlite`이면 snapshot과 문서 토큰이 `SNAPSHOT_SQLITE_PATH` SQLite DB의 `snapshots` 테이블에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 DB catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=heed`이면 snapshot과 문서 토큰이 `SNAPSHOT_HEED_PATH` heed LMDB 디렉터리의 `snapshots` database에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 heed catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=jammdb`이면 snapshot과 문서 토큰이 `SNAPSHOT_JAMMDB_PATH` jammdb 파일의 `snapshots` bucket에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 jammdb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=fjall`이면 snapshot과 문서 토큰이 `SNAPSHOT_FJALL_PATH` fjall DB 디렉터리의 `snapshots` keyspace에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 fjall catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=persy`이면 snapshot과 문서 토큰이 `SNAPSHOT_PERSY_PATH` persy 파일의 `snapshots` segment와 `snapshots_by_doc_id` index에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 persy catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=native_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_NATIVE_DB_PATH` native_db 파일의 primary-key catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 native_db catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=parity_db`이면 snapshot과 문서 토큰이 `SNAPSHOT_PARITY_DB_PATH` parity-db 디렉터리의 ordered `snapshots` column에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 parity-db BTree catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=redb`이면 snapshot과 문서 토큰이 `SNAPSHOT_REDB_PATH` redb 파일의 `snapshots` 테이블에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 redb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=pickledb`이면 snapshot과 문서 토큰이 `SNAPSHOT_PICKLEDB_PATH` PickleDB 파일의 key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 PickleDB catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=microkv`이면 snapshot과 문서 토큰이 `SNAPSHOT_MICROKV_PATH` base path에 대응하는 MicroKV 파일 `<path>.kv`의 key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 MicroKV catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=sled`이면 snapshot과 문서 토큰이 `SNAPSHOT_SLED_PATH` sled DB 디렉터리의 key-value 엔트리에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 sled catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=rustbreak`이면 snapshot과 문서 토큰이 `SNAPSHOT_RUSTBREAK_PATH` rustbreak 단일 파일 catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 rustbreak catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=yedb`이면 snapshot과 문서 토큰이 `SNAPSHOT_YEDB_PATH` yedb 디렉터리의 `snapshots/<doc_id>` key에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 yedb catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=btree_store`이면 snapshot과 문서 토큰이 `SNAPSHOT_BTREE_STORE_PATH` btree-store 단일 파일의 `snapshots` bucket key-value catalog에 저장된다. 기본 local ownership 모드에서는 앱 시작 시 btree-store catalog에서 room을 eager hydrate하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=s3`이면 snapshot과 문서 토큰이 `SNAPSHOT_S3_ENDPOINT` / `SNAPSHOT_S3_BUCKET` / `SNAPSHOT_S3_PREFIX` 조합의 S3 object key `<prefix><doc_id>.json`에 저장된다. startup hydrate는 bucket listing 뒤 각 object를 읽어 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SNAPSHOT_STORE=managed`이면 snapshot과 문서 토큰이 `SNAPSHOT_MANAGED_BASE_URL` 아래의 external snapshot service `GET /v1/snapshots`, `GET|PUT|DELETE /v1/snapshots/:doc_id`를 통해 저장된다. 기본 local ownership 모드에서는 startup catalog lookup 뒤 eager hydrate를 수행하고, distributed ownership 모드에서는 문서 catalog만 읽은 뒤 실제 room restore는 ownership 확인 이후 on-demand로 수행한다.
- `SqliteSnapshotStore`는 row-level upsert로 기존 snapshot을 교체하며, 잘못된 timestamp나 손상된 row는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `HeedSnapshotStore`는 LMDB-backed named database upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `JammdbSnapshotStore`는 single-file B+ tree bucket upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `FjallSnapshotStore`는 LSM-tree keyspace upsert 뒤 `PersistMode::SyncAll`로 journal을 동기화해 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `PersySnapshotStore`는 single-file copy-on-write segment update와 `doc_id -> record_id` replace index를 함께 사용해 기존 snapshot을 교체하며, 손상된 snapshot payload나 dangling index entry는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `ParityDbSnapshotStore`는 ordered BTree column upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- `RedbSnapshotStore`는 key-value upsert로 기존 snapshot을 교체하며, 손상된 snapshot payload는 `GET /api/documents` 카탈로그 생성 중 warning과 함께 건너뛰고 직접 load 시에는 corrupt snapshot 오류로 취급한다.
- 기본 `LocalRoomLocator`는 모든 문서를 현재 프로세스 소유로 해석한다.
- `StaticRoomLocator`는 `ROOM_OWNER_HINTS_PATH`의 문서별 owner 힌트를 읽고, 현재 `NODE_ID`와 다른 owner를 가진 문서에 대해 `409 conflict`와 owner 힌트를 반환한다.
- `FileRoomLocator`는 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`을 읽고, 현재 `NODE_ID`와 다른 node가 active owner로 기록돼 있으며 `expires_at`이 아직 지나지 않았으면 `409 conflict`와 `owner.node_id` 및 optional `owner.base_url`를 반환한다.
- `SqliteRoomLocator`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` row를 읽고, 현재 `NODE_ID`와 다른 node가 active owner로 기록돼 있으며 `expires_at`이 아직 지나지 않았으면 `409 conflict`와 `owner.node_id` 및 optional `owner.base_url`를 반환한다.
- `ROOM_COORDINATOR=noop`은 아무 side effect 없이 통과하고, `ROOM_COORDINATOR=logging`은 `NODE_ID`와 `doc_id` 기준 lifecycle log만 남긴다.
- `ROOM_COORDINATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`에 canonical lease state (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 atomic write로 남기고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 시 compare-and-release 방식으로 정리한다. `NODE_BASE_URL`이 주어지면 이 값도 canonical origin으로 정규화해 함께 기록한다.
- `ROOM_COORDINATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블에 같은 canonical lease state를 upsert하고, active room 동안 background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 시 `node_id + lease_id + epoch` compare-and-delete로 정리한다. `NODE_BASE_URL`이 주어지면 canonical origin으로 정규화한 `base_url`도 함께 기록한다.
- `ROOM_LOCATOR=file`과 `ROOM_COORDINATOR=file`은 같은 `ROOM_COORDINATOR_STATE_DIR`를 공유해야 하며, 멀티 노드에서 쓰려면 각 노드가 같은 디렉터리를 읽고 쓸 수 있어야 한다.
- `ROOM_LOCATOR=sqlite`와 `ROOM_COORDINATOR=sqlite`는 같은 `ROOM_COORDINATOR_SQLITE_PATH`를 공유해야 하며, 실제 owner handoff를 원하면 shared snapshot store도 함께 맞춰야 한다.
- WebSocket 첫 세션 시작과 마지막 세션 종료 시점에 `RoomCoordinator` hook이 호출되도록 런타임 경계가 이미 연결돼 있다.
- 현재 file-backed lease state는 shared filesystem 위에서만 동작하는 best-effort 구현이다. crash 뒤에는 `expires_at` 경과 후에만 stale로 간주된다.
- `SqliteRoomCoordinator`/`SqliteRoomLocator`는 shared SQLite DB에서 transactional lease compare-and-swap을 수행한다. 실제 owner handoff는 `SNAPSHOT_STORE=sqlite` 같은 shared snapshot store와 함께 구성했을 때만 안전하게 활성화해야 한다.
- `ManagedRoomCoordinator`는 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service에 `POST /v1/leases/:doc_id/acquire|renew|release`를 호출해 same canonical lease contract를 유지하고, background heartbeat로 `renewed_at`/`expires_at`을 갱신한 뒤 마지막 세션 종료 시 compare-and-release를 요청한다. `ManagedRoomLocator`는 같은 service의 `GET /v1/leases/:doc_id`를 읽어 non-local owner를 판단한다.
- `ManagedSnapshotStore`는 `SNAPSHOT_MANAGED_BASE_URL` 아래의 external snapshot service에 `GET /v1/snapshots`, `GET|PUT|DELETE /v1/snapshots/:doc_id`를 호출해 document catalog와 full-state Yrs snapshot을 유지한다. optional `SNAPSHOT_MANAGED_AUTH_TOKEN`이 설정되면 모든 요청에 `Authorization: Bearer <token>` 헤더를 붙인다.
- `S3SnapshotStore`는 `SNAPSHOT_S3_ENDPOINT`, `SNAPSHOT_S3_BUCKET`, `SNAPSHOT_S3_PREFIX` 조합 아래의 S3-compatible object storage에 `<prefix><doc_id>.json` object를 저장하고, optional `SNAPSHOT_S3_SESSION_TOKEN`을 포함한 SigV4 요청으로 catalog/list/load/save/delete를 수행한다.
- managed lease service는 `Authorization: Bearer <ROOM_COORDINATION_MANAGED_AUTH_TOKEN>` 헤더를 선택적으로 받을 수 있고, conflict 시 현재 lease record를 `409` body로 반환해야 한다.

## Lease / Heartbeat Coordination Contract

- authoritative coordination store는 최소 `get`, `acquire`, `renew`, `release` 네 동작을 제공해야 한다. 현재 저장소에는 이 계약을 만족하는 SQLite 구현이 포함된다.
- owner record는 최소 `doc_id`, `node_id`, optional `base_url`, `lease_id`, `acquired_at`, `renewed_at`, `expires_at`, `epoch`를 저장해야 한다.
- `owner.base_url`을 노출하는 경우 현재 `StaticRoomLocator`와 같은 규칙을 따라 path/query 없는 origin-only absolute `http://` 또는 `https://` URL만 허용하고, 응답에는 canonical origin (`scheme://authority`)으로 실어야 한다.
- non-local owner conflict 응답은 항상 `x-collab-owner-node-id`를 포함하고, `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 포함해야 한다. redirect URL은 owner origin 뒤에 현재 요청의 path/query를 그대로 붙인 값이어야 한다.
- `lease_id`는 compare-and-swap 기준값이다. `renew`와 `release`는 현재 holder의 `lease_id`와 `node_id`가 모두 일치할 때만 성공해야 한다.
- `epoch`는 lease 재획득마다 증가하는 fencing token이다. snapshot write, redirect metadata, future async side effect는 이 값을 함께 기록해 stale owner가 늦게 도착한 작업을 덮어쓰지 못하게 해야 한다.
- `acquire`는 active lease가 없거나 `expires_at <= now`인 경우에만 새 owner를 기록해야 한다.
- `renew`는 첫 WebSocket 세션 시작 직후 background heartbeat loop에서 주기적으로 실행해야 하며, room이 active인 동안 `expires_at`을 앞으로 민다.
- `release`는 마지막 세션 종료 후 snapshot 저장이 성공한 뒤에만 호출해야 한다. snapshot 저장 실패 시 lease를 즉시 반환하지 말고 TTL 만료까지 기존 owner를 유지해야 한다.
- locator는 `expires_at`이 지나기 전까지는 non-local owner를 authoritative하게 취급하고, 만료 뒤에만 stale owner로 간주해야 한다. 단순 파일 mtime이나 로컬 clock drift만으로 조기 handoff를 결정하지 않는다.
- 권장 기본값은 `heartbeat_interval=10s`, `lease_ttl=30s`, `stale_after_missed_heartbeats=2`다. 즉, owner는 TTL의 절반보다 짧은 간격으로 renew를 시도하고, 다른 노드는 마지막 `expires_at`이 지난 뒤에만 ownership takeover를 시도한다.
- crash 복구 경로는 `owner crash -> renew 중단 -> expires_at 경과 -> 새 owner acquire -> snapshot restore -> room activate` 순서를 따른다. awareness는 재게시 허용 범위로 두고 내구성 복구 대상에는 포함하지 않는다.
- 현재 저장소의 `FileRoomCoordinator`/`FileRoomLocator`는 이 계약의 file-backed 준비 구현을 제공한다. canonical lease record, compare-and-release, background heartbeat renew, `expires_at` 기반 stale 판정은 로컬/shared filesystem 경계에서 검증할 수 있지만 여전히 best-effort rehearsal mode로만 사용해야 한다.
- 현재 저장소의 `SqliteRoomCoordinator`/`SqliteRoomLocator`는 같은 계약을 shared SQLite DB row에 매핑한 authoritative CAS 구현을 제공한다.
- 현재 저장소의 `ManagedRoomCoordinator`/`ManagedRoomLocator`는 external lease service를 쓰는 multi-host coordination backend를 제공하고, `ManagedSnapshotStore`는 같은 방식의 external durability backend를 제공한다. `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 `SNAPSHOT_STORE=sqlite`와 결합한 owner handoff rehearsal, `SNAPSHOT_STORE=managed` 자체의 저장/복구 경계, 그리고 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 `SNAPSHOT_STORE=managed`와 결합한 actual handoff rehearsal까지 모두 회귀 테스트로 검증됐다.

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
