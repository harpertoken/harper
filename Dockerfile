# Build stage
FROM rust:latest AS builder

WORKDIR /app

# Copy dependency files
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:latest

# Install ca-certificates for HTTPS requests
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy config files
COPY config ./config

# Copy the binary from builder stage
COPY --from=builder /app/target/release/harper .

# Set the binary as executable
RUN chmod +x harper

# Set environment variables (can be overridden)
ENV DATABASE_PATH=./harper.db

# Expose any ports if needed (Harper is CLI, so probably not)
# EXPOSE 8080

# Set the entrypoint
ENTRYPOINT ["./harper"]