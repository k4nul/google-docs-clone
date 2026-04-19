# API

## Error Response Shape

입력 검증 실패, 인증 실패, 접근 거절은 다음 JSON 구조로 반환된다.

```json
{
  "error": "bad_request",
  "message": "id must be a valid UUID, received `not-a-uuid`"
}
```

room ownership conflict처럼 non-local owner 힌트를 함께 주는 경우에는 `owner` 객체가 추가될 수 있다.

```json
{
  "error": "conflict",
  "message": "document `00000000-0000-0000-0000-000000000000` is owned by another collaboration node",
  "owner": {
    "node_id": "node-b",
    "base_url": "http://127.0.0.1:5001"
  }
}
```

- authoritative coordination resolver도 같은 `owner.node_id` / optional `owner.base_url` shape를 유지해야 한다. 현재 저장소의 `ROOM_LOCATOR=sqlite|managed`도 이 shape를 그대로 사용한다.
- `owner.base_url`이 존재하면 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 하고, 응답에는 canonical origin (`scheme://authority`)으로 반환한다.
- 같은 non-local owner conflict 응답은 JSON body 외에도 `x-collab-owner-node-id` 헤더를 포함한다.
- `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 포함한다.
- `x-collab-redirect-location`과 `Location` 값은 owner origin 뒤에 현재 요청 path/query를 그대로 붙인 absolute URL이어야 한다. 예를 들어 `GET /ws/:doc_id?source=edge`가 remote owner로 거절되면 `Location: https://node-b.internal/ws/:doc_id?source=edge` 형태가 된다.

## HTTP Endpoints

### `GET /api/health`

Response:

```json
{
  "status": "ok",
  "service": "backend",
  "timestamp": "2026-04-17T14:00:00Z"
}
```

### `GET /api/documents`

- `Authorization: Bearer <API_TOKEN>` 헤더가 필요하다.

Response:

```json
{
  "documents": [
    {
      "id": "00000000-0000-0000-0000-000000000000",
      "title": "Document 00000000-0000-0000-0000-000000000000",
      "created_at": "2026-04-17T14:00:00Z",
      "updated_at": "2026-04-17T14:00:00Z"
    }
  ]
}
```

active room과 snapshot store에 남아 있는 persisted document catalog를 합쳐 문서 목록을 반환한다.

### `POST /api/documents`

- `Authorization: Bearer <API_TOKEN>` 헤더가 필요하다.

Request body:

```json
{
  "title": "Design notes"
}
```

- `title`은 선택값이다.
- `title`이 비어 있거나 누락되면 기본 제목 `Document {uuid}`를 사용한다.
- 서버가 새 UUID를 생성하고 해당 문서 room을 메모리 및 snapshot store에 등록한다.
- 응답의 `credentials.access_token`은 이후 문서 상세 조회, 삭제, WebSocket 연결에 사용한다.

Response: `201 Created`

```json
{
  "document": {
    "id": "00000000-0000-0000-0000-000000000000",
    "title": "Design notes",
    "created_at": "2026-04-17T14:00:00Z",
    "updated_at": "2026-04-17T14:00:00Z"
  },
  "credentials": {
    "access_token": "11111111-1111-1111-1111-111111111111"
  }
}
```

### `GET /api/documents/:id`

