# Build Stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy only dependency files first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer gets cached)
RUN cargo build --release && rm -rf src target/release/deps/imgzen*

# Copy actual source
COPY src ./src

# Build the real application
RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

COPY --from=builder /usr/src/app/target/release/imgzen /usr/local/bin/imgzen

ENTRYPOINT ["imgzen"]
