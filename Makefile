# Copyright 2026 harpertoken
#
# Licensed under the Apache License, Version 2.0

.PHONY: help test test-release build build-debug run run-server dev fmt lint doc clean install update audit deny check all

help: ## Show available targets
	@printf "\nHarper\n\n"
	@printf "Usage:\n  make <target>\n\n"
	@printf "Targets:\n"
	@grep -E '^[a-zA-Z_-]+:.*?##' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?##"} {printf "  %-14s %s\n", $$1, $$2}'

test: ## Run tests (unit + integration)
	@cargo test --all-features --workspace

test-release: ## Run tests in release mode
	@cargo test --release --workspace

build: ## Build release
	@cargo build --release --workspace

build-debug: ## Build debug
	@cargo build --workspace

run: ## Run harper-ui
	@cargo run -p harper-ui

run-server: ## Run harper-mcp-server
	@cargo run -p harper-mcp-server

dev: ## Development mode (watch if available)
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x run; \
	else \
		printf "cargo-watch not installed. Install with:\n  cargo install cargo-watch\n"; \
		cargo run -p harper-ui; \
	fi

fmt: ## Check formatting
	@cargo fmt --all --check

lint: ## Run clippy
	@cargo clippy --all-targets --all-features --workspace -- -D warnings

doc: ## Generate docs
	@cargo doc --no-deps --document-private-items --all-features --workspace --open

clean: ## Clean artifacts
	@cargo clean
	@rm -f chat_sessions.db session_*.txt

install: ## Install dev tools
	@cargo install cargo-audit cargo-deny cargo-watch

update: ## Update dependencies
	@cargo update

audit: ## Security audit
	@cargo audit || true

deny: ## Dependency check
	@cargo deny check || true

check: fmt lint doc ## Quick check
	@printf "✓ check complete\n"

all: check test test-release build build-debug ## Run everything
	@printf "✓ all tasks complete\n"
