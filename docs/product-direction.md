# Product Direction Change

Date: 2026-05-27
Last updated: 2026-05-28

This document is an implementation directive for the next automation pass. It records the requested product direction change only. Do not treat this document as evidence that the current implementation already behaves this way.

## Scope

The Google Docs clone should move from a developer/demo-oriented collaborative editor toward a user-facing document workspace. The next implementation pass must remove internal debug/demo surfaces, restore document credential behavior, adopt shadcn/ui for the frontend interface, and make document cards useful to end users.

## Required Changes

### 1. Restore Credential Token Flow

The project previously had document credential token behavior that was later made optional or unused because it complicated the mid-project flow. Restore it as a first-class access control feature.

Requirements:

- Document creation must return a document credential token and the frontend must retain the token needed to reopen or operate on that document.
- Protected document operations must require the matching credential token again. At minimum, detail, rename/title update, delete, security settings, and collaboration entry should be covered.
- WebSocket authentication must have an explicit browser-compatible contract. Because browsers cannot attach arbitrary WebSocket headers, define and document one supported approach such as a query token, a short-lived websocket ticket, or a protocol auth message before realtime sync starts.
- Do not expose document credential tokens in public list metadata, UI debug panels, or logs.
- Update backend and frontend API docs when the contract is implemented.

Acceptance criteria:

- Opening a document without the required credential is blocked or prompts for the credential.
- Opening the same document with the correct credential works across refreshes.
- Collaboration WebSocket refuses unauthenticated access.
- Existing storage adapters continue preserving the document credential token in snapshots.

### 2. Move Frontend Design To shadcn/ui

Replace the current custom visual system with shadcn/ui-based UI.

Requirements:

- Adopt shadcn/ui conventions for buttons, inputs, cards, dialogs, dropdowns, tabs, tooltips, badges, scroll areas, and form controls.
- Replace or retire the bespoke `DesignSystem` and large custom CSS surfaces where shadcn components cover the same role.
- Use Tailwind/CSS variable tokens compatible with shadcn theming.
- Keep the interface quiet, document-workspace oriented, and dense enough for repeated use.
- Preserve editor functionality while changing the shell and document-list UI.

Acceptance criteria:

- Home, document card list, editor shell, toolbar, side panels, dialogs, and security settings use shadcn-style primitives.
- The UI no longer looks like the existing custom dashboard skin.
- Mobile and desktop layouts remain usable.

### 3. Remove Export JSON

The JSON export feature is no longer needed.

Requirements:

- Remove the Export JSON action and any copy/download JSON UI.
- Remove JSON export-specific helper code, tests, labels, and documentation.
- Keep DOCX import unless a separate decision removes it.
- If JSON import exists only to support JSON export snapshots, evaluate and remove it in the same pass.

Acceptance criteria:

- Users cannot export a document as JSON from the UI.
- Editor copy and documentation no longer advertise JSON export.
- Tests do not assert JSON export behavior.

### 4. Fix Document Rename After Create

There appears to be no usable path to rename a document after creating it. Confirm the behavior and fix it.

Requirements:

- Add or expose a title update flow for backend documents.
- Prefer an inline editable title in the editor header, with save/error states.
- Add a backend endpoint if missing, for example `PATCH /api/documents/:id` with a title payload.
- The title update must persist through snapshot storage and remain visible in the document list after refresh.
- Apply credential token checks to rename operations after token restoration.

Acceptance criteria:

- Create document, rename it, refresh the editor, return to list, and see the updated title.
- Empty or whitespace-only titles are rejected or normalized consistently.
- Failed rename attempts show a user-facing error.

### 5. Replace Sample Content

Existing sample content that reads like planning material for cloning popular projects should be removed. Samples may remain only if they are original creative writing or neutral demo documents.

Requirements:

- Remove all "popular project clone coding" planning copy from frontend mock data, seeded backend snapshots, docs, and tests.
- Replace sample documents with original creative content. Examples: short fictional notes, invented story outlines, personal brainstorming, worldbuilding notes, or neutral writing exercises.
- Samples must not imply the app is intended for cloning existing products.

Acceptance criteria:

- Searching the repository for clone-coding planning copy finds no user-facing sample content.
- Fallback/sample document cards and sample editor content use original creative writing.

### 6. Remove Open Sample Editor

The "Open sample editor" action does not have a clear user-facing purpose and should be removed.

Requirements:

- Remove the home page action that opens a local sample editor.
- If backend is unavailable, fallback samples may still be displayed as read-only/list placeholders only if needed for local demo resilience.
- Empty states should guide users to create a document, not open a sample editor.

Acceptance criteria:

- No visible button or link says "Open sample editor".
- Empty state copy does not instruct users to open a sample editor.

### 7. Simplify Summary Metrics

The current three tiles for `Documents`, `Data source`, and `Collaboration` are mostly not meaningful to users. Keep only the document count, or fold the count naturally into the list heading.

Requirements:

