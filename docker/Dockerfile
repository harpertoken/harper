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

# Build stage
FROM rust:1.92.0 AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy workspace crates
COPY lib ./lib

# Build application
RUN cargo build --release

# Test stage
FROM rust:1.92.0 AS tester

WORKDIR /app

# Install system dependencies first to leverage Docker layer caching.
RUN apt-get update && apt-get install -y --no-install-recommends libsqlite3-dev pkg-config && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Reuse build artifacts from the builder stage to avoid re-downloading dependencies.
COPY --from=builder /usr/local/cargo/registry /usr/local/cargo/registry

# Copy config files needed for tests
COPY config ./config

# Copy workspace and tests
COPY lib ./lib
COPY tests ./tests

    # Run E2E tests in debug mode as some tests may not work in release.
    RUN cargo test --workspace --test integration --test session_service_test

# Runtime stage
FROM debian:bookworm-slim

# Install ca-certificates for HTTPS requests
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy config files
COPY config ./config

# Copy the binary from builder stage
COPY --from=builder /app/target/release/harper .

# Set the binary as executable
RUN chmod +x harper

# Create and switch to a non-root user
RUN useradd --create-home appuser
RUN chown -R appuser:appuser /app
USER appuser

# Set environment variables (can be overridden)
ENV DATABASE_PATH=/app/data/harper.db

# Expose any ports if needed (Harper is CLI, so probably not)
# EXPOSE 8080

# Set the entrypoint
ENTRYPOINT ["./harper"]
