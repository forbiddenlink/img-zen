use clap::Parser;
use walkdir::WalkDir;
use std::path::Path;
use std::env;
use anyhow::{Context, Result};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Directories to scan for images
    #[arg(short, long, env = "INPUT_PATHS", default_value = "./")]
    paths: String,

    /// Glob patterns to ignore
    #[arg(short, long, env = "INPUT_IGNORE", default_value = "node_modules/**, target/**, dist/**, .git/**")]
    ignore: String,

    /// Formats to generate
    #[arg(short, long, env = "INPUT_GENERATE_FORMATS", default_value = "avif, webp")]
    generate_formats: String,

    /// Responsive widths to generate (comma separated)
    #[arg(short, long, env = "INPUT_RESPONSIVE_WIDTHS", default_value = "")]
    responsive_widths: String,

    /// Inject lazy loading
    #[arg(short = 'l', long, env = "INPUT_INJECT_LAZY_LOADING", default_value = "true")]
    inject_lazy_loading: String,

    /// Quality for AVIF/WebP/JPEG encoding (1-100)
    #[arg(short, long, env = "INPUT_QUALITY", default_value = "75")]
    quality: f32,

    /// PNG optimization level (0-6, higher = slower but smaller)
    #[arg(long, env = "INPUT_PNG_LEVEL", default_value = "2")]
    png_level: u8,
}

struct Stats {
    original_size: u64,
    saved_bytes: u64,
    generated_bytes: u64,
    files_count: usize,
    file_reports: Vec<FileReport>,
}

struct FileReport {
    path: String,
    original_size: u64,
    saved_bytes: u64,
    generated_formats: Vec<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Parse paths
    let paths: Vec<&str> = args.paths.split(',').map(|s| s.trim()).collect();

    // Build glob matcher for ignore patterns
    let mut glob_builder = globset::GlobSetBuilder::new();
    for pattern in args.ignore.split(',').map(|s| s.trim()) {
        if !pattern.is_empty() {
            if let Ok(glob) = globset::Glob::new(pattern) {
                glob_builder.add(glob);
            }
        }
    }
    let ignore_globs = glob_builder.build().unwrap_or_else(|_| globset::GlobSet::empty());
    
    // Parse formats
    let formats: Vec<&str> = args.generate_formats.split(',').map(|s| s.trim()).collect();
    let gen_avif = formats.contains(&"avif");
    let gen_webp = formats.contains(&"webp");

    // Parse responsive widths
    let responsive_widths: Vec<u32> = args.responsive_widths
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();

    println!("ImgZen Starting...");
    println!("Scanning paths: {:?}", paths);
    if !responsive_widths.is_empty() {
        println!("Responsive widths: {:?}", responsive_widths);
    }
    
    let stats = Arc::new(Mutex::new(Stats {
        original_size: 0,
        saved_bytes: 0,
        generated_bytes: 0,
        files_count: 0,
        file_reports: Vec::new(),
    }));

