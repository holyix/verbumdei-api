#!/usr/bin/env bash
set -euo pipefail

MONGO_URI="${MONGO_URI:-mongodb://127.0.0.1:27017}"
MONGO_DB="${MONGO_DB:-verbumdei}"
COLLECTION="${COLLECTION:-eras}"
DATA_FILE="${DATA_FILE:-data/eras.json}"

if [[ ! -f "$DATA_FILE" ]]; then
  echo "Data file not found: $DATA_FILE" >&2
  exit 1
fi

if ! command -v mongoimport >/dev/null 2>&1; then
  echo "mongoimport not found. Please install MongoDB tools." >&2
  exit 1
fi

echo "Loading $DATA_FILE into ${MONGO_URI}/${MONGO_DB}.${COLLECTION}..."
mongoimport \
  --uri "$MONGO_URI" \
  --db "$MONGO_DB" \
  --collection "$COLLECTION" \
  --drop \
  --jsonArray \
  "$DATA_FILE"

echo "Done."