- `Authorization: Bearer <access_token>` 헤더가 필요하다.
- Path parameter `id`는 UUID 형식이어야 한다.
- 현재 노드 ownership을 `RoomLocator` 경계로 먼저 확인하고, active room이 없으면 snapshot store에서 문서를 on-demand로 복구한다.
- 문서가 없으면 `404` JSON 에러를 반환한다.
- 토큰이 없으면 `401`, 토큰이 문서와 맞지 않으면 `403`을 반환한다.
- `ROOM_LOCATOR=static`, `ROOM_LOCATOR=file`, `ROOM_LOCATOR=sqlite`, `ROOM_LOCATOR=managed`, 또는 동등한 authoritative resolver가 현재 노드 비소유를 보고하면 local restore 대신 `409` JSON 에러로 중단한다. 이때 owner 힌트가 있으면 `owner.node_id`와 optional `owner.base_url`를 함께 반환한다. 기본 `LocalRoomLocator` 구성에서는 이 경로가 발생하지 않는다.
- `ROOM_OWNER_HINTS_PATH`에 선언하는 `owner.node_id`와 `owner.base_url`은 trim 후 저장된다.
- `owner.base_url`은 선택값이지만, 사용할 경우 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 하며 응답에는 canonical origin (`scheme://authority`) 형태로 반환된다.
- `ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`의 active owner lease state를 읽는다. 해당 state에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- `ROOM_LOCATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 `room_leases` row를 읽는다. 해당 row에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- `ROOM_LOCATOR=managed`는 `ROOM_COORDINATION_MANAGED_BASE_URL`의 `GET /v1/leases/:doc_id`를 읽는다. 해당 lease record에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- non-local owner conflict 응답은 `x-collab-owner-node-id` 헤더를 항상 포함한다.
- `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 포함하고, redirect URL은 현재 요청의 path/query를 그대로 유지한다.
- `ROOM_LOCATOR=file`과 `ROOM_LOCATOR=sqlite`는 persisted `expires_at`이 지나기 전까지 다른 node lease를 authoritative하게 취급하고, 만료 뒤에만 stale owner로 간주한다.
- `ROOM_LOCATOR=managed`를 포함한 authoritative coordination resolver는 stale 판단을 `expires_at` 기반 lease 만료로 수행해야 하며, 그 결과를 동일한 `409` owner metadata shape로 노출해야 한다.
- UUID 형식이 아니면 `400`과 JSON 에러 응답을 반환한다.

Response:

```json
{
  "document": {
    "id": "00000000-0000-0000-0000-000000000000",
    "title": "Design notes",
    "created_at": "2026-04-17T14:00:00Z",
    "updated_at": "2026-04-17T14:00:00Z"
  }
}
```

### `DELETE /api/documents/:id`

- `Authorization: Bearer <access_token>` 헤더가 필요하다.
- Path parameter `id`는 UUID 형식이어야 한다.
- 문서가 존재하면 room과 문서 메타데이터를 함께 제거한다.
- 문서가 없으면 `404` JSON 에러 응답을 반환한다.
- 토큰이 없으면 `401`, 토큰이 문서와 맞지 않으면 `403`을 반환한다.

If an active collaboration WebSocket session is still attached to the document, the delete request returns `409 Conflict` with the standard JSON error shape.

Response: `204 No Content`

## WebSocket Path

### `GET /ws/:doc_id`