- Remove `Backend connected` / `Data source` UI.
- Remove `Realtime enabled` / `Collaboration` summary tile.
- Preserve the document count somewhere useful.

Acceptance criteria:

- The home page no longer has three metric tiles.
- Users can still see how many documents are in the current list.

### 8. Remove Workspace Configuration And Editing Flow Panels

The `Workspace configuration` and `Editing flow` sections expose implementation/debug details that are not meaningful to end users.

Requirements:

- Remove the workspace configuration panel.
- Remove the editing flow panel.
- Remove user-facing display of API base URL, websocket URL, legacy token state, and import/export debug copy.

Acceptance criteria:

- Home page has no configuration/debug panels aimed at developers.
- User-facing copy focuses on documents and editing, not environment wiring.

### 9. Ensure Document List Scrolls

When document count grows, the user must be able to scroll through the list naturally.

Requirements:

- Confirm whether the current document list already scrolls correctly.
- If not, constrain the document list area and use a scrollable container, preferably shadcn `ScrollArea`.
- Avoid layout shifts when many cards are present.

Acceptance criteria:

- A large document list remains usable on desktop and mobile.
- Header/create/search controls remain accessible enough for normal navigation.

### 10. Make Document Cards User-Facing

Document cards should keep collaborator information, but remove backend implementation labels.

Requirements:

- Keep collaborator count and collaborator names where available.
- Remove user-facing labels such as `Backend`, `Backend document ...`, `Source: Backend API`, and similar implementation-source copy.
- Show a short preview of the document content instead of backend/source metadata.
- The preview must be derived from the actual document content, sanitized to plain text, and truncated to the first useful portion.
- If content preview is unavailable, show a neutral placeholder such as `No preview available` rather than backend metadata.

Acceptance criteria:

- Document cards show title, collaborators, updated time, and content preview.
- Document cards do not expose backend/source implementation labels.
- Search can include title and allowed preview text, but should not depend on removed source/status labels.

### 11. Hide Preview For Secured Documents

Keep the document-card changes above, but respect document-level security settings.

Requirements:

- Add or expose a document security setting that can hide content previews from the document list.
- When preview hiding is enabled, the list must not show document body summary, first paragraph, or any derived preview text.
- The hidden preview state should still show useful non-sensitive metadata such as title, updated time, and collaborator names/count.
- Backend list responses should not send preview text for documents whose security settings hide preview. Do not rely only on frontend redaction.
- Credential token checks must apply to reading or changing security settings.

Acceptance criteria:

- A secured document card shows no content preview.
- Network responses for secured document list entries do not include hidden preview text.
- Users with the correct credential can open the document, while the list remains redacted.

### 12. Make The Recent Files Area User-Friendly

The current `Recent workspace files` heading and related copy feel developer-oriented. Replace this area with a user-facing recent documents section that emphasizes document names and direct reopening.

Requirements:

- Replace `Recent workspace files` with a simpler user-facing label such as `Recent files`, `Recent documents`, or a Korean equivalent if the UI language is localized.
- Show the latest document names prominently and make each name/card a clear link to reopen the document.
- Avoid explaining backend state, workspace internals, source labels, or implementation flow in this section.
- Keep search and create actions if they remain useful, but ensure the section reads like a document app rather than a developer dashboard.
- Do not show `Using local sample documents` or similar fallback/debug notices to end users. If backend loading fails, show a normal user-facing empty/error state such as `Documents are temporarily unavailable` with a retry action.
- If fallback sample data is kept for development resilience, it must not be labeled as local sample data in the product UI.

Acceptance criteria:

- The home page does not display `Recent workspace files`.
- The recent section displays document titles as primary clickable links/cards.
- The home page does not display `Using local sample documents`.
- Backend or fallback state is communicated only through user-facing language, not implementation labels.

## Implementation Order

1. Define and document the credential token contract.
2. Add backend APIs needed for title update, preview metadata, and security settings.
3. Update frontend API clients and state for credential-aware document access.
4. Migrate UI shell and reusable controls to shadcn/ui.
5. Remove JSON export, sample editor entry points, metrics/debug panels, and backend-source labels.
6. Replace sample content with original creative content.
7. Replace the `Recent workspace files` and `Using local sample documents` copy with user-facing recent-file behavior.
8. Add list scrolling and preview redaction checks.
9. Update tests and docs to match the new product direction.

## Regression Checks

Frontend:

```bash
cd Front-End
npm run build
npm run lint
npm run test
npm run typecheck
```

Backend:

```bash
cd Back-End
./scripts/verify.sh core
./scripts/verify.sh websocket
```

Manual cross-stack checks:

- Create document -> rename -> refresh -> list title updates.
- Create secured document -> list hides preview -> open with credential works.
- Open document without credential -> access is denied or credential prompt appears.
- Many documents -> list scrolls on desktop and mobile.
- Recent files show document names as clear links/cards.
- No JSON export, Open sample editor, workspace configuration, editing flow, source/backend metadata, or clone-coding sample copy remains visible.
- No `Recent workspace files` or `Using local sample documents` copy remains visible.
