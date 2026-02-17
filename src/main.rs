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
}

struct Stats {
    original_size: u64,
    optimized_size: u64,
    saved_bytes: u64,
    generated_bytes: u64,
    files_count: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Parse paths
    let paths: Vec<&str> = args.paths.split(',').map(|s| s.trim()).collect();
    let ignore_patterns: Vec<&str> = args.ignore.split(',').map(|s| s.trim()).collect();
    
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
        optimized_size: 0,
        saved_bytes: 0,
        generated_bytes: 0,
        files_count: 0,
    }));

    // Collect all image files
    let mut image_files = Vec::new();
    for path in &paths {
        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext = ext.to_lowercase();
                    if ["jpg", "jpeg", "png"].contains(&ext.as_str()) {
                         let path_str = path.to_string_lossy();
                         if !ignore_patterns.iter().any(|pat| path_str.contains(&pat.replace("**", ""))) {
                             image_files.push(path.to_owned());
                         }
                    }
                }
            }
        }
    }

    println!("Found {} images.", image_files.len());

    image_files.par_iter().for_each(|path| {
        if let Err(e) = process_image(path, gen_avif, gen_webp, &responsive_widths, &stats) {
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
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        let ext = ext.to_lowercase();
                        if ext == "html" || ext == "htm" {
                             let path_str = path.to_string_lossy();
                             if !ignore_patterns.iter().any(|pat| path_str.contains(&pat.replace("**", ""))) {
                                 html_files.push(path.to_owned());
                             }
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
        println!("::set-output name=saved-size::{}", humansize::format_size(final_stats.saved_bytes, humansize::DECIMAL));
        
        let report = format!(
            "### ImgZen Optimization Report 🚀\n\n| Metric | Value |\n|---|---|\n| Processed Files | {} |\n| Original Size | {} |\n| **Saved Size** | **{}** |\n| Generated Assets | {} |",
            final_stats.files_count,
            humansize::format_size(final_stats.original_size, humansize::DECIMAL),
            humansize::format_size(final_stats.saved_bytes, humansize::DECIMAL),
            humansize::format_size(final_stats.generated_bytes, humansize::DECIMAL)
        );
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
        Settings {
            element_content_handlers: vec![
                element!("img", |el| {
                    if el.get_attribute("loading").is_none() {
                        el.set_attribute("loading", "lazy")?;
                    }
                    Ok(())
                })
            ],
            ..Settings::default()
        },
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

fn process_image(path: &Path, gen_avif: bool, gen_webp: bool, responsive_widths: &[u32], stats: &Arc<Mutex<Stats>>) -> Result<()> {
    // Load image
    let img = image::open(path).with_context(|| format!("Failed to open image {:?}", path))?;
    let metadata = std::fs::metadata(path)?;
    let original_bytes = std::fs::read(path)?;
    let original_size = metadata.len();

    // Optimize Original if PNG
    let mut saved_here = 0;
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if ext.to_lowercase() == "png" {
             let options = oxipng::Options::from_preset(2);
             if let Ok(optimized_data) = oxipng::optimize_from_memory(&original_bytes, &options) {
                 if optimized_data.len() < original_bytes.len() {
                     std::fs::write(path, &optimized_data)?;
                     saved_here = original_size as u64 - optimized_data.len() as u64;
                 }
             }
        }
    }

    let (width, height) = (img.width(), img.height());
    let _rgb = img.to_rgb8(); // Unused but kept for structure if needed
    let rgba = img.to_rgba8();

    let mut generated_size = 0;

    // Helper to generate formats for a buffer
    let save_formats = |stem_path: &Path, img_rgba: &image::RgbaImage, w: u32, h: u32| -> Result<u64> {
        let mut gen_size = 0;
        if gen_webp {
            let webp_path = stem_path.with_extension("webp");
            let encoder = webp::Encoder::from_rgba(img_rgba, w, h);
            let webp_data = encoder.encode(75.0); 
            std::fs::write(&webp_path, &*webp_data)?;
            gen_size += webp_data.len() as u64;
        }
        
        if gen_avif {
             let avif_path = stem_path.with_extension("avif");
             use rgb::FromSlice;
             let pixels = img_rgba.as_raw().as_rgba();
             let img_buffer = imgref::Img::new(pixels, w as usize, h as usize);
             let encoder = ravif::Encoder::new().with_quality(75.0).with_speed(4);
             let result = encoder.encode_rgba(img_buffer)
                .map_err(|e| anyhow::anyhow!("AVIF encoding failed: {:?}", e))?;
             std::fs::write(&avif_path, &result.avif_file)?;
             gen_size += result.avif_file.len() as u64;
        }
        Ok(gen_size)
    };

    // Generate for original size
    generated_size += save_formats(path, &rgba, width, height)?;

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
                    
                    generated_size += save_formats(&new_path, &resized, resized.width(), resized.height())?;
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
    }

    Ok(())
}

// fn saved_len(path: &Path) -> Result<usize> {
//    Ok(std::fs::metadata(path)?.len() as usize)
// }
