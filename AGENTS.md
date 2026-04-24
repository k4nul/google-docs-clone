{
  "repo": "frontend",
  "goal": "bootstrap-collaborative-editor-ui",
  "stack": {
    "framework": "react",
    "bundler": "vite",
    "language": "typescript",
    "editor": "tiptap",
    "sync": "yjs",
    "provider": "y-websocket"
  },
  "docs": {
    "root": "docs",
    "agent_rules": "docs/agent-rules.md",
    "setup": "docs/setup.md",
    "architecture": "docs/architecture.md",
    "roles": "docs/roles.md",
    "conventions": "docs/conventions.md",
    "checklist": "docs/checklist.md"
  },
  "roles": {
    "pm_integration": "A",
    "frontend_editor_ui": "B",
    "backend_realtime_api": "C",
    "qa_docs_devops": "D"
  },
  "ownership_enforcement": {
    "codeowners_baseline": "@System-Docs-H",
    "role_scoped_paths_documented": true,
    "dedicated_github_handles_pending": true
  },
  "env": {
    "api_base": "VITE_API_BASE_URL",
    "ws_base": "VITE_WS_URL"
  },
  "commands": {
    "dev": "npm run dev",
    "build": "npm run build",
    "preview": "npm run preview",
    "lint": "npm run lint",
    "test": "npm run test",
    "typecheck": "npm run typecheck"
  },
  "commit_policy": {
    "format": "type(scope): subject",
    "allowed_types": [
      "feat",
      "fix",
      "docs",
      "style",
      "refactor",
      "test",
      "chore",
      "perf",
      "build",
      "ci",
      "rename",
      "remove"
    ],
    "allowed_scopes": [
      "ui",
      "editor",
      "auth",
      "api",
      "state",
      "router",
      "styles",
      "docs",
      "repo"
    ],
    "subject_rules": {
      "tense": "present",
      "lowercase_first_letter": true,
      "no_trailing_period": true,
      "avoid_vague_subjects": true,
      "describe_exact_change": true
    },
    "change_isolation": {
      "single_purpose_per_commit": true,
      "do_not_mix_feature_and_refactor": true,
      "separate_formatting_only_commits": true,
      "split_large_docs_and_feature_changes": true
    }
  },
  "branch_policy": {
    "base_branch": "main",
    "single_purpose_per_branch": true,
    "preferred_name_format": "<type>/<scope>-<short-kebab-description>",
    "prefer_short_lived_branches": true,
    "reconcile_with_latest_main_before_pr": true,
    "wip_prefix_allowed_for_experiments": true,
    "rename_wip_branch_before_merge": true,
    "examples": [
      "feat/websocket-document-sync",
      "fix/storage-file-snapshot-catalog",
      "docs/repo-readme-refresh"
    ]
  },
  "harness": {
    "path_base": "repo_root",
    "powershell_restricted_commands": {
      "dev": "npm.cmd run dev",
      "build": "npm.cmd run build",
      "preview": "npm.cmd run preview",
      "lint": "npm.cmd run lint",
      "test": "npm.cmd run test",
      "typecheck": "npm.cmd run typecheck"
    },
    "ci_workflow": ".github/workflows/ci.yml",
    "required_checks": ["build", "lint", "test", "typecheck"],
    "codeowners_file": ".github/CODEOWNERS"
  },
  "done": {
    "scaffolded": true,
    "deps_installed": true,
    "docs_written": true,
    "build_ready": true,
    "test_ready": true
  }
}
