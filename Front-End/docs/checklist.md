# Checklist

## Bootstrap Complete

- [x] Vite React TypeScript app를 현재 디렉터리에 초기화
- [x] npm lockfile 생성 및 npm 유지
- [x] strict mode와 `@/*` alias 구성
- [x] collaborative editor 최소 셸 생성
- [x] Yjs document + websocket provider 초기화 로직 분리
- [x] DOCX import utility 생성
- [x] README와 `docs/` 문서 작성
- [x] smoke test 추가
- [x] GitHub Actions quality gate 구성
- [x] CODEOWNERS baseline 파일 추가

## Next TODO

- [x] 실제 documents list/detail API 연동
- [x] websocket auth와 reconnect 정책 정의
- [x] import UI에서 `.docx` 업로드 후 editor ingest 연결
- [x] presence participant list와 connection status indicator 고도화
- [x] persisted draft/save mutation 및 error handling 추가
- [x] shadcn-compatible shared UI primitives로 bespoke `DesignSystem.tsx` surface retired
- [x] 루트 `.github/CODEOWNERS`의 `@System-Docs-H` baseline으로 문서상 A/B/C/D 역할 구간을 enforcing owner 경계에 연결

## External Owner Provisioning

- dedicated GitHub users/teams가 준비되면 `@System-Docs-H` baseline owner를 실제 팀/사용자 핸들로 교체한다. 이 계정 provisioning은 프론트엔드 구현 완료나 로컬 검증 완료를 막는 구현 gate가 아니다.
