#!/bin/bash
# Harper Working Features Test

cd /Users/niladri/harper

echo "=== Harper Working Features Test ==="
echo ""

# Kill existing server
pkill -f "harper --server" 2>/dev/null || true
sleep 1

# Start server
target/release/harper --server &
SERVER_PID=$!
sleep 4

echo "=== WORKING: Health Check ==="
curl -s http://127.0.0.1:8081/health | jq .

echo ""
echo "=== WORKING: Code Review (uses Ollama) ==="
curl -s -X POST http://127.0.0.1:8081/api/review \
  -H "Content-Type: application/json" \
  -d '{
    "file_path": "test.rs",
    "content": "fn main() {\n    println!(\"test\");\n}",
    "language": "rust"
  }' | jq '{summary: .summary, findings_count: (.findings | length)}'

echo ""
echo "=== WORKING: Session List ==="
curl -s http://127.0.0.1:8081/api/sessions | jq '.'

echo ""
echo "=== NOT IMPLEMENTED: Chat (placeholder) ==="
curl -s -X POST http://127.0.0.1:8081/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "list files"}' | jq .

# Clean up
kill $SERVER_PID 2>/dev/null || true
echo ""
echo "=== Summary ==="
echo "✅ Working: /health, /api/review, /api/sessions"
echo "❌ Placeholder: /api/chat (needs implementation)"
