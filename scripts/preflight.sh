#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MODE="${1:-all}"
STATUS=0

pass() {
    printf '[pass] %s\n' "$1"
}

fail() {
    printf '[fail] %s\n' "$1" >&2
    return 1
}

check_git_write() {
    local probe_path

    probe_path="$ROOT_DIR/.git/codex-preflight-$$.lock"
    if : >"$probe_path" 2>/dev/null && rm -f "$probe_path"; then
        pass "git metadata directory is writable"
        return 0
    fi

    fail "cannot create lock files under .git; staging and commit would be blocked"
}

check_github_dns() {
    if command -v getent >/dev/null 2>&1; then
        if getent ahostsv4 github.com >/dev/null 2>&1 || getent hosts github.com >/dev/null 2>&1; then
            pass "github.com DNS lookup succeeded"
            return 0
        fi
    elif command -v nslookup >/dev/null 2>&1; then
        if nslookup github.com >/dev/null 2>&1; then
            pass "github.com DNS lookup succeeded"
            return 0
        fi
    elif git ls-remote --heads origin HEAD >/dev/null 2>&1; then
        pass "origin remote is reachable"
        return 0
    fi

    fail "github.com DNS lookup failed; push and remote verification are likely blocked"
}

check_socket_bind() {
    local probe_log

    probe_log="$(mktemp)"
    if cargo test --locked websocket_endpoint_accepts_document_connections -- --exact \
        >"$probe_log" 2>&1; then
        rm -f "$probe_log"
        pass "socket-backed websocket probe passed"
        return 0
    fi

    if grep -q "Cannot create socket address for use" "$probe_log"; then
        rm -f "$probe_log"
        fail "runner cannot bind socket addresses; websocket verification lane is blocked"
        return 1
    fi

    cat "$probe_log" >&2
    rm -f "$probe_log"
    fail "websocket probe failed for a non-environment reason"
}

run_mode() {
    case "$MODE" in
        commit)
            check_git_write || STATUS=1
            ;;
        publish)
            check_git_write || STATUS=1
            check_github_dns || STATUS=1
            ;;
        websocket)
            check_socket_bind || STATUS=1
            ;;
        all)
            check_git_write || STATUS=1
            check_github_dns || STATUS=1
            check_socket_bind || STATUS=1
            ;;
        *)
            printf 'usage: %s [commit|publish|websocket|all]\n' "${BASH_SOURCE[0]}" >&2
            return 2
            ;;
    esac
}

cd "$ROOT_DIR"
run_mode
exit "$STATUS"
