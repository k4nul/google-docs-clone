# Backend Collaborative Server

Axum, Tokio, Yrs 기반으로 시작하는 협업 편집 백엔드 부트스트랩 프로젝트입니다.

## 프로젝트 개요

문서 단위의 실시간 협업 서버를 Rust로 안전하게 시작할 수 있도록 최소 실행 구조를 제공합니다. 현재 단계에서는 HTTP 헬스체크, 문서 생성/조회/삭제 API, 문서별 WebSocket 진입점, in-memory room registry, snapshot 저장 추상화를 포함합니다.

## 해결하려는 문제

협업 편집 시스템은 HTTP API, WebSocket 세션, 문서별 상태 관리, CRDT 동기화 경계를 초기에 잘 나누지 않으면 빠르게 복잡해집니다. 이 레포는 그 복잡도를 초기에 제어하기 위해 compile-safe한 기본 골격과 문서화를 함께 제공합니다.

## 핵심 기능

- `GET /api/health` 헬스체크
- `GET /api/documents` 현재 메모리상 문서 목록 조회
- `POST /api/documents` 문서 생성 및 room 초기화
- `GET /api/documents/:id` 기존 문서 상세 조회
- `DELETE /api/documents/:id` 문서 및 room 제거
- `GET /ws/:doc_id` 문서별 협업 WebSocket 진입점
- `DashMap` 기반 room registry
- `yrs-axum` 기반 broadcast group 연결
- `SnapshotStore` trait 및 in-memory adapter

## 기술 스택

- Rust
- Axum
- Tokio
- Yrs
- yrs-axum
- DashMap
- Tracing / tracing-subscriber

## 로컬 실행 방법

```bash
cp .env.example .env
cargo run
```

기본 실행 주소는 `127.0.0.1:4000`입니다. 기본 `FRONTEND_ORIGIN`은 `http://localhost:3000`으로 설정되어 있어 로컬 프런트엔드 개발 서버와 포트가 겹치지 않습니다.

## API/WS 개요

- HTTP base path: `/api`
- Health: `GET /api/health`
- Documents: `GET /api/documents`, `POST /api/documents`, `GET /api/documents/:id`, `DELETE /api/documents/:id`
- Collaboration WebSocket: `GET /ws/:doc_id`

먼저 `POST /api/documents`로 문서를 생성하면 해당 `doc_id` room이 메모리에 준비되고, 이후 `GET /api/documents/:id`로 상세를 조회하거나 같은 `doc_id`로 WebSocket에 연결해 협업 세션을 시작할 수 있습니다. 존재하지 않는 문서 ID로 상세 조회나 WebSocket 연결을 시도하면 `404`를 반환합니다. WebSocket 핸드셰이크의 `Origin` 헤더는 `FRONTEND_ORIGIN`과 정확히 일치해야 합니다.

## 폴더 구조 요약

```text
.
|-- AGENTS.md
|-- README.md
|-- .env.example
|-- docs/
|-- src/
|   |-- app.rs
|   |-- collab/
|   |-- config.rs
|   |-- errors.rs
|   |-- models/
|   |-- routes/
|   |-- state.rs
|   |-- lib.rs
|   `-- main.rs
`-- tests/
```

## 환경변수 요약

- `HOST`: 서버 바인드 호스트
- `PORT`: 서버 바인드 포트
- `FRONTEND_ORIGIN`: CORS 허용 origin
- `RUST_LOG`: tracing 필터 설정

## 현재 범위

- 단일 프로세스 in-memory room 관리
- room snapshot 저장/복구 확장 포인트
- 문서별 WebSocket 협업 세션 진입
- API/앱 상태/설정/에러 모듈 분리
- 테스트 가능한 앱 빌더 제공

## 비범위

- 데이터베이스 연동
- 외부 영속 저장소 구현
- 인증 및 권한 관리
- 문서 수정용 REST API
- 멀티 노드 분산 동기화

## 향후 확장 방향

- 인증 레이어 및 문서 접근 제어 추가
- awareness metadata 확장
- 외부 저장소 adapter 및 startup hydration 추가
- provider / frontend editor 연동 계약 고도화
