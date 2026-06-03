# Architecture

## Feature-Based Folder Structure

- `src/app`: 앱 진입점과 라우팅
- `src/pages`: route-level page
- `src/features/editor`: editor shell, toolbar, extension 조합
- `src/features/documents`: 문서 목록 관점 feature
- `src/lib/collab`: Yjs document/provider 생성과 해제 로직
- `src/lib/api`: backend API helper
- `src/lib/import`: DOCX import utility
- `src/lib/export`: DOCX export utility
- `src/shared/config`: 환경변수 파싱
- `src/shared/types`: 공용 타입
- `src/shared/ui`: 공용 레이아웃

## Boundary Rules

- `src/main.tsx`와 `src/app/*`는 React root, router provider, route table만 담당한다.
- `src/pages/*`는 URL param, route-level loading/error state, backend metadata orchestration, feature composition을 담당한다. Tiptap extension, Yjs binary protocol, low-level fetch request construction은 page에서 직접 다루지 않는다.
- `src/features/editor/*`는 editor shell, toolbar, Tiptap extension composition, realtime connection lifecycle UI를 담당한다. backend REST 호출과 document credential persistence는 `src/lib/api` 경계를 통해서만 사용한다.
- `src/features/documents/*`는 document list presentation과 user action surface를 담당한다. document transport shape 변환은 `src/lib/api/documents.ts`에 둔다.
- `src/lib/api/*`는 fetch, URL construction, backend response mapping, document credential storage를 담당한다. `pages`나 `features`를 import하지 않는다.
- `src/lib/collab/*`는 Y.Doc, WebSocket endpoint construction, Yjs/Yrs binary sync protocol, provider cleanup을 담당한다. React component state와 DOM rendering을 import하지 않는다.
- `src/lib/import`와 `src/lib/export`는 file conversion boundary다. editor feature는 import/export result만 받아 editor content에 적용한다.
- `src/shared/*`는 cross-feature config, type, UI primitive만 제공한다. shared module이 route-specific orchestration을 소유하지 않는다.

## Route Structure

- `/`: backend document list page with user-facing unavailable state
- `/docs/:docId`: collaborative editor page with document detail lookup

## Realtime Flow

`EditorPage` -> `GET /api/documents/:id` -> `EditorShell` -> `createCollaborationConnection()` -> `Y.Doc` 생성 -> `WebsocketProvider` 연결 시도 -> Tiptap `Collaboration` extension 연결 -> `CollaborationCaret`는 provider가 있을 때만 활성화

핵심 포인트:

- provider base는 `VITE_WS_URL`이 있으면 그 값을 쓰고, 없으면 현재 브라우저 origin에서 `ws(s)://<current-host>/ws`로 자동 계산한다. 실제 room endpoint는 `/ws/:docId?access_token=<document-token>`으로 구성한다.
- API base URL도 `VITE_API_BASE_URL`이 없으면 `<current-origin>/api`로 자동 계산한다.
- browser WebSocket은 임의의 `Authorization` 헤더를 붙일 수 없으므로 프론트엔드는 backend UUID 문서, 저장된 document credential, origin 정책에 맞춰 연결한다.
- 문서 credential이 없으면 editor는 내용을 열지 않고 access token 입력을 요구한다. 저장된 credential로 문서 detail 조회가 실패하면 metadata unavailable 상태와 token 재입력 폼을 표시하고 realtime provider를 열지 않는다.
- connection state, sync 여부, participant count, awareness participant list는 editor 화면에서 확인한다.
- collaboration 사용 시 `StarterKit.history`는 비활성화한다.

## Import / Export Utilities

- 위치: `src/lib/import/docxImport.ts`
- 목적: DOCX 파일을 Mammoth로 HTML 변환 후 DOMPurify로 sanitize
- `src/features/util/FileManager.tsx`는 DOCX 업로드 결과를 editor에 넣기 전에 같은 sanitize 경계를 통과시킨다.
- 출력: editor content ingest에 연결 가능한 typed payload
- 위치: `src/lib/export/documentExport.ts`
- 목적: editor HTML을 `docx` 패키지의 paragraph/run model로 변환한 뒤 DOCX blob으로 pack
- `src/features/util/FileManager.tsx`는 editor가 준비된 뒤 `Import DOCX`와 `Export DOCX` action을 제공한다.

## API Boundary

- `src/lib/api/httpClient.ts`는 `buildApiUrl()`, `apiGet()`, `apiPost()`, `apiPatch()`, JSON error payload parsing을 담당한다.
- `src/lib/api/documents.ts`는 backend `GET /documents`, `GET /documents/:id`, `POST /documents`, `PATCH /documents/:id` 응답을 frontend camelCase document shape로 변환한다.
- document list summary는 backend가 제공하는 preview/summary를 우선 사용하고, preview가 없거나 hidden이면 사용자용 placeholder를 표시한다.
- `GET /documents`와 `POST /documents`는 `VITE_API_TOKEN`을 admin bearer token으로 사용한다. `POST /documents` 응답의 `credentials.access_token`은 문서별로 `localStorage`에 저장되고, detail/rename REST 요청과 WebSocket `access_token` query parameter에 재사용된다.
