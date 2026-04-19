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

## Current Status

- bootstrap 범위의 백엔드 구현 작업은 모두 완료됐다.
- 현재 브랜치 `backend-realtime-api`는 역할 중심 기본 협업 브랜치다. 이전 작업 slug 기반 브랜치 `storage-temp-handling`에서 이름만 정리해 이어받았고, 현재는 `origin/backend-realtime-api`와 동기화 상태를 기준으로 관리한다.
- 최신 landed 변경은 `feat(sync): add room coordinator lifecycle hooks` (`c23bbdc`)이며, `RoomCoordinator` no-op 경계와 WebSocket first/last session lifecycle hook을 도입해 future lease/heartbeat coordinator가 snapshot persist 이후 handoff 지점을 붙일 수 있게 정리했다.
- storage hardening 변경인 `fix(storage): save snapshots atomically and clean stale temp files` (`ee800ef`)와 `fix(storage): clean stale temp snapshots on startup` (`188e6f4`)도 현재 브랜치와 원격 브랜치에 반영돼 있다.
- 현재 런타임은 여전히 단일 프로세스 owner 판정과 static owner hints 수준으로 유지되며, 실제 멀티 프로세스 활성화는 지원하지 않는다.
- 다음 구현 후보는 외부 snapshot store와 owner coordination 저장소가 필요한 멀티 프로세스 room 분산 지원이며, 현재 저장소 범위에서는 roadmap blocked 상태다.
- sandboxed run에서는 commit/push/WebSocket 통합 테스트가 계속 차단될 수 있다. 현재 스크립트와 운영 규칙은 이 차단을 없애는 것이 아니라 조기에 탐지하고 unrestricted 실행으로 분리하는 용도다.

## WS / Yrs Follow-up Items

- [x] incoming awareness payload server validation added
- [x] awareness metadata에 사용자 정보 구조 정의
- [x] snapshot 복구 시점과 room eviction 정책 정의
- [x] 다중 프로세스 환경에서 room 분산 전략 검토
- [x] `yrs-axum` upstream 변화에 맞춘 provider compatibility 검증 자동화
- [x] `RoomLocator` ownership resolver 경계를 route/WS restore 전에 도입
- [x] config-driven `StaticRoomLocator`와 owner hint 응답 계약 추가
- [x] `RoomCoordinator` session lifecycle hook 경계 추가

## Execution Log

