# Architecture

## Module Structure

- `src/app.rs`: 앱 조립, CORS, HTTP tracing layer
- `src/config.rs`: 환경변수 로딩, 기본값, 값 검증
- `src/state.rs`: 전역 `AppState`, room registry 접근, 허용된 프런트엔드 origin 보관
- `src/errors.rs`: 공통 에러 타입과 HTTP 응답 변환
- `src/auth.rs`: Bearer 토큰 파싱과 인증 경계
- `src/routes`: REST endpoint 집합
- `src/collab`: Yrs room registry와 WebSocket 협업 경계
- `src/models`: 문서 placeholder 모델
- `src/storage`: snapshot store trait과 memory/file adapter

## Request Flow

1. `main.rs`가 환경변수를 읽고 tracing을 초기화한다.
2. `app.rs`가 `AppState`와 라우트를 조합해 `Router`를 만든다.
3. `/api/*` 요청은 `routes` 모듈로 들어간다.
4. 문서 목록/생성은 관리용 `API_TOKEN`을 검증하고, 문서 상세/삭제는 문서별 `access_token`을 검증한다.
5. 문서 단위 room에 닿는 요청은 `AppState`의 `RoomLocator` 경계로 현재 노드 ownership을 먼저 확인한다.
6. route handler는 `AppState`를 통해 registry를 조회하고 JSON 응답을 반환한다.

## WebSocket / Collab Flow

- Incoming awareness updates pass through a custom Yrs protocol layer before shared room awareness state is mutated.

1. 클라이언트가 `GET /ws/:doc_id`로 업그레이드를 요청한다.
2. `collab/ws.rs`가 `doc_id` 형식을 검증하고 `Origin` 헤더가 `FRONTEND_ORIGIN`과 일치하는지 확인한다.
3. 같은 핸들러가 `Authorization: Bearer <access_token>`을 검증한다.
4. 같은 경계가 `RoomLocator`로 현재 노드 ownership을 확인한다.
5. 검증이 통과하면 `doc_id`에 해당하는 room을 조회하거나 snapshot store에서 on-demand로 복구한다.
6. room은 `Yrs Doc`, `Awareness`, lazy `BroadcastGroup`을 가진다.
7. 클라이언트는 연결 직후 awareness state를 `user`, optional `selection`, `client` 구조로 게시한다.
8. 업그레이드된 socket은 `AxumSink` / `AxumStream`으로 감싸진다.
9. `BroadcastGroup::subscribe`가 해당 문서의 협업 세션을 처리한다.
10. 마지막 WebSocket 세션이 종료되면 room snapshot을 저장하고 idle room을 registry에서 제거한다.

## Room Registry Structure

- 저장소는 `DashMap<Uuid, Arc<Room>>`
- key는 문서 ID
- value는 placeholder 문서 메타데이터와 Yrs awareness, lazy broadcast group
- 문서 메타데이터에는 외부 응답으로 노출하지 않는 `access_token`이 포함된다.
- 문서 API와 WebSocket 엔트리포인트가 같은 registry를 공유한다.
- awareness payload의 canonical shape는 `AwarenessState { user, selection?, client }`이며, 사용자 식별과 색상 규칙은 서버 모델과 문서에서 함께 관리한다.

## Persistence Extension Points

