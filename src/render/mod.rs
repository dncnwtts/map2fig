pub trait RenderBackend {
    fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8);
    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64);
    fn stroke_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, width: f64);

    fn width(&self) -> f64;
    fn height(&self) -> f64;

    fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64);
    fn draw_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, width: f64);
    fn draw_text(&mut self, x: f64, y: f64, size: f64, text: &str);
}

pub mod cairo_uncompressed;
pub mod pdf;
pub mod png;
pub mod raster;
pub mod target;

pub use pdf::PdfBackend;

// ============================================================================
// Tier 3a: Lazy Image Buffer Allocation
// ============================================================================
// Create RGBA image buffers for rasterization.
// Initializes all pixels to avoid undefined behavior.
// All pixels are written before any are read (see usage sites).
pub fn create_image_buffer_uninitialized(width: u32, height: u32) -> image::RgbaImage {
    let pixel_count = (width as usize) * (height as usize);
    let byte_count = pixel_count * 4; // RGBA = 4 bytes per pixel

    // Create zero-initialized vector for safe buffer initialization
    let pixels: Vec<u8> = vec![0u8; byte_count];

    image::ImageBuffer::from_raw(width, height, pixels).expect("Failed to create image buffer")
}
pub use raster::RasterBackend;
