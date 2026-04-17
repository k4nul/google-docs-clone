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
  "commands": {
    "run": "cargo run",
    "check": "cargo check",
    "fmt": "cargo fmt --check",
    "test": "cargo test"
  }
}
