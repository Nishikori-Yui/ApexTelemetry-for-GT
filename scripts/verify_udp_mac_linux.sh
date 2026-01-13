#!/usr/bin/env bash
set -euo pipefail

if [ "${1:-}" = "" ]; then
  echo "Usage: $0 <PS5_IP>"
  echo "Example: $0 192.168.50.105"
  exit 1
fi

PS5_IP="$1"
HB_SECONDS="${HB_SECONDS:-10}"
CAPTURE_EXTRA="${CAPTURE_EXTRA:-5}"
TCPDUMP_IFACE="${TCPDUMP_IFACE:-any}"

echo "Starting tcpdump on ${TCPDUMP_IFACE} (udp/33739 + udp/33740)..."
sudo tcpdump -n -i "$TCPDUMP_IFACE" "udp port 33739 or udp port 33740" -vv -l &
TCPDUMP_PID=$!

python3 - <<PY
import socket, time
ps5_ip = "$PS5_IP"
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
for _ in range(int("$HB_SECONDS")):
    sock.sendto(b"A", (ps5_ip, 33739))
    time.sleep(1)
print("heartbeat done")
PY

sleep "$CAPTURE_EXTRA"
sudo kill -2 "$TCPDUMP_PID"
echo "Done. If needed, adjust HB_SECONDS/CAPTURE_EXTRA/TCPDUMP_IFACE env vars."
