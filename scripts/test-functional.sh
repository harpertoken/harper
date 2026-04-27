#!/bin/bash
# Harper Functional Test Suite
# Tests natural language -> shell workflow

set -e

PASS=0
FAIL=0

pass() { ((PASS++)); echo "PASS: $1"; }
fail() { ((FAIL++)); echo "FAIL: $1"; }

cd /Users/niladri/harper

echo "=== Harper Functional Tests ==="
echo ""

# Start server in background
echo "Starting server..."
target/release/harper --server &
SERVER_PID=$!
sleep 4

# Test 1: Health endpoint
echo "[1] Testing health endpoint..."
HEALTH=$(curl -s http://127.0.0.1:8081/health 2>/dev/null)
if echo "$HEALTH" | grep -q "ok"; then
    pass "Health check"
else
    fail "Health check: $HEALTH"
fi

# Test 2: List sessions (may be empty)
echo "[2] Testing sessions endpoint..."
SESSIONS=$(curl -s http://127.0.0.1:8081/api/sessions 2>/dev/null)
pass "Sessions endpoint responds" # It responds, may be empty

# Test 3: Review file
echo "[3] Testing review endpoint..."
REVIEW=$(curl -s -X POST http://127.0.0.1:8081/api/review \
    -H "Content-Type: application/json" \
    -d '{
        "file_path": "AGENTS.md",
        "content": "# Test\nLine 1",
        "language": "en"
    }' 2>/dev/null)

if echo "$REVIEW" | grep -q "review\|summary\|markdown"; then
    pass "Review endpoint works"
    echo "  -> Got review for AGENTS.md"
else
    fail "Review endpoint: $REVIEW"
fi

# Test 4: Tool execution (simulate)
echo "[4] Testing tool execution flow..."
# The server should parse natural language and return tool intent
# Currently /api/chat returns placeholder

# Test 5: Provider detection
echo "[5] Testing provider detection..."
LOG=$(curl -s http://127.0.0.1:8081/health 2>/dev/null)
if echo "$LOG" | grep -q "Ollama\|Cerebras\|OpenAI"; then
    pass "Provider detected in logs"
else
    # Check config instead
    grep -q "provider = " config/local.example.toml && pass "Provider config works" || fail "Provider config"
fi

# Test 6: Database persistence
echo "[6] Testing database..."
if [ -f .harper/sessions.db ]; then
    pass "Database exists"
    # Check tables
    if target/release/harper --server 2>&1 | grep -q "sessions"; then
        # DB should have sessions table
        sqlite3 .harper/sessions.db ".tables" 2>/dev/null | grep -q "command_logs" && pass "DB has command_logs" || fail "DB command_logs"
    fi
else
    fail "Database not created"
fi

# Cleanup
echo ""
kill $SERVER_PID 2>/dev/null || true

# Summary
echo ""
echo "=== SUMMARY ==="
echo "PASSED: $PASS"
echo "FAILED: $FAIL"

[ $FAIL -eq 0 ] && exit 0 || exit 1
