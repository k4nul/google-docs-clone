#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LANE="${1:-core}"
CARGO_BIN="${CARGO:-cargo}"

CORE_SKIP_FILTERS=(
    "--skip" "backend_role_completion_gate"
    "--skip" "delete_document_endpoint_rejects_documents_with_active_websocket_sessions"
    "--skip" "delete_document_endpoint_allows_delete_after_websocket_session_closes"
    "--skip" "websocket_endpoint_"
)

run() {
    printf '==> %s\n' "$*"
    "$@"
}

run_core_lane() {
    run "$CARGO_BIN" fmt --check
    run "$CARGO_BIN" check --locked
    if [[ -n "${BACKEND_ROLE_COMPLETION_NESTED:-}" ]]; then
        run "$CARGO_BIN" test --locked --lib -- "${CORE_SKIP_FILTERS[@]}"
        run "$CARGO_BIN" test --locked --test docs_snapshot_store_lists -- "${CORE_SKIP_FILTERS[@]}"
        run "$CARGO_BIN" test --locked --test env_example -- "${CORE_SKIP_FILTERS[@]}"
        run "$CARGO_BIN" test --locked --test health -- "${CORE_SKIP_FILTERS[@]}"
    else
        run "$CARGO_BIN" test --locked -- "${CORE_SKIP_FILTERS[@]}"
    fi
}

run_websocket_lane() {
    run "$ROOT_DIR/scripts/preflight.sh" websocket
    if [[ -n "${BACKEND_ROLE_COMPLETION_NESTED:-}" ]]; then
        run "$CARGO_BIN" test --locked --test health websocket_endpoint_
        run "$CARGO_BIN" test --locked --test health delete_document_endpoint_rejects_documents_with_active_websocket_sessions
        run "$CARGO_BIN" test --locked --test health delete_document_endpoint_allows_delete_after_websocket_session_closes
    else
        run "$CARGO_BIN" test --locked websocket_endpoint_
        run "$CARGO_BIN" test --locked delete_document_endpoint_rejects_documents_with_active_websocket_sessions
        run "$CARGO_BIN" test --locked delete_document_endpoint_allows_delete_after_websocket_session_closes
    fi
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
