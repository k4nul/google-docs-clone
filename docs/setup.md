# Setup

## Build

```bash
cargo check
```

## Run

```bash
cp .env.example .env
cargo run
```

기본 바인드 주소는 `127.0.0.1:4000`입니다.
기본 `FRONTEND_ORIGIN`은 `http://localhost:3000`이므로 로컬 프런트엔드 개발 서버를 별도 포트에서 띄우는 흐름을 바로 재현할 수 있습니다.
기본 `API_TOKEN`은 `dev-admin-token`이며, 개발 환경에서는 이 토큰으로 문서 생성/목록 API를 호출합니다.
기본 `SNAPSHOT_STORE`는 `memory`이며, 프로세스 재시작 뒤에도 문서 snapshot을 유지하려면 `SNAPSHOT_STORE=file`과 `SNAPSHOT_DIR`, `SNAPSHOT_STORE=sqlite`와 `SNAPSHOT_SQLITE_PATH`, 또는 `SNAPSHOT_STORE=managed`와 `SNAPSHOT_MANAGED_BASE_URL`을 함께 설정합니다.
기본 `ROOM_LOCATOR`는 `local`이며, `static`으로 바꾸면 `NODE_ID`와 `ROOM_OWNER_HINTS_PATH`를 함께 설정해 문서별 owner 힌트를 읽습니다. `file`로 바꾸면 `ROOM_COORDINATOR_STATE_DIR` 아래의 active room state JSON을 읽고, `sqlite`로 바꾸면 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블을 읽어 현재 노드 비소유 문서를 거절합니다. `managed`로 바꾸면 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service `GET /v1/leases/:doc_id`를 읽어 현재 노드 비소유 문서를 거절합니다.
기본 `ROOM_COORDINATOR`는 `noop`이며, `logging`으로 바꾸면 room 활성/비활성 lifecycle을 `NODE_ID` 기준 tracing log로만 남깁니다. `file`로 바꾸면 `ROOM_COORDINATOR_STATE_DIR` 아래에 active room lease JSON을 남기고, `sqlite`로 바꾸면 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블에 lease row를 남긴 뒤 `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS` / `ROOM_COORDINATOR_LEASE_TTL_SECS`에 맞춰 heartbeat를 갱신합니다. `managed`로 바꾸면 같은 heartbeat/TTL 정책을 유지한 채 `ROOM_COORDINATION_MANAGED_BASE_URL` 아래의 external lease service `POST /v1/leases/:doc_id/acquire|renew|release`를 호출합니다.

## Test

```bash
./scripts/verify.sh core
./scripts/preflight.sh publish
./scripts/verify.sh websocket
```

- `preflight.sh commit`/`publish`는 stage/commit/push 차단을 점검한다.
- `verify.sh core`는 socket bind나 `.git` 쓰기 가능 여부와 무관한 검증만 실행한다.
- `verify.sh websocket`는 WebSocket/삭제 통합 테스트처럼 socket bind가 필요한 검증만 실행한다.

## Environment Variables

