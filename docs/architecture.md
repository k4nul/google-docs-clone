# Architecture

## Module Structure

- `src/app.rs`: 앱 조립, CORS, HTTP tracing layer
- `src/config.rs`: 환경변수 로딩, 기본값, 값 검증
- `src/state.rs`: 전역 `AppState`, room registry 접근, 허용된 프런트엔드 origin 보관
- `src/errors.rs`: 공통 에러 타입과 HTTP 응답 변환
- `src/routes`: REST endpoint 집합
- `src/collab`: Yrs room registry와 WebSocket 협업 경계
- `src/models`: 문서 placeholder 모델
- `src/storage`: snapshot store trait과 기본 in-memory adapter

## Request Flow

1. `main.rs`가 환경변수를 읽고 tracing을 초기화한다.
2. `app.rs`가 `AppState`와 라우트를 조합해 `Router`를 만든다.
3. `/api/*` 요청은 `routes` 모듈로 들어간다.
4. route handler는 `AppState`를 통해 registry를 조회하고 JSON 응답을 반환한다.

## WebSocket / Collab Flow

1. 클라이언트가 `GET /ws/:doc_id`로 업그레이드를 요청한다.
2. `collab/ws.rs`가 `doc_id` 형식을 검증하고 `Origin` 헤더가 `FRONTEND_ORIGIN`과 일치하는지 확인한다.
3. 검증이 통과하면 `doc_id`에 해당하는 room을 조회하거나 생성한다.
4. room은 `Yrs Doc`, `Awareness`, lazy `BroadcastGroup`을 가진다.
5. 업그레이드된 socket은 `AxumSink` / `AxumStream`으로 감싸진다.
6. `BroadcastGroup::subscribe`가 해당 문서의 협업 세션을 처리한다.

## Room Registry Structure

- 저장소는 `DashMap<Uuid, Arc<Room>>`
- key는 문서 ID
- value는 placeholder 문서 메타데이터와 Yrs awareness, lazy broadcast group
- 문서 API와 WebSocket 엔트리포인트가 같은 registry를 공유한다.

## Persistence Extension Points

- `SnapshotStore` trait이 `load/save/delete` 경계를 정의하고, `RoomRegistry`가 이 trait에만 의존한다.
- `Room::snapshot()`은 Yrs document를 full-state update로 직렬화하고 문서 metadata를 함께 저장한다.
- `Room::from_snapshot()`은 저장된 update를 다시 apply해 room을 복구한다.
- 현재는 `InMemorySnapshotStore`만 연결되며, future adapter는 같은 trait으로 file/db/object storage를 대체할 수 있다.
- `GET /api/documents/:id`와 `GET /ws/:doc_id`는 active room이 없어도 snapshot store에서 문서를 복구한 뒤 처리할 수 있다.
- startup hydration과 eviction-after-save 정책은 아직 없으므로 persisted document catalog 구성은 다음 단계로 남겨 둔다.
