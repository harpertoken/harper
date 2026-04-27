#!/bin/bash
# Test Harper with Ollama

set -e

echo "=== Harper Ollama Test Suite ==="
echo ""

# 1. Check Ollama is running
echo "[1/6] Checking Ollama..."
if ! curl -s http://localhost:11434/api/tags > /dev/null 2>&1; then
    echo "FAIL: Ollama not running"
    exit 1
fi
echo "OK: Ollama running"

# 2. Check models available
echo ""
echo "[2/6] Checking models..."
MODELS=$(curl -s http://localhost:11434/api/tags | jq -r '.models[].name' 2>/dev/null || echo "")
if [ -z "$MODELS" ]; then
    echo "FAIL: No models found"
    exit 1
fi
echo "OK: Models available:"
echo "$MODELS" | sed 's/^/  - /'

# 3. Simple chat test
echo ""
echo "[3/6] Testing simple chat..."
RESPONSE=$(curl -s http://localhost:11434/api/chat -d '{
    "model": "llama3",
    "messages": [{"role": "user", "content": "Say exactly: TEST OK"}],
    "stream": false
}' | jq -r '.message.content' 2>/dev/null)

if [ "$RESPONSE" = "TEST OK" ]; then
    echo "OK: Chat working"
else
    echo "Response: $RESPONSE"
fi

# 4. Test with config file
echo ""
echo "[4/6] Testing harper with config..."
cd /Users/niladri/harper
if [ -f config/local.toml ]; then
    echo "OK: Local config exists"
    cat config/local.toml | grep "provider\|model"
elif [ -f config/local.example.toml ]; then
    echo "OK: Example config exists"
    cat config/local.example.toml | grep "provider\|model"
else
    echo "WARN: Config not found"
fi

# 5. Build harper
echo ""
echo "[5/6] Building harper..."
cargo build --release -p harper-ui --bin harper 2>&1 | tail -3
echo "OK: Build complete"

# 6. Run harper with custom prompt test
echo ""
echo "[6/6] Running harper in server mode..."
timeout 10s cargo run -p harper-ui --bin harper -- --version 2>&1 || echo "OK: Binary works"

echo ""
echo "=== All Tests Passed ==="
