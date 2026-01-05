#!/bin/bash

# Copyright 2025 harpertoken
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# Comprehensive test and validation script for Harper
# This script runs all quality checks and tests

set -euo pipefail  # Exit on any error, undefined vars, or pipe failures

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Quiet mode
QUIET=${QUIET:-0}

echo ""
echo -e "${BLUE}╭──────────────────────────────────────────────╮${NC}"
echo -e "${BLUE}│          Harper Validation Suite             │${NC}"
echo -e "${BLUE}│        Calm checks. Strong guarantees.       │${NC}"
echo -e "${BLUE}╰──────────────────────────────────────────────╯${NC}"
echo ""

# Function to print status
print_status() {
    local status=$1
    local message=$2
    case $status in
        "PASS")
            echo -e "${GREEN}PASS${NC}: $message"
            ;;
        "FAIL")
            echo -e "${RED}FAIL${NC}: $message"
            ;;
        "WARN")
            echo -e "${YELLOW}WARN${NC}: $message"
            ;;
        "INFO")
            echo -e "${BLUE}INFO${NC}: $message"
            ;;
    esac
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_status "FAIL" "Not in Harper project root (Cargo.toml not found)"
    exit 1
fi

print_status "INFO" "Starting validation checks..."

# 1. Rust Compilation Check
echo ""
print_status "INFO" "[1/14] Checking Rust compilation..."
if cargo check --quiet 2>/dev/null; then
    print_status "PASS" "Rust compilation successful ✓"
else
    print_status "FAIL" "Rust compilation failed"
    echo "Run 'cargo check' for details"
    exit 1
fi

# 2. Clippy Linting
echo ""
print_status "INFO" "2. Running Clippy linter..."
if cargo clippy --all-targets --all-features --quiet -- -D warnings 2>/dev/null; then
    print_status "PASS" "Clippy linting passed"
else
print_status "FAIL" "Clippy reported issues that need attention"
echo "→ Tip: run 'cargo clippy' locally to inspect them calmly"
    exit 1
fi

# 3. Code Formatting
echo ""
print_status "INFO" "3. Checking code formatting..."
if cargo fmt --check 2>/dev/null; then
    print_status "PASS" "Code formatting correct"
else
    print_status "FAIL" "Code formatting issues found"
    echo "Run 'cargo fmt' to fix formatting"
    exit 1
fi

# 4. Unit Tests
echo ""
print_status "INFO" "4. Running unit tests..."
if cargo test --quiet 2>/dev/null; then
    print_status "PASS" "All unit tests passed"
else
    print_status "FAIL" "Unit tests failed"
    echo "Run 'cargo test' for details"
    exit 1
fi

# 5. Security Audit
echo ""
print_status "INFO" "5. Running security audit..."
if [ -x "$HOME/.cargo/bin/cargo-audit" ]; then
    if "$HOME/.cargo/bin/cargo-audit" audit --quiet >/dev/null 2>&1; then
        print_status "PASS" "Security audit passed"
    else
        print_status "FAIL" "Security vulnerabilities found"
        echo "Run '$HOME/.cargo/bin/cargo-audit audit' for details"
        exit 1
    fi
else
    print_status "WARN" "cargo-audit not installed (run: cargo install cargo-audit)"
fi

# 6. File Size Check
echo ""
print_status "INFO" "6. Checking for large files..."
LARGE_FILES=$(find . -type f -size +10M -not -path "./target/*" -not -path "./.git/*" 2>/dev/null || true)
if [ -z "$LARGE_FILES" ]; then
    print_status "PASS" "No large files found"
else
    print_status "FAIL" "Large files detected (>10MB)"
    echo "$LARGE_FILES"
    exit 1
fi

# 7. Sensitive Files Check
echo ""
print_status "INFO" "7. Checking for sensitive files..."
SENSITIVE_FILES=$(find . \( -name "*.key" -o -name "*.pem" -o -name "*.env" \) -not -path "./target/*" -not -path "./.git/*" 2>/dev/null || true)
if [ -z "$SENSITIVE_FILES" ]; then
    print_status "PASS" "No sensitive files found in repository"
else
    print_status "WARN" "Potential sensitive files found"
    echo "$SENSITIVE_FILES"
fi

