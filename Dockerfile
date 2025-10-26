# Build stage
FROM rust:latest as builder

WORKDIR /app

# Copy everything
COPY Cargo.toml ./
COPY src ./src

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/postgres-api /app/postgres-api

EXPOSE 8080

CMD ["/app/postgres-api"]