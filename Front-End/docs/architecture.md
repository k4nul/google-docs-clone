# Architecture

## Feature-Based Folder Structure

- `src/app`: 앱 진입점과 라우팅
- `src/pages`: route-level page
- `src/features/editor`: editor shell, toolbar, extension 조합
- `src/features/documents`: 문서 목록 관점 feature
- `src/lib/collab`: Yjs document/provider 생성과 해제 로직
- `src/lib/api`: backend API helper
- `src/lib/import`: DOCX import utility
- `src/shared/config`: 환경변수 파싱
- `src/shared/types`: 공용 타입
- `src/shared/ui`: 공용 레이아웃

## Route Structure

- `/`: backend document list page with local sample fallback
- `/docs/:docId`: collaborative editor page with backend metadata lookup

## Realtime Flow

`EditorPage` -> `GET /api/documents/:id` -> `EditorShell` -> `createCollaborationConnection()` -> `Y.Doc` 생성 -> `WebsocketProvider` 연결 시도 -> Tiptap `Collaboration` extension 연결 -> `CollaborationCaret`는 provider가 있을 때만 활성화

핵심 포인트:

- provider URL은 `VITE_WS_URL`이 있으면 그 값을 쓰고, 없으면 현재 브라우저 origin에서 `ws(s)://<current-host>/ws`로 자동 계산한다.
- API base URL도 `VITE_API_BASE_URL`이 없으면 `<current-origin>/api`로 자동 계산한다.
- browser WebSocket은 임의의 `Authorization` 헤더를 붙일 수 없으므로 프론트엔드는 backend UUID 문서와 origin 정책에 맞춰 연결한다.
- 문서 detail 조회가 실패하면 editor는 local-only provider로 열리고 realtime endpoint는 비활성 상태로 표시된다.
- connection state, transport state, sync 여부, participant count, awareness participant list는 editor 화면에서 확인한다.
- collaboration 사용 시 `StarterKit.history`는 비활성화한다.

## Import Utility

- 위치: `src/lib/import/docxImport.ts`
- 목적: DOCX 파일을 Mammoth로 HTML 변환 후 DOMPurify로 sanitize
- 출력: 향후 editor content ingest에 연결 가능한 typed payload

## API Boundary

- `src/lib/api/httpClient.ts`는 `buildApiUrl()`, `apiGet()`, `apiPost()`, JSON error payload parsing을 담당한다.
- `src/lib/api/documents.ts`는 backend `GET /documents`, `GET /documents/:id`, `POST /documents` 응답을 frontend camelCase document shape로 변환한다.
- `POST /documents`는 현재 backend local-development contract에 맞춰 토큰 없이 동작한다. `VITE_API_TOKEN`이 있으면 legacy compatibility header만 추가한다.