- `SnapshotStore` trait이 `load/save/delete` 경계를 정의하고, `RoomRegistry`가 이 trait에만 의존한다.
- `RoomLocator` trait이 "현재 프로세스가 이 문서의 authoritative owner인가"라는 진입 경계를 정의하고, `AppState`가 route/WS 진입 전에 이 trait만 호출한다.
- `room_locator_from_config`는 현재 `LocalRoomLocator` 또는 file-backed `StaticRoomLocator`를 런타임에 선택한다.
- `Room::snapshot()`은 Yrs document를 full-state update로 직렬화하고 문서 metadata를 함께 저장한다.
- `Room::from_snapshot()`은 저장된 update를 다시 apply해 room을 복구한다.
- 각 room은 active WebSocket session 수를 추적하고, 마지막 세션 종료 시에만 snapshot 저장 후 eviction을 시도한다.
- 문서 삭제는 active WebSocket session 수가 0일 때만 허용하며, 세션이 남아 있으면 `409 conflict`로 거절한다.
- 현재는 `InMemorySnapshotStore`와 `FileSnapshotStore`가 연결되며, future adapter는 같은 trait으로 db/object storage를 대체할 수 있다.
- `GET /api/documents/:id`와 `GET /ws/:doc_id`는 active room이 없어도 snapshot store에서 문서를 복구한 뒤 처리할 수 있다.
- `GET /api/documents`는 active room과 snapshot store catalog를 합쳐 eviction 이후에도 문서 메타데이터를 유지한다.
- 앱 시작 시 snapshot catalog를 순회해 저장된 문서를 room registry로 hydrate한다.
- `FileSnapshotStore`는 catalog/hydrate 경로에서 corrupt snapshot 파일을 warning과 함께 건너뛰어 단일 손상 파일이 전체 startup/listing 실패로 번지지 않게 한다.
- `FileSnapshotStore`는 같은 디렉터리의 임시 파일에 snapshot을 먼저 쓴 뒤 `rename`으로 교체해 partial write가 마지막 정상 snapshot을 직접 덮어쓰지 않도록 한다.
- interrupted save가 남긴 `.tmp` 파일은 `FileSnapshotStore` 초기화 시점에 정리되며, catalog/hydrate는 계속 `.json` snapshot만 복구 대상으로 취급한다.
- 문서 삭제 시 `FileSnapshotStore`는 본 snapshot과 같은 문서 ID를 가진 stale `.tmp` 파일도 함께 제거해 temp artifact가 누적되지 않게 한다.
- `Config.snapshot_store`가 `memory`/`file` 어댑터 선택을 담당하고, `file` 모드에서는 `SNAPSHOT_DIR/<doc_id>.json` 파일이 문서 metadata와 Yrs full-state update를 함께 저장한다.

## Multi-Process Distribution Strategy

- 현재 `RoomRegistry`는 프로세스 로컬 `DashMap`에 의존하므로, 같은 `doc_id`를 여러 프로세스가 동시에 소유하면 Yrs update ordering과 awareness fan-out이 분리된다.
- 따라서 다중 프로세스 운영의 첫 번째 불변조건은 "한 시점에 한 문서는 정확히 한 협업 프로세스만 authoritative owner가 된다"로 둔다.
- 권장 1차 전략은 L7 sticky routing보다 명시적 room ownership 조회를 우선하는 것이다. sticky session만으로는 프로세스 재시작, scale-in, reconnect 시 ownership drift를 막기 어렵다.
- 진입 플로우는 `GET /ws/:doc_id` 전에 API gateway 또는 thin coordination layer가 `doc_id -> owner node` 매핑을 조회하고, 현재 노드가 owner가 아니면 redirect 또는 proxy 방식으로 owner에 라우팅하는 형태를 기준으로 한다.
- owner node는 room 활성 중에는 주기적으로 lease/heartbeat를 갱신하고, lease 만료 시에만 다른 노드가 snapshot restore 후 ownership을 인계받는다.
- snapshot store는 프로세스 공용 저장소여야 하며, owner handoff 직전 마지막 full-state update가 내구적으로 저장되어야 한다.
- awareness는 durability 대상이 아니므로 owner handoff 시 재게시를 허용하고, 문서 본문 CRDT update와 분리해 취급한다.
- cross-node fan-out이 필요해지는 시점 전까지는 한 room의 WebSocket 세션을 모두 owner node에 붙이는 방식이 가장 단순하다. node 간 pub/sub 복제는 ownership 우회가 아니라 장애 복구 보조 경로로만 고려한다.
- 구현 확장 포인트는 `RoomRegistry` 앞단에 `RoomLocator` 또는 동등한 ownership resolver를 두고, 현재 `get_or_restore` 호출 전에 authoritative node 결정을 끼워 넣는 형태가 가장 경계에 맞다.
- 현재 저장소에는 이 경계를 구현한 기본 `LocalRoomLocator`와 file-backed `StaticRoomLocator`가 들어가 있다.
- `StaticRoomLocator`는 문서별 owner 힌트를 읽어 현재 `NODE_ID`와 다른 owner를 가진 room 요청을 조기에 차단하고 optional `base_url` 힌트를 응답에 실어준다.
- 현재는 `InMemorySnapshotStore`와 로컬 `FileSnapshotStore`만 있으므로 실제 멀티 프로세스 활성화는 여전히 blocked 상태다. 여러 프로세스가 함께 쓰는 외부 snapshot store와 owner coordination 저장소가 준비되기 전까지는 단일 프로세스 배포를 운영 규칙으로 유지한다.
