# Frontend Bootstrap

이 프로젝트는 React + TypeScript + Vite 기반 collaborative editor frontend의 초기 부트스트랩입니다. 문서 목록 라우트와 Tiptap/Yjs 기반 협업 에디터 셸, DOCX import utility, 개발 규약 문서를 한 번에 정리한 상태입니다.

## 프로젝트 목적

- collaborative editor UI의 최소 실행 가능한 프론트엔드 구조 제공
- 실시간 협업을 위한 Yjs + websocket provider 연결 지점 선행 구성
- 향후 backend API, import pipeline, presence 기능이 붙을 수 있는 feature-based 구조 마련

## 기술 스택

- React 19
- TypeScript (strict mode)
- Vite
- React Router
- Tiptap
- Yjs + y-websocket
- Mammoth + DOMPurify
- Vitest + Testing Library
- ESLint + Prettier

## 실행 명령

```bash
npm install
npm run dev
npm run build
npm run lint
npm run test
npm run typecheck
npm run preview
```

Windows PowerShell에서 실행 정책 때문에 `npm.ps1`이 차단되면 같은 명령을 `npm.cmd run <task>` 형태로 실행합니다. 예: `npm.cmd run build`

## 폴더 구조 요약

```text
src/
  app/                앱 부트스트랩과 라우팅
  pages/              route entry page
  features/
    editor/           Tiptap editor shell과 toolbar
    documents/        문서 목록 mock data와 document-facing feature
  lib/
    api/              API client helpers
    collab/           Yjs document/provider 초기화 및 사용자 placeholder
    import/           DOCX import utility
  shared/
    config/           환경변수 파싱
    types/            공용 타입
    ui/               공용 레이아웃
  test/               test setup
docs/                 규약, 역할, 아키텍처, 체크리스트
```

## Docs Index

- [Agent Rules](./docs/agent-rules.md)
- [Setup](./docs/setup.md)
- [Architecture](./docs/architecture.md)
- [Roles](./docs/roles.md)
- [Conventions](./docs/conventions.md)
- [Checklist](./docs/checklist.md)

## Harness / CI

- AGENTS 문서 경로는 repo root 기준 relative path로 유지합니다.
- 품질 게이트는 [.github/workflows/ci.yml](./.github/workflows/ci.yml)에서 `build`, `lint`, `test`, `typecheck`라는 독립 check name으로 자동 검증합니다.
- [AGENTS.md](./AGENTS.md)의 A/B/C/D는 현재 GitHub owner 핸들이 아니라 책임 구간 역할 문서입니다.
- [.github/CODEOWNERS](./.github/CODEOWNERS)는 실제 enforcement 관점에서 현재 `@System-Docs-H`를 baseline owner로 사용하고, 경로 구간은 역할 문서와 같은 범위로 정렬해 둡니다.

## 현재 범위

- `/` 문서 목록 placeholder page
- `/docs/:docId` collaborative editor page
- websocket URL이 없을 때도 안전하게 동작하는 local-only collaboration shell
- DOCX HTML import utility와 sanitize path

## 비범위

- 실제 인증/권한 처리
- 영속 문서 저장 API 연동
- 운영용 presence UI, comment system, revision history UI
- 완성형 import/upload 화면

## Backend Integration Points

- 문서 목록/상세 조회: `@/lib/api/httpClient.ts`
- 실시간 협업 provider: `@/lib/collab/connection.ts`
- editor route entry: `@/pages/EditorPage.tsx`
- DOCX ingest path: `@/lib/import/docxImport.ts`

## 환경변수 요약

- `VITE_API_BASE_URL`: REST API base URL
- `VITE_WS_URL`: Yjs websocket provider base URL

예시는 [.env.example](./.env.example)에 있습니다.
