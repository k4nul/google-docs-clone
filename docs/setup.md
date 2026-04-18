# Setup

## Build

```bash
cargo check
```

## Run

```bash
cp .env.example .env
cargo run
```

기본 바인드 주소는 `127.0.0.1:4000`입니다.
기본 `FRONTEND_ORIGIN`은 `http://localhost:3000`이므로 로컬 프런트엔드 개발 서버를 별도 포트에서 띄우는 흐름을 바로 재현할 수 있습니다.
기본 `API_TOKEN`은 `dev-admin-token`이며, 개발 환경에서는 이 토큰으로 문서 생성/목록 API를 호출합니다.

## Test

```bash
cargo fmt --check
cargo test
```

## Environment Variables

- `HOST`: 서버가 바인드할 호스트명 또는 IP
- `PORT`: 서버 포트
- `FRONTEND_ORIGIN`: CORS 허용 origin
- `RUST_LOG`: tracing subscriber 필터
- `API_TOKEN`: 문서 생성 및 목록 조회용 Bearer 토큰

## Local Development Procedure

1. `.env.example`을 기준으로 로컬 환경값을 준비한다.
2. `cargo check`로 의존성과 컴파일 상태를 먼저 확인한다.
3. `cargo run`으로 서버를 올리고 `/api/health`를 확인한다.
4. `Authorization: Bearer <API_TOKEN>`으로 `POST /api/documents`를 호출해 문서를 만들고 응답의 `access_token`을 확보한다.
5. 문서 상세 조회, 삭제, WebSocket 연결에는 `Authorization: Bearer <access_token>`을 사용한다.
6. WebSocket 접속 시 `Origin` 헤더를 `FRONTEND_ORIGIN`과 맞춰 `/ws/:doc_id`에 접속한다.
7. 작업 마무리 전 `cargo fmt --check`와 `cargo test`를 실행한다.
