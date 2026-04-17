# Agent Rules

- 큰 구조 변경 전에 관련 문서를 먼저 확인하고 필요하면 같은 작업 안에서 함께 갱신한다.
- 모든 변경은 `cargo check`, `cargo fmt --check`, `cargo test`가 녹색인 상태를 유지하도록 마무리한다.
- API, route, 환경변수, WebSocket 계약이 바뀌면 `README.md`와 `docs/api.md`를 함께 동기화한다.
- `src/app.rs`, `src/config.rs`, `src/state.rs`, `src/errors.rs`, `src/routes`, `src/collab`의 책임 경계를 존중한다.
- persistence가 아직 없으므로 상태 저장이 필요해 보이는 기능은 먼저 확장 포인트로 설계하고 문서에 명시한다.
- broken integration이 의심되는 실험 코드는 바로 route에 넣지 않고 경계 모듈로 격리한다.
