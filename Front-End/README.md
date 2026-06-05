# Front-End Collaborative Editor

React + TypeScript + Vite 기반의 collaborative editor 프론트엔드 저장소입니다. 현재는 Tiptap, Yjs, 커스텀 WebSocket provider를 중심으로 실시간 편집 셸을 구성했고, 백엔드 문서 API와 협업 WebSocket 계약에 맞춰 실제 로컬 연동이 가능하도록 정리했습니다.

## 핵심 기능

- 백엔드 `GET /api/documents` 기반 최근 문서 목록, body preview, redacted preview 상태, 사용자용 unavailable 상태 제공
- `/docs/:docId` 경로에서 collaborative editor 셸 제공
- Tiptap 기반 rich text editor 구성
- Yjs binary sync protocol 기반 실시간 협업 연결 구조 분리
- `VITE_WS_URL`이 없으면 현재 브라우저 origin에서 WebSocket URL 자동 계산
- DOCX import/export와 HTML sanitize 유틸리티 제공
- `src/lib/api` 경계에서 documents list/detail/create/security 응답 변환
- 문서 생성 응답의 credential을 저장하고 상세 조회, 제목 변경, WebSocket 연결에 재사용
- 편집 중 realtime snapshot persistence 상태를 autosave UI로 표시
- `build`, `lint`, `test`, `typecheck` 품질 게이트 유지

## 모듈 의존성

### 외부 라이브러리

| 구분 | 사용 라이브러리 | 목적 |
| --- | --- | --- |
| UI / 앱 구조 | `react`, `react-dom`, `react-router-dom` | 화면 렌더링, 라우팅 |
| 번들링 / 언어 | `vite`, `typescript` | 개발 서버, 번들링, 타입 안정성 |
| 에디터 | `@tiptap/react`, `@tiptap/starter-kit`, `@tiptap/extension-link`, `@tiptap/pm` | 에디터 UI와 기본 편집 기능 |
| 협업 | `@tiptap/extension-collaboration`, `@tiptap/extension-collaboration-caret`, `@tiptap/y-tiptap`, `yjs`, `lib0`, `y-protocols` | 공동 편집 상태 동기화와 커서 표시 |
| 문서 import/export | `mammoth`, `dompurify`, `docx` | DOCX -> HTML 변환, sanitize, editor HTML -> DOCX 변환 |
| 품질 도구 | `eslint`, `prettier`, `vitest`, `@testing-library/react`, `jsdom` | 정적 검사, 포맷, 테스트 |

### 내부 모듈 의존성

| 모듈 | 역할 | 주요 의존 대상 |
| --- | --- | --- |
| `src/app`, `src/pages` | 앱 진입점, 라우팅, 페이지 조립 | `src/features/*`, `src/shared/*`, `src/lib/*` |
| `src/features/editor` | 에디터 셸, 툴바, extension 조합 | `src/lib/collab`, `src/shared/ui` |
| `src/features/documents` | 문서 목록 화면 구성 | `src/lib/api`, `src/shared/ui` |
| `src/lib/collab` | `Y.Doc`, provider 생성과 해제 | `yjs`, `lib0`, `y-protocols` |
| `src/lib/api` | API URL 생성, fetch helper | `VITE_API_BASE_URL` 또는 현재 origin |
| `src/lib/import` | DOCX import와 sanitize | `mammoth`, `dompurify` |
| `src/lib/export` | editor HTML을 DOCX blob으로 변환 | `docx` |
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

## 현재 구현 상태

이 섹션은 2026-06-04 기준 현재 소스와 `docs/checklist.md`에 맞춘 프론트엔드 상태를 정리한다. 과거 bootstrap 일정 대신 실제 구현된 흐름과 남은 작업을 기준으로 본다.

### 완료된 흐름

- Vite React TypeScript 앱, strict mode, `@/*` alias, GitHub Actions 품질 게이트가 구성되어 있다.
- 홈 화면은 `GET /api/documents`로 최근 문서를 불러오고, 실패하면 사용자용 unavailable 상태와 retry 흐름을 보여준다.
- `New document`는 `POST /api/documents`로 UUID 문서를 만들고, 응답의 `credentials.access_token`을 문서별 `localStorage` credential map에 저장한 뒤 `/docs/:docId`로 이동한다.
- 편집기 화면은 저장된 문서 credential로 `GET /api/documents/:id` metadata를 확인한 뒤에만 editor와 협업 provider를 연다.
- `src/lib/collab/connection.ts`의 `BinaryWebsocketProvider`가 Yjs/Yrs v1 binary sync, awareness, reconnect, periodic resync, token-redacted connection logging을 담당한다.
- browser WebSocket 제약에 맞춰 문서 credential은 `/ws/:docId?access_token=<document-token>` query parameter로 전달한다.
- editor content 변경은 Yjs update를 통해 백엔드 snapshot persistence로 전송되며, 편집기 헤더는 연결/저장/일시 중단 상태를 autosave UI로 표시한다.
- DOCX import는 sanitize 경계를 거쳐 editor content에 반영되고, DOCX export는 editor HTML을 `docx` package model로 변환한다.
- presence participant list, connection status, typing state, last sync time이 editor details surface에 표시된다.