- `Authorization: Bearer <access_token>` 헤더가 필요하다.
- `doc_id`는 UUID 형식이어야 한다.
- 문서는 먼저 `POST /api/documents`로 생성되어 있어야 한다.
- WebSocket 핸드셰이크의 `Origin` 헤더는 `FRONTEND_ORIGIN`과 정확히 일치해야 한다.
- 같은 `doc_id`를 사용하는 클라이언트는 같은 Yrs broadcast group에 연결된다.
- 현재 노드 ownership을 `RoomLocator` 경계로 먼저 확인하고, active room이 없으면 snapshot store에서 room을 on-demand로 복구한다.
- 내부 `RoomCoordinator` hook은 `ROOM_COORDINATOR` 설정에 따라 `noop`, `logging`, `file`, `sqlite`, 또는 `managed` 모드로 동작하며, 현재 단계에서는 HTTP/WS 계약 자체를 바꾸지 않는다.
- 마지막 WebSocket 세션이 종료되면 최신 snapshot을 저장한 뒤 idle room을 메모리에서 제거한다.
- `doc_id`가 UUID 형식이 아니면 `400` JSON 에러 응답을 반환한다.
- 토큰이 없으면 `401`, 토큰이 문서와 맞지 않으면 `403` JSON 에러 응답을 반환한다.
- 문서가 존재하지 않으면 업그레이드 전에 `404` JSON 에러 응답을 반환한다.
- `Origin` 헤더가 없거나 허용되지 않으면 업그레이드 전에 `403` JSON 에러 응답을 반환한다.
- `ROOM_LOCATOR=static`, `ROOM_LOCATOR=file`, `ROOM_LOCATOR=sqlite`, `ROOM_LOCATOR=managed`, 또는 동등한 authoritative resolver가 현재 노드 비소유를 보고하면 업그레이드 전에 `409` JSON 에러 응답을 반환한다. 이때 owner 힌트가 있으면 `owner.node_id`와 optional `owner.base_url`를 함께 반환한다. 기본 `LocalRoomLocator` 구성에서는 이 경로가 발생하지 않는다.
- `ROOM_OWNER_HINTS_PATH`에 선언하는 `owner.node_id`와 `owner.base_url`은 trim 후 저장된다.
- `owner.base_url`은 선택값이지만, 사용할 경우 path/query 없는 origin-only absolute `http://` 또는 `https://` URL이어야 하며 응답에는 canonical origin (`scheme://authority`) 형태로 반환된다.
- `ROOM_LOCATOR=file`은 `ROOM_COORDINATOR_STATE_DIR/<doc_id>.json`의 active owner lease state를 읽는다. 해당 state에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- `ROOM_LOCATOR=sqlite`는 `ROOM_COORDINATOR_SQLITE_PATH`의 active owner lease row를 읽는다. 해당 row에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- `ROOM_LOCATOR=managed`는 `ROOM_COORDINATION_MANAGED_BASE_URL`의 `GET /v1/leases/:doc_id` 응답을 읽는다. 해당 lease record에 `base_url`이 있으면 이 경로의 conflict 응답도 `owner.node_id`와 함께 canonical `owner.base_url`을 포함한다.
- non-local owner conflict 응답은 `x-collab-owner-node-id` 헤더를 항상 포함한다.
- `owner.base_url`이 있으면 `x-collab-owner-base-url`, `x-collab-redirect-location`, `Location` 헤더도 함께 포함하고, redirect URL은 현재 요청의 path/query를 그대로 유지한다.
- `ROOM_COORDINATOR=file`은 첫 active session에서 file-backed lease를 acquire하고, background heartbeat로 `renewed_at`/`expires_at`을 갱신하며, 마지막 session 종료 뒤 snapshot persist가 끝난 다음 compare-and-release로 lease를 정리한다.
- `ROOM_COORDINATOR=sqlite`는 첫 active session에서 SQLite-backed lease row를 acquire하고, background heartbeat로 `renewed_at`/`expires_at`을 갱신하며, 마지막 session 종료 뒤 snapshot persist가 끝난 다음 `node_id + lease_id + epoch` compare-and-delete로 lease를 정리한다.
- `ROOM_COORDINATOR=managed`는 첫 active session에서 managed lease service `POST /v1/leases/:doc_id/acquire`를 호출하고, background heartbeat로 `POST /v1/leases/:doc_id/renew`를 반복하며, 마지막 session 종료 뒤 snapshot persist가 끝난 다음 `POST /v1/leases/:doc_id/release`로 compare-and-release를 요청한다.
- `ROOM_LOCATOR=file|sqlite|managed`와 동등한 authoritative coordination resolver는 모두 lease 만료 전까지 기존 owner를 authoritative하게 취급하고, `expires_at` 경과 뒤에만 ownership handoff를 허용해야 한다.

## Frontend Contract Notes

- incoming awareness JSON is validated against `AwarenessState`; malformed JSON, blank required identifiers, or invalid `user.color` values are rejected before room awareness state is updated.

- 프런트엔드는 관리 API 호출 시 `Authorization: Bearer <API_TOKEN>`을 넣어야 한다.
- 문서 생성 응답의 `credentials.access_token`을 저장하고, 같은 문서의 상세 조회, 삭제, WebSocket 연결에 재사용해야 한다.
- WebSocket 연결 경로는 문서 ID 단위로 고정하고, 브라우저 origin은 `FRONTEND_ORIGIN`과 일치해야 한다.
- 연결 후 게시하는 Yrs awareness state는 아래 구조를 표준으로 사용한다.

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

- `user.id`, `user.name`, `client.id`, `client.kind`는 trim 후 빈 문자열이면 안 된다.
- `user.color`는 `#RRGGBB` 형식의 hex color를 사용한다.
- `selection`은 선택 사항이며, 커서 위치를 보내지 않을 때는 생략할 수 있다.
- 외부 인증 연동과 사용자 프로필의 source of truth는 아직 별도 계약에 포함하지 않는다.
