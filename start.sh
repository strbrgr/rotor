#!/usr/bin/env bash
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN="$REPO/server/target/debug"
PIDS=()

cleanup() {
  echo ""
  echo "[rotor] Shutting down..."
  for pid in "${PIDS[@]:-}"; do
    kill "$pid" 2>/dev/null || true
  done
  wait 2>/dev/null || true
  echo "[rotor] Done."
}
trap cleanup EXIT INT TERM

wait_for_port() {
  local host=$1 port=$2 label=$3
  printf "[rotor] Waiting for %s" "$label"
  until nc -z "$host" "$port" 2>/dev/null; do
    printf "."
    sleep 0.5
  done
  echo " ready"
}

if [[ ! -f "$REPO/server/.env" ]]; then
  echo "[rotor] ERROR: .env not found at $REPO/server/.env — see README for required variables."
  exit 1
fi

# Export env vars so all child processes inherit them (dotenvy reads from CWD,
# but binaries run from repo root where there is no .env)
set -a
# shellcheck source=/dev/null
source "$REPO/server/.env"
set +a

# Docker services
echo "[rotor] Starting Iggy + QuestDB..."
docker compose -f "$REPO/server/docker-compose.yml" --env-file "$REPO/server/.env" up -d

wait_for_port 127.0.0.1 8090 "Iggy"
wait_for_port 127.0.0.1 9000 "QuestDB"

# Build all server binaries once
echo "[rotor] Building server binaries..."
cargo build --manifest-path "$REPO/server/Cargo.toml" --bins 2>&1

# Gateway (sensors connect to :8080)
echo "[rotor] Starting gateway..."
"$BIN/gateway" &
PIDS+=($!)
wait_for_port 127.0.0.1 8080 "gateway"

# Writer and SSE can start in parallel (both connect to Iggy :8090)
echo "[rotor] Starting writer..."
"$BIN/writer" &
PIDS+=($!)

echo "[rotor] Starting SSE server..."
"$BIN/sse" &
PIDS+=($!)
wait_for_port 127.0.0.1 3001 "SSE"

# Sensors (connect to gateway :8080)
echo "[rotor] Starting sensors..."
"$BIN/sensor" 100 &
PIDS+=($!)
"$BIN/sensor" 150 &
PIDS+=($!)

# UI
echo "[rotor] Starting UI..."
cd "$REPO/ui" && npm install --silent && npm run dev &
PIDS+=($!)

echo "[rotor] All services running. Ctrl+C to stop."
wait
