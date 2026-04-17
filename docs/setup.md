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

## Local Development Procedure

1. `.env.example`을 기준으로 로컬 환경값을 준비한다.
2. `cargo check`로 의존성과 컴파일 상태를 먼저 확인한다.
3. `cargo run`으로 서버를 올리고 `/api/health`를 확인한다.
4. 필요 시 `GET /api/documents/:id`로 room을 만든 뒤 `Origin` 헤더를 `FRONTEND_ORIGIN`과 맞춰 `/ws/:doc_id`에 접속한다.
5. 작업 마무리 전 `cargo fmt --check`와 `cargo test`를 실행한다.
