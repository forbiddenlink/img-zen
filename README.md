# ImgZen

A GitHub Action, written in Rust, that optimizes the images in a repository. It optimizes PNG files, generates AVIF and WebP variants, optionally generates resized copies, and rewrites your HTML to use lazy loading and `<picture>` elements. It is for developers with static sites who want smaller images without configuring a build pipeline.

## Features

- Scans the folders you list for PNG and JPEG source images and optimizes PNG originals losslessly with `oxipng`.
- Leaves JPEG originals untouched unless you set `lossy-jpeg: true` (lossy, re-encoded with `mozjpeg`; it drops EXIF, ICC and orientation data).
- Generates `AVIF` and `WebP` variants next to each image.
- Generates resized variants for the widths you give (off by default).
- Rewrites HTML: adds `loading="lazy"` to `<img>` tags and wraps images that have variants in `<picture>` (both on by default; the original `<img>` stays as the fallback).
- Writes a size-savings report to the job summary and to the `report` and `saved-size` outputs. It does not post comments on pull requests.
- A `check` mode that changes no files and fails the step when a PNG could be optimized further, for gating CI without a bot commit.

## Usage

Create a workflow file (e.g., `.github/workflows/optimize-images.yml`):

```yaml
name: Optimize Images
on:
  push:
    paths:
      - '**.png'
      - '**.jpg'
      - '**.jpeg'
  pull_request:
    paths:
      - '**.png'
      - '**.jpg'
      - '**.jpeg'

jobs:
  imgzen:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run ImgZen
        uses: forbiddenlink/img-zen@main # no release tags exist yet; pin a commit SHA for stability
        with:
          paths: 'public/images, assets'
          ignore: 'node_modules/**'
          generate-formats: 'avif, webp'
          responsive-widths: '640, 1024, 1920' # Optional: Generate resized variants
          inject-lazy-loading: 'true' # Optional: Inject loading="lazy" in HTML
          
      - name: Commit Changes
        uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: "chore: optimize images with ImgZen"
```

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `paths` | Comma-separated list of directories to scan. | `./` |
| `ignore` | Comma-separated list of glob patterns to ignore. | `node_modules/**, target/**, dist/**` (the binary also ignores `.git/**`) |
| `generate-formats` | Formats to generate (avif, webp). | `avif, webp` |
| `responsive-widths` | Comma-separated list of widths (px) to generate resized variants for. | `(Disabled)` |
| `inject-lazy-loading` | Inject `loading="lazy"` into HTML `<img>` tags. | `true` |
| `inject-picture` | Wrap `<img>` tags that have generated AVIF/WebP variants in `<picture>` with `srcset` widths. The original `<img>` stays as the fallback. Images without variants, with their own `srcset`, or already in a `<picture>` are left alone. | `true` |
| `quality` | Quality for AVIF/WebP encoding, and for JPEG originals when `lossy-jpeg` is `true` (1-100). | `75` |
| `png-level` | PNG optimization level (0-6, higher = slower but smaller). | `2` |
| `lossy-jpeg` | Re-encode JPEG originals in place at `quality`. Lossy, and drops EXIF, ICC profile and orientation data. By default JPEG originals are not modified (PNG originals are still losslessly optimized). | `false` |
| `mode` | `optimize` rewrites images in place. `check` changes no files and fails the step when a PNG could be losslessly optimized further (JPEGs count only with `lossy-jpeg: true`). | `optimize` |

## Outputs

| Output | Description |
|--------|-------------|
| `saved-size` | Total size saved by optimization. |
| `report` | Markdown report of the optimization. |

## Local development and testing

1. **Build**:

   ```bash
   cargo build --release
   ```

2. **Run Locally**:

   ```bash
   # Set inputs via args
   ./target/release/imgzen --paths ./sample_images --responsive-widths 500
   ```

3. **Run unit tests**:

   ```bash
   cargo test
   ```

4. **Run E2E Tests**:

   ```bash
   chmod +x tests/e2e.sh
   ./tests/e2e.sh
   ```

   `tests/partial_failure.py` and `tests/invalid_inputs.py` are further regression scripts that take the path to the built binary.

## License

MIT. See [LICENSE](LICENSE).
