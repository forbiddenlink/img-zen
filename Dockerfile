# Build Stage
FROM rust:1.75-slim-bookworm as builder

WORKDIR /usr/src/app
COPY . .

# Install build dependencies if needed (e.g., for avif/ravif via nasm if not pure native)
# The 'image' crate with 'avif-native' might need some system deps or just static build.
# For simplicity in 'avif-native' (ravif), it usually works out of box or needs nasm.
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

RUN cargo build --release

# Runtime Stage
FROM debian:bookworm-slim

COPY --from=builder /usr/src/app/target/release/imgzen /usr/local/bin/imgzen

ENTRYPOINT ["imgzen"]
