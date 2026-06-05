use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("Back-End should live under the repository root")
        .to_path_buf()
}

fn assert_contains(haystack: &str, needle: &str, context: &str) {
    assert!(
        haystack.contains(needle),
        "{context} is missing required text: {needle}"
    );
}

fn assert_not_contains(haystack: &str, needle: &str, context: &str) {
    assert!(
        !haystack.contains(needle),
        "{context} still contains retired text: {needle}"
    );
}

fn latest_section_bullet<'a>(document: &'a str, heading: &str) -> &'a str {
    let mut in_section = false;

    for line in document.lines() {
        let trimmed = line.trim();

        if trimmed == heading {
            in_section = true;
            continue;
        }

        if !in_section {
            continue;
        }

        if trimmed.starts_with("## ") {
            break;
        }

        if trimmed.starts_with("- ") {
            return trimmed;
        }
    }

    panic!("{heading} should include a status bullet")
}

fn command_output(output: &Output) -> String {
    format!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn qa_docs_devops_completion_gate() {
    let root = repo_root();
    let root_ci = include_str!("../../.github/workflows/ci.yml");
    let codeowners = include_str!("../../.github/CODEOWNERS");
    let root_testing_doc = include_str!("../../docs/testing.md");
    let frontend_readme = include_str!("../../Front-End/README.md");
    let frontend_setup = include_str!("../../Front-End/docs/setup.md");
    let frontend_roles = include_str!("../../Front-End/docs/roles.md");
    let frontend_conventions = include_str!("../../Front-End/docs/conventions.md");
    let backend_readme = include_str!("../README.md");
    let backend_setup = include_str!("../docs/setup.md");
    let checklist = include_str!("../docs/checklist.md");
    let verify_script = include_str!("../scripts/verify.sh");
    let preflight_script = include_str!("../scripts/preflight.sh");

    for (path, doc) in [
        ("docs/testing.md", root_testing_doc),
        ("Front-End/README.md", frontend_readme),
        ("Front-End/docs/setup.md", frontend_setup),
        ("Front-End/docs/conventions.md", frontend_conventions),
    ] {
        assert_contains(doc, ".github/workflows/ci.yml", path);
        assert_not_contains(doc, "Front-End/.github/workflows/ci.yml", path);
        assert_not_contains(doc, "package-local mirror", path);
    }
    assert_contains(
        frontend_roles,
        ".github/CODEOWNERS",
        "Front-End/docs/roles.md",
    );
    assert_not_contains(
        frontend_roles,
        "Front-End/.github/CODEOWNERS",
        "Front-End/docs/roles.md",
    );

    assert!(
        !root.join("Front-End/.github/workflows/ci.yml").exists(),
        "frontend-local workflow mirror should not exist because GitHub only reads repository-level workflows"
    );
    assert!(
        !root.join("Front-End/.github/CODEOWNERS").exists(),
        "frontend-local CODEOWNERS should not exist because repository-level CODEOWNERS is the active owner file"
    );

    for needle in [
        "working-directory: Front-End",
        "cache-dependency-path: Front-End/package-lock.json",
        "npm ci",
        "npm run build",
        "npm run lint",
        "npm run test",
        "npm run typecheck",
        "working-directory: Back-End",
        "rustup toolchain install stable --profile minimal --component rustfmt",
        "./scripts/verify.sh core",
    ] {
        assert_contains(root_ci, needle, ".github/workflows/ci.yml");
    }
    assert_not_contains(root_ci, "pkg-config libssl-dev", ".github/workflows/ci.yml");

    for needle in [
        "* @System-Docs-H",
        "/.github/ @System-Docs-H",
        "/Front-End/package-lock.json @System-Docs-H",
        "/Front-End/src/lib/collab/ @System-Docs-H",
        "/Back-End/Cargo.toml @System-Docs-H",
        "/Back-End/scripts/ @System-Docs-H",
        "/Back-End/tests/ @System-Docs-H",
    ] {
        assert_contains(codeowners, needle, ".github/CODEOWNERS");
    }

    for needle in [
        "./scripts/verify.sh core",
        "./scripts/preflight.sh publish",
        "./scripts/verify.sh websocket",
        "cargo check --features full-snapshot-stores",
    ] {
        assert_contains(backend_readme, needle, "Back-End/README.md");
        assert_contains(backend_setup, needle, "Back-End/docs/setup.md");
    }

    for needle in [
        "CORE_SKIP_FILTERS",
        "--skip\" \"qa_docs_devops_completion_gate",
        "run_core_lane",
        "run_websocket_lane",
    ] {
        assert_contains(verify_script, needle, "Back-End/scripts/verify.sh");
    }
    for needle in [
        "git -C \"$ROOT_DIR\" rev-parse --git-path",
        "check_git_write",
        "check_github_dns",
        "check_socket_bind",
        "commit)",
        "publish)",
        "websocket)",
    ] {
        assert_contains(preflight_script, needle, "Back-End/scripts/preflight.sh");
    }

    let preflight_commit = Command::new(root.join("Back-End/scripts/preflight.sh"))
        .arg("commit")
        .current_dir(root.join("Back-End"))
        .output()
        .expect("preflight commit command should run");
    if !preflight_commit.status.success() {
        let output = command_output(&preflight_commit);
        assert!(
            output.contains("cannot create lock files under .git")
                || output.contains("Read-only file system"),
            "commit preflight should either pass or report the generated-worktree git metadata blocker.\n{output}"
        );
    }

    assert_contains(
        checklist,
        "- [x] D: QA / Docs / DevOps owner complete and `cargo test --test qa_docs_devops_completion_gate -- --nocapture` is green.",
        "Back-End/docs/checklist.md",
    );
    let d_status = latest_section_bullet(checklist, "## QA / Docs / DevOps Current Status");
    assert_contains(d_status, "D 역할 종료 확인", "latest D status");
    assert_not_contains(d_status, "미완료", "latest D status");
}
