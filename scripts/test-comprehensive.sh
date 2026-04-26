#!/bin/bash
# Harper Comprehensive Test Suite

set -e

PASS=0
FAIL=0

pass() { ((PASS++)); echo "PASS: $1"; }
fail() { ((FAIL++)); echo "FAIL: $1"; }

echo "=== Harper Comprehensive Test Suite ==="
echo ""

cd /Users/niladri/harper

# === Basic Tests ===

echo "=== BASIC TESTS ==="

[ -f target/release/harper ] && pass "Binary exists" || fail "Binary exists"
target/release/harper --version >/dev/null 2>&1 && pass "Binary runs" || fail "Binary runs"

# === Config Tests ===

echo ""
echo "=== CONFIG TESTS ==="

grep -q 'provider = "Cerebras"' config/local.toml && pass "Config: Cerebras provider" || fail "Config: Cerebras"
grep -q 'qwen-3-32b-a4b' config/local.toml && pass "Config: Cerebras model" || fail "Config: Cerebras model"
[ -f config/default.toml ] && pass "Config: default.toml exists" || fail "Config: default.toml"
[ -f config/env.example ] && pass "Config: env.example exists" || fail "Config: env.example"

# === Environment Tests ===

echo ""
echo "=== ENVIRONMENT TESTS ==="

[ -f .env.example ] && pass "Env: .env.example exists" || fail "Env: .env.example"
grep -q CEREBRAS_API_KEY .env.example && pass "Env: Cerebras key" || fail "Env: Cerebras key"
grep -q OLLAMA .env.example && pass "Env: Ollama var" || fail "Env: Ollama var"

# === Code Tests ===

echo ""
echo "=== CODE TESTS ==="

cargo check -p harper-core >/dev/null 2>&1 && pass "Code: harper-core compiles" || fail "Code: harper-core"
cargo check -p harper-ui >/dev/null 2>&1 && pass "Code: harper-ui compiles" || fail "Code: harper-ui"
cargo check -p harper-mcp-server >/dev/null 2>&1 && pass "Code: harper-mcpServer" || fail "Code: harper-mcp-server"

# === Ollama Tests ===

echo ""
echo "=== OLLAMA TESTS ==="

curl -s http://localhost:11434/api/tags >/dev/null 2>&1 && pass "Ollama: running" || fail "Ollama: running"
curl -s http://localhost:11434/api/tags | grep -q llama3 && pass "Ollama: llama3 model" || fail "Ollama: llama3"

# Test chat endpoint
CHAT_RESP=$(curl -s http://localhost:11434/api/chat -d '{
    "model": "llama3",
    "messages": [{"role": "user", "content": "Hi"}],
    "stream": false
}' | jq -r '.message.content' 2>/dev/null)

[ -n "$CHAT_RESP" ] && pass "Ollama: chat endpoint" || fail "Ollama: chat endpoint"

# === Agent Intent Tests ===

echo ""
echo "=== AGENT INTENT TESTS ==="

grep -q "read a file -> use read_file" lib/harper-core/src/agent/prompt.rs && pass "Intent: read_file mapping" || fail "Intent: read_file"
grep -q "run a command -> use run_command" lib/harper-core/src/agent/prompt.rs && pass "Intent: run_command mapping" || fail "Intent: run_command"
grep -q "write_file" lib/harper-core/src/agent/prompt.rs && pass "Intent: write_file tool" || fail "Intent: write_file"
grep -q "search_replace" lib/harper-core/src/agent/prompt.rs && pass "Intent: search_replace tool" || fail "Intent: search_replace"
grep -q "git_diff\|git_add\|git_commit" lib/harper-core/src/agent/prompt.rs && pass "Intent: git tools" || fail "Intent: git tools"

# === Approval Tests ===

echo ""
echo "=== APPROVAL TESTS ==="

grep -q "approver" lib/harper-core/src/tools/shell.rs && pass "Approval: shell has approver" || fail "Approval: shell"
grep -q "approve(" lib/harper-core/src/tools/shell.rs && pass "Approval: approve called" || fail "Approval: approve"
grep -q "Execute command?" lib/harper-core/src/tools/shell.rs && pass "Approval: prompt shown" || fail "Approval: prompt"

# === Provider Tests ===

echo ""
echo "=== PROVIDER TESTS ==="

grep -q 'CEREBRAS' lib/harper-core/src/core/models.rs && pass "Provider: Cerebras model" || fail "Provider: Cerebras"
grep 'CEREBRAS' lib/harper-core/src/core/models.rs | grep -q 'const' && pass "Provider: Cerebras defined" || fail "Provider: Cerebras define"
grep -q 'cerebras' lib/harper-ui/src/auth.rs && pass "Provider: Cerebras auth" || fail "Provider: Cerebras auth"
grep -q 'CEREBRAS_API_KEY' lib/harper-core/src/runtime/config.rs && pass "Provider: Cerebras env" || fail "Provider: Cerebras env"

# === Documentation Tests ===

echo ""
echo "=== DOC TESTS ==="

[ -f docs/PRIVACY.md ] && pass "Docs: PRIVACY.md" || fail "Docs: PRIVACY.md"
grep -q Cerebras docs/PRIVACY.md && pass "Docs: Cerebras in privacy" || fail "Docs: Cerebras privacy"
grep -q cerebras docs/user-guide/keychain.md && pass "Docs: Cerebras in keychain" || fail "Docs: Cerebras keychain"
[ -f GEMINI.md ] && pass "Docs: GEMINI.md exists" || fail "Docs: GEMINI.md"

# === AGENTS.md Tests ===

echo ""
echo "=== AGENTS.MD TESTS ==="

[ -f AGENTS.md ] && pass "AGENTS: file exists" || fail "AGENTS: file exists"
grep -q "User Intent Recognition" AGENTS.md && pass "AGENTS: intent section" || fail "AGENTS: intent"
grep -q "read_file" AGENTS.md && pass "AGENTS: read_file tool" || fail "AGENTS: read_file"
grep -q "search_replace" AGENTS.md && pass "AGENTS: search_replace" || fail "AGENTS: search_replace"
grep -q "run_command" AGENTS.md && pass "AGENTS: run_command" || fail "AGENTS: run_command"

# === Website Tests ===

echo ""
echo "=== WEBSITE TESTS ==="

[ -f website/index.html ] && pass "Website: index.html" || fail "Website: index"
grep -q Cerebras website/index.html && pass "Website: Cerebras listed" || fail "Website: Cerebras"
[ -f website/server.html ] && pass "Website: server.html" || fail "Website: server"

# === Summary ===

echo ""
echo "=== SUMMARY ==="
echo "PASSED: $PASS"
echo "FAILED: $FAIL"
echo ""

if [ $FAIL -eq 0 ]; then
    echo "All tests passed!"
    exit 0
else
    echo "Some tests failed"
    exit 1
fi
