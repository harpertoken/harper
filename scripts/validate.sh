#!/bin/bash

set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

TOTAL_STEPS=14
PASSED_STEPS=0
WARNED_STEPS=0

print_banner() {
    echo
    echo -e "${BLUE}╭──────────────────────────────────────────────╮${NC}"
    echo -e "${BLUE}│          Harper Validation Suite             │${NC}"
    echo -e "${BLUE}│        Build. Lint. Test. Verify.            │${NC}"
    echo -e "${BLUE}╰──────────────────────────────────────────────╯${NC}"
    echo
}

print_status() {
    local status=$1
    local message=$2
    case $status in
        PASS) echo -e "${GREEN}PASS${NC}: $message" ;;
        FAIL) echo -e "${RED}FAIL${NC}: $message" ;;
        WARN) echo -e "${YELLOW}WARN${NC}: $message" ;;
        INFO) echo -e "${BLUE}INFO${NC}: $message" ;;
    esac
}

run_command_with_log() {
    local log_file
    log_file=$(mktemp)

    if "$@" >"$log_file" 2>&1; then
        rm -f "$log_file"
        return 0
    fi

    echo "--- command output ---"
    cat "$log_file"
    rm -f "$log_file"
    return 1
}

run_required_step() {
    local step=$1
    local title=$2
    local fail_message=$3
    local tip_message=$4
    shift 4

    echo
    print_status "INFO" "[$step/$TOTAL_STEPS] $title"
    if run_command_with_log "$@"; then
        print_status "PASS" "$title"
        PASSED_STEPS=$((PASSED_STEPS + 1))
    else
        print_status "FAIL" "$fail_message"
        if [ -n "$tip_message" ]; then
            echo "$tip_message"
        fi
        exit 1
    fi
}

run_optional_step() {
    local step=$1
    local title=$2
    shift 2

    echo
    print_status "INFO" "[$step/$TOTAL_STEPS] $title"
    if "$@"; then
        print_status "PASS" "$title"
        PASSED_STEPS=$((PASSED_STEPS + 1))
    else
        print_status "WARN" "$title"
        WARNED_STEPS=$((WARNED_STEPS + 1))
    fi
}

check_project_root() {
    if [ ! -f "Cargo.toml" ]; then
        print_status "FAIL" "Not in Harper project root (Cargo.toml not found)"
        exit 1
    fi
}

check_large_files() {
    local large_files
    large_files=$(find . -type f -size +10M -not -path "./target/*" -not -path "./.git/*" 2>/dev/null || true)
    if [ -n "$large_files" ]; then
        print_status "FAIL" "Large files detected (>10MB)"
        echo "$large_files"
        return 1
    fi
    return 0
}

check_sensitive_files() {
    local sensitive_files
    sensitive_files=$(find . \( -name "*.key" -o -name "*.pem" -o -name "*.env" \) -not -path "./target/*" -not -path "./.git/*" 2>/dev/null || true)
    if [ -n "$sensitive_files" ]; then
        print_status "WARN" "Potential sensitive files found"
        echo "$sensitive_files"
        WARNED_STEPS=$((WARNED_STEPS + 1))
    else
        print_status "PASS" "No sensitive files found in repository"
        PASSED_STEPS=$((PASSED_STEPS + 1))
    fi
    return 0
}

check_yaml() {
    if ! command -v yamllint >/dev/null 2>&1; then
        print_status "WARN" "yamllint not installed (run: pip install yamllint)"
        WARNED_STEPS=$((WARNED_STEPS + 1))
        return 0
    fi

    local yaml_files
    yaml_files=$(find . \( -name "*.yml" -o -name "*.yaml" \) -not -path "./target/*" -not -path "./.git/*" 2>/dev/null || true)
    if [ -z "$yaml_files" ]; then
        print_status "PASS" "No YAML files to validate"
        PASSED_STEPS=$((PASSED_STEPS + 1))
        return 0
    fi

    if echo "$yaml_files" | xargs yamllint >/dev/null 2>&1; then
        print_status "PASS" "YAML files are valid"
        PASSED_STEPS=$((PASSED_STEPS + 1))
        return 0
    fi

    print_status "FAIL" "YAML validation failed"
    echo "Run 'yamllint .github/workflows/*.yml docker/docker-compose.yml' for details"
    return 1
}

check_todos() {
    local todo_count
    todo_count=$(
        rg --glob '*.rs' --count-matches \
            '(//|///|/\*|\*|//!).*(TODO|FIXME|XXX)' \
            . 2>/dev/null || true
    )
    todo_count=$(
        printf '%s\n' "$todo_count" \
            | awk -F: '{sum += $NF} END {print sum + 0}'
    )

    if [ "$todo_count" -eq 0 ]; then
        print_status "PASS" "No unresolved TODO comments found"
        PASSED_STEPS=$((PASSED_STEPS + 1))
    else
        print_status "WARN" "Found $todo_count unresolved TODO/FIXME/XXX comments"
        echo "Consider resolving these or documenting why they're needed"
        WARNED_STEPS=$((WARNED_STEPS + 1))
    fi
    return 0
}

