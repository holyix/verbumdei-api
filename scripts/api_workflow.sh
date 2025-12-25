#!/usr/bin/env bash
set -euo pipefail

API_BASE="http://127.0.0.1:8080/v1"
MONGO_HOST="127.0.0.1"
MONGO_PORT=27017

# Check Mongo is reachable before running API calls.
if ! nc -z "$MONGO_HOST" "$MONGO_PORT" 2>/dev/null; then
  echo "MongoDB is not reachable at ${MONGO_HOST}:${MONGO_PORT}. Start Mongo and retry." >&2
  exit 1
fi

echo "MongoDB reachable at ${MONGO_HOST}:${MONGO_PORT}"

# Create a question
echo "Creating question..."
CREATE_RES="$(curl -sS -X POST "$API_BASE/questions" \
  -H 'Content-Type: application/json' \
  -d '{
    "stage": 1,
    "prompt": "Sample prompt?",
    "options": [
      { "text": "Yes", "correct": true, "explanation": null },
      { "text": "No", "correct": false, "explanation": "Because ..." },
      { "text": "Maybe", "correct": false, "explanation": null },
      { "text": "Later", "correct": false, "explanation": null }
    ],
    "tags": ["demo"]
  }')"
echo "Create response: $CREATE_RES"

QUESTION_ID="$(echo "$CREATE_RES" | jq -r '.id')"
if [[ -z "$QUESTION_ID" || "$QUESTION_ID" == "null" ]]; then
  echo "Failed to extract question id from create response" >&2
  exit 1
fi
echo "Created question id: $QUESTION_ID"

# Get by id
echo "Fetching question..."
curl -sS "$API_BASE/questions/$QUESTION_ID" | jq .

# List with pagination
echo "Listing questions..."
curl -sS "$API_BASE/questions?limit=5&offset=0" | jq .

# Delete
echo "Deleting question..."
DEL_STATUS="$(curl -s -o /dev/null -w '%{http_code}' -X DELETE "$API_BASE/questions/$QUESTION_ID")"
echo "Delete status: $DEL_STATUS"
if [[ "$DEL_STATUS" != "204" ]]; then
  echo "Delete failed with status $DEL_STATUS" >&2
  exit 1
fi

# Confirm gone
echo "Confirming deletion (should be 404)..."
curl -s -o /dev/null -w '%{http_code}\n' "$API_BASE/questions/$QUESTION_ID"
