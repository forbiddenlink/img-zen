# Build Stage
FROM rust:1.99-slim-bookworm@sha256:24707e70a4f1e23d9a7b25202ec43126d28a52294b96aad5c670df1fd1f1d400 as builder

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
FROM debian:bookworm-slim@sha256:99c0b25c46d0214d5a2091092e7908907dcaecd6a962cde0ac700acee222b6f3

COPY --from=builder /usr/src/app/target/release/imgzen /usr/local/bin/imgzen

ENTRYPOINT ["imgzen"]
