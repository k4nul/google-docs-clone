use std::{
    env,
    path::{Path, PathBuf},
    process::{Command, Output},
};

const NESTED_GATE_ENV: &str = "BACKEND_ROLE_COMPLETION_NESTED";
const GATE_COMMAND: &str = "cargo test --test backend_role_completion_gate -- --nocapture";
const WINDOWS_SQLITE_STATUS_MARKER: &str = "WINDOWS_SQLITE_SHIM_COMPATIBILITY_DONE";
const FINAL_STATUS_MARKER: &str = "역할 종료 확인";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn bash_compatible_windows_path(path: &str) -> String {
    path.split(';')
        .filter(|entry| !entry.is_empty())
        .flat_map(|entry| {
            let normalized = entry.replace('\\', "/");
            let bytes = normalized.as_bytes();
            if bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic() {
                let drive = (bytes[0] as char).to_ascii_lowercase();
                vec![
                    format!("/mnt/{drive}{}", &normalized[2..]),
                    format!("/{drive}{}", &normalized[2..]),
                ]
            } else {
                vec![normalized]
            }
        })
        .collect::<Vec<_>>()
        .join(":")
}

fn windows_command_path(command_name: &str) -> Option<String> {
    let output = Command::new("where.exe").arg(command_name).output().ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_owned)
        .next()
}

fn windows_command_parent(command_name: &str) -> Option<String> {
    windows_command_path(command_name).and_then(|path| {
        Path::new(&path)
            .parent()
            .map(|parent| parent.display().to_string())
    })
}

fn first_bash_compatible_windows_path(path: &str) -> String {
    bash_compatible_windows_path(path)
        .split(':')
        .next()
        .unwrap_or(path)
        .to_owned()
}