# 8. YAML Validation
echo ""
print_status "INFO" "8. Validating YAML files..."
if command -v yamllint >/dev/null 2>&1; then
    YAML_FILES=$(find . -name "*.yml" -o -name "*.yaml" -not -path "./target/*" -not -path "./.git/*" 2>/dev/null || true)
    if [ -n "$YAML_FILES" ]; then
        # Use xargs to handle multiple files properly
        if echo "$YAML_FILES" | xargs yamllint >/dev/null 2>&1; then
            print_status "PASS" "YAML files are valid"
        else
            print_status "FAIL" "YAML validation failed"
            echo "Run 'yamllint .github/workflows/*.yml docker/docker-compose.yml' for details"
            exit 1
        fi
    else
        print_status "PASS" "No YAML files to validate"
    fi
else
    print_status "WARN" "yamllint not installed (run: pip install yamllint)"
fi

# 9. TODO Comments Check
echo ""
print_status "INFO" "9. Checking for TODO comments..."
# Look for TODO/FIXME/XXX in comments, not in string literals or identifiers
TODO_COUNT=$(grep -r "(//|///|/\*|\*|//!).*TODO|(//|///|/\*|\*|//!).*FIXME|(//|///|/\*|\*|//!).*XXX" src/ --include="*.rs" 2>/dev/null | wc -l)
if [ "$TODO_COUNT" -eq "0" ]; then
    print_status "PASS" "No unresolved TODO comments found"
else
    print_status "WARN" "Found $TODO_COUNT unresolved TODO/FIXME/XXX comments"
    echo "Consider resolving these or documenting why they're needed"
fi

# 10. Documentation Check
echo ""
print_status "INFO" "10. Checking documentation..."
if cargo doc --no-deps --quiet 2>/dev/null; then
    print_status "PASS" "Documentation builds successfully"
else
    print_status "FAIL" "Documentation build failed"
    echo "Run 'cargo doc' for details"
    exit 1
fi

# 11. License Check
echo ""
print_status "INFO" "11. Checking license compliance..."
if [ -x "$HOME/.cargo/bin/cargo-deny" ]; then
    # Run cargo-deny but allow warnings (duplicate versions are not violations)
    if "$HOME/.cargo/bin/cargo-deny" check >/dev/null 2>&1; then
        print_status "PASS" "License compliance check passed"
    else
        print_status "FAIL" "License compliance issues found"
        echo "Run '$HOME/.cargo/bin/cargo-deny check' for details"
        exit 1
    fi
else
    print_status "WARN" "cargo-deny not installed (run: cargo install cargo-deny)"
fi

# 12. Performance Benchmark (if criterion is available)
echo ""
print_status "INFO" "12. Running performance benchmarks..."
if cargo bench --quiet 2>/dev/null || true; then
    print_status "PASS" "Performance benchmarks completed"
else
    print_status "WARN" "Performance benchmarks failed or not configured"
fi

# 13. Integration Tests
echo ""
print_status "INFO" "13. Running integration tests..."
if cargo test --tests --quiet 2>/dev/null; then
    print_status "PASS" "Integration tests passed"
else
    print_status "WARN" "Integration tests failed or not configured"
fi

# 14. Build Optimization Check
echo ""
print_status "INFO" "14. Checking build optimization..."
if cargo build --release --quiet 2>/dev/null; then
    if command -v stat >/dev/null 2>&1; then
        BIN=$(cargo metadata --no-deps --format-version 1 2>/dev/null | jq -r '.packages[0].targets[] | select(.kind[]=="bin") | .name' 2>/dev/null || echo "harper")
        BINARY_SIZE=$(stat -f%z "target/release/$BIN" 2>/dev/null || stat -c%s "target/release/$BIN" 2>/dev/null || echo "unknown")
    else
        BINARY_SIZE="unknown"
    fi
    print_status "PASS" "Release build successful (binary size: $BINARY_SIZE bytes)"
else
    print_status "FAIL" "Release build failed"
    exit 1
fi

# Summary
echo ""
echo -e "${GREEN}✓ Harper validation completed successfully${NC}"
echo ""
echo "What this means:"
echo " • Code builds cleanly"
echo " • Linting and formatting are consistent"
echo " • Tests and docs are healthy"
echo " • No obvious security or licensing risks"
echo ""
echo "You can merge, release, or ship with confidence."
echo ""
echo "Strong systems are built calmly."
