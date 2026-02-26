#!/usr/bin/env bash
set -euo pipefail

API_URL="${API_URL:-http://localhost:3001}"

echo "=== NEXUS Seed Data ==="
echo ""

# Register a test user.
echo "Creating test user..."
RESPONSE=$(curl -sf -X POST "${API_URL}/api/v1/auth/register" \
    -H "Content-Type: application/json" \
    -d '{"username": "testuser", "email": "test@nexus.dev", "password": "testpass123"}' 2>&1) || {
    echo "User may already exist, trying login..."
    RESPONSE=$(curl -sf -X POST "${API_URL}/api/v1/auth/login" \
        -H "Content-Type: application/json" \
        -d '{"email": "test@nexus.dev", "password": "testpass123"}')
}

TOKEN=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['token'])")
USER_ID=$(echo "$RESPONSE" | python3 -c "import sys,json; print(json.load(sys.stdin)['user_id'])")

echo "User ID: $USER_ID"
echo "Token: ${TOKEN:0:20}..."
echo ""

# Send a test message in integrated mode.
echo "Sending test message (integrated mode)..."
curl -sf -X POST "${API_URL}/api/v1/chat" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer ${TOKEN}" \
    -d '{"message": "The media is always biased against ordinary people.", "mode": "integrated"}' | python3 -m json.tool

echo ""

# Send a test analysis.
echo "Running Perspective analysis..."
curl -sf -X POST "${API_URL}/api/v1/analyze" \
    -H "Content-Type: application/json" \
    -H "Authorization: Bearer ${TOKEN}" \
    -d '{"text": "Economic growth is the only path to prosperity. The free market, left to its own devices, naturally produces the best outcomes for everyone."}' | python3 -m json.tool

echo ""
echo "=== Seed data complete ==="