fn run_script(script: &Path, arg: &str) -> Output {
    let root = repo_root();
    let mut command = if cfg!(windows) {
        let relative_script = script
            .strip_prefix(&root)
            .unwrap_or(script)
            .to_string_lossy()
            .replace('\\', "/");
        let mut cargo_path_entries = Vec::new();
        if let Some(cargo_parent) = windows_command_parent("cargo") {
            cargo_path_entries.push(cargo_parent);
        }
        if let Ok(user_profile) = env::var("USERPROFILE") {
            cargo_path_entries.push(format!(r"{user_profile}\.cargo\bin"));
        }
        if cargo_path_entries.is_empty()
            && let (Ok(home_drive), Ok(home_path)) = (env::var("HOMEDRIVE"), env::var("HOMEPATH"))
        {
            cargo_path_entries.push(format!(r"{home_drive}{home_path}\.cargo\bin"));
        }
        let cargo_path_prefix = if cargo_path_entries.is_empty() {
            String::new()
        } else {
            let cargo_bin = bash_compatible_windows_path(&cargo_path_entries.join(";"));
            format!("export PATH=\"{cargo_bin}:$PATH\"; ")
        };
        let cargo_command_prefix = windows_command_path("cargo")
            .or_else(|| {
                env::var("USERPROFILE")
                    .ok()
                    .map(|user_profile| format!(r"{user_profile}\.cargo\bin\cargo.exe"))
            })
            .map(|cargo_path| {
                format!(
                    "export CARGO=\"{}\"; ",
                    first_bash_compatible_windows_path(&cargo_path)
                )
            })
            .unwrap_or_default();
        let mut command = Command::new("bash");
        command.arg("-c");
        command.arg(format!(
            "export {NESTED_GATE_ENV}=1; {cargo_command_prefix}{cargo_path_prefix}bash ./{relative_script} {arg}"
        ));
        command
    } else {
        let mut command = Command::new(script);
        command.arg(arg);
        command
    };

    command
        .current_dir(root)
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

fn latest_current_status_entry(checklist: &str) -> &str {
    let mut in_current_status_section = false;

    for line in checklist.lines() {
        let trimmed = line.trim();

        if trimmed == "## Current Status" {
            in_current_status_section = true;
            continue;
        }

        if !in_current_status_section {
            continue;
        }

        if trimmed.starts_with("## ") {
            break;
        }

        if trimmed.starts_with("- ") {
            return trimmed;
        }
    }

    panic!("checklist current status section should include a status bullet")
}

fn latest_status_entry_marks_completion(status_entry: &str) -> bool {
    status_entry
        .split_once(": ")
        .map(|(_, headline)| {
            headline.starts_with(FINAL_STATUS_MARKER)
                || headline.starts_with(WINDOWS_SQLITE_STATUS_MARKER)
        })
        .unwrap_or(false)
}

#[test]
fn backend_role_completion_gate_uses_first_current_status_bullet() {
    let checklist = r#"# Checklist

## Current Status

종료 판정 규칙: 최상단 항목만 본다.
- 2026-04-26: 역할 종료 확인. completion gate 통과, 추가 작업 없음.
- 2026-04-25: 미완료 다음 작업 1건으로 후속 작업을 진행했다.
"#;

    assert_eq!(
        latest_current_status_entry(checklist),
        "- 2026-04-26: 역할 종료 확인. completion gate 통과, 추가 작업 없음."
    );
}

#[test]
fn backend_role_completion_gate_requires_explicit_completion_marker() {
    assert!(latest_status_entry_marks_completion(
        "- 2026-04-26: 역할 종료 확인. completion gate 통과, 추가 작업 없음."
    ));
    assert!(latest_status_entry_marks_completion(
        "- 2026-04-27: WINDOWS_SQLITE_SHIM_COMPATIBILITY_DONE. Windows SQLite shim role gate passed."
    ));
    assert!(!latest_status_entry_marks_completion(
        "- 2026-04-26: 미완료 다음 작업 1건으로 후속 작업을 진행했다."
    ));
    assert!(!latest_status_entry_marks_completion(
        "- 2026-04-26: 미완료 다음 작업 1건으로 이전 역할 종료 확인 entry를 무효화했다."
    ));
}

#[test]
fn backend_role_completion_gate() {
    if env::var_os(NESTED_GATE_ENV).is_some() {
        return;
    }

    let prompt = include_str!("../.codex/cron-prompt.txt");
    let gitattributes = include_str!("../.gitattributes");
    let agent_rules = include_str!("../docs/agent-rules.md");
    let roles = include_str!("../docs/roles.md");
    let checklist = include_str!("../docs/checklist.md");
    let health_tests = include_str!("../tests/health.rs");
    let rusqlite_shim = include_str!("../vendor/rusqlite/src/lib.rs");
    let preflight_script = include_str!("../scripts/preflight.sh");
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
            && prompt.contains("즉시 종료하라")
            && prompt.contains(FINAL_STATUS_MARKER),
        "cron prompt must stop immediately when the completion gate passes:\n{prompt}"
    );

    assert!(
        roles.contains("## C: Backend Realtime / API Owner")
            && roles.contains("HTTP API, room registry, WebSocket 협업 흐름, CRDT 서버 구조 유지"),
        "roles document no longer defines the backend realtime/API owner scope:\n{roles}"
    );

    assert!(
        agent_rules.contains("platform-specific")
            && agent_rules.contains("WINDOWS_SQLITE_SHIM_COMPATIBILITY_PLAN")
            && agent_rules.contains("vendor/rusqlite/src/lib.rs"),
        "agent rules must record the Windows SQLite shim troubleshooting path:\n{agent_rules}"
    );
    assert!(
        gitattributes.contains("*.sh text eol=lf"),
        "bash verification scripts must stay LF-only for Windows role gates:\n{gitattributes}"
    );
    assert!(
        roles.contains("Windows SQLite shim portability")
            && roles.contains("platform-specific failure"),
        "roles document must assign the Windows SQLite shim fix to C and verification to D:\n{roles}"
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
    let latest_status_entry = latest_current_status_entry(checklist);
    assert!(
        latest_status_entry_marks_completion(latest_status_entry),
        "latest checklist status entry must explicitly mark completion before the gate can pass.\nlatest entry: {latest_status_entry}"
    );
    assert!(
        !latest_status_entry.contains("미완료 다음 작업"),
        "latest checklist status entry still advertises unfinished work.\nlatest entry: {latest_status_entry}"
    );
    for needle in [
        "WINDOWS_SQLITE_SHIM_COMPATIBILITY_PLAN",
        "WINDOWS_SQLITE_SHIM_COMPATIBILITY_DONE",
        "vendor/rusqlite/src/lib.rs",
        "PermissionDenied (os error 5)",
        "cargo test --locked --lib",
    ] {
        assert!(
            checklist.contains(needle),
            "checklist is missing the Windows SQLite shim plan/completion marker: {needle}"
        );
    }
    assert!(
        rusqlite_shim.contains("fn replace_data_file")
            && rusqlite_shim.contains("#[cfg(windows)]")
            && rusqlite_shim.contains("fn sync_parent_directory"),
        "vendored rusqlite shim must keep the Windows-compatible persistence helpers"
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
    assert!(
        verify_script.contains("CARGO_BIN")
            && verify_script.contains(NESTED_GATE_ENV)
            && preflight_script.contains("CARGO_BIN")
            && preflight_script.contains(NESTED_GATE_ENV),
        "Windows role gates must keep cargo override and nested target narrowing in verify scripts"
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
