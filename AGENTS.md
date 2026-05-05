{
  "repo": "backend",
  "goal": "bootstrap-collaborative-editor-server",
  "stack": {
    "language": "rust",
    "framework": "axum",
    "runtime": "tokio",
    "crdt": "yrs",
    "ws_adapter": "yrs-axum"
  },
  "docs": {
    "root": "docs",
    "agent_rules": "docs/agent-rules.md",
    "setup": "docs/setup.md",
    "architecture": "docs/architecture.md",
    "api": "docs/api.md",
    "roles": "docs/roles.md",
    "conventions": "docs/conventions.md",
    "checklist": "docs/checklist.md"
  },
  "owners": {
    "pm_integration": "A",
    "frontend_editor_ui": "B",
    "backend_realtime_api": "C",
    "qa_docs_devops": "D"
  },
  "env": {
    "host": "HOST",
    "port": "PORT",
    "frontend_origin": "FRONTEND_ORIGIN",
    "rust_log": "RUST_LOG"
  },
  "platform_rules": {
    "windows_sqlite_shim": {
      "issue": "Windows cargo test failed with PermissionDenied (os error 5) in vendored rusqlite persistence; do not classify it as missing SQLite.",
      "plan": "Track WINDOWS_SQLITE_SHIM_COMPATIBILITY_PLAN in docs/checklist.md. Owner C fixes vendor/rusqlite/src/lib.rs and owner D verifies cargo test --locked --lib plus ./scripts/verify.sh core before completion."
    }
  },
  "commands": {
    "run": "cargo run",
    "check": "cargo check",
    "fmt": "cargo fmt --check",
    "test": "cargo test"
  },
  "commit": {
    "message_format": "type(scope): subject",
    "types": [
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
    "scopes": [
      "api",
      "sync",
      "yrs",
      "auth",
      "db",
      "websocket",
      "storage",
      "config",
      "docs",
      "repo"
    ],
    "subject_rules": [
      "present-tense",
      "lowercase-first-letter",
      "no-trailing-period",
      "specific-change-description"
    ],
    "work_rules": [
      "single-purpose-per-commit",
      "do-not-mix-refactor-and-behavior-change",
      "update-related-docs-and-tests-on-schema-or-api-change",
      "run-build-test-lint-when-possible",
      "mark-uncertain-work-as-todo-or-blocked"
    ]
  }
}
