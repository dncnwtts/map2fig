/// LaTeX rendering using system pdflatex with caching
///
/// Uses system pdflatex which is widely available on most Unix-like systems.
/// Renders LaTeX strings to PDF, converts to PNG, and caches results.
/// Falls back to Unicode approximation if LaTeX is unavailable.
use std::io::Write;
use std::path::PathBuf;
use sha2::{Sha256, Digest};
use tempfile::TempDir;

/// Get cache directory for rendered LaTeX images
fn get_cache_dir() -> PathBuf {
    let cache_dir = directories::ProjectDirs::from("", "", "map2fig")
        .map(|dirs| dirs.cache_dir().to_path_buf())
        .unwrap_or_else(|| {
            std::env::home_dir()
                .map(|h| h.join(".cache/map2fig"))
                .unwrap_or_else(|| PathBuf::from(".map2fig_cache"))
        })
        .join("latex");
    
    let _ = std::fs::create_dir_all(&cache_dir);
    cache_dir
}

/// Generate cache key from LaTeX source
fn get_cache_key(latex_str: &str, font_size: u32) -> String {
    let mut hasher = Sha256::new();
    hasher.update(latex_str.as_bytes());
    hasher.update(font_size.to_string().as_bytes());
    format!("{:x}.png", hasher.finalize())
}

