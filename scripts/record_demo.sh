#!/usr/bin/env bash
set -euo pipefail

BASE_URL="${APEXTELEMETRY_BASE_URL:-http://127.0.0.1:10086}"
INTERVAL_SEC="${APEXTELEMETRY_RECORD_INTERVAL:-1}"

cleanup() {
  curl -s -X POST "${BASE_URL}/demo/record/stop" >/dev/null 2>&1 || true
}

trap cleanup EXIT

echo "start: ${BASE_URL}/demo/record/start"
curl -fsS -X POST "${BASE_URL}/demo/record/start" >/dev/null

recording_seen=0
while true; do
  status_json="$(curl -fsS "${BASE_URL}/demo/record/status" || true)"
  if [[ -z "${status_json}" ]]; then
    echo "status: unavailable"
    sleep "${INTERVAL_SEC}"
    continue
  fi

  read -r mode active armed frames path < <(python3 - "${status_json}" <<'PY'
import json
import sys

payload = json.loads(sys.argv[1])
mode = payload.get("mode", "unknown")
active = str(payload.get("active", False)).lower()
armed = str(payload.get("armed", False)).lower()
frames = payload.get("frames", 0)
path = payload.get("path") or "-"
print(mode, active, armed, frames, path)
PY
)

  now="$(date +%H:%M:%S)"
  printf "[%s] mode=%s active=%s armed=%s frames=%s path=%s\n" \
    "${now}" "${mode}" "${active}" "${armed}" "${frames}" "${path}"

  if [[ "${mode}" == "recording" ]]; then
    recording_seen=1
  fi
  if [[ "${recording_seen}" == "1" && "${mode}" != "recording" ]]; then
    break
  fi
  sleep "${INTERVAL_SEC}"
done
