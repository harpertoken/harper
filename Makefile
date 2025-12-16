.PHONY: help test build clean fmt lint doc install run dev check all

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

# Default target
help:
	@echo "Harper - AI Agent"
	@echo ""
	@echo "Available commands:"
	@echo "  make test     - Run all tests and checks"
	@echo "  make build    - Build the project"
	@echo "  make run      - Run the application"
	@echo "  make dev      - Run in development mode"
	@echo "  make fmt      - Format code"
	@echo "  make lint     - Run linter"
	@echo "  make doc      - Generate documentation"
	@echo "  make clean    - Clean build artifacts"
	@echo "  make install  - Install dependencies"
	@echo "  make check    - Quick check (fmt + lint)"
	@echo "  make all      - Run everything (check + test + build)"

# Run comprehensive tests
test:
	@./scripts/test.sh

# Build the project
build:
	cargo build --release

# Run the application
run:
	cargo run

# Development mode with file watching (requires cargo-watch)
dev:
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x run; \
	else \
		echo "cargo-watch not installed. Run: cargo install cargo-watch"; \
		cargo run; \
	fi

# Format code
fmt:
	cargo fmt --all

# Run linter
lint:
	cargo clippy --all-targets --all-features --workspace -- -D warnings

# Generate documentation
doc:
	cargo doc --no-deps --document-private-items --all-features --workspace --open

# Clean build artifacts
clean:
	cargo clean
	rm -f chat_sessions.db
	rm -f session_*.txt

# Install development dependencies
install:
	cargo install cargo-audit cargo-deny cargo-watch cargo-llvm-cov

# Quick check
check: fmt lint
	@echo "Quick check completed"

# Run everything
all: check test build
	@echo "All tasks completed successfully"

# Update dependencies
update:
	cargo update

# Security audit
audit:
	cargo audit

# Dependency check
deny:
	cargo deny check

# Updated for v0.1.6
