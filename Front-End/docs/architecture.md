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

- `/`: document list placeholder page
- `/docs/:docId`: collaborative editor page

## Realtime Flow

`EditorPage` -> `EditorShell` -> `createCollaborationConnection()` -> `Y.Doc` 생성 -> `WebsocketProvider` 연결 시도 -> Tiptap `Collaboration` extension 연결 -> `CollaborationCaret`는 provider가 있을 때만 활성화

핵심 포인트:

- provider URL은 `VITE_WS_URL`이 있으면 그 값을 쓰고, 없으면 현재 브라우저 origin에서 `ws(s)://<current-host>/ws`로 자동 계산한다.
- API base URL도 `VITE_API_BASE_URL`이 없으면 `<current-origin>/api`로 자동 계산한다.
- collaboration 사용 시 `StarterKit.history`는 비활성화한다.

## Import Utility

- 위치: `src/lib/import/docxImport.ts`
- 목적: DOCX 파일을 Mammoth로 HTML 변환 후 DOMPurify로 sanitize
- 출력: 향후 editor content ingest에 연결 가능한 typed payload

## API Boundary

- 현재는 `buildApiUrl()`와 `apiGet()`만 두고 실제 fetch contract는 최소화했다.
- backend 준비 시 documents query/mutation 로직은 `src/lib/api` 아래에서 확장한다.
