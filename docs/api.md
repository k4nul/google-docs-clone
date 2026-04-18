# API

## Error Response Shape

유효성 검증 실패나 접근 거절은 다음 JSON 구조로 반환한다.

```json
{
  "error": "bad_request",
  "message": "id must be a valid UUID, received `not-a-uuid`"
}
```

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

현재 메모리에 생성된 room 기준으로 문서 목록을 반환한다.

### `POST /api/documents`

Request body:

```json
{
  "title": "Design notes"
}
```

- `title`은 선택값이다.
- `title`이 비어 있거나 누락되면 기본 제목 `Document {uuid}`를 사용한다.
- 서버가 새 UUID를 생성하고 해당 문서 room을 메모리에 등록한다.

Response: `201 Created`

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

### `GET /api/documents/:id`

- Path parameter `id`는 UUID 형식이어야 한다.
- 문서가 없으면 `404`와 JSON 에러 응답을 반환한다.
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

- Path parameter `id`는 UUID 형식이어야 한다.
- 문서가 존재하면 room과 문서 메타데이터를 함께 제거한다.
- 문서가 없으면 `404`와 JSON 에러 응답을 반환한다.

Response: `204 No Content`

## WebSocket Path

### `GET /ws/:doc_id`

- `doc_id`는 UUID 형식이어야 한다.
- 문서는 먼저 `POST /api/documents`로 생성되어 있어야 한다.
- WebSocket 핸드셰이크의 `Origin` 헤더는 `FRONTEND_ORIGIN`과 정확히 일치해야 한다.
- 같은 `doc_id`를 사용하는 클라이언트는 같은 Yrs broadcast group에 연결된다.
- 현재 구현은 in-memory room registry를 사용한다.
- `doc_id`가 UUID 형식이 아니면 `400` JSON 에러 응답을 반환한다.
- 문서가 존재하지 않으면 업그레이드 전에 `404` JSON 에러 응답을 반환한다.
- `Origin` 헤더가 없거나 허용되지 않으면 업그레이드 전에 `403` JSON 에러 응답을 반환한다.

## Frontend Contract Notes

- 프런트엔드는 문서 진입 전에 `POST /api/documents`로 새 문서를 만들거나, 이미 생성된 ID에 대해 `GET /api/documents/:id`로 존재 여부를 확인한다.
- WebSocket 연결 경로는 문서 ID 단위로 고정되고, 브라우저 origin은 `FRONTEND_ORIGIN`과 일치해야 한다.
- 인증, 사용자 메타데이터, persistence 관련 필드는 아직 계약에 포함하지 않는다.
