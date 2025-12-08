# syntax=docker/dockerfile:1

# Stage 1: Build the Rust application
FROM rust:alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev pkgconfig openssl-dev openssl-libs-static

# Set the working directory
WORKDIR /app

# Copy Cargo files first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create dummy source files to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    echo "pub fn dummy() {}" > src/lib.rs && \
    cargo build --release && \
    rm -rf src

# Copy the actual source code
COPY src ./src

# Build the actual application
RUN touch src/lib.rs src/main.rs && \
    cargo build --release

# Stage 2: Create a minimal runtime image
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache libgcc

# Create a non-root user
RUN addgroup -g 1000 appuser && \
    adduser -D -u 1000 -G appuser appuser

# Set the working directory
WORKDIR /app

# Copy the binary from the builder stage
COPY --from=builder /app/target/release/german-bridge-backend /app/german-bridge-backend

# Change ownership
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose the port the app runs on
EXPOSE 8080

# Run the application
CMD ["/app/german-bridge-backend"]