/// Check if pdflatex is available on system
fn check_pdflatex() -> bool {
    std::process::Command::new("which")
        .arg("pdflatex")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if pdftoppm (from poppler) is available
fn check_pdftoppm() -> bool {
    std::process::Command::new("which")
        .arg("pdftoppm")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if pdf2svg is available on system (for vector PDF embedding)
fn check_pdf2svg() -> bool {
    std::process::Command::new("which")
        .arg("pdf2svg")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Check if ImageMagick convert is available on system
fn check_convert() -> bool {
    std::process::Command::new("which")
        .arg("convert")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Render LaTeX to high-quality PNG for vector-like appearance
/// Uses higher DPI for near-vector quality without actual vectorization
pub fn render_latex_to_hires_png(latex_str: &str, font_size_pt: u32, dpi: u32) -> Option<RenderedLatex> {
    // Check if LaTeX tools are available
    if !check_pdflatex() || !check_pdftoppm() {
        return None;
    }
    
    // Check cache first using DPI in the key
    let hash_value = {
        let mut hasher = Sha256::new();
        hasher.update(latex_str.as_bytes());
        hasher.update(font_size_pt.to_string().as_bytes());
        hasher.update(dpi.to_string().as_bytes());
        format!("{:x}", hasher.finalize())
    };
    let cache_key = format!("{}_{}.png", hash_value, dpi);
    
    let cache_path = get_cache_dir().join(&cache_key);
    
    if cache_path.exists()
        && let Ok(image_data) = std::fs::read(&cache_path)
            && let Ok((width, height)) = extract_png_dimensions(&image_data) {
                return Some(RenderedLatex { image_data, width, height });
            }
    
    // Create temporary directory for compilation
    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(_) => return None,
    };
    
    let temp_path = temp_dir.path();
    
    // Create minimal LaTeX document with substantial vertical padding
    // Use invisible vertical rule to force bounding box size (standalone class is strict)
    let latex_doc = format!(
        r#"\documentclass[{}pt]{{standalone}}
\usepackage{{amsmath,amssymb,mathtools,xcolor}}
\pagestyle{{empty}}
\begin{{document}}
\rule{{0pt}}{{0.6cm}}\\
{}
\end{{document}}"#,
        font_size_pt, latex_str
    );
    
    // Write LaTeX file
    let tex_path = temp_path.join("document.tex");
    match std::fs::File::create(&tex_path) {
        Ok(mut file) => {
            if file.write_all(latex_doc.as_bytes()).is_err() {
                return None;
            }
        }
        Err(_) => return None,
    }
    
    // Run pdflatex
    let output = std::process::Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg("-no-shell-escape")
        .arg(&tex_path)
        .current_dir(temp_path)
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    // Convert PDF to PNG using pdftoppm at specified DPI
    let pdf_path = temp_path.join("document.pdf");
    let png_path = temp_path.join("document.png");
    
    let dpi_str = dpi.to_string();
    let output = std::process::Command::new("pdftoppm")
        .arg("-singlefile")
        .arg("-png")
        .arg("-r").arg(&dpi_str)  // Custom DPI for vector-like quality
        .arg(&pdf_path)
        .arg(png_path.with_extension(""))
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    // Read PNG
    let image_data = std::fs::read(&png_path).ok()?;
    let (width, height) = extract_png_dimensions(&image_data).ok()?;
    
    // Cache the result
    let _ = std::fs::write(&cache_path, &image_data);
    
    Some(RenderedLatex { image_data, width, height })
}

/// Rendered LaTeX output as PNG
#[derive(Clone, Debug)]
pub struct RenderedLatex {
    /// PNG image bytes
    pub image_data: Vec<u8>,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// Rendered LaTeX output as SVG (vector format)
#[derive(Clone, Debug)]
pub struct RenderedLatexSvg {
    /// SVG file content as string
    pub svg_data: String,
    /// Approximate width in points (from SVG viewBox)
    pub width: f64,
    /// Approximate height in points (from SVG viewBox)
    pub height: f64,
}

/// Render LaTeX string to SVG (vector format) using pdf2svg
///
/// Falls back to using ImageMagick convert if pdf2svg is unavailable.
/// Uses tectonic or system pdflatex for PDF generation.
///
/// # Arguments
/// * `latex_str` - LaTeX source (e.g., "$K_{\\mathrm{CMB}}$")
/// * `font_size_pt` - Font size in points
///
/// # Returns
/// * `Some(RenderedLatexSvg)` - SVG data with dimensions
/// * `None` - If LaTeX and all conversion tools unavailable
pub fn render_latex_to_svg(latex_str: &str, font_size_pt: u32) -> Option<RenderedLatexSvg> {
    // Check if we have at least one conversion tool
    if !check_pdf2svg() && !check_convert() {
        return None;
    }
    
    // Generate PDF using tectonic or system pdflatex
    let pdf_bytes = render_latex_to_pdf(latex_str, font_size_pt)?;
    
    // Create temporary directory for SVG conversion
    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(_) => return None,
    };
    
    let temp_path = temp_dir.path();
    let pdf_path = temp_path.join("document.pdf");
    let svg_path = temp_path.join("document.svg");
    
    // Write PDF to temp file
    match std::fs::write(&pdf_path, &pdf_bytes) {
        Ok(_) => {},
        Err(_) => return None,
    }
    
    // Try pdf2svg first (best quality)
    if check_pdf2svg() {
        let output = std::process::Command::new("pdf2svg")
            .arg(&pdf_path)
            .arg(&svg_path)
            .output()
            .ok()?;
        
        if output.status.success()
            && let Ok(svg_data) = std::fs::read_to_string(&svg_path)
                && let Some((width, height)) = extract_svg_dimensions(&svg_data) {
                    return Some(RenderedLatexSvg { svg_data, width, height });
                }
    }
    
    // Fallback to ImageMagick convert
    if check_convert() {
        let output = std::process::Command::new("convert")
            .arg("-density").arg("150")
            .arg(format!("{}[0]", pdf_path.display())) // Convert PDF page 1
            .arg(&svg_path)
            .output()
            .ok()?;
        
        if output.status.success()
            && let Ok(svg_data) = std::fs::read_to_string(&svg_path)
                && let Some((width, height)) = extract_svg_dimensions(&svg_data) {
                    return Some(RenderedLatexSvg { svg_data, width, height });
                }
    }
    
    None
}

/// Render LaTeX to SVG (vector format) then convert to PNG
/// 
/// This function uses pdf2svg to create vector output, then rasterizes to PNG
/// for embedding in Cairo. The PDF is generated at high quality, so the rasterization
/// preserves vector sharpness better than direct PDFLaTeX→PNG conversion.
///
/// # Arguments
/// * `latex_str` - LaTeX source (e.g., "$K_{\\mathrm{CMB}}$")
/// * `font_size_pt` - Font size in points
///
/// # Returns
/// * `Some(RenderedLatex)` if rendering successful
/// * `None` if pdf2svg or LaTeX unavailable
pub fn render_latex_to_svg_png(latex_str: &str, font_size_pt: u32) -> Option<RenderedLatex> {
    // Try to render to SVG first
    let _rendered_svg = render_latex_to_svg(latex_str, font_size_pt)?;
    
    // Convert SVG to PNG using ImageMagick or similar (requires external tool)
    // For now, we skip this and rely on the existing PDF→PNG pipeline
    // TODO: Implement SVG→PNG conversion using usvg or similar
    
    None
}

/// Extract width and height from SVG viewBox attribute
fn extract_svg_dimensions(svg_data: &str) -> Option<(f64, f64)> {
    // Look for viewBox="x y width height" pattern
    let start = svg_data.find("viewBox=\"")?;
    let after_viewbox = &svg_data[start + 9..];
    let end = after_viewbox.find("\"")?;
    let viewbox_str = &after_viewbox[..end];
    
    let parts: Vec<&str> = viewbox_str.split_whitespace().collect();
    if parts.len() < 4 {
        return None;
    }
    
    let width = parts[2].parse::<f64>().ok()?;
    let height = parts[3].parse::<f64>().ok()?;
    
    Some((width, height))
}

/// Render LaTeX to PDF using Tectonic command-line (preferred pure-Rust implementation)
/// 
/// Tectonic is self-contained and doesn't require system TeX installation.
/// This provides the best quality and avoids platform-specific issues.
///
/// # Arguments
/// * `latex_str` - LaTeX source (e.g., "$K_{\\mathrm{CMB}}$")
/// * `font_size_pt` - Font size in points
///
/// # Returns
/// * `Some(Vec<u8>)` - PDF file bytes
/// * `None` - If tectonic unavailable or compilation fails
pub fn render_latex_to_pdf_tectonic(latex_str: &str, font_size_pt: u32) -> Option<Vec<u8>> {
    // Create temporary directory for compilation
    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(_) => return None,
    };
    
    let temp_path = temp_dir.path();
    let pdf_path = temp_path.join("document.pdf");
    
    // Create minimal LaTeX document with substantial vertical padding
    // Use invisible vertical rule to force bounding box size (standalone class is strict)
    let latex_doc = format!(
        r#"\documentclass[{}pt]{{standalone}}
\usepackage{{amsmath,amssymb,mathtools,xcolor}}
\pagestyle{{empty}}
\begin{{document}}
\rule{{0pt}}{{0.6cm}}\\
{}
\end{{document}}"#,
        font_size_pt, latex_str
    );
    
    // Write LaTeX file
    let tex_path = temp_path.join("document.tex");
    match std::fs::File::create(&tex_path) {
        Ok(mut file) => {
            if file.write_all(latex_doc.as_bytes()).is_err() {
                return None;
            }
        }
        Err(_) => return None,
    }
    
    // Use Tectonic CLI to compile (simpler invocation without -X)
    let output = std::process::Command::new("tectonic")
        .arg(&tex_path)
        .current_dir(temp_path)
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    // Read the generated PDF
    std::fs::read(&pdf_path).ok()
}

/// Render LaTeX to PDF bytes (vector format)
///
/// Uses system pdflatex for PDF generation. pdflatex is widely available on Unix-like systems
/// and doesn't require a Rust build dependency.
///
/// # Arguments
/// * `latex_str` - LaTeX source (e.g., "$K_{\\mathrm{CMB}}$")
/// * `font_size_pt` - Font size in points
///
/// # Returns
/// * `Some(Vec<u8>)` - PDF file bytes
/// * `None` - If pdflatex is not available
pub fn render_latex_to_pdf(latex_str: &str, font_size_pt: u32) -> Option<Vec<u8>> {
    // Use system pdflatex directly
    if !check_pdflatex() {
        return None;
    }
    
    // Create temporary directory for compilation
    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(_) => return None,
    };
    
    let temp_path = temp_dir.path();
    
    // Create minimal LaTeX document with substantial vertical padding
    // Use invisible vertical rule to force bounding box size (standalone class is strict)
    let latex_doc = format!(
        r#"\documentclass[{}pt]{{standalone}}
\usepackage{{amsmath,amssymb,mathtools,xcolor}}
\pagestyle{{empty}}
\begin{{document}}
\rule{{0pt}}{{0.6cm}}\\
{}
\end{{document}}"#,
        font_size_pt, latex_str
    );
    
    // Write LaTeX file
    let tex_path = temp_path.join("document.tex");
    match std::fs::File::create(&tex_path) {
        Ok(mut file) => {
            if file.write_all(latex_doc.as_bytes()).is_err() {
                return None;
            }
        }
        Err(_) => return None,
    }
    
    // Run pdflatex
    let output = std::process::Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg("-no-shell-escape")
        .arg(&tex_path)
        .current_dir(temp_path)
        .output()
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    // Read the generated PDF
    let pdf_path = temp_path.join("document.pdf");
    std::fs::read(&pdf_path).ok()
}

/// Render LaTeX string to PNG image
/// 
/// Uses system pdflatex if available, caches results.
/// If LaTeX unavailable, returns None to fall back to Unicode.
///
/// # Arguments
/// * `latex_str` - LaTeX source (e.g., "$K_{\\mathrm{CMB}}$")
/// * `font_size_pt` - Font size in points
///
/// # Returns
/// * `Some(RenderedLatex)` if rendering successful
/// * `None` if LaTeX unavailable on system
pub fn render_latex_to_png(latex_str: &str, font_size_pt: u32) -> Option<RenderedLatex> {
    // Check if LaTeX tools are available
    if !check_pdflatex() || !check_pdftoppm() {
        return None;
    }
    
    // Check cache first
    let cache_key = get_cache_key(latex_str, font_size_pt);
    let cache_path = get_cache_dir().join(&cache_key);
    
    if cache_path.exists()
        && let Ok(image_data) = std::fs::read(&cache_path)
            && let Ok((width, height)) = extract_png_dimensions(&image_data) {
                return Some(RenderedLatex { image_data, width, height });
            }
    
    // Create temporary directory for compilation
    let temp_dir = match TempDir::new() {
        Ok(dir) => dir,
        Err(_) => return None,
    };
    
    let temp_path = temp_dir.path();
    
    // Create minimal LaTeX document with substantial vertical padding
    // Use invisible vertical rule to force bounding box size (standalone class is strict)
    let latex_doc = format!(
        r#"\documentclass[{}pt]{{standalone}}
\usepackage{{amsmath,amssymb,mathtools,xcolor}}
\pagestyle{{empty}}
\begin{{document}}
\rule{{0pt}}{{0.6cm}}\\
{}
\end{{document}}"#,
        font_size_pt, latex_str
    );
    
    // Write LaTeX file
    let tex_path = temp_path.join("document.tex");
    match std::fs::File::create(&tex_path) {
        Ok(mut file) => {
            if file.write_all(latex_doc.as_bytes()).is_err() {
                return None;
            }
        }
        Err(_) => return None,
    }
    
    // Run pdflatex
    let output = std::process::Command::new("pdflatex")
        .arg("-interaction=nonstopmode")
        .arg("-no-shell-escape")
        .arg(&tex_path)
        .current_dir(temp_path)
        .output()
        .ok()?;
    
    if !output.status.success() {
        eprintln!("pdflatex compilation failed for: {}", latex_str);
        return None;
    }
    
    // Convert PDF to PNG using pdftoppm
    let pdf_path = temp_path.join("document.pdf");
    let png_path = temp_path.join("document.png");
    
    let output = std::process::Command::new("pdftoppm")
        .arg("-singlefile")
        .arg("-png")
        .arg("-r").arg("150")  // 150 DPI for good quality
        .arg(&pdf_path)
        .arg(png_path.with_extension(""))
        .output()
        .ok()?;
    
    if !output.status.success() {
        eprintln!("pdftoppm failed for: {}", latex_str);
        return None;
    }
    
    // Read PNG
    let image_data = std::fs::read(&png_path).ok()?;
    let (width, height) = extract_png_dimensions(&image_data).ok()?;
    
    // Cache the result
    let _ = std::fs::write(&cache_path, &image_data);
    
    Some(RenderedLatex { image_data, width, height })
}

/// Extract PNG dimensions from IHDR chunk
fn extract_png_dimensions(png_data: &[u8]) -> Result<(u32, u32), String> {
    if png_data.len() < 24 {
        return Err("PNG too short".to_string());
    }
    
    if &png_data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("Invalid PNG signature".to_string());
    }
    
    let width = u32::from_be_bytes([
        png_data[16], png_data[17], png_data[18], png_data[19]
    ]);
    let height = u32::from_be_bytes([
        png_data[20], png_data[21], png_data[22], png_data[23]
    ]);
    
    Ok((width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_cache_key_uniqueness() {
        let key1 = get_cache_key("$x^2$", 12);
        let key2 = get_cache_key("$x^2$", 12);
        assert_eq!(key1, key2);
        
        let key3 = get_cache_key("$x^3$", 12);
        assert_ne!(key1, key3);
        
        let key4 = get_cache_key("$x^2$", 14);
        assert_ne!(key1, key4);
    }
    
    #[test]
    fn test_png_dimension_parsing() {
        // Minimal 1x1 PNG
        let png = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
            0x00, 0x00, 0x00, 0x02,  // width: 2
            0x00, 0x00, 0x00, 0x03,  // height: 3
            0x08, 0x02, 0x00, 0x00, 0x00, 0x00,
        ];
        
        let (w, h) = extract_png_dimensions(&png).unwrap();
        assert_eq!(w, 2);
        assert_eq!(h, 3);
    }
    
    #[test]
    fn test_svg_rendering() {
        // Skip if pdflatex or conversion tools not available
        if !check_pdflatex() || (!check_pdf2svg() && !check_convert()) {
            eprintln!("Skipping SVG test: pdflatex or conversion tools not available");
            return;
        }
        
        // Test simple LaTeX
        if let Some(svg) = render_latex_to_svg("$K$", 10) {
            assert!(svg.width > 0.0, "SVG width should be > 0");
            assert!(svg.height > 0.0, "SVG height should be > 0");
            assert!(svg.svg_data.len() > 0, "SVG data should not be empty");
            eprintln!("SVG render test: {}x{} with {} bytes", svg.width, svg.height, svg.svg_data.len());
        } else {
            eprintln!("SVG rendering returned None");
        }
    }
    
    #[test]
    fn test_svg_dimension_extraction() {
        let svg_data = r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 50">
  <text>Test</text>
</svg>"#;
        
        if let Some((w, h)) = extract_svg_dimensions(svg_data) {
            assert_eq!(w, 100.0);
            assert_eq!(h, 50.0);
        } else {
            panic!("Failed to extract SVG dimensions");
        }
    }
}