- `HOST`: 서버가 바인드할 호스트명 또는 IP
- `PORT`: 서버 포트
- `FRONTEND_ORIGIN`: CORS 허용 origin
- `RUST_LOG`: tracing subscriber 필터
- `API_TOKEN`: 문서 생성 및 목록 조회용 Bearer 토큰
- `SNAPSHOT_STORE`: `memory`, `file`, `sqlite`, 또는 `managed`
- `SNAPSHOT_DIR`: file snapshot store 루트 디렉터리
- `SNAPSHOT_SQLITE_PATH`: sqlite snapshot store DB 파일 경로
- `SNAPSHOT_MANAGED_BASE_URL`: `SNAPSHOT_STORE=managed`일 때 external snapshot service base URL
- `SNAPSHOT_MANAGED_AUTH_TOKEN`: managed snapshot service에 보낼 optional Bearer 토큰
- `SNAPSHOT_MANAGED_TIMEOUT_SECS`: managed snapshot service HTTP timeout(초)
- `ROOM_LOCATOR`: `local`, `static`, `file`, `sqlite`, 또는 `managed`
- `ROOM_COORDINATOR`: `noop`, `logging`, `file`, `sqlite`, 또는 `managed`
- `ROOM_COORDINATOR_STATE_DIR`: `ROOM_COORDINATOR=file`일 때 active room state JSON 루트 디렉터리이며, `ROOM_LOCATOR=file`이 같은 디렉터리를 읽는다
- `ROOM_COORDINATOR_SQLITE_PATH`: `ROOM_COORDINATOR=sqlite`일 때 active room lease row를 저장하는 SQLite DB 파일 경로이며, `ROOM_LOCATOR=sqlite`가 같은 DB를 읽는다
- `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS`: `ROOM_COORDINATOR=file|sqlite|managed`일 때 lease heartbeat 갱신 간격(초)
- `ROOM_COORDINATOR_LEASE_TTL_SECS`: `ROOM_COORDINATOR=file|sqlite|managed`일 때 lease 만료 TTL(초)
- `ROOM_COORDINATION_MANAGED_BASE_URL`: `ROOM_LOCATOR=managed` 또는 `ROOM_COORDINATOR=managed`일 때 external lease service base URL
- `ROOM_COORDINATION_MANAGED_AUTH_TOKEN`: managed coordination service에 보낼 optional Bearer 토큰
- `ROOM_COORDINATION_MANAGED_TIMEOUT_SECS`: managed coordination service HTTP timeout(초)
- `NODE_ID`: 현재 collaboration node 식별자
- `NODE_BASE_URL`: 현재 collaboration node를 다른 노드에 안내할 때 사용할 canonical origin-only base URL. `ROOM_COORDINATOR=file|sqlite|managed` lease state와 `ROOM_LOCATOR=file|sqlite|managed` conflict 응답의 `owner.base_url`에 반영된다.
- non-local owner `409 conflict`가 발생하면 ingress/proxy가 바로 사용할 수 있도록 `x-collab-owner-node-id` 헤더가 항상 붙고, `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 붙는다.
- `ROOM_OWNER_HINTS_PATH`: `ROOM_LOCATOR=static`일 때 owner hints JSON 파일 경로

## Static Room Locator File

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

- 힌트에 없는 문서는 현재 노드 소유로 간주한다.
- `node_id`와 `base_url`은 trim 후 저장된다.
- `base_url`은 선택값이며, 있으면 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 한다.
- 유효한 `base_url`은 canonical origin (`scheme://authority`) 형태로 non-local owner `409` 응답의 owner metadata로 전달된다.

## File Room Locator State