- 2026-04-20: blocked 상태인 멀티 프로세스 roadmap 준비 작업으로 `RoomCoordinator` lifecycle 경계를 도입했다. `src/collab/coordinator.rs`에 기본 `NoopRoomCoordinator`와 trait을 추가하고 `src/state.rs`, `src/collab/ws.rs`에서 WebSocket 첫 세션 시작 시 activate, 마지막 세션 종료 후 snapshot persist/eviction 처리 뒤 deactivate hook을 타도록 연결했다. `src/collab/rooms.rs`의 session teardown 결과를 구조화해 lease handoff 후보 구현이 idle 여부를 안정적으로 판단할 수 있게 정리했고, `tests/health.rs`에는 first/last session hook 호출과 activation failure rollback 회귀 테스트를 추가했다. 관련 아키텍처/운영 문서도 같은 경계로 맞췄다. 검증은 `cargo fmt --check`, `cargo check`, `cargo test websocket_room_coordinator_tracks_first_and_last_session -- --nocapture`, `cargo test websocket_room_activation_failure_does_not_leak_active_sessions -- --nocapture`, `cargo test`, `./scripts/preflight.sh publish` 순서로 실행했고 모두 통과했다.
- 2026-04-19: blocked 상태인 멀티 프로세스 roadmap 준비 작업으로 `StaticRoomLocator` owner hints 정규화를 보강했다. `src/collab/locator.rs`에서 hints JSON의 `node_id`/`base_url`을 trim 후 저장하도록 바꿨고, `owner.base_url`은 path/query 없는 origin-only absolute `http://`/`https://` URL만 허용한 뒤 canonical origin (`scheme://authority`)으로 정규화해 응답 metadata에 실리도록 고정했다. 관련 unit test를 추가하고 `README.md`, `docs/setup.md`, `docs/api.md`를 같은 계약으로 맞췄다. 검증은 `cargo test static_room_locator -- --nocapture`, `cargo fmt --check`, `cargo check`, `cargo test`, `./scripts/preflight.sh publish` 순서로 실행했고 모두 통과했다.
- 2026-04-19: blocked 상태인 멀티 프로세스 roadmap의 준비 작업으로 `StaticRoomLocator` owner hint 검증을 보강했다. `src/collab/locator.rs`에서 `owner.base_url`이 비어 있지 않은 경우 absolute `http://`/`https://` URL인지 fail-fast 검증하도록 바꿨고, 잘못된 hints 파일은 startup 시 `AppError::Config`로 즉시 중단되게 정리했다. 관련 unit test와 `README.md`, `docs/setup.md`, `docs/api.md`도 같은 계약으로 맞췄다.
- 2026-04-19: Codex 자동화가 예전 작업 slug 기반 브랜치 `storage-temp-handling`을 계속 사용해 역할 이름과 어긋나는 문제를 정리했다. 현재 협업 브랜치 이름을 `backend-realtime-api`로 변경했고, 이후 문서 기준도 역할 중심 브랜치명으로 맞췄다. repo 내부 검색으로는 이 이름을 생성하는 하드코딩 코드는 없었고, 저장소 안에는 checklist 기록만 남아 있었다. 따라서 repo 차원 조치는 현재 브랜치/원격 브랜치 이름 정리와 문서 규칙 추가로 제한했다.
- 2026-04-19: 상태 문서 기준 미완료 다음 작업 1건을 checklist status reconciliation으로 확정했다. 이번 run에서는 `docs/checklist.md`의 `Current Status`를 최신 landed commit `973308b` 기준으로 갱신했고, 이미 landed 된 `FileSnapshotStore` atomic replace 변경을 계속 미완료로 반복 기록하던 stale execution log를 정리해 milestone 중심 로그로 압축했다. 검증은 `git status --short --branch`, `git log --oneline --decorate -n 12`, `git rev-list --left-right --count origin/storage-temp-handling...HEAD`, `cargo fmt --check`, `./scripts/verify.sh core`, `git diff --check -- docs/checklist.md`로 수행했고 모두 통과했다. publish 계열 확인으로 `./scripts/preflight.sh publish`, `git add -- docs/checklist.md`, `git commit -m "docs(docs): reconcile checklist stale execution log"`, `git push origin storage-temp-handling`를 sandboxed 실행으로 시도했을 때는 `.git/index.lock` 또는 `.git/codex-preflight-2.lock` 생성이 `Read-only file system`으로 차단됐고 push도 `Could not resolve host: github.com`으로 실패했다. 이 항목의 재발 방지 범위는 sandbox 차단 제거가 아니라 조기 탐지와 unrestricted fallback 분리이며, 이후 unrestricted 실행에서는 같은 commit/push가 정상 완료됐다.
- 2026-04-19: `RoomLocator` 경계를 테스트 주입 수준에서 실제 런타임 설정 경계로 확장했다. `src/config.rs`에 `ROOM_LOCATOR`, `NODE_ID`, `ROOM_OWNER_HINTS_PATH`를 추가했고, `src/collab/locator.rs`에 file-backed `StaticRoomLocator`와 config factory를 구현했다. non-local owner는 이제 단순 `409` 문자열 대신 optional `owner.node_id` / `owner.base_url` metadata를 포함해 응답하며, `README.md`, `docs/setup.md`, `docs/api.md`, `docs/architecture.md`, `.env.example`도 새 계약에 맞춰 갱신했다.
- 2026-04-19: `RoomLocator` ownership resolver 경계를 도입해 `src/state.rs`, `src/routes/documents.rs`, `src/collab/ws.rs`가 document room restore 전에 owner 판정을 먼저 통과하도록 연결했다. 관련 unit test와 document detail non-local owner rejection 회귀 테스트, architecture/API/conventions 문서를 함께 갱신했다.
- 2026-04-19: `scripts/preflight.sh`와 `scripts/verify.sh`를 정리해 `.git`/DNS/socket 환경 차단을 core code verification과 분리했다. `verify.sh core`는 socket-free 검증만, `verify.sh websocket`는 socket bind가 필요한 통합 테스트만 실행하도록 정리했다.
- 2026-04-19: `FileSnapshotStore` 초기화 시 interrupted save가 남긴 stale `.tmp` snapshot을 정리하도록 보강했고, startup hydrate가 stale temp artifact에 오염되지 않는 회귀 테스트를 추가했다.
- 2026-04-19: `FileSnapshotStore` 저장 경로를 same-directory temp file write 후 `rename` 기반 atomic replace로 바꾸고, stale temp cleanup까지 포함하는 storage hardening 변경을 `ee800ef fix(storage): save snapshots atomically and clean stale temp files`로 반영했다.
- 2026-04-18: `RoomRegistry::delete_document`가 활성 WebSocket 세션이 남아 있는 문서 삭제를 `409 conflict`로 차단하도록 보강했고, 세션 종료 후 삭제 허용 회귀 테스트와 관련 문서를 갱신했다.
- 2026-04-18: `FileSnapshotStore` catalog/hydrate 경로가 손상된 snapshot 파일을 warning과 함께 건너뛰도록 보강해 단일 corrupt file이 전체 startup/listing 실패로 번지지 않게 했다.
- 2026-04-18: `SNAPSHOT_STORE` (`memory`/`file`)와 `SNAPSHOT_DIR` 설정을 추가하고 `FileSnapshotStore`를 연결했다. 앱 시작 시 snapshot catalog도 hydrate되며, snapshot round-trip 및 재시작 복구 테스트를 함께 추가했다.
- 2026-04-18: `ValidatingProtocol`을 추가해 `/ws/:doc_id` 경로의 incoming awareness payload를 `AwarenessState` 계약으로 검증하도록 고정했다.
- 2026-04-18: 문서 생성/삭제 API를 추가하고, 존재하지 않는 문서에 대한 `GET /api/documents/:id` 및 `GET /ws/:doc_id`가 `404`를 반환하도록 명시 생성 기반 흐름으로 정리했다.
- 2026-04-18: `SnapshotStore` trait과 `InMemorySnapshotStore`를 도입해 room save/restore 경계를 정리했고, active room이 없어도 stored snapshot이 있으면 on-demand restore 경로를 타도록 만들었다.
- 2026-04-18: 관리용 `API_TOKEN`과 문서별 `access_token`을 도입해 문서 생성/목록과 문서 상세/삭제/WebSocket 접근을 분리 보호했다.
- 2026-04-18: 협업 참가자 awareness 표준 구조(`user`, optional `selection`, `client`)와 기본 검증 규칙을 추가하고 문서와 테스트를 같은 계약으로 맞췄다.
- 2026-04-18: room이 active WebSocket session 수를 추적하고 마지막 세션 종료 시 최신 snapshot을 저장한 뒤 eviction하도록 정리했다.
- 2026-04-18: `tests/health.rs`에 y-sync `SyncStep1/SyncStep2` 핸드셰이크와 update broadcast를 실제 WebSocket 경로에서 검증하는 provider compatibility 회귀 테스트를 추가했다.
- 2026-04-18: frontend editor provider end-to-end 상호운용 TODO를 완료 처리했다. 실제 `/ws/:doc_id` 경로에서 두 클라이언트의 초기 동기화와 provider update broadcast를 검증한다.
- 2026-04-18: 단일 프로세스 `RoomRegistry`를 유지한 상태에서 다중 프로세스 room 분산 전략을 문서화했다. 한 문서당 단일 owner 프로세스, 공용 snapshot store, lease 기반 ownership handoff, `RoomLocator` 확장 포인트를 운영 규칙으로 정리했다.
- 2026-04-18: `SnapshotStore::list_documents` 경계를 추가해 `GET /api/documents`가 active room과 persisted snapshot catalog를 함께 반환하도록 정리했다.
- 2026-04-18: 앱 시작 시 `RoomRegistry::hydrate_from_store`로 snapshot catalog를 선로딩하도록 정리했다.
