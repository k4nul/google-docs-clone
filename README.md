# Front-End Collaborative Editor

React + TypeScript + Vite 기반의 collaborative editor 프론트엔드 저장소입니다. 현재는 Tiptap, Yjs, 커스텀 WebSocket provider를 중심으로 실시간 편집 셸을 구성했고, 백엔드 문서 API와 협업 WebSocket 계약에 맞춰 실제 로컬 연동이 가능하도록 정리했습니다.

## 핵심 기능

- 문서 목록 placeholder 페이지 제공
- `/docs/:docId` 경로에서 collaborative editor 셸 제공
- Tiptap 기반 rich text editor 구성
- Yjs binary sync protocol 기반 실시간 협업 연결 구조 분리
- `VITE_WS_URL`이 없으면 현재 브라우저 origin에서 WebSocket URL 자동 계산
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
| 협업 | `@tiptap/extension-collaboration`, `@tiptap/extension-collaboration-caret`, `@tiptap/y-tiptap`, `yjs`, `lib0`, `y-protocols` | 공동 편집 상태 동기화와 커서 표시 |
| 문서 import | `mammoth`, `dompurify` | DOCX -> HTML 변환, sanitize |
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

API와 WebSocket 기본값은 현재 브라우저가 접속한 origin을 사용합니다. 문서 생성 기능을 쓰려면 `VITE_API_TOKEN`은 별도로 설정해야 합니다.

```bash
# optional, only needed when the backend is not served through the same origin
VITE_API_BASE_URL=http://localhost:4000/api
VITE_API_TOKEN=dev-admin-token
VITE_WS_URL=ws://localhost:4000/ws
```

- `VITE_API_BASE_URL`: REST API base URL. 없으면 `<current-origin>/api`를 사용합니다.
- `VITE_API_TOKEN`: 문서 생성용 admin API token
- `VITE_WS_URL`: collaboration websocket base URL. 없으면 `ws(s)://<current-host>/ws`를 사용합니다.

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

- `/`: 문서 목록 placeholder 페이지
- `/docs/:docId`: collaborative editor 페이지

## 백엔드 연동 흐름

현재 프런트는 백엔드 README의 문서 생성/협업 연결 계약에 맞춰 동작합니다.

1. 홈 화면에서 `Create backend editor`를 클릭합니다.
2. 프런트가 `POST /api/documents`를 `Authorization: Bearer <VITE_API_TOKEN>` 헤더와 함께 호출합니다.
3. 백엔드가 문서를 만들고 `document.id`를 응답합니다.
4. 프런트는 `/docs/:docId`로 이동합니다.
5. 협업 연결은 `ws://host/ws/:docId` 형태로 열립니다.

즉, mock 문서 경로가 아니라 백엔드가 생성한 UUID 문서 경로로 들어가야 실제 협업이 연결됩니다.

## 이번 문제와 해결

이번 로컬 연동에서 연결이 실패한 이유는 한 가지가 아니라 여러 단계가 겹쳐 있었습니다.

### 1. 잘못된 WebSocket 주소 사용

초기 설정은 `ws://localhost:1234`를 가리켰지만, 실제 백엔드는 `127.0.0.1:4000`에서 실행 중이었습니다. 그래서 프런트가 잘못된 포트로 연결을 시도해 `disconnected`가 발생했습니다.

해결:

- `VITE_WS_URL`을 `ws://localhost:4000`으로 수정
- 프런트 dev 서버 재시작

### 2. mock 문서 ID 사용

홈 화면에서 사용하던 `launch-plan` 같은 mock 문서 ID는 백엔드 WebSocket이 요구하는 UUID 문서가 아니었습니다. 백엔드는 `/ws/:doc_id`에서 UUID 형식 문서만 허용합니다.

해결:

- 홈 화면에 `Create backend editor` 버튼 추가
- `POST /api/documents`로 실제 문서를 생성한 뒤 해당 UUID로 이동

### 3. 문서 경로 전달 정리

문서 생성 후 협업 페이지로 이동할 때 불필요한 query parameter를 붙이지 않고, 문서 ID만으로 이동하도록 단순화했습니다.

해결:

- 프런트 문서 생성 응답에서 `document.id`만 사용
- 프런트 라우팅을 `/docs/:docId` 형태로 고정
- WebSocket endpoint를 `ws://host/ws/:docId`로 단순화

### 4. awareness payload shape 불일치

서버는 awareness payload를 검증합니다. 초기에 프런트는 서버 계약에 없는 shape를 보내고 있어 collaborator 정보가 반영되지 않을 수 있었습니다.

해결:

- awareness local state를 `user`, `client`, 선택적 `selection` 구조에 맞춰 정리
- 계약 외 필드 전송 제거

### 5. React StrictMode로 인한 개발 모드 이중 연결/해제

개발 모드에서는 `StrictMode` 때문에 `useEffect`가 mount-cleanup-mount 순서로 한 번 더 실행됩니다. 협업 소켓처럼 연결 부작용이 있는 코드에서는 연결 직후 cleanup이 먼저 실행되어, 서버에는 불완전한 바이너리 메시지가 들어가고 브라우저에는 `WebSocket is closed before the connection is established`가 보일 수 있었습니다.

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
VITE_API_TOKEN=dev-admin-token
VITE_WS_URL=ws://localhost:4000
```

3. 프런트 dev 서버를 재시작
4. 홈 화면에서 `Open mock editor`가 아니라 `Create backend editor`를 클릭
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
