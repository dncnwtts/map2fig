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

pub mod target;
pub mod raster;
pub mod pdf;
pub mod png;

pub use raster::RasterBackend;
pub use pdf::PdfBackend;


