#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${1:-https://pathscale-be.sylvanbloch.workers.dev}"
BASE_URL="${BASE_URL%/}"

WS_URL="${BASE_URL/https:\/\//wss://}"
WS_URL="${WS_URL/http:\/\//ws://}"

PASS=0
FAIL=0

ok()   { echo "  PASS: $*"; PASS=$((PASS + 1)); }
fail() { echo "  FAIL: $*"; FAIL=$((FAIL + 1)); }

echo "Checking $BASE_URL"
echo

if ! command -v websocat &>/dev/null; then
    echo "ERROR: websocat not found (cargo install websocat)"
    exit 1
fi

# ---- Axum REST: GET / ----
echo "GET /"
BODY=$(curl -sf "$BASE_URL/") && true
if [ "${BODY:-}" = "Hello, World!" ]; then
    ok "response = '$BODY'"
else
    fail "expected 'Hello, World!', got '${BODY:-<empty>}'"
fi
echo

# ---- Axum REST: WS /ws echo ----
echo "WS $WS_URL/ws (axum echo)"
WS_BODY=$(echo "test" | timeout 5 websocat "$WS_URL/ws") && true
if [ "${WS_BODY:-}" = "hello: test" ]; then
    ok "sent 'test', got '$WS_BODY'"
else
    fail "expected 'hello: test', got '${WS_BODY:-<empty>}'"
fi
echo

# ---- AddLead (method 41001, public — no auth required) ----
echo "WS AddLead (method 41001)"
REQ='{"method":41001,"seq":1,"params":{"name":"check.sh","telegram":"@check_sh_test"}}'
RESP=$(echo "$REQ" | timeout 10 websocat --no-close -n1 "$WS_URL:8082" 2>/dev/null) && true

if echo "${RESP:-}" | grep -q '"type":"Immediate"'; then
    if echo "${RESP:-}" | grep -q '"method":41001'; then
        ok "got Immediate response for method 41001"
    else
        fail "got Immediate but wrong method — response: ${RESP:-<empty>}"
    fi
elif echo "${RESP:-}" | grep -q '"type":"Error"'; then
    fail "server returned error — response: ${RESP:-<empty>}"
else
    fail "unexpected response: ${RESP:-<empty>}"
fi
echo

# ---- Wrong method (expect Error, confirms server is processing requests) ----
echo "WS invalid method (expect Error response)"
REQ='{"method":99999,"seq":2,"params":{}}'
RESP=$(echo "$REQ" | timeout 10 websocat --no-close -n1 "$WS_URL:8082/signal" 2>/dev/null) && true

if echo "${RESP:-}" | grep -q '"type":"Error"'; then
    ok "got Error for unknown method (server is healthy)"
else
    fail "expected Error response, got: ${RESP:-<empty>}"
fi
echo

echo "$PASS passed, $FAIL failed"
[ "$FAIL" -eq 0 ]
