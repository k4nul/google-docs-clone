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

- [ ] 문서 생성/삭제 API 추가
- [ ] persistence adapter trait 정의 및 snapshot 저장 전략 도입
- [ ] 인증과 문서 접근 제어 추가
- [ ] frontend editor provider와 end-to-end 상호운용 테스트 추가

## WS / Yrs Follow-up Items

- [ ] awareness metadata에 사용자 정보 구조 정의
- [ ] snapshot 복구 시점과 room eviction 정책 정의
- [ ] 다중 프로세스 환경에서 room 분산 전략 검토
- [ ] `yrs-axum` upstream 변화에 맞춘 provider compatibility 검증 자동화
