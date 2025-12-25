#!/usr/bin/env bash
set -euo pipefail

API_HOST="${API_HOST:-127.0.0.1}"
API_PORT="${API_PORT:-8080}"
API_BASE="http://${API_HOST}:${API_PORT}"

MONGO_HOST="${MONGO_HOST:-127.0.0.1}"
MONGO_PORT="${MONGO_PORT:-27017}"

# Ensure Mongo is reachable before starting the API.
if ! nc -z "$MONGO_HOST" "$MONGO_PORT" 2>/dev/null; then
  echo "MongoDB is not reachable at ${MONGO_HOST}:${MONGO_PORT}. Start Mongo and retry." >&2
  exit 1
fi

echo "Starting API server in background..."
cargo run > /tmp/verbumdei-api.log 2>&1 &
SERVER_PID=$!
trap 'kill $SERVER_PID >/dev/null 2>&1 || true' EXIT

echo "Waiting for server to be ready at ${API_BASE}/health..."
for i in {1..30}; do
  if curl -fsS "${API_BASE}/health" >/dev/null 2>&1; then
    echo "Server is up."
    break
  fi
  sleep 1
  if ! kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    echo "Server process exited prematurely. Check /tmp/verbumdei-api.log" >&2
    exit 1
  fi
  if [[ $i -eq 30 ]]; then
    echo "Timed out waiting for server to start. Check /tmp/verbumdei-api.log" >&2
    exit 1
  fi
done

./scripts/api_workflow.sh
