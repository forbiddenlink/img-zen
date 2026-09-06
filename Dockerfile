# Build Stage
FROM rust:1.75-slim-bookworm@sha256:70c2a016184099262fd7cee46f3d35fec3568c45c62f87e37f7f665f766b1f74 as builder

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
FROM debian:bookworm-slim@sha256:88200866dfff7ea7f5cbcb6ec7c8a701889efe6fe859fe64d6990e4b07ea4171

COPY --from=builder /usr/src/app/target/release/imgzen /usr/local/bin/imgzen

ENTRYPOINT ["imgzen"]
