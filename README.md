# Front-End Collaborative Editor

React + TypeScript + Vite 기반의 collaborative editor 프론트엔드 저장소입니다. 현재는 Tiptap, Yjs, `y-websocket`을 중심으로 실시간 편집 셸을 구성했고, backend 연동 전에도 빌드와 테스트가 가능한 bootstrap 상태를 목표로 유지합니다.

## 핵심 기능

- 문서 목록 placeholder 페이지 제공
- `/docs/:docId` 경로에서 collaborative editor 셸 제공
- Tiptap 기반 rich text editor 구성
- `Yjs` + `y-websocket` 기반 실시간 협업 연결 구조 분리
- `VITE_WS_URL`이 없을 때도 동작하는 local-only collaboration 모드 지원
- `.docx`를 HTML로 변환하고 sanitize 하는 import 유틸리티 제공
- API 연동 확장을 위한 `src/lib/api` 경계 유지
- `build`, `lint`, `test`, `typecheck` 품질 게이트 유지

## 모듈 의존성

### 외부 라이브러리

| 구분 | 사용 라이브러리 | 목적 |
| --- | --- | --- |
| UI / 앱 구조 | `react`, `react-dom`, `react-router-dom` | 화면 렌더링, 라우팅 |
| 번들링 / 언어 | `vite`, `typescript` | 개발 서버, 번들링, 타입 안정성 |
| 에디터 | `@tiptap/react`, `@tiptap/starter-kit`, `@tiptap/extension-link`, `@tiptap/pm` | 에디터 UI와 기본 편집 기능 |
| 협업 | `@tiptap/extension-collaboration`, `@tiptap/extension-collaboration-caret`, `@tiptap/y-tiptap`, `yjs`, `y-websocket` | 공동 편집 상태 동기화와 커서 표시 |
| 문서 import | `mammoth`, `dompurify` | DOCX -> HTML 변환, sanitize |
| 품질 도구 | `eslint`, `prettier`, `vitest`, `@testing-library/react`, `jsdom` | 정적 검사, 포맷, 테스트 |

### 내부 모듈 의존성

| 모듈 | 역할 | 주요 의존 대상 |
| --- | --- | --- |
| `src/app`, `src/pages` | 앱 진입점, 라우팅, 페이지 조립 | `src/features/*`, `src/shared/*`, `src/lib/*` |
| `src/features/editor` | 에디터 셸, 툴바, extension 조합 | `src/lib/collab`, `src/shared/ui` |
| `src/features/documents` | 문서 목록 화면 구성 | `src/lib/api`, `src/shared/ui` |
| `src/lib/collab` | `Y.Doc`, provider 생성과 해제 | `yjs`, `y-websocket` |
| `src/lib/api` | API URL 생성, fetch helper | `VITE_API_BASE_URL` |
| `src/lib/import` | DOCX import와 sanitize | `mammoth`, `dompurify` |
| `src/shared/config`, `src/shared/types`, `src/shared/ui` | 공용 설정, 타입, UI | 여러 feature에서 공통 사용 |

## 역할 분담

현재 프론트엔드 담당자는 1명이며, 이 저장소의 프론트엔드 범위를 전체 담당합니다.

| 역할 | 현재 운영 방식 | 담당 범위 |
| --- | --- | --- |
| 프론트엔드 오너 | 1인 전체 담당 | 요구사항 반영, 화면 구현, editor UI, 상태 연결, 문서 갱신, 테스트와 배포 전 검증 |

운영 원칙은 아래와 같습니다.

- 담당자는 1명이지만 기능은 브랜치별로 분리해서 작업한다.
- API, route, provider 계약이 바뀌면 코드와 함께 `docs/`도 갱신한다.
- backend/QA 협업이 확대되면 세부 역할 구분은 `docs/roles.md` 기준을 따른다.

## 개발 계획

목표일: `2026-05-07`

### 1주차

- bootstrap 상태를 안정화하고 `build`, `lint`, `test`, `typecheck`가 항상 통과되는 기준선을 유지한다.
- 문서 목록과 editor 진입 흐름을 점검하고 `src/lib/api` 경유 구조를 기준으로 화면 연동 준비를 마친다.
- 협업 에디터의 기본 UX와 라우트 진입 구조를 정리한다.

### 2주차

- documents list/detail API 연동을 진행하고 데이터 흐름을 실제 화면에 연결한다.
- websocket auth, reconnect 정책, connection 상태 처리 기준을 정리한다.
- presence UI와 collaboration 상태 표시 등 실시간 협업 경험을 보강한다.

### 3주차

