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
기본 `SNAPSHOT_STORE`는 `memory`이며, 프로세스 재시작 뒤에도 문서 snapshot을 유지하려면 `SNAPSHOT_STORE=file`과 `SNAPSHOT_DIR`를 함께 설정합니다.
기본 `ROOM_LOCATOR`는 `local`이며, `static`으로 바꾸면 `NODE_ID`와 `ROOM_OWNER_HINTS_PATH`를 함께 설정해 문서별 owner 힌트를 읽습니다.
기본 `ROOM_COORDINATOR`는 `noop`이며, `logging`으로 바꾸면 room 활성/비활성 lifecycle을 `NODE_ID` 기준 tracing log로만 남깁니다. `file`로 바꾸면 `ROOM_COORDINATOR_STATE_DIR` 아래에 active room state JSON을 남깁니다.

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
- `SNAPSHOT_STORE`: `memory` 또는 `file`
- `SNAPSHOT_DIR`: file snapshot store 루트 디렉터리
- `ROOM_LOCATOR`: `local` 또는 `static`
- `ROOM_COORDINATOR`: `noop`, `logging`, 또는 `file`
- `ROOM_COORDINATOR_STATE_DIR`: `ROOM_COORDINATOR=file`일 때 active room state JSON 루트 디렉터리
- `NODE_ID`: 현재 collaboration node 식별자
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

## Local Development Procedure

1. `.env.example`을 기준으로 로컬 환경값을 준비한다.
2. `cargo check`로 의존성과 컴파일 상태를 먼저 확인한다.
3. `cargo run`으로 서버를 올리고 `/api/health`를 확인한다.
4. `Authorization: Bearer <API_TOKEN>`으로 `POST /api/documents`를 호출해 문서를 만들고 응답의 `access_token`을 확보한다.
5. 문서 상세 조회, 삭제, WebSocket 연결에는 `Authorization: Bearer <access_token>`을 사용한다.
6. WebSocket 접속 시 `Origin` 헤더를 `FRONTEND_ORIGIN`과 맞춰 `/ws/:doc_id`에 접속한다.
7. 작업 시작 전에 `./scripts/verify.sh core`로 코드 경로를 먼저 검증하고, publish 전에는 `./scripts/preflight.sh publish`, WebSocket 검증 전에는 `./scripts/preflight.sh websocket`로 환경 차단을 확인한다.
8. 작업 마무리 전 `./scripts/verify.sh core`를 다시 실행하고, socket bind 가능한 러너에서는 `./scripts/verify.sh websocket`까지 실행한다.
9. `ROOM_LOCATOR=static`을 쓰는 경우에는 `NODE_ID`와 `ROOM_OWNER_HINTS_PATH`를 함께 맞추고, non-local owner 문서에 대해 `409 conflict`와 `owner` metadata가 반환되는지 확인한다.
10. 재시작 복구를 검증하려면 `SNAPSHOT_STORE=file`로 서버를 띄운 뒤 문서를 만든 다음 프로세스를 재시작해 같은 문서 ID가 hydrate되는지 확인한다.
