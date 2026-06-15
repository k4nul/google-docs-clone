#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LANE="${1:-core}"
CARGO_BIN="${CARGO:-cargo}"
NESTED_TARGET_DIR=""

if [[ -n "${BACKEND_ROLE_COMPLETION_NESTED:-}" ]]; then
    NESTED_TARGET_DIR="${BACKEND_ROLE_COMPLETION_TARGET_DIR:-$ROOT_DIR/target/backend-role-completion-nested}"
    mkdir -p "$NESTED_TARGET_DIR"
fi

CORE_ONLY_SKIP_FILTERS=(
    "--skip" "backend_role_completion_gate"
    "--skip" "project_completion_gate"
    "--skip" "qa_docs_devops_completion_gate"
)

WEBSOCKET_TEST_FILTERS=(
    "websocket_endpoint_"
    "websocket_room_coordinator_tracks_first_and_last_session"
    "websocket_room_activation_failure_does_not_leak_active_sessions"
    "delete_document_endpoint_rejects_documents_with_active_websocket_sessions"
    "delete_document_endpoint_allows_delete_after_websocket_session_closes"
    "document_detail_restores_latest_sqlite_snapshot_after_managed_owner_handoff"
    "app_state_restores_latest_managed_snapshot_after_managed_owner_handoff"
    "app_state_uses_managed_room_coordination_from_config"
    "app_state_uses_managed_snapshot_store_from_config"
    "app_state_uses_s3_snapshot_store_from_config"
    "managed_snapshot_store_"
    "s3_snapshot_store_"
)

CORE_SKIP_FILTERS=("${CORE_ONLY_SKIP_FILTERS[@]}")
for test_filter in "${WEBSOCKET_TEST_FILTERS[@]}"; do
    CORE_SKIP_FILTERS+=("--skip" "$test_filter")
done

run() {
    printf '==> %s\n' "$*"
    "$@"
}

run_cargo() {
    if [[ -n "$NESTED_TARGET_DIR" ]]; then
        run env "CARGO_TARGET_DIR=$NESTED_TARGET_DIR" "$CARGO_BIN" "$@"
    else
        run "$CARGO_BIN" "$@"
    fi
}

run_websocket_filter() {
    local test_filter="$1"

    if [[ -n "${BACKEND_ROLE_COMPLETION_NESTED:-}" ]]; then
        run_cargo test --locked --test health "$test_filter"
    else
        run_cargo test --locked "$test_filter"
    fi
}

run_core_lane() {
    run_cargo fmt --check
    run_cargo check --locked
    if [[ -n "${BACKEND_ROLE_COMPLETION_NESTED:-}" ]]; then
        run_cargo test --locked --lib -- "${CORE_SKIP_FILTERS[@]}"
        run_cargo test --locked --test docs_snapshot_store_lists -- "${CORE_SKIP_FILTERS[@]}"
        run_cargo test --locked --test env_example -- "${CORE_SKIP_FILTERS[@]}"
        run_cargo test --locked --test health -- "${CORE_SKIP_FILTERS[@]}"
    else
        run_cargo test --locked -- "${CORE_SKIP_FILTERS[@]}"
    fi
}

run_websocket_lane() {
    run "$ROOT_DIR/scripts/preflight.sh" websocket

    for test_filter in "${WEBSOCKET_TEST_FILTERS[@]}"; do
        run_websocket_filter "$test_filter"
    done
}

run_all_lanes() {
    run_core_lane
    run_websocket_lane
}

cd "$ROOT_DIR"

case "$LANE" in
    core)
        run_core_lane
        ;;
    websocket)
        run_websocket_lane
        ;;
    all)
        run_all_lanes
        ;;
    *)
        printf 'usage: %s [core|websocket|all]\n' "${BASH_SOURCE[0]}" >&2
        exit 2
        ;;
esac