- `.docx` 업로드 이후 sanitize 결과를 editor ingest 흐름에 연결한다.
- draft 저장, 오류 처리, 예외 상태 메시지 등 마감 전 안정화 작업을 수행한다.
- README, `docs/`, CI 검증 기준을 최종 점검하고 배포 전 문서 상태를 정리한다.

## 커밋 규칙

모든 커밋은 `type(scope): subject` 형식을 사용합니다.

### 허용 type

- `feat`
- `fix`
- `docs`
- `style`
- `refactor`
- `test`
- `chore`
- `perf`
- `build`
- `ci`
- `rename`
- `remove`

### 허용 scope

- `ui`
- `editor`
- `auth`
- `api`
- `state`
- `router`
- `styles`
- `docs`
- `repo`

### 작성 규칙

- subject는 현재형으로 작성한다.
- subject 첫 글자는 소문자로 시작한다.
- subject 끝에 마침표를 붙이지 않는다.
- 무엇이 바뀌었는지 바로 드러나게 구체적으로 작성한다.
- 한 커밋에는 한 가지 목적만 담는다.
- 기능 추가와 리팩터링은 같은 커밋에 섞지 않는다.
- 포맷팅만 한 변경은 별도 커밋으로 분리한다.
- 문서 변경과 기능 변경이 모두 큰 경우에는 커밋을 분리한다.

### 예시

- `feat(editor): add collaboration status badge`
- `fix(api): guard empty base url`
- `docs(repo): rewrite readme for team workflow`

## PR 규칙

- PR은 한 가지 목적만 담는 작은 단위로 만든다.
- PR 생성 전 최신 `main` 기준으로 충돌을 정리한다.
- PR 생성 전 로컬에서 아래 명령을 모두 확인한다.
  - `npm run build`
  - `npm run lint`
  - `npm run test`
  - `npm run typecheck`
- 같은 품질 게이트가 `.github/workflows/ci.yml`에서도 통과되어야 한다.
- UI 변경이 있으면 PR 설명에 변경 화면이나 동작 요약을 함께 남긴다.
- API, route, provider 계약이 바뀌면 관련 `docs/` 문서도 함께 포함한다.
- README나 운영 규칙 변경도 PR 설명에 이유를 명확히 적는다.
- 직접 `main`에 푸시하지 않고 PR로 병합한다.

## 브랜치 규칙

- 모든 작업 브랜치는 `main`에서 분기한다.
- 브랜치 하나당 작업 목적 하나만 담당한다.
- 브랜치 이름은 `<type>/<scope>-<short-kebab-description>` 형식을 권장한다.
- 담당자는 한 명이지만 기능마다 별도 브랜치를 만든다.
- 문서 전용 수정도 가능하면 별도 브랜치로 분리한다.
- 장기 브랜치보다 짧고 작은 브랜치를 선호한다.
- 실험성 작업은 `wip/` 접두사를 사용할 수 있지만, merge 전에는 목적이 드러나는 이름으로 정리한다.
- PR 생성 전 최신 `main` 기준으로 충돌을 정리하고 품질 게이트를 다시 실행한다.

### 브랜치 예시

- `feat/websocket-document-sync`
- `fix/storage-file-snapshot-catalog`
- `docs/repo-readme-refresh`

## 실행 방법

### 요구 사항

- Node.js 20 이상
- npm

### 환경 변수

`.env.example`를 참고해 `.env.local`을 구성합니다.

```bash
VITE_API_BASE_URL=http://localhost:4000/api
VITE_WS_URL=ws://localhost:1234
```

- `VITE_API_BASE_URL`: REST API base URL
- `VITE_WS_URL`: Yjs websocket provider base URL

`VITE_WS_URL`이 비어 있으면 editor는 local-only Yjs mode로 동작합니다.

### 설치

```bash
npm install
```

### 개발 서버 실행

```bash
npm run dev
```

Windows PowerShell에서 `npm.ps1` 실행이 차단되면 아래 형식을 사용합니다.

```powershell
npm.cmd run dev
```

### 검증 및 빌드 명령

```bash
npm run build
npm run lint
npm run test
npm run typecheck
npm run preview
```

### 기본 라우트

- `/`: 문서 목록 placeholder 페이지
- `/docs/:docId`: collaborative editor 페이지

## 참고 문서

- [docs/agent-rules.md](docs/agent-rules.md)
- [docs/setup.md](docs/setup.md)
- [docs/architecture.md](docs/architecture.md)
- [docs/roles.md](docs/roles.md)
- [docs/conventions.md](docs/conventions.md)
- [docs/checklist.md](docs/checklist.md)
- [AGENTS.md](AGENTS.md)
