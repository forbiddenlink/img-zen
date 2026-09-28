# Build Stage
FROM rust:1.98-slim-bookworm@sha256:ff521445a372125ed4f76e1453a1f8098f2d05332d1601d30db1c1f62757e730 as builder

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
FROM debian:bookworm-slim@sha256:3783cc01769c7b2b1b83a5c5ad96c815348e28ed7da68e2e3687004faa906251

COPY --from=builder /usr/src/app/target/release/imgzen /usr/local/bin/imgzen

ENTRYPOINT ["imgzen"]
