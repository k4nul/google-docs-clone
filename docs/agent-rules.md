# Agent Rules

- 큰 구조 변경 전에 관련 문서를 먼저 확인하고 필요하면 같은 작업 안에서 함께 갱신한다.
- 모든 변경은 `cargo check`, `cargo fmt --check`, `cargo test`가 녹색인 상태를 유지하도록 마무리한다.
- API, route, 환경변수, WebSocket 계약이 바뀌면 `README.md`와 `docs/api.md`를 함께 동기화한다.
- `src/app.rs`, `src/config.rs`, `src/state.rs`, `src/errors.rs`, `src/routes`, `src/collab`의 책임 경계를 존중한다.
- persistence가 아직 없으므로 상태 저장이 필요해 보이는 기능은 먼저 확장 포인트로 설계하고 문서에 명시한다.
- broken integration이 의심되는 실험 코드는 바로 route에 넣지 않고 경계 모듈로 격리한다.
- 커밋 메시지는 반드시 `type(scope): subject` 형식을 사용한다.
- 허용되는 `type`은 `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`, `perf`, `build`, `ci`, `rename`, `remove`만 사용한다.
- `scope`는 백엔드 문맥에 맞게 `api`, `sync`, `yrs`, `auth`, `db`, `websocket`, `storage`, `config`, `docs`, `repo` 중에서 구체적으로 선택한다.
- `subject`는 현재형, 소문자 시작, 마침표 없음, 변경 내용을 직접 설명하는 문장 조각으로 작성한다.
- 한 커밋에는 한 가지 목적만 담고, 리팩토링과 동작 변경을 섞지 않는다.
- 스키마 또는 API 계약을 바꾸면 관련 문서와 테스트를 같은 작업 안에서 함께 갱신한다.
- 빌드, 테스트, 린트가 가능하면 실행하고 결과를 남긴다.
- 불확실한 구현은 추측으로 밀어 넣지 말고 `TODO` 또는 blocked 상태로 명시한다.
