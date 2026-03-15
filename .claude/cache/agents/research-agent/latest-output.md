# Research Report: ImgZen Project Analysis and Recommendations

Generated: 2026-02-25

## Executive Summary

ImgZen is a well-architected Rust-based GitHub Action for image optimization with AVIF/WebP generation, responsive images, and lazy loading injection. The codebase is functional but early-stage (~288 lines), using solid libraries (oxipng, ravif, image crate). Key gaps compared to mature tools include: no JPEG optimization, missing PR comment reports, deprecated GitHub Action output syntax, no Docker/Cargo caching, limited format support, and missing quality/compression controls.

## Research Question

Analyze the ImgZen project to identify improvements and next steps, including best practices for Rust image optimization, modern format support, GitHub Action patterns, and competitor feature comparison.

---

## Codebase Analysis

### Current Implementation Summary

| Component | Implementation | Status |
|-----------|---------------|--------|
| **PNG Optimization** | oxipng v9.0 with preset 2 | Working |
| **AVIF Generation** | ravif v0.11 (quality=75, speed=4) | Working |
| **WebP Generation** | webp v0.2 crate (quality=75) | Working |
| **Responsive Images** | Lanczos3 resize via image crate | Working |
| **Lazy Loading** | lol_html for HTML rewriting | Working |
| **Parallel Processing** | rayon for parallel image processing | Working |
| **JPEG Optimization** | **Not implemented** | Gap |
| **PR Comments** | **Not implemented** | Gap |

### Dependencies (Cargo.toml)

```toml
image = "0.24"        # Core image processing
oxipng = "9.0"        # PNG optimization
ravif = "0.11"        # AVIF encoding
webp = "0.2"          # WebP encoding
lol_html = "1.1"      # HTML streaming parser
rayon = "1.8"         # Parallel processing
clap = "4.4"          # CLI parsing with env support
```

### Architecture Observations

1. **Single-file implementation** (`src/main.rs`, 288 lines) - good for simplicity, may need refactoring as features grow
2. **Proper parallel processing** with rayon and Arc<Mutex<Stats>>
3. **Good error handling** with anyhow context
4. **Environment variable integration** via clap's `env` feature for GitHub Actions
5. **Glob-based ignore patterns** are basic (string contains check vs proper glob matching)

### Issues Found

1. **Deprecated GitHub Actions syntax**: Uses `::set-output` (deprecated since October 2022), should use `$GITHUB_OUTPUT` file
2. **No JPEG optimization**: Only PNG files are optimized; JPEGs just get converted
3. **Glob pattern matching is naive**: Uses `path_str.contains(&pat.replace("**", ""))` instead of proper glob library
4. **No quality controls**: AVIF/WebP quality hardcoded to 75.0
5. **No compression level controls**: oxipng preset hardcoded to 2
6. **No skip-if-larger logic for WebP/AVIF**: Always writes even if larger than original
7. **No caching in Dockerfile**: Rebuilds entire Cargo dependency tree every time

---

## Key Findings

### Finding 1: Rust Image Optimization Ecosystem Has Matured

**Rimage** is a comprehensive alternative that provides:
- Multi-format support (JpegXL, MozJpeg, oxipng, WebP, AVIF)
- Built-in quality presets
- Parallel processing

**MozJPEG Rust options**:
- `mozjpeg-rs`: Pure Rust, 6% faster than C mozjpeg, produces smaller files
- `jpegli-rs` / `zenjpeg`: Port of Google's jpegli with adaptive quantization
- `mozjpeg-sys`: FFI bindings to C mozjpeg

**Recommendation**: Add MozJPEG support for JPEG optimization (currently a major gap).

