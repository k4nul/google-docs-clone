# Checklist

## Bootstrap Complete Checklist

- [x] Rust binary crate initialized at repository root
- [x] Axum / Tokio / Yrs / yrs-axum dependencies added
- [x] app/config/state/errors/routes/collab modules separated
- [x] `.env.example` created
- [x] `AGENTS.md`, `README.md`, `/docs` created
- [x] health endpoint implemented
- [x] documents endpoints implemented
- [x] WebSocket collaboration entrypoint implemented
- [x] in-memory room registry implemented
- [x] integration tests added

## Next Step TODO

- [x] 문서 생성/삭제 API 추가
- [x] persistence adapter trait 정의 및 snapshot 저장 전략 도입
- [x] 인증과 문서 접근 제어 추가
- [x] frontend editor provider와 end-to-end 상호운용 테스트 추가

## WS / Yrs Follow-up Items

- [x] incoming awareness payload server validation added

- [x] awareness metadata에 사용자 정보 구조 정의
- [x] snapshot 복구 시점과 room eviction 정책 정의
- [x] 다중 프로세스 환경에서 room 분산 전략 검토
- [x] `yrs-axum` upstream 변화에 맞춘 provider compatibility 검증 자동화

## Execution Log

- 2026-04-18: 자동화 run에서 file snapshot store 작업을 검증 완료했다. `cargo check`, `cargo fmt --check`, `cargo test`가 모두 통과했고 `SNAPSHOT_STORE=file` 설정 경로와 재시작 hydrate 테스트 결과를 기준으로 커밋 준비 상태를 확인했다.
- 2026-04-18: `SNAPSHOT_STORE` (`memory`/`file`)와 `SNAPSHOT_DIR` 설정을 추가하고 `FileSnapshotStore`를 연결했다. 앱 시작 시 file snapshot catalog도 hydrate되며, snapshot round-trip unit test와 재시작 복구 integration test, `.env.example`, README, setup, architecture 문서를 함께 갱신했다.
- 2026-04-18: Added `ValidatingProtocol` so `/ws/:doc_id` validates incoming awareness payloads against `AwarenessState`. Invalid JSON shape or field values are rejected before shared room awareness mutates, and unit/integration tests plus related docs were updated.

- 2026-04-18: 문서 생성/삭제 API를 추가하고 문서 자동 생성 흐름을 명시 생성 기반으로 정리했다. `GET /api/documents/:id`와 `GET /ws/:doc_id`는 이제 존재하지 않는 문서에 대해 `404`를 반환한다. 관련 테스트, README, API 문서를 함께 갱신했다.
- 2026-04-18: `SnapshotStore` trait과 `InMemorySnapshotStore`를 추가하고 `RoomRegistry`가 snapshot save/restore 경계를 통해 room을 복구할 수 있도록 정리했다. `GET /api/documents/:id`와 `GET /ws/:doc_id`는 active room이 없어도 stored snapshot이 있으면 복구 경로를 탄다. unit/integration test와 README, architecture/conventions 문서를 함께 갱신했다.
- 2026-04-18: 관리용 `API_TOKEN`과 문서별 `access_token`을 도입해 문서 생성/목록 조회와 문서 상세/삭제/WebSocket 접근을 분리 보호했다. 관련 라우트/WS 테스트를 보강하고 `.env.example`, README, setup/api/architecture/conventions 문서를 함께 갱신했다.
- 2026-04-18: `src/models/awareness.rs`에 협업 참가자 awareness 표준 구조(`user`, optional `selection`, `client`)와 기본 검증 규칙을 추가했다. README, API, architecture, conventions 문서를 같은 계약으로 맞추고 unit test로 직렬화 shape와 필드 검증을 고정했다.
- 2026-04-18: room이 active WebSocket session 수를 추적하고 마지막 세션 종료 시 최신 snapshot을 저장한 뒤 eviction하도록 정리했다. `GET /api/documents/:id`와 `GET /ws/:doc_id`의 on-demand restore 시점을 문서화하고 unit/integration test와 README, API, architecture, conventions 문서를 함께 갱신했다.
- 2026-04-18: `tests/health.rs`에 y-sync `SyncStep1/SyncStep2` 핸드셰이크와 update broadcast를 실제 WebSocket 경로에서 검증하는 provider compatibility 회귀 테스트를 추가했다. `yrs-axum` 업스트림 변경이 서버-클라이언트 기본 호환성을 깨뜨리는지 `cargo test`로 자동 확인할 수 있다.
- 2026-04-18: frontend editor provider end-to-end 상호운용 항목을 완료 처리했다. `tests/health.rs`의 WebSocket 회귀 테스트가 문서 생성 후 두 클라이언트의 y-sync 초기 동기화와 provider update broadcast를 실제 `/ws/:doc_id` 경로에서 검증한다.
- 2026-04-18: 단일 프로세스 `RoomRegistry`를 유지한 상태에서 다중 프로세스 room 분산 전략을 문서화했다. 한 문서당 단일 owner 프로세스, 공용 snapshot store, lease 기반 ownership handoff, `RoomLocator` 확장 포인트를 architecture/README/conventions에 정리했고, 외부 저장소 전까지는 단일 프로세스 배포를 운영 규칙으로 명시했다.
- 2026-04-18: `SnapshotStore::list_documents` 경계를 추가해 `GET /api/documents`가 active room과 persisted snapshot catalog를 함께 반환하도록 정리했다. idle eviction 이후에도 문서 목록에서 메타데이터가 유지되도록 unit/integration test와 README/API/architecture 문서를 함께 갱신했다.
- 2026-04-18: 앱 시작 시 `RoomRegistry::hydrate_from_store`로 snapshot catalog를 선로딩하도록 정리했다. `AppState::with_snapshot_store`가 초기화 중 저장된 문서를 room registry에 복원하고, unit/integration test 및 README/architecture 문서를 함께 갱신했다.
