# Build stage
FROM rust:1.82.0 AS builder

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Build dependencies with a dummy main file
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release

# Copy application source
COPY src ./src

# Build application (will use cached dependencies)
RUN cargo build --release

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