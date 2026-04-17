# Architecture

## Module Structure

- `src/app.rs`: 앱 조립, CORS, HTTP tracing layer
- `src/config.rs`: 환경변수 로딩, 기본값, 값 검증
- `src/state.rs`: 전역 `AppState`, room registry 접근, 허용된 프런트엔드 origin 보관
- `src/errors.rs`: 공통 에러 타입과 HTTP 응답 변환
- `src/routes`: REST endpoint 집합
- `src/collab`: Yrs room registry와 WebSocket 협업 경계
- `src/models`: 문서 placeholder 모델

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

- `Room`에 snapshot provider를 붙일 수 있도록 메타데이터와 CRDT 상태를 분리해 두었다.
- 현재는 메모리 전용이므로 프로세스 재시작 시 문서 상태가 유지되지 않는다.
- 다음 단계에서는 document repository trait과 snapshot serialize/restore 경계를 추가하는 것이 자연스럽다.
