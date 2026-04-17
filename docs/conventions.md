# Conventions

## Naming

- React component: PascalCase
- hook/function/util: camelCase
- route param, file path segment: kebab-case
- shared type name: 도메인 의미가 드러나는 명사형

## Folder Rules

- route entry는 `src/pages`
- feature-specific UI와 로직은 `src/features/<feature>`
- backend/infra helper는 `src/lib`
- 여러 feature가 공유하는 타입과 UI는 `src/shared`
- 문서는 반드시 `docs/` 아래에 둔다
- 하네스와 문서의 경로 표기는 repo root 기준 relative path를 사용한다

## State Handling

- page-local state는 component 내부에 유지한다
- collaboration connection lifecycle은 `src/lib/collab`에서 관리한다
- 전역 상태 라이브러리는 실제 필요가 생기기 전까지 추가하지 않는다

## API Access Rules

- 직접 `fetch`를 분산 호출하지 않고 `src/lib/api`를 경유한다
- API base URL은 `VITE_API_BASE_URL`만 사용한다
- route component는 raw endpoint string을 조합하지 않고 helper를 사용한다

## CSS Strategy

- 전역 토큰과 레이아웃은 `src/index.css`에 둔다
- 불필요한 UI 라이브러리 추가를 피한다
- editor-specific styling은 전역 CSS 안에서 명시적으로 네이밍한다

## Test Rules

- 최소 smoke test를 유지한다
- route/page 테스트는 사용자 관점 텍스트 기준으로 작성한다
- utility test는 import/sanitize 경로의 안정성을 우선 검증한다

## Branch / PR Conventions

- 브랜치 접두사는 기본적으로 `codex/`를 사용한다
- 작은 단위 PR을 선호한다
- PR 전 `build`, `lint`, `test`, `typecheck`를 모두 통과시킨다
- 동일한 검증은 `.github/workflows/ci.yml`에서도 자동 실행한다
- baseline code owner 설정은 `.github/CODEOWNERS`에서 관리한다
- 계약 변경이 있으면 코드보다 먼저 또는 동시에 `docs/`를 갱신한다
