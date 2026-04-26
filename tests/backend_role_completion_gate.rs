use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const NESTED_GATE_ENV: &str = "BACKEND_ROLE_COMPLETION_NESTED";
const GATE_COMMAND: &str = "cargo test --test backend_role_completion_gate -- --nocapture";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run_script(script: &Path, arg: &str) -> Output {
    Command::new(script)
        .arg(arg)
        .current_dir(repo_root())
        .env(NESTED_GATE_ENV, "1")
        .output()
        .unwrap_or_else(|error| panic!("failed to execute `{}`: {error}", script.display()))
}

fn combined_output(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn backend_role_completion_gate() {
    if env::var_os(NESTED_GATE_ENV).is_some() {
        return;
    }

    let prompt = include_str!("../.codex/cron-prompt.txt");
    let roles = include_str!("../docs/roles.md");
    let checklist = include_str!("../docs/checklist.md");
    let health_tests = include_str!("../tests/health.rs");
    let verify_script = include_str!("../scripts/verify.sh");

    assert!(
        prompt.contains("Rust 서버")
            && prompt.contains("실시간 동기화")
            && prompt.contains("동시 편집 충돌 처리")
            && prompt.contains("저장/복구 구조"),
        "cron prompt no longer describes the backend realtime role:\n{prompt}"
    );
    assert!(
        prompt.contains(GATE_COMMAND),
        "cron prompt must run the completion gate before repeating work:\n{prompt}"
    );
    assert!(
        prompt.contains("추가 구현, 커밋, 푸시, `cargo clean`을 수행하지 말고")
            && prompt.contains("즉시 종료하라"),
        "cron prompt must stop immediately when the completion gate passes:\n{prompt}"
    );

    assert!(
        roles.contains("## C: Backend Realtime / API Owner")
            && roles.contains("HTTP API, room registry, WebSocket 협업 흐름, CRDT 서버 구조 유지"),
        "roles document no longer defines the backend realtime/API owner scope:\n{roles}"
    );

    let unchecked_items: Vec<_> = checklist
        .lines()
        .filter(|line| line.trim_start().starts_with("- [ ]"))
        .collect();
    assert!(
        unchecked_items.is_empty(),
        "checklist still has unchecked items: {unchecked_items:?}"
    );
    assert!(
        checklist.contains("- bootstrap 범위의 백엔드 구현 작업은 모두 완료됐다."),
        "checklist must explicitly declare the backend bootstrap scope complete"
    );
    for needle in [
        "- shared SQLite snapshot/lease 조합의 실제 owner handoff는 두 노드 앱을 동시에 띄운 end-to-end 테스트로 검증됐다.",
        "- managed coordination lease service와 shared SQLite snapshot store를 묶은 실제 owner handoff도 두 노드 앱 회귀 테스트로 검증됐다.",
        "- managed coordination lease service와 managed snapshot service를 함께 묶은 실제 owner handoff도 두 노드 앱 회귀 테스트로 검증됐다.",
    ] {
        assert!(
            checklist.contains(needle),
            "completion checklist is missing terminal backend verification marker: {needle}"
        );
    }

    for needle in [
        "async fn health_endpoint_returns_ok_payload()",
        "async fn create_document_endpoint_creates_document_and_lists_it()",
        "async fn documents_endpoint_lists_snapshot_backed_documents_after_room_eviction()",
        "async fn websocket_endpoint_supports_yrs_sync_handshake_and_update_broadcast()",
        "async fn websocket_endpoint_restores_latest_sqlite_snapshot_after_owner_handoff()",
        "async fn document_detail_restores_latest_sqlite_snapshot_after_managed_owner_handoff()",
        "async fn app_state_restores_latest_managed_snapshot_after_managed_owner_handoff()",
    ] {
        assert!(
            health_tests.contains(needle),
            "health regression inventory is missing required backend acceptance coverage: {needle}"
        );
    }
    assert!(
        verify_script.contains("run_websocket_lane")
            && verify_script.contains("websocket_endpoint_")
            && verify_script.contains(
                "delete_document_endpoint_rejects_documents_with_active_websocket_sessions"
            ),
        "verify script must keep the websocket verification lane split out of the core lane"
    );

    let verify_core = run_script(&repo_root().join("scripts/verify.sh"), "core");
    assert!(
        verify_core.status.success(),
        "backend role is not complete until `./scripts/verify.sh core` is green.\n{}",
        combined_output(&verify_core)
    );

    let websocket_preflight = run_script(&repo_root().join("scripts/preflight.sh"), "websocket");
    if websocket_preflight.status.success() {
        let verify_websocket = run_script(&repo_root().join("scripts/verify.sh"), "websocket");
        assert!(
            verify_websocket.status.success(),
            "backend role is not complete until `./scripts/verify.sh websocket` is green when socket bind is available.\n{}",
            combined_output(&verify_websocket)
        );
    } else {
        let preflight_output = combined_output(&websocket_preflight);
        assert!(
            preflight_output.contains(
                "runner cannot bind socket addresses; websocket verification lane is blocked"
            ),
            "websocket preflight failed for an unexpected reason.\n{preflight_output}"
        );
    }
}
