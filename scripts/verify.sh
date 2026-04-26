#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LANE="${1:-core}"

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
    run cargo fmt --check
    run cargo check --locked
    run cargo test --locked -- "${CORE_SKIP_FILTERS[@]}"
}

run_websocket_lane() {
    run "$ROOT_DIR/scripts/preflight.sh" websocket
    run cargo test --locked websocket_endpoint_
    run cargo test --locked delete_document_endpoint_rejects_documents_with_active_websocket_sessions
    run cargo test --locked delete_document_endpoint_allows_delete_after_websocket_session_closes
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