### 남은 작업

- dedicated GitHub users/teams를 준비해 문서상 A/B/C/D 역할 구간을 실제 CODEOWNERS enforcing owner로 연결해야 한다.
- 현재 UI는 `src/shared/ui/DesignSystem.tsx`와 `src/index.css` 기반 custom design system이다. shadcn/ui 전환은 `docs/product-direction.md`의 다음 제품 방향 작업으로 남아 있다.

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
- 같은 품질 게이트가 루트 `.github/workflows/ci.yml`의 frontend job에서도 통과되어야 한다. 프론트엔드 전용 package-local workflow mirror는 두지 않고 루트 workflow를 단일 CI entry point로 유지한다.
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

- Node.js 20.19+, 22.12+, or 24+
- npm

### 환경 변수

API와 WebSocket 기본값은 현재 브라우저가 접속한 origin을 사용합니다. `VITE_API_TOKEN`은 backend `API_TOKEN`과 같은 값이며, 문서 목록과 문서 생성 요청에 사용됩니다. 문서 생성 뒤에는 응답의 `credentials.access_token`을 브라우저 `localStorage`에 저장하고, 상세 조회/제목 변경/WebSocket 연결에 재사용합니다.

```bash
# optional, only needed when the backend is not served through the same origin
VITE_API_BASE_URL=http://localhost:4000/api
VITE_API_TOKEN=dev-admin-token
VITE_WS_URL=ws://localhost:4000
```

- `VITE_API_BASE_URL`: REST API base URL. 없으면 `<current-origin>/api`를 사용합니다.
- `VITE_API_TOKEN`: 문서 목록/생성을 위한 backend admin API token입니다.
- `VITE_WS_URL`: collaboration websocket origin/base host. 없으면 `ws(s)://<current-host>/ws`를 사용합니다. provider는 이 값에서 `/ws/:docId?access_token=<document-token>` endpoint를 구성합니다.

현재 origin 기반 기본값을 쓰면 `localhost`, DDNS, 새 도메인, HTTPS 전환 시 프론트 환경변수를 바꾸지 않아도 됩니다.

환경 변수를 수정했다면 `npm run dev`를 반드시 다시 시작해야 합니다.

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

- `/`: 최근 문서 목록 페이지
- `/docs/:docId`: collaborative editor 페이지

## 백엔드 연동 흐름

현재 프런트는 백엔드 README의 문서 생성/협업 연결 계약에 맞춰 동작합니다. `POST /api/documents` 응답 credential을 저장한 뒤 `GET /api/documents/:id`, `PATCH /api/documents/:id`, `/ws/:docId?access_token=<token>`에 재사용합니다. `GET /api/documents`의 `preview`는 최근 문서 카드에 표시하고, `preview_hidden=true` 또는 `hide_preview=true`이면 body preview 대신 redacted 상태를 표시합니다. 저장된 credential이 없으면 편집기 페이지는 문서 내용을 열지 않고 access token 입력을 요구합니다.

1. 홈 화면에서 `New document`를 클릭합니다.
2. 프런트가 `VITE_API_TOKEN`을 bearer token으로 사용해 `POST /api/documents`를 호출합니다.
3. 백엔드가 문서를 만들고 `document.id`와 `credentials.access_token`을 응답합니다.
4. 프런트는 credential을 `localStorage`에 저장하고 `/docs/:docId`로 이동합니다.
5. 프런트가 저장된 credential로 `GET /api/documents/:id` detail metadata를 확인합니다.
6. 협업 연결은 `ws://host/ws/:docId?access_token=<document-token>` 형태로 열립니다.

문서 목록을 불러올 수 없으면 홈 화면은 사용자용 unavailable 상태와 재시도 버튼을 표시합니다. 실제 협업 WebSocket은 백엔드가 생성한 UUID 문서에서만 정상 연결됩니다.

## 로컬 협업 연결 체크포인트

