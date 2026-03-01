#!/usr/bin/env bash
# dev/test_miabis_e2e.sh — End-to-end integration test for the miabis project.
#
# Spins up a local Blaze instance, loads a minimal MIABIS-on-FHIR 1.0.0
# test bundle, starts a mock Beam proxy, and runs focus against it.
# Verifies that the returned MeasureReport contains the expected counts.
#
# Prerequisites:
#   - Docker (with the compose plugin)
#   - Python 3
#   - focus binary: built via `cargo build` (or `cargo build --release`)
#
# Usage:
#   dev/test_miabis_e2e.sh [path/to/focus/binary]
#
# If no binary path is given the script looks for:
#   ./target/debug/focus  ./target/release/focus
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BLAZE_PORT=18080
BLAZE_URL="http://localhost:${BLAZE_PORT}/fhir"
PROXY_PORT=18082
FOCUS_APP_ID="focus.proxy1.broker.samply.de"
BUNDLE="${REPO_ROOT}/resources/test/miabis_e2e_bundle.json"

RESULT_FILE="$(mktemp /tmp/miabis_e2e_result_XXXXXX.json)"
PROXY_PY="$(mktemp /tmp/miabis_e2e_proxy_XXXXXX.py)"
PASS=0

cleanup() {
    kill "$FOCUS_PID" 2>/dev/null || true
    wait "$FOCUS_PID" 2>/dev/null || true
    kill "$PROXY_PID" 2>/dev/null || true
    wait "$PROXY_PID" 2>/dev/null || true
    docker rm -f miabis-e2e-blaze 2>/dev/null || true
    rm -f "$RESULT_FILE" "$PROXY_PY"
}
trap cleanup EXIT

# ── 1. Locate focus binary ────────────────────────────────────────────────────
if [[ -n "${1:-}" ]]; then
    FOCUS_BIN="$1"
elif [[ -x "${REPO_ROOT}/target/debug/focus" ]]; then
    FOCUS_BIN="${REPO_ROOT}/target/debug/focus"
elif [[ -x "${REPO_ROOT}/target/release/focus" ]]; then
    FOCUS_BIN="${REPO_ROOT}/target/release/focus"
else
    echo "ERROR: focus binary not found. Run 'cargo build' first, or pass the path as an argument." >&2
    exit 1
fi

echo "==> Using focus binary: ${FOCUS_BIN}"

# ── 2. Check prerequisites ────────────────────────────────────────────────────
for cmd in docker python3 curl; do
    command -v "$cmd" >/dev/null 2>&1 || { echo "ERROR: $cmd not found in PATH" >&2; exit 1; }
done

# ── 3. Start Blaze ────────────────────────────────────────────────────────────
echo "==> Starting Blaze on port ${BLAZE_PORT}..."
docker run -d --rm \
    --name miabis-e2e-blaze \
    -p "${BLAZE_PORT}:8080" \
    -e JAVA_TOOL_OPTIONS="-Xmx512m" \
    samply/blaze:latest >/dev/null

echo -n "==> Waiting for Blaze..."
for i in $(seq 1 60); do
    curl -sf "${BLAZE_URL}/metadata" >/dev/null 2>&1 && break
    printf '.'; sleep 2
done
echo " ready."

# ── 4. Load test bundle ───────────────────────────────────────────────────────
echo "==> Loading MIABIS test bundle..."
LOAD_RESULT=$(curl -sf -X POST "${BLAZE_URL}" \
    -H "Content-Type: application/fhir+json" \
    -d "@${BUNDLE}")
ENTRY_COUNT=$(echo "$LOAD_RESULT" | python3 -c "
import sys, json
r = json.load(sys.stdin)
print(len(r.get('entry', [])))
")
echo "    Loaded ${ENTRY_COUNT} resources (2 Patient, 2 Condition, 3 Specimen expected)"

# ── 5. Build AST task payload (empty AND = all patients) ──────────────────────
AST_JSON='{"ast":{"operand":"AND","children":[]},"id":"a6f1ccf3-ebf1-424f-9d69-4e5d135f2340"}'
AST_B64=$(echo -n "$AST_JSON" | base64 -w0)
BODY_JSON="{\"lang\":\"ast\",\"payload\":\"${AST_B64}\"}"
BODY_B64=$(echo -n "$BODY_JSON" | base64 -w0)
TASK_ID="$(python3 -c 'import uuid; print(uuid.uuid4())')"

# ── 6. Write and start mock Beam proxy ────────────────────────────────────────
cat > "$PROXY_PY" <<PYEOF
import base64, json, os, threading
from http.server import BaseHTTPRequestHandler, HTTPServer

TASK_ID   = "${TASK_ID}"
FOCUS_APP = "${FOCUS_APP_ID}"
BODY_B64  = "${BODY_B64}"
RESULT_FILE = "${RESULT_FILE}"
PORT      = ${PROXY_PORT}

TASK = {
    "id": TASK_ID, "from": "mock-broker.local.test", "to": [FOCUS_APP],
    "ttl": "3600s",
    "failure_strategy": {"retry": {"backoff_millisecs": 1000, "max_tries": 3}},
    "metadata": {"project": "miabis"},
    "body": BODY_B64,
}
task_served = False
result_done = threading.Event()

class Handler(BaseHTTPRequestHandler):
    def log_message(self, fmt, *args): pass
    def send_json(self, code, data):
        body = json.dumps(data).encode()
        self.send_response(code)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)
    def do_GET(self):
        global task_served
        if self.path.startswith('/v1/tasks'):
            if not task_served:
                task_served = True
                self.send_json(200, [TASK])
            else:
                self.send_json(200, [])
        elif self.path == '/v1/health':
            self.send_json(200, {'status': 'ok'})
        else:
            self.send_json(404, {})
    def do_PUT(self):
        parts = self.path.strip('/').split('/')
        if len(parts) == 5 and parts[0]=='v1' and parts[1]=='tasks' and parts[3]=='results':
            self._handle_result()
        else:
            self.send_json(404, {})
    def do_POST(self): self.do_PUT()
    def _handle_result(self):
        length = int(self.headers.get('Content-Length', 0))
        raw = self.rfile.read(length)
        try:
            env = json.loads(raw)
            if env.get('status') == 'claimed':
                self.send_json(201, {})
                return
            decoded = base64.b64decode(env.get('body', '')).decode()
            with open(RESULT_FILE, 'w') as f:
                f.write(decoded)
        except Exception:
            pass
        self.send_json(201, {})
        result_done.set()

