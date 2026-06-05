# Roles

## A: PM / Integration

- 책임: 범위 정의, 전체 일정 정리, 프런트와 백엔드 계약 조율, 통합 우선순위 확정
- 입력물: 요구사항, 일정, 외부 의존성 상태, 팀 결정 사항
- 출력물: 통합 계획, 마일스톤, 계약 변경 요청 정리
- Handoff Point: 기능 우선순위와 계약 변경안을 B와 C에 전달하고 검증 요청을 D에 넘긴다.

## B: Frontend Editor / UI Owner

- 책임: 같은 repository의 `Front-End/` workspace에서 편집기 UI, provider 연결, 문서 진입 흐름, 사용자 상호작용을 구현하고 backend API/WS 계약 변경점을 함께 조율
- 입력물: API 계약, WebSocket 경로 규약, 제품 요구사항
- 출력물: `Front-End/` workspace 구현, 연결 검증 결과, backend에 반영할 UI/API 계약 변경점
- Handoff Point: 필요한 API/WS 변경 사항을 C와 A에 전달하고 사용자 플로우 검증 결과를 D와 공유한다.

## C: Backend Realtime / API Owner

- 책임: HTTP API, room registry, WebSocket 협업 흐름, CRDT 서버 구조 유지
- 추가 책임: storage portability와 Windows SQLite shim portability를 포함해 `vendor/rusqlite/src/lib.rs` 같은 backend-owned test unblocker를 수정한다.
- 입력물: 계약 요구사항, 프런트엔드 연결 조건, 운영 제약
- 출력물: 서버 구현, API 계약 반영, 런타임 안정성 개선안
- Handoff Point: 구현된 엔드포인트와 변경 사항을 A와 B에 공유하고 테스트 포인트를 D에 전달한다.

## D: QA / Docs / DevOps Owner

- 책임: 테스트 실행, 문서 최신화, 실행 절차 검증, 릴리스/운영 준비
- 추가 책임: platform-specific failure를 환경 차이로 기록하고, 역할 완료 전 `cargo test --locked --lib`, `cargo test --locked`, `./scripts/verify.sh core` 같은 지정 검증이 통과했는지 확인한다.
- 입력물: 구현 변경 내역, 계약 변경 내역, 실행 로그, 테스트 결과
- 출력물: 검증 결과, 운영 체크리스트, 문서 업데이트, 배포 준비 상태
- Handoff Point: 검증 완료 상태를 A에 보고하고 발견된 결함이나 문서 누락을 B/C에 되돌린다.