    // Collect all image files
    let mut image_files = Vec::new();
    for path in &paths {
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() && !ignore_globs.is_match(path) {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext = ext.to_lowercase();
                    if ["jpg", "jpeg", "png"].contains(&ext.as_str()) {
                        image_files.push(path.to_owned());
                    }
                }
            }
        }
    }

    println!("Found {} images.", image_files.len());

    let quality = args.quality;
    let png_level = args.png_level;

    image_files.par_iter().for_each(|path| {
        if let Err(e) = process_image(path, gen_avif, gen_webp, &responsive_widths, quality, png_level, &stats) {
            eprintln!("Failed to process {:?}: {}", path, e);
        }
    });

    // Lazy loading injection
    if args.inject_lazy_loading == "true" {
        println!("Injecting lazy loading...");
        // Collect HTML files
        let mut html_files = Vec::new();
        for path in &paths {
             for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_file() && !ignore_globs.is_match(path) {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        let ext = ext.to_lowercase();
                        if ext == "html" || ext == "htm" {
                            html_files.push(path.to_owned());
                        }
                    }
                }
             }
        }
        
        html_files.par_iter().for_each(|path| {
            if let Err(e) = inject_lazy_loading(path) {
                eprintln!("Failed to inject lazy loading in {:?}: {}", path, e);
            }
        });
    }

    let final_stats = stats.lock().unwrap();
    println!("Optimization complete!");
    println!("Processed {} files.", final_stats.files_count);
    println!("Original Size: {}", humansize::format_size(final_stats.original_size, humansize::DECIMAL));
    println!("Saved: {}", humansize::format_size(final_stats.saved_bytes, humansize::DECIMAL));
    println!("Generated Assets: {}", humansize::format_size(final_stats.generated_bytes, humansize::DECIMAL));

    // GitHub Actions Output
    if env::var("GITHUB_ACTIONS").is_ok() {
        // Build per-file table
        let mut file_table = String::from("| File | Original | Saved | Formats |\n|---|---|---|---|\n");
        for fr in &final_stats.file_reports {
            let formats = if fr.generated_formats.is_empty() {
                "-".to_string()
            } else {
                fr.generated_formats.join(", ")
            };
            file_table.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                fr.path,
                humansize::format_size(fr.original_size, humansize::DECIMAL),
                humansize::format_size(fr.saved_bytes, humansize::DECIMAL),
                formats
            ));
        }

        let report = format!(
            "### ImgZen Optimization Report\n\n\
            | Metric | Value |\n|---|---|\n\
            | Processed Files | {} |\n\
            | Original Size | {} |\n\
            | **Saved Size** | **{}** |\n\
            | Generated Assets | {} |\n\n\
            <details>\n<summary>Per-file details</summary>\n\n{}\n</details>",
            final_stats.files_count,
            humansize::format_size(final_stats.original_size, humansize::DECIMAL),
            humansize::format_size(final_stats.saved_bytes, humansize::DECIMAL),
            humansize::format_size(final_stats.generated_bytes, humansize::DECIMAL),
            file_table
        );

        // Write to GITHUB_OUTPUT (replaces deprecated ::set-output)
        if let Ok(output_path) = env::var("GITHUB_OUTPUT") {
            use std::io::Write;
            if let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(&output_path) {
                let _ = writeln!(file, "saved-size={}", humansize::format_size(final_stats.saved_bytes, humansize::DECIMAL));
                // Write multiline report using heredoc syntax
                let _ = writeln!(file, "report<<EOF\n{}\nEOF", report);
            }
        }

        if let Ok(summary_file) = env::var("GITHUB_STEP_SUMMARY") {
             let _ = std::fs::write(&summary_file, &report);
        }
    }
    
    Ok(())
}

fn inject_lazy_loading(path: &Path) -> Result<()> {
    let content = std::fs::read_to_string(path)?;
    let mut output = Vec::new();
    use lol_html::{HtmlRewriter, Settings, element};
    
    let mut rewriter = HtmlRewriter::new(
        Settings::new()
            .append_element_content_handler(
                element!("img", |el| {
                    if el.get_attribute("loading").is_none() {
                        el.set_attribute("loading", "lazy")?;
                    }
                    Ok(())
                })
            ),
        |c: &[u8]| {
            output.extend_from_slice(c);
        }
    );
    rewriter.write(content.as_bytes())?;
    rewriter.end()?;
    
    let new_content = String::from_utf8(output)?;
    if new_content != content {
        std::fs::write(path, new_content)?;
    }
    
    Ok(())
}

