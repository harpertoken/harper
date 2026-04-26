#!/bin/bash
# Harper TUI Intent Test via Server
# Tests natural language -> tool mapping via API

set -e

cd /Users/niladri/harper

echo "=== Harper Intent Recognition Test ==="
echo ""

# Kill existing server
pkill -f "harper --server" 2>/dev/null || true
sleep 1

# Start server
echo "Starting server..."
target/release/harper --server &
SERVER_PID=$!
sleep 4

echo "Testing intents via /api/chat endpoint..."
echo ""

# Test cases - each maps specific intent to tool
declare -a TEST_CASES=(
    "list files in current directory:ls"
    "run git status:git status"
    "check the README.md:read_file"
    "search for function main:grep"
    "create a new test file:write_file"
    "update this code:search_replace"
)

for test in "${TEST_CASES[@]}"; do
    INPUT="${test%%:*}"
    EXPECTED="${test##*:}"

    RESULT=$(curl -s -X POST http://127.0.0.1:8081/api/chat \
        -H "Content-Type: application/json" \
        -d "{\"message\": \"$INPUT\"}" 2>/dev/null)

    # Check if response contains expected tool or command
    if echo "$RESULT" | grep -qi "$EXPECTED"; then
        echo "✓ '$INPUT' -> $EXPECTED"
    else
        echo "✗ '$INPUT' -> (looking for: $EXPECTED)"
        echo "  Response: $(echo "$RESULT" | jq -r '.message' 2>/dev/null | head -c 100)"
    fi
done

echo ""
echo "=== Review Endpoint Test (fully working) ==="

# Review endpoint uses the LLM properly
RESPONSE=$(curl -s -X POST http://127.0.0.1:8081/api/review \
    -H "Content-Type: application/json" \
    -d '{
        "file_path": "test.rs",
        "content": "fn main() {\n    println!(\"Hello\");\n}",
        "language": "rust"
    }' 2>/dev/null)

SUMMARY=$(echo "$RESPONSE" | jq -r '.summary' 2>/dev/null)
if [ -n "$SUMMARY" ]; then
    echo "✓ Review endpoint works:"
    echo "  -> $(echo "$SUMMARY" | head -c 150)"
else
    echo "✗ Review endpoint failed"
fi

# Cleanup
kill $SERVER_PID 2>/dev/null || true
echo ""
echo "Done!"
