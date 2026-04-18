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
- [ ] 인증과 문서 접근 제어 추가
- [ ] frontend editor provider와 end-to-end 상호운용 테스트 추가

## WS / Yrs Follow-up Items

- [ ] awareness metadata에 사용자 정보 구조 정의
- [ ] snapshot 복구 시점과 room eviction 정책 정의
- [ ] 다중 프로세스 환경에서 room 분산 전략 검토
- [ ] `yrs-axum` upstream 변화에 맞춘 provider compatibility 검증 자동화

## Execution Log

- 2026-04-18: 문서 생성/삭제 API를 추가하고 문서 자동 생성 흐름을 명시 생성 기반으로 정리했다. `GET /api/documents/:id`와 `GET /ws/:doc_id`는 이제 존재하지 않는 문서에 대해 `404`를 반환한다. 관련 테스트, README, API 문서를 함께 갱신했다.
- 2026-04-18: `SnapshotStore` trait과 `InMemorySnapshotStore`를 추가하고 `RoomRegistry`가 snapshot save/restore 경계를 통해 room을 복구할 수 있도록 정리했다. `GET /api/documents/:id`와 `GET /ws/:doc_id`는 active room이 없어도 stored snapshot이 있으면 복구 경로를 탄다. unit/integration test와 README, architecture/conventions 문서를 함께 갱신했다.