fn process_image(path: &Path, gen_avif: bool, gen_webp: bool, responsive_widths: &[u32], quality: f32, png_level: u8, stats: &Arc<Mutex<Stats>>) -> Result<()> {
    // Load image
    let img = image::open(path).with_context(|| format!("Failed to open image {:?}", path))?;
    let metadata = std::fs::metadata(path)?;
    let original_bytes = std::fs::read(path)?;
    let original_size = metadata.len();

    // Optimize Original (PNG or JPEG)
    let mut saved_here = 0;
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext_lower = ext.to_lowercase();
        if ext_lower == "png" {
             let options = oxipng::Options::from_preset(png_level);
             if let Ok(optimized_data) = oxipng::optimize_from_memory(&original_bytes, &options) {
                 if optimized_data.len() < original_bytes.len() {
                     std::fs::write(path, &optimized_data)?;
                     saved_here = original_size as u64 - optimized_data.len() as u64;
                 }
             }
        } else if ext_lower == "jpg" || ext_lower == "jpeg" {
            // Optimize JPEG with mozjpeg
            let rgb = img.to_rgb8();
            let (w, h) = (img.width() as usize, img.height() as usize);
            let jpeg_result: Option<Vec<u8>> = (|| {
                let mut comp = mozjpeg::Compress::new(mozjpeg::ColorSpace::JCS_RGB);
                comp.set_size(w, h);
                comp.set_quality(quality);
                let mut output = Vec::new();
                let mut started = comp.start_compress(&mut output).ok()?;
                let _ = started.write_scanlines(rgb.as_raw());
                started.finish().ok()?;
                Some(output)
            })();
            if let Some(optimized_data) = jpeg_result {
                if optimized_data.len() < original_bytes.len() {
                    std::fs::write(path, &optimized_data)?;
                    saved_here = original_size as u64 - optimized_data.len() as u64;
                }
            }
        }
    }

    let (width, height) = (img.width(), img.height());
    let rgba = img.to_rgba8();

    let mut generated_size = 0;

    // Helper to generate formats for a buffer (skip if larger than source)
    // Returns (generated_size, list of formats generated)
    let save_formats = |stem_path: &Path, img_rgba: &image::RgbaImage, w: u32, h: u32, source_size: u64| -> Result<(u64, Vec<String>)> {
        let mut gen_size = 0;
        let mut formats_generated = Vec::new();

        if gen_webp {
            let webp_path = stem_path.with_extension("webp");
            let encoder = webp::Encoder::from_rgba(img_rgba, w, h);
            let webp_data = encoder.encode(quality);
            // Only write if smaller than source
            if (webp_data.len() as u64) < source_size {
                std::fs::write(&webp_path, &*webp_data)?;
                gen_size += webp_data.len() as u64;
                formats_generated.push("webp".to_string());
            }
        }

        if gen_avif {
             let avif_path = stem_path.with_extension("avif");
             use rgb::FromSlice;
             let pixels = img_rgba.as_raw().as_rgba();
             let img_buffer = imgref::Img::new(pixels, w as usize, h as usize);
             let encoder = ravif::Encoder::new().with_quality(quality).with_speed(4);
             let result = encoder.encode_rgba(img_buffer)
                .map_err(|e| anyhow::anyhow!("AVIF encoding failed: {:?}", e))?;
             // Only write if smaller than source
             if (result.avif_file.len() as u64) < source_size {
                 std::fs::write(&avif_path, &result.avif_file)?;
                 gen_size += result.avif_file.len() as u64;
                 formats_generated.push("avif".to_string());
             }
        }
        Ok((gen_size, formats_generated))
    };

    // Generate for original size
    let mut all_formats = Vec::new();
    let (gen_size, formats) = save_formats(path, &rgba, width, height, original_size)?;
    generated_size += gen_size;
    all_formats.extend(formats);

    // Generate Responsive Versions
    for &target_width in responsive_widths {
        if width > target_width {
            let filter = image::imageops::FilterType::Lanczos3;
            let resized = image::imageops::resize(&img, target_width, (height as f64 * (target_width as f64 / width as f64)) as u32, filter);
            // resized returns RgbaImage (ImageBuffer)
            
            // Construct new filename: stem-width.ext
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let new_filename = format!("{}-{}.{}", stem, target_width, ext);
                    let new_path = path.with_file_name(new_filename);
                    
                    // Save resized original format? (e.g. jpg -> jpg)
                    // Usually yes.
                    resized.save(&new_path)?;
                    generated_size += std::fs::metadata(&new_path)?.len();

                    // Save modernized formats for resized
                    let _resized_path_base = path.with_file_name(format!("{}-{}", stem, target_width)); // base for extension replacement
                    // Wait, save_formats takes path with extension effectively or stem?
                    // My helper uses with_extension on the input path.
                    // If I pass 'foo-640.jpg', with_extension("webp") -> 'foo-640.webp'. Correct.
                    
                    let resized_size = std::fs::metadata(&new_path)?.len();
                    let (gen_size, _) = save_formats(&new_path, &resized, resized.width(), resized.height(), resized_size)?;
                    generated_size += gen_size;
                }
            }
        }
    }
    
    {
        let mut s = stats.lock().unwrap();
        s.original_size += original_size;
        s.saved_bytes += saved_here;
        s.generated_bytes += generated_size;
        s.files_count += 1;
        s.file_reports.push(FileReport {
            path: path.to_string_lossy().to_string(),
            original_size,
            saved_bytes: saved_here,
            generated_formats: all_formats,
        });
    }

    Ok(())
}

// fn saved_len(path: &Path) -> Result<usize> {
//    Ok(std::fs::metadata(path)?.len() as usize)
// }