로컬에서 협업 연결이 끊기거나 `disconnected` 상태가 계속되면 아래 경계를 순서대로 확인합니다. 각 항목은 현재 구현이 의존하는 연결 조건입니다.

### 1. 잘못된 WebSocket 주소 사용

프런트의 `VITE_WS_URL`은 백엔드의 WebSocket host를 가리켜야 합니다. 기본 로컬 백엔드는 `127.0.0.1:4000`에서 실행되므로 direct backend URL을 쓸 때는 `ws://localhost:4000`을 사용합니다.

해결:

- `VITE_WS_URL`을 `ws://localhost:4000`으로 수정
- 프런트 dev 서버 재시작

### 2. backend에 없는 fallback 문서 사용

홈 화면의 fallback 문서 카드는 백엔드 문서 목록을 불러오지 못할 때 개발용으로 렌더링될 수 있지만, 백엔드 persisted document라고 보장하지 않습니다. WebSocket 협업은 UUID 형식뿐 아니라 `POST /api/documents`로 생성되어 백엔드 catalog에 존재하는 문서에서 정상 동작합니다.

해결:

- 홈 화면에 `New document` 버튼 추가
- `POST /api/documents`로 실제 문서를 생성한 뒤 해당 UUID로 이동

### 3. 문서 경로 전달 정리

문서 생성 후 협업 페이지로 이동할 때 browser route에는 문서 ID만 넣고, credential은 WebSocket endpoint에만 붙입니다.

해결:

- 프런트 문서 생성 응답에서 `document.id`만 사용
- 프런트 라우팅을 `/docs/:docId` 형태로 고정
- WebSocket endpoint는 `ws://host/ws/:docId?access_token=<document-token>`로 구성하고, browser route에는 token을 붙이지 않음

### 4. awareness payload shape 불일치

서버는 awareness payload를 검증합니다. collaborator 정보가 반영되지 않으면 프런트 awareness local state가 서버 계약의 `user`, `client`, 선택적 `selection` 구조를 유지하는지 확인합니다.

해결:

- awareness local state를 `user`, `client`, 선택적 `selection` 구조에 맞춰 정리
- 계약 외 필드 전송 제거

### 5. React StrictMode로 인한 개발 모드 이중 연결/해제

개발 모드에서 연결 직후 cleanup이 먼저 실행되면 서버에 불완전한 binary message가 들어가고 브라우저에는 `WebSocket is closed before the connection is established`가 보일 수 있습니다. 현재 구현은 `src/main.tsx`에서 React `StrictMode`를 사용하지 않고, provider cleanup을 지연/취소할 수 있게 정리했습니다.

해결:

- `src/main.tsx`에서 `StrictMode` 제거
- provider cleanup 로직을 보강해 연결 중 소켓 정리 시 불필요한 조기 종료를 줄임
- 콘솔에 endpoint, close code, wasClean, reason을 남기도록 디버그 로그 추가

## 디버깅 체크리스트

협업 연결이 안 붙으면 아래 순서로 확인합니다.

1. 백엔드가 `http://localhost:4000`에서 실행 중인지 확인
2. 프런트 `.env`에 아래 값이 들어있는지 확인

```bash
VITE_API_BASE_URL=http://localhost:4000/api
VITE_WS_URL=ws://localhost:4000
```

3. 프런트 dev 서버를 재시작
4. 홈 화면에서 문서 목록을 확인하거나 `New document`를 클릭
5. 브라우저 URL이 `/docs/<uuid>` 형태인지 확인
6. 콘솔에 아래 로그가 찍히는지 확인

```text
[collab] websocket connect requested {
  endpoint: "ws://localhost:4000/ws/<uuid>",
  roomId: "<uuid>",
  status: "connecting"
}
```

7. 백엔드 로그에 `websocket collaboration session started` 이후 경고가 없는지 확인

## 현재 로그 포인트

협업 연결 디버깅을 위해 브라우저 콘솔에 아래 로그를 남깁니다.

- `websocket connect requested`
- `websocket connected`
- `websocket error`
- `websocket closed`

`websocket closed` 로그에는 `code`, `wasClean`, `reason`, `willReconnect`가 포함됩니다.

## 참고 문서

- [docs/agent-rules.md](docs/agent-rules.md)
- [docs/setup.md](docs/setup.md)
- [docs/architecture.md](docs/architecture.md)
- [docs/roles.md](docs/roles.md)
- [docs/conventions.md](docs/conventions.md)
- [docs/checklist.md](docs/checklist.md)
- [AGENTS.md](AGENTS.md)
