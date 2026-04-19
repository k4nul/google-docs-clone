# Conventions

## Module Naming

- 앱 조립은 `app`
- 설정은 `config`
- 공통 에러는 `errors`
- 라우트는 `routes`
- 실시간 협업은 `collab`
- 직렬화 모델은 `models`
- 저장 경계는 `storage`

## Error Handling

- 정상 흐름 제어에 panic을 사용하지 않는다.
- 공통 실패 타입은 `AppError`와 `AppResult`를 사용한다.
- 외부 실패 응답은 JSON `error` / `message` 형태로 고정하고 안정적인 메시지로 제한한다.
- path 파라미터 검증 실패와 WebSocket origin 거절도 공통 에러 타입으로 매핑한다.
- 인증 누락은 `401 unauthorized`, 토큰 불일치는 `403 forbidden`으로 구분한다.
- 활성 협업 세션이 남아 있는 문서를 삭제하려는 요청은 `409 conflict`로 응답한다.

## Config Rules

- 환경변수 파싱은 `config.rs`에 모은다.
- 기본값은 코드에 명시하되 빈 문자열과 잘못된 포맷은 에러로 처리한다.
- 새 환경변수를 추가하면 `.env.example`, `README.md`, `docs/api.md` 또는 `docs/setup.md`를 함께 갱신한다.
- 관리용 토큰과 문서별 토큰은 응답/로그에 불필요하게 노출하지 않는다.

## Route Rules

- incoming WebSocket awareness payloads must be validated in the `collab` boundary before they reach shared room state.

- HTTP route는 `src/routes` 아래에 둔다.
- WebSocket 협업 엔트리포인트는 `src/collab/ws.rs`에 둔다.
- route handler는 가능한 한 얇게 유지하고 상태 조회/생성은 registry 계층에 위임한다.
- 문서 detail/delete나 WebSocket처럼 room-affined 요청은 `get_or_restore` 전에 `RoomLocator` 경계를 먼저 통과시킨다.
- awareness payload 계약은 `src/models/awareness.rs`에 두고, 클라이언트가 그대로 재사용할 수 있는 camelCase JSON shape를 유지한다.

## Logging / Tracing Rules

- 요청 단위 기본 추적은 `TraceLayer`를 사용한다.
- WebSocket 연결 시작/종료, 실패는 문서 ID와 함께 기록한다.
- 마지막 WebSocket 세션 종료 후 snapshot 저장과 idle room eviction 결과도 문서 ID와 함께 기록한다.
- 첫 WebSocket 세션 시작과 마지막 세션 종료에 연결되는 `RoomCoordinator` hook 실패도 문서 ID와 함께 warning으로 기록한다.
- WebSocket origin 거절도 문서 ID와 함께 기록한다.
- 다중 프로세스 확장 전까지는 "문서당 단일 owner 프로세스" 가정을 깨는 우회 구현을 넣지 않는다.
- `StaticRoomLocator`의 owner hints는 운영 힌트일 뿐이며, lease/heartbeat 기반 coordination store를 대체하는 것으로 간주하지 않는다.
- lease/heartbeat 같은 owner coordination side effect는 `RoomCoordinator` 경계 뒤에만 붙이고, snapshot 저장 전 handoff를 허용하지 않는다.
- authoritative coordination backend는 `lease_id` compare-and-swap과 `epoch` fencing token을 함께 유지해야 한다.
- stale owner 판단은 `expires_at` 기준으로만 하고, file mtime 같은 로컬 heuristic로 조기 회수를 시도하지 않는다.
- filesystem 기반 `FileRoomCoordinator`/`FileRoomLocator`는 rehearsal 경계일 뿐이며, sqlite/managed coordination backend와 shared snapshot store를 대체하는 production authority로 간주하지 않는다.
- 로그 레벨 정책은 `RUST_LOG`로 조정한다.

## Test Rules

- 최소 한 개 이상의 endpoint 검증 테스트를 유지한다.
- 통합 테스트는 앱 빌더를 통해 실제 Router를 띄운다.
- 새 route를 추가하면 happy path 기준 smoke test를 함께 추가한다.
- 공유 계약 모델을 추가하면 직렬화 shape와 기본 검증 규칙에 대한 unit test를 함께 둔다.
- socket bind가 필요한 통합 테스트를 추가하면 `scripts/verify.sh`의 websocket lane과 core skip 목록을 함께 갱신한다.
- 환경 제약으로 실패할 수 있는 검증은 `scripts/preflight.sh`로 조기 탐지 가능한 형태를 우선 제공한다.
- `.git` 쓰기나 remote DNS처럼 publish 전용 조건은 `verify.sh core`에 섞지 말고 `preflight.sh commit`/`publish`로 분리한다.

## Commit Rules

- Codex 작업 브랜치는 일회성 작업 slug보다 역할 중심 이름을 우선한다. 현재 기본 협업 브랜치 이름은 `backend-realtime-api`다.
- 커밋 메시지 형식은 `type(scope): subject`를 사용한다.
- `type`은 `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `build`, `ci`, `rename`, `remove`만 허용한다.
- `scope`는 백엔드 변경 의도를 드러내도록 `api`, `sync`, `yrs`, `auth`, `db`, `websocket`, `storage`, `config`, `docs`, `repo` 중에서 선택한다.
- `subject`는 현재형으로 쓰고, 첫 글자는 소문자로 시작하며, 마침표를 붙이지 않는다.
- `subject`에는 `update`, `fix bug`, `work` 같은 모호한 표현 대신 실제 변경 내용을 적는다.
- 한 커밋에는 한 가지 목적만 담고, 리팩토링과 동작 변경을 섞지 않는다.
- 스키마나 API 변경이 있으면 관련 문서와 테스트를 함께 갱신한다.
