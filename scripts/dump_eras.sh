#!/usr/bin/env bash
set -euo pipefail

MONGO_URI="${MONGO_URI:-mongodb://127.0.0.1:27017}"
MONGO_DB="${MONGO_DB:-verbumdei}"
COLLECTION="${COLLECTION:-eras}"
OUT_FILE="${OUT_FILE:-data/eras.json}"

if ! command -v mongoexport >/dev/null 2>&1; then
  echo "mongoexport not found. Please install MongoDB tools." >&2
  exit 1
fi

echo "Exporting ${MONGO_URI}/${MONGO_DB}.${COLLECTION} to ${OUT_FILE}..."
mongoexport \
  --uri "$MONGO_URI" \
  --db "$MONGO_DB" \
  --collection "$COLLECTION" \
  --jsonArray \
  --out "$OUT_FILE"

echo "Done."
