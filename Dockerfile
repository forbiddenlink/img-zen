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
FROM debian:bookworm-slim@sha256:7c7b2c966bc9ee8cedfeef67e0e279108992c77681fa595db4a9d65c06ccc587

COPY --from=builder /usr/src/app/target/release/imgzen /usr/local/bin/imgzen

ENTRYPOINT ["imgzen"]
