# Roles

A/B/C/D는 현재 GitHub owner 핸들이 아니라 책임 구간 역할 문서다. 실제 owner mapping은 `Front-End/.github/CODEOWNERS`에서 관리하며, dedicated GitHub 팀/사용자 핸들이 준비되기 전까지는 `@System-Docs-H`가 baseline owner를 맡는다.

## A: PM / Integration

- 책임:
  요구사항 정리, 우선순위 관리, cross-team integration 일정 조율
- CODEOWNERS scope:
  `AGENTS.md`, `README.md`
- 입력물:
  제품 요구사항, 일정, backend/frontend dependency map
- 출력물:
  스코프 확정, 통합 체크포인트, integration acceptance 기준
- handoff point:
  route/API/provider contract를 B와 C에게 전달하고 D에게 검증 기준을 넘긴다

## B: Frontend Editor / UI Owner

- 책임:
  editor UI, route shell, document list, import entry UI, interaction polish
- CODEOWNERS scope:
  `src/app`, `src/pages`, `src/features/editor`, `src/features/documents`, `src/shared/ui`, `src/index.css`
- 입력물:
  A가 정리한 요구사항, C가 제공한 API/provider contract
- 출력물:
  editor page, document page, reusable UI component, frontend state wiring
- handoff point:
  API shape 확정 후 C와 연결하고, 테스트 가능한 UI 변경 사항을 D에게 넘긴다

## C: Backend Realtime / API Owner

- 책임:
  document CRUD API, Yjs websocket infra, auth-aware realtime contract 정의
- CODEOWNERS scope:
  `src/lib/api`, `src/lib/collab`, `src/shared/config`, `src/shared/types`
- 입력물:
  A의 통합 요구사항, B의 UI data needs
- 출력물:
  REST/WS contract, mock/staging endpoint, provider 운영 기준
- handoff point:
  `VITE_API_BASE_URL`, `VITE_WS_URL`에 연결 가능한 명세와 샘플 payload를 B와 D에게 전달한다

## D: QA / Docs / DevOps Owner

- 책임:
  테스트 전략, 문서 정확성, CI, build/lint/typecheck 기준 유지
- CODEOWNERS scope:
  `.github`, `docs`, `src/lib/import`, `src/test`, `.env.example`, `package.json`, `package-lock.json`, `eslint.config.js`, `tsconfig*.json`, `vite.config.ts`, `index.html`
- 입력물:
  A의 acceptance 기준, B/C의 변경사항과 계약 문서
- 출력물:
  검증 결과, release checklist, 문서 갱신, CI 품질 게이트
- handoff point:
  PR 검증 후 결과를 A에게 회신하고, 누락된 문서를 B/C에 피드백한다
