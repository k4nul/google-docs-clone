# Agent Rules

- 변경은 항상 작은 단위로 나눈다.
- 큰 리팩터링은 명시적 합의 없이는 수행하지 않는다.
- 기존 파일과 계약은 최대한 보존하고 필요한 부분만 수정한다.
- 문서 경로는 repo root 기준 relative path로 적는다.
- API, route, import contract를 바꾸면 구현보다 먼저 문서를 갱신한다.
- PR 전에는 반드시 `npm run build`, `npm run test`, `npm run lint`, `npm run typecheck`를 검증한다.
- Windows PowerShell 자동화에서 `npm.ps1`이 차단되면 `npm.cmd run <task>`를 사용한다.
- 동일한 품질 게이트는 CI에서도 유지한다.
- editor/provider 코드는 기능별로 분리하고, 한 파일에 과도하게 몰아넣지 않는다.
- backend가 준비되지 않은 상태에서도 compile-safe를 유지한다.
- 신규 TODO는 빌드를 깨지 않는 범위에서만 남긴다.
