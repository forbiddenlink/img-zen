# ImgZen 🖼️⚡

**ImgZen** is a high-performance, automatic image optimizer for GitHub Actions, built with Rust. It makes your web assets lighter and faster without any configuration.

## Features

- **🚀 Performance-First**: Built with Rust for blazing speed in CI/CD pipelines.
- **📦 Zero Configuration**: Works out of the box. Just add the action.
- **✨ Modern Formats**: Automatically generates `AVIF` and `WebP` variants.
- **📉 Smart Optimization**: Optimizes original PNG/JPEG files using industry-standard tools (`oxipng`).
- **📱 Responsive Images**: Generates resized versions of your images for responsive layouts (configurable).
- **⚡ Lazy Loading**: Automatically injects `loading="lazy"` into your HTML image tags (optional).
- **📊 Detailed Reporting**: precise "Saved X MB" reports in your PRs.

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
      - '**.webp'
  pull_request:
    paths:
      - '**.png'
      - '**.jpg'
      - '**.jpeg'
      - '**.webp'

jobs:
  imgzen:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Run ImgZen
        uses: ./ # Or your-username/imgzen@v1
        with:
          paths: 'public/images, assets'
          ignore: 'node_modules/**'
          generate-formats: 'avif, webp'
          responsive-widths: '640, 1024, 1920' # Optional: Generate resized variants
          inject-lazy-loading: 'true' # Optional: Inject loading="lazy" in HTML
          
      - name: Commit Changes
        uses: stefanzweifel/git-auto-commit-action@v5
        with:
          commit_message: "⚡ ImgZen: Optimized images"
```

## Inputs

| Input | Description | Default |
|-------|-------------|---------|
| `paths` | Comma-separated list of directories to scan. | `./` |
| `ignore` | Comma-separated list of glob patterns to ignore. | `node_modules/**, target/**, dist/**, .git/**` |
| `generate-formats` | Formats to generate (avif, webp). | `avif, webp` |
| `responsive-widths` | Comma-separated list of widths (px) to generate resized variants for. | `(Disabled)` |
| `inject-lazy-loading` | Inject `loading="lazy"` into HTML `<img>` tags. | `true` |

## Local Development & Testing

1. **Build**:

   ```bash
   cargo build --release
   ```

2. **Run Locally**:

   ```bash
   # Set inputs via args
   ./target/release/imgzen --paths ./sample_images --responsive-widths 500
   ```

3. **Run E2E Tests**:

   ```bash
   chmod +x tests/e2e.sh
   ./tests/e2e.sh
   ```

## License

MIT