- `ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`을 읽어 active owner lease를 판정한다.
- 이 모드는 `ROOM_COORDINATOR=file`이 남긴 state를 소비하는 전제이므로, 멀티 노드에서 사용하려면 각 노드가 같은 `ROOM_COORDINATOR_STATE_DIR`를 읽고 쓸 수 있어야 한다.
- `NODE_BASE_URL`이 설정된 노드가 `ROOM_COORDINATOR=file`을 사용하면 lease state에 canonical `base_url`도 기록되고, `ROOM_LOCATOR=file`의 non-local owner `409` 응답에도 `owner.base_url`이 함께 실린다.
- current file-backed state는 canonical lease record (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 저장하고, stale owner 판단은 `expires_at` 기준으로만 수행한다.
- `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS`는 0보다 커야 하고 `ROOM_COORDINATOR_LEASE_TTL_SECS`보다 작아야 한다.
- 이 구현은 shared filesystem 위에서만 best-effort로 동작한다.

## Sqlite Room Locator State

- `ROOM_LOCATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` 테이블을 읽어 active owner lease를 판정한다.
- 이 모드는 `ROOM_COORDINATOR=sqlite`가 남긴 lease row를 그대로 소비하므로, 실제 owner handoff를 원하면 각 노드가 같은 SQLite DB 파일을 lock-capable storage 위에서 공유해야 한다.
- `NODE_BASE_URL`이 설정된 노드가 `ROOM_COORDINATOR=sqlite`를 사용하면 lease row에 canonical `base_url`도 기록되고, `ROOM_LOCATOR=sqlite`의 non-local owner `409` 응답에도 `owner.base_url`이 함께 실린다.
- sqlite lease row도 canonical lease record (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 저장하고, stale owner 판단은 `expires_at` 기준으로만 수행한다.
- `ROOM_COORDINATOR_HEARTBEAT_INTERVAL_SECS`는 0보다 커야 하고 `ROOM_COORDINATOR_LEASE_TTL_SECS`보다 작아야 한다.
- 이 구현은 shared SQLite DB에서 transactional compare-and-swap을 제공하지만, 실제 handoff를 안전하게 쓰려면 `SNAPSHOT_STORE=sqlite` 같은 shared snapshot store를 함께 구성해야 한다.

## Managed Room Coordination Service

- `ROOM_LOCATOR=managed`와 `ROOM_COORDINATOR=managed`는 같은 `ROOM_COORDINATION_MANAGED_BASE_URL`을 공유해야 한다.
- base URL은 absolute `http://` 또는 `https://` URL이어야 하며 query string은 허용하지 않는다. path prefix는 허용되며, 실제 요청은 그 뒤에 `/v1/leases/:doc_id` 및 `/v1/leases/:doc_id/acquire|renew|release`가 붙는다.
- optional `ROOM_COORDINATION_MANAGED_AUTH_TOKEN`이 설정되면 모든 managed lease 요청에 `Authorization: Bearer <token>` 헤더가 붙는다.
- lookup은 `GET /v1/leases/:doc_id`, acquire는 `POST /v1/leases/:doc_id/acquire`, renew는 `POST /v1/leases/:doc_id/renew`, release는 `POST /v1/leases/:doc_id/release`를 사용한다.
- `GET`과 성공한 `acquire`/`renew` 응답은 canonical lease record (`doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`)를 JSON으로 반환해야 한다.
- `acquire` 또는 `renew`/`release` conflict는 `409`와 현재 active lease record를 JSON body로 반환해야 한다.
- 이 구현은 coordination storage를 external service로 분리하지만, 실제 handoff를 안전하게 쓰려면 여전히 `SNAPSHOT_STORE=sqlite` 같은 shared snapshot store를 함께 구성해야 한다.

## Managed Snapshot Service

- `SNAPSHOT_STORE=managed`는 `SNAPSHOT_MANAGED_BASE_URL`을 통해 external snapshot service를 사용한다.
- base URL은 absolute `http://` 또는 `https://` URL이어야 하며 query string은 허용하지 않는다. path prefix는 허용되며, 실제 요청은 그 뒤에 `/v1/snapshots`와 `/v1/snapshots/:doc_id`가 붙는다.
- optional `SNAPSHOT_MANAGED_AUTH_TOKEN`이 설정되면 모든 managed snapshot 요청에 `Authorization: Bearer <token>` 헤더가 붙는다.
- catalog lookup은 `GET /v1/snapshots`, load는 `GET /v1/snapshots/:doc_id`, save는 `PUT /v1/snapshots/:doc_id`, delete는 `DELETE /v1/snapshots/:doc_id`를 사용한다.
- `GET /v1/snapshots` 응답은 `{"documents":[...]}` shape로 document catalog를 반환해야 한다.
- `GET /v1/snapshots/:doc_id` 응답은 `{"document": {...}, "update": [...]}` shape로 full-state snapshot을 반환해야 한다. `document`에는 internal restore에 필요한 `id`, `title`, `created_at`, `updated_at`, `access_token`이 모두 포함돼야 한다.
- `PUT /v1/snapshots/:doc_id`는 같은 JSON payload를 받아 해당 문서 snapshot을 upsert해야 한다.
- `DELETE /v1/snapshots/:doc_id`는 문서 snapshot이 없어도 idempotent하게 성공해도 된다.
- 이 구현은 shared SQLite를 넘어서는 durability surface를 제공하지만, `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`와 결합한 실제 owner handoff rehearsal은 아직 다음 단계다.

## Future Coordination Store Rollout Contract

- 실제 멀티 호스트 handoff를 shared SQLite DB 밖의 coordination plane으로 옮기려면 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`를 같은 외부 lease service에 연결한다.
- 그 backend는 최소 `GET /v1/leases/:doc_id`, `POST /v1/leases/:doc_id/acquire`, `POST /v1/leases/:doc_id/renew`, `POST /v1/leases/:doc_id/release` 네 API를 제공해야 한다.
- lease record는 `doc_id`, `node_id`, optional `base_url`, `lease_id`, `epoch`, `acquired_at`, `renewed_at`, `expires_at`를 저장해야 한다.
- `owner.base_url`을 응답에 노출하려면 현재 static hints와 같은 canonical origin 규칙을 따라야 한다.
- `renew`는 active room 동안 heartbeat loop로 반복되어야 하고, `release`는 마지막 세션 종료 뒤 snapshot 저장이 성공했을 때만 허용된다.
- stale owner 판단은 반드시 `expires_at` 기준으로만 해야 한다. 로컬 파일 timestamp나 프로세스 uptime만으로 handoff를 결정하지 않는다.
- 권장 기본값은 `heartbeat_interval=10s`, `lease_ttl=30s`, `max_missed_heartbeats_before_stale=2`다.
- 현재 저장소에는 filesystem rehearsal용 coordination surface, SQLite-backed authoritative coordination surface, external lease service를 쓰는 managed coordination surface, 그리고 external snapshot service를 쓰는 managed durability surface가 함께 있다. shared snapshot durability 후보로 `SNAPSHOT_STORE=sqlite`를 쓸 수 있고, 외부 durability 후보로 `SNAPSHOT_STORE=managed`를 쓸 수 있다. 이를 `ROOM_LOCATOR=sqlite` / `ROOM_COORDINATOR=sqlite` 또는 `ROOM_LOCATOR=managed` / `ROOM_COORDINATOR=managed`에 연결해 owner lease와 snapshot durability를 분리 구성할 수 있다. 다만 managed coordination과 managed durability를 묶은 actual handoff rehearsal은 아직 future work다.

## Local Development Procedure

1. `.env.example`을 기준으로 로컬 환경값을 준비한다.
2. `cargo check`로 의존성과 컴파일 상태를 먼저 확인한다.
3. `cargo run`으로 서버를 올리고 `/api/health`를 확인한다.
4. `Authorization: Bearer <API_TOKEN>`으로 `POST /api/documents`를 호출해 문서를 만들고 응답의 `access_token`을 확보한다.
5. 문서 상세 조회, 삭제, WebSocket 연결에는 `Authorization: Bearer <access_token>`을 사용한다.
6. WebSocket 접속 시 `Origin` 헤더를 `FRONTEND_ORIGIN`과 맞춰 `/ws/:doc_id`에 접속한다.
7. 작업 시작 전에 `./scripts/verify.sh core`로 코드 경로를 먼저 검증하고, publish 전에는 `./scripts/preflight.sh publish`, WebSocket 검증 전에는 `./scripts/preflight.sh websocket`로 환경 차단을 확인한다.
8. 작업 마무리 전 `./scripts/verify.sh core`를 다시 실행하고, socket bind 가능한 러너에서는 `./scripts/verify.sh websocket`까지 실행한다.
9. `ROOM_LOCATOR=static`을 쓰는 경우에는 `NODE_ID`와 `ROOM_OWNER_HINTS_PATH`를 함께 맞추고, non-local owner 문서에 대해 `409 conflict`, `owner` metadata, `x-collab-owner-node-id` 헤더가 반환되는지 확인한다. `ROOM_LOCATOR=file`을 쓰는 경우에는 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json` state를 준비하고, `ROOM_LOCATOR=sqlite`를 쓰는 경우에는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` row를 준비한다. `ROOM_LOCATOR=managed`를 쓰는 경우에는 managed lease service `GET /v1/leases/:doc_id`가 current lease record를 반환하도록 준비한 뒤 같은 응답이 `owner.node_id`, optional `owner.base_url`, optional `x-collab-redirect-location`/`Location` 기준으로 반환되는지 확인한다.
10. 재시작 복구를 검증하려면 `SNAPSHOT_STORE=file`, `SNAPSHOT_STORE=sqlite`, 또는 `SNAPSHOT_STORE=managed`로 서버를 띄운 뒤 문서를 만든 다음 프로세스를 재시작해 같은 문서 ID가 hydrate되는지 확인한다. 단, `ROOM_LOCATOR != local` 또는 `ROOM_COORDINATOR=file|sqlite|managed` 같은 distributed ownership 모드에서는 startup eager hydrate 대신 ownership 확인 뒤 on-demand restore가 일어나므로, 실제 owner handoff 검증은 snapshot store와 authoritative coordination backend를 함께 맞춘 뒤 이전 owner 종료 후 새 owner의 detail/WS 진입이 최신 snapshot을 복구하는지 확인해야 한다.