- Source: [Rimage - Lib.rs](https://lib.rs/crates/rimage)
- Source: [mozjpeg-rs - Lib.rs](https://lib.rs/crates/mozjpeg-rs)

### Finding 2: Modern Format Browser Support is Excellent in 2026

| Format | Browser Support | Recommendation |
|--------|----------------|----------------|
| **WebP** | Universal (Baseline for 30+ months) | Safe default fallback |
| **AVIF** | All modern browsers (Chrome, Firefox, Safari) | Primary format for photos |
| **JPEG XL** | Safari 17+ only (disabled in Chrome/Firefox) | Wait for broader support |

**Strategy for 2026**: AVIF first, WebP fallback, original as final fallback.

- Source: [RUMvision - Modern Image Formats](https://www.rumvision.com/blog/modern-image-formats-webp-avif-browser-support/)
- Source: [Can I Use - AVIF](https://caniuse.com/avif)

### Finding 3: GitHub Action Best Practices for Performance

**Caching strategies**:
1. **Docker layer caching**: Use `cache-from` and `cache-to` with `type=gha`
2. **Cargo caching**: Use `Swatinem/rust-cache` action for target/ and registry
3. **Pre-built binaries**: Consider publishing pre-built binaries instead of building in-action

**Deprecated syntax to fix**:
```bash
# OLD (deprecated)
echo "::set-output name=saved-size::$VALUE"

# NEW (required)
echo "saved-size=$VALUE" >> $GITHUB_OUTPUT
```

- Source: [Docker Docs - GitHub Actions Cache](https://docs.docker.com/build/cache/backends/gha/)
- Source: [Uffizzi - Optimizing Rust Builds](https://www.uffizzi.com/blog/optimizing-rust-builds-for-faster-github-actions-pipelines)

### Finding 4: Competitor Features Comparison

| Feature | ImgZen | Calibre Image Actions | imgproxy | Squoosh |
|---------|--------|----------------------|----------|---------|
| AVIF generation | Yes | Yes | Yes | Yes |
| WebP generation | Yes | Yes | Yes | Yes |
| JPEG optimization | **No** | Yes (MozJPEG) | Yes | Yes |
| PNG optimization | Yes | Yes | Yes | Yes |
| PR comment report | **No** | Yes | N/A | N/A |
| Visual diff links | **No** | Yes | N/A | Yes |
| Quality controls | **No** | Yes | Yes | Yes |
| Size threshold | **No** | Yes (5% default) | N/A | N/A |
| Responsive images | Yes | No | Yes | No |
| Lazy loading | Yes | No | No | No |
| SVG optimization | **No** | Yes | No | No |
| GIF optimization | **No** | Yes | Yes | No |

**ImgZen's unique advantages**: Responsive image generation, lazy loading injection
**Key gaps**: No JPEG optimization, no PR comments, no quality controls

- Source: [Calibre Image Actions](https://github.com/calibreapp/image-actions)
- Source: [Image Actions 2.0 Blog](https://calibreapp.com/blog/image-actions-2.0)

### Finding 5: PR Comment Reports are Expected

Mature image optimization actions provide:
1. **Compression summary comment** on the PR
2. **Per-file size comparison** (before/after)
3. **Visual diff links** to compare image quality
4. **Skip threshold** (e.g., only commit if >5% smaller)

Example comment format from Calibre Image Actions:
```markdown
## Image Optimization Results

| Image | Before | After | Savings |
|-------|--------|-------|---------|
| hero.png | 2.3 MB | 890 KB | 62% |
| logo.jpg | 156 KB | 98 KB | 37% |

**Total savings: 1.5 MB (58%)**
```

- Source: [DEV.to - Compress Images with GitHub Actions](https://dev.to/github/compress-images-for-the-web-with-github-actions-29a3)

---

## Recommendations

### Quick Wins (Low Effort, High Impact)

#### 1. Fix Deprecated GitHub Actions Output Syntax
**Effort**: 15 minutes | **Impact**: High (prevents future breakage)

```rust
// Replace:
println!("::set-output name=saved-size::{}", ...);

// With:
if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&output_file)?;
    writeln!(file, "saved-size={}", humansize::format_size(...))?;
}
```

#### 2. Add Docker Layer Caching
**Effort**: 30 minutes | **Impact**: High (90% faster builds)

Update Dockerfile:
```dockerfile
# Build Stage with cache mounts
FROM rust:1.75-slim-bookworm as builder
WORKDIR /usr/src/app

# Install deps first (cacheable)
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release && rm -rf src

# Now copy real source
COPY . .
RUN touch src/main.rs && cargo build --release
```

#### 3. Add Skip-If-Larger Logic
**Effort**: 30 minutes | **Impact**: Medium (avoids bloat)

Only write AVIF/WebP if smaller than original:
```rust
if webp_data.len() < original_bytes.len() {
    std::fs::write(&webp_path, &*webp_data)?;
    gen_size += webp_data.len() as u64;
}
```

#### 4. Add Quality Input Parameters
**Effort**: 1 hour | **Impact**: Medium (user control)

```yaml
# action.yml
inputs:
  quality:
    description: 'Compression quality (1-100)'
    default: '75'
  png-optimization-level:
    description: 'PNG optimization level (0-6)'
    default: '2'
```

### Medium-Term Improvements (1-2 days each)

#### 5. Add JPEG Optimization with MozJPEG
**Effort**: 1 day | **Impact**: High (major feature gap)

Add to Cargo.toml:
```toml
mozjpeg = "0.10"
```

Implement JPEG optimization similar to PNG:
```rust
if ext.to_lowercase() == "jpg" || ext.to_lowercase() == "jpeg" {
    let optimized = mozjpeg::compress(&original_bytes, quality)?;
    if optimized.len() < original_bytes.len() {
        std::fs::write(path, &optimized)?;
        saved_here = original_size - optimized.len() as u64;
    }
}
```

#### 6. Add PR Comment Report
**Effort**: 1-2 days | **Impact**: High (developer experience)

Requires:
1. Collect per-file stats during processing
2. Generate markdown table
3. Use `gh pr comment` or GitHub API to post

Output structure:
```rust
struct FileResult {
    path: String,
    original_size: u64,
    optimized_size: u64,
    formats_generated: Vec<String>,
}
```

#### 7. Fix Glob Pattern Matching
**Effort**: 2 hours | **Impact**: Medium (correctness)

Use the `glob` crate properly (already in dependencies):
```rust
use glob::Pattern;

let ignore_patterns: Vec<Pattern> = args.ignore
    .split(',')
    .filter_map(|s| Pattern::new(s.trim()).ok())
    .collect();

// Then check:
if !ignore_patterns.iter().any(|pat| pat.matches(&path_str)) {
    // process
}
```

#### 8. Add `<picture>` Element Generation
**Effort**: 1 day | **Impact**: Medium (HTML modernization)

Generate HTML snippets for optimal format delivery:
```html
<picture>
  <source srcset="image.avif" type="image/avif">
  <source srcset="image.webp" type="image/webp">
  <img src="image.jpg" loading="lazy" alt="">
</picture>
```

### Longer-Term Features (1+ weeks)

#### 9. Add SVG Optimization (SVGO)
**Effort**: 3-5 days | **Impact**: Medium

Options:
- Shell out to `svgo` CLI (simple)
- Use `usvg` Rust crate for parsing, custom optimization

#### 10. Add GIF to WebP/AVIF Animation Conversion
**Effort**: 1 week | **Impact**: Medium

Use `image` crate's GIF decoder with frame iteration, encode to animated WebP.

#### 11. Pre-built Binary Releases
**Effort**: 3-5 days | **Impact**: High (CI speed)

- Use GitHub Actions to build binaries for linux-x64, linux-arm64
- Publish as GitHub releases
- Update action.yml to download pre-built binary instead of Docker build

#### 12. Add Visual Diff URL Generation
**Effort**: 1 week | **Impact**: Medium (DX)

Generate comparison URLs using services like:
- GitHub's built-in image diff (for committed images)
- Upload to temporary hosting for before/after comparison

---

## Implementation Priority Matrix

| Priority | Item | Effort | Impact | Dependencies |
|----------|------|--------|--------|--------------|
| **P0** | Fix deprecated output syntax | 15 min | High | None |
| **P0** | Add Docker caching | 30 min | High | None |
| **P1** | Add JPEG optimization | 1 day | High | mozjpeg crate |
| **P1** | Add quality controls | 1 hour | Medium | None |
| **P1** | Add skip-if-larger | 30 min | Medium | None |
| **P2** | PR comment report | 1-2 days | High | GitHub API |
| **P2** | Fix glob matching | 2 hours | Medium | None |
| **P3** | Picture element generation | 1 day | Medium | HTML parser |
| **P3** | Pre-built binaries | 3-5 days | High | CI setup |
| **P4** | SVG optimization | 3-5 days | Medium | svgo or usvg |
| **P4** | GIF animation support | 1 week | Medium | image crate |

---

## Open Questions

1. **JPEG XL support?** - Browser support is limited (Safari only in 2026). Consider as opt-in feature flag.

2. **Lossless vs Lossy modes?** - Current implementation is lossy-only. Some users may want lossless WebP/AVIF for graphics.

3. **Integration testing strategy?** - Current e2e.sh is basic. Consider adding visual regression tests.

4. **Marketplace publishing?** - Action uses local reference (`./`). Plan for GitHub Marketplace publishing?

5. **Size limits?** - Should there be max image dimension/file size limits to prevent memory issues?

---

## Sources

### Rust Libraries
- [Rimage - Lib.rs](https://lib.rs/crates/rimage)
- [mozjpeg-rs - Lib.rs](https://lib.rs/crates/mozjpeg-rs)
- [jpegli-rs - crates.io](https://crates.io/crates/jpegli-rs)
- [Oxipng - GitHub](https://github.com/shssoichiro/oxipng)

### Image Formats
- [Modern Image Formats - RUMvision](https://www.rumvision.com/blog/modern-image-formats-webp-avif-browser-support/)
- [Can I Use - AVIF](https://caniuse.com/avif)
- [WebP vs JPEG vs AVIF - FreeImages](https://blog.freeimages.com/post/webp-vs-jpeg-vs-avif-best-format-for-web-photos)
- [AVIF vs WebP 2026 - Elementor](https://elementor.com/blog/webp-vs-avif/)

### GitHub Actions
- [GitHub Actions Cache](https://github.com/actions/cache)
- [Docker GHA Cache Docs](https://docs.docker.com/build/cache/backends/gha/)
- [Optimizing Rust Builds - Uffizzi](https://www.uffizzi.com/blog/optimizing-rust-builds-for-faster-github-actions-pipelines)
- [Docker Layer Caching Guide - Blacksmith](https://www.blacksmith.sh/blog/cache-is-king-a-guide-for-docker-layer-caching-in-github-actions)

### Competitor Tools
- [Calibre Image Actions](https://github.com/calibreapp/image-actions)
- [Image Actions 2.0 Blog](https://calibreapp.com/blog/image-actions-2.0)
- [Squoosh](https://squoosh.app/)
- [Smashing Magazine - Image Optimization Tools](https://www.smashingmagazine.com/2022/07/powerful-image-optimization-tools/)