server = HTTPServer(('0.0.0.0', PORT), Handler)
t = threading.Thread(target=server.serve_forever, daemon=True)
t.start()
result_done.wait(timeout=120)
threading.Event().wait(timeout=2)
server.shutdown()
PYEOF

echo "==> Starting mock Beam proxy on port ${PROXY_PORT}..."
python3 "$PROXY_PY" &
PROXY_PID=$!
sleep 1

# ── 7. Run focus ──────────────────────────────────────────────────────────────
echo "==> Running focus..."
BEAM_PROXY_URL="http://localhost:${PROXY_PORT}" \
BEAM_APP_ID_LONG="${FOCUS_APP_ID}" \
API_KEY="test" \
ENDPOINT_URL="${BLAZE_URL}/" \
OBFUSCATE="no" \
ENDPOINT_TYPE="blaze" \
RUST_LOG="warn" \
"${FOCUS_BIN}" &
FOCUS_PID=$!

wait "$PROXY_PID" || true
kill "$FOCUS_PID" 2>/dev/null || true
wait "$FOCUS_PID" 2>/dev/null || true

# ── 8. Verify MeasureReport ───────────────────────────────────────────────────
echo "==> Verifying MeasureReport..."
python3 - <<PYEOF
import json, sys

with open("${RESULT_FILE}") as f:
    report = json.load(f)

if report.get('resourceType') == 'OperationOutcome':
    for issue in report.get('issue', []):
        print(f"  ERROR: {issue.get('diagnostics', '?')}")
    sys.exit(1)

failures = []

def pop(group):
    return (group.get('population') or [{}])[0].get('count', -1)

def stratum_map(group, stratifier_text):
    for strat in group.get('stratifier', []):
        codes = strat.get('code') or []
        if any(c.get('text') == stratifier_text for c in codes):
            return {s['value']['text']: (s.get('population') or [{}])[0].get('count', 0)
                    for s in strat.get('stratum', []) if 'value' in s}
    return {}

groups = {g['code']['text']: g for g in report.get('group', [])}

# Expected counts derived from resources/test/miabis_e2e_bundle.json
checks = [
    # (description, actual, expected)
    ("patient population",   pop(groups['patient']),   2),
    ("diagnosis population", pop(groups['diagnosis']), 2),
    ("specimen population",  pop(groups['specimen']),  3),
]

gender = stratum_map(groups['patient'], 'gender')
checks += [
    ("gender female", gender.get('female', 0), 1),
    ("gender male",   gender.get('male', 0),   1),
]

diag = stratum_map(groups['diagnosis'], 'diagnosis')
checks += [
    ("diagnosis C34", diag.get('C34', 0), 1),
    ("diagnosis C18", diag.get('C18', 0), 1),
]

sample = stratum_map(groups['specimen'], 'sample_kind')
checks += [
    ("sample_kind blood-plasma",  sample.get('blood-plasma', 0),  1),
    ("sample_kind tissue-ffpe",   sample.get('tissue-ffpe', 0),   1),
    ("sample_kind whole-blood",   sample.get('whole-blood', 0),   1),
]

temp = stratum_map(groups['specimen'], 'storage_temperature')
checks += [
    ("storage_temperature LN", temp.get('LN', 0), 2),
    ("storage_temperature RT", temp.get('RT', 0), 1),
]

cust = stratum_map(groups['patient'], 'Custodian')
checks += [
    ("Custodian collection-test-A", cust.get('collection-test-A', 0), 1),
    ("Custodian collection-test-B", cust.get('collection-test-B', 0), 1),
]

for desc, actual, expected in checks:
    status = "PASS" if actual == expected else "FAIL"
    print(f"  [{status}] {desc}: {actual} (expected {expected})")
    if actual != expected:
        failures.append(desc)

if failures:
    print(f"\nFAILED: {len(failures)} check(s) did not pass.")
    sys.exit(1)
else:
    print(f"\nAll {len(checks)} checks passed.")
PYEOF

echo "==> E2E test complete."
