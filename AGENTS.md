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