check_cargo_audit() {
    if ! command -v cargo-audit >/dev/null 2>&1 && [ ! -x "$HOME/.cargo/bin/cargo-audit" ]; then
        print_status "WARN" "cargo-audit not installed (run: cargo install cargo-audit)"
        WARNED_STEPS=$((WARNED_STEPS + 1))
        return 0
    fi

    local audit_bin="cargo-audit"
    if ! command -v cargo-audit >/dev/null 2>&1; then
        audit_bin="$HOME/.cargo/bin/cargo-audit"
    fi

    if run_command_with_log "$audit_bin" audit --quiet; then
        print_status "PASS" "Security audit passed"
        PASSED_STEPS=$((PASSED_STEPS + 1))
        return 0
    fi

    print_status "FAIL" "Security audit found vulnerabilities"
    echo "Run '$audit_bin audit' for details"
    return 1
}

check_cargo_deny() {
    if ! command -v cargo-deny >/dev/null 2>&1 && [ ! -x "$HOME/.cargo/bin/cargo-deny" ]; then
        print_status "WARN" "cargo-deny not installed (run: cargo install cargo-deny)"
        WARNED_STEPS=$((WARNED_STEPS + 1))
        return 0
    fi

    local deny_bin="cargo-deny"
    if ! command -v cargo-deny >/dev/null 2>&1; then
        deny_bin="$HOME/.cargo/bin/cargo-deny"
    fi

    if run_command_with_log "$deny_bin" check; then
        print_status "PASS" "cargo-deny policy checks passed"
        PASSED_STEPS=$((PASSED_STEPS + 1))
        return 0
    fi

    print_status "FAIL" "cargo-deny reported dependency policy issues"
    echo "Run '$deny_bin check' for details"
    return 1
}

check_benchmarks() {
    if cargo bench --workspace --quiet >/dev/null 2>&1; then
        return 0
    fi
    return 1
}

check_integration_tests() {
    if cargo test --tests --workspace --quiet >/dev/null 2>&1; then
        return 0
    fi
    return 1
}

release_binary_size() {
    if ! command -v stat >/dev/null 2>&1; then
        echo "unknown"
        return 0
    fi

    local bin
    bin=$(cargo metadata --no-deps --format-version 1 2>/dev/null | jq -r '.packages[0].targets[] | select(.kind[]=="bin") | .name' 2>/dev/null || echo "harper")
    stat -f%z "target/release/$bin" 2>/dev/null || stat -c%s "target/release/$bin" 2>/dev/null || echo "unknown"
}

print_summary() {
    echo
    echo -e "${GREEN}✓ Harper validation completed${NC}"
    echo
    echo "Summary:"
    echo " • Passed steps: $PASSED_STEPS/$TOTAL_STEPS"
    echo " • Warning steps: $WARNED_STEPS"
    echo " • Required checks: all passed"
    echo
}

main() {
    check_project_root
    print_banner
    print_status "INFO" "Starting validation checks..."

    run_required_step 1 "Rust compilation check" \
        "Rust compilation failed" \
        "Run 'cargo check --workspace' for details" \
        cargo check --workspace --quiet

    run_required_step 2 "Clippy linter" \
        "Clippy reported issues that need attention" \
        "Run 'cargo clippy --all-targets --all-features --workspace -- -A clippy::pedantic -D warnings' for details" \
        cargo clippy --all-targets --all-features --workspace --quiet -- -A clippy::pedantic -D warnings

    run_required_step 3 "Formatting check" \
        "Code formatting issues found" \
        "Run 'cargo fmt --all' to fix formatting" \
        cargo fmt --all -- --check

    run_required_step 4 "Workspace tests" \
        "Workspace tests failed" \
        "Run 'cargo test --workspace' for details" \
        cargo test --workspace --quiet

    echo
    print_status "INFO" "[5/$TOTAL_STEPS] Security audit"
    check_cargo_audit

    echo
    print_status "INFO" "[6/$TOTAL_STEPS] Large file check"
    if check_large_files; then
        print_status "PASS" "No large files found"
        PASSED_STEPS=$((PASSED_STEPS + 1))
    else
        exit 1
    fi

    echo
    print_status "INFO" "[7/$TOTAL_STEPS] Sensitive file check"
    check_sensitive_files

    echo
    print_status "INFO" "[8/$TOTAL_STEPS] YAML validation"
    if ! check_yaml; then
        exit 1
    fi

    echo
    print_status "INFO" "[9/$TOTAL_STEPS] TODO/FIXME/XXX scan"
    check_todos

    run_required_step 10 "Documentation build" \
        "Documentation build failed" \
        "Run 'cargo doc --no-deps --workspace' for details" \
        cargo doc --no-deps --workspace --quiet

    echo
    print_status "INFO" "[11/$TOTAL_STEPS] cargo-deny policy check"
    if ! check_cargo_deny; then
        exit 1
    fi

    run_optional_step 12 "Workspace benchmarks" check_benchmarks
    run_optional_step 13 "Integration tests" check_integration_tests

    run_required_step 14 "Release build" \
        "Release build failed" \
        "Run 'cargo build --release --workspace' for details" \
        cargo build --release --workspace --quiet

    print_status "PASS" "Release build successful (binary size: $(release_binary_size) bytes)"
    print_summary
}

main "$@"
