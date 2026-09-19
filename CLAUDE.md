# ImgZen

A GitHub Action, written in Rust, that automatically optimizes images in a repo:
generates AVIF/WebP variants, optimizes PNG/JPEG (`oxipng`, `mozjpeg`), optionally
generates responsive widths, and can inject `loading="lazy"` into HTML `<img>` tags.
Runs as a Docker-based action and reports size savings back to the PR.

Repo: https://github.com/forbiddenlink/img-zen

## Stack

- Rust (edition 2021), Cargo (Cargo.lock present)
- Key deps: `clap` (CLI/env args), `image`, `ravif`, `webp`, `oxipng`, `mozjpeg`
  (encoders), `lol_html` (HTML rewriting for lazy-loading injection), `rayon`
  (parallelism), `walkdir` + `globset` (file discovery)
- Ships as a Docker action (`action.yml` -> `Dockerfile`, multi-stage Rust build)

## Commands

- `cargo build --release` - build the binary
- `./target/release/imgzen --paths ./sample_images --responsive-widths 500` - run
  locally against a directory
- `chmod +x tests/e2e.sh && ./tests/e2e.sh` - end-to-end test script

## Layout

- `src/main.rs` - CLI entry point and optimization pipeline
- `src/bin/gen_test_image.rs` - helper binary to generate a test image
- `tests/e2e.sh` - shell-driven end-to-end test; `tests/test_run/` holds its
  fixture/output image
- `action.yml` - GitHub Action definition (inputs: `paths`, `ignore`,
  `generate-formats`, `responsive-widths`, `inject-lazy-loading`, `quality`,
  `png-level`; outputs: `saved-size`, `report`)
- `Dockerfile` - two-stage build (Rust builder -> `debian:bookworm-slim` runtime),
  pinned base images by digest

## Conventions

- No env vars beyond what `clap`'s `env` feature exposes for CLI args (no secrets
  or app config to set)
- CI: `.github/workflows/ci.yml`, `dependabot-automerge.yml`, `scorecard.yml`

## Gotchas

- The action always runs via Docker (`runs.using: 'docker'` in `action.yml`), so
  local testing needs either the compiled binary directly or a Docker build - the
  Dockerfile is the source of truth for the runtime environment, not `cargo run`.
