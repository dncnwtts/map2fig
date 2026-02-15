use ab_glyph::FontRef;
use image::{Rgba, RgbaImage};

use crate::render::RenderBackend;
use crate::render::target::PixelSource;

pub struct RasterBackend<'a> {
    pub img: &'a mut RgbaImage,
    color: Rgba<u8>,
    font: FontRef<'a>,
}

impl<'a> RasterBackend<'a> {
    pub fn new(img: &'a mut RgbaImage, font: FontRef<'a>) -> Self {
        Self {
            img,
            color: Rgba([0, 0, 0, 255]),
            font,
        }
    }
}

impl<'a> RenderBackend for RasterBackend<'a> {
    fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.color = Rgba([r, g, b, a]);
    }

    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let x0 = x.round() as i32;
        let y0 = y.round() as i32;
        let x1 = (x + w).round() as i32;
        let y1 = (y + h).round() as i32;

        for py in y0.max(0)..y1.min(self.img.height() as i32) {
            for px in x0.max(0)..x1.min(self.img.width() as i32) {
                self.img.put_pixel(px as u32, py as u32, self.color);
            }
        }
    }

    fn stroke_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, _width: f64) {
        imageproc::drawing::draw_antialiased_line_segment_mut(
            self.img,
            (x0 as i32, y0 as i32),
            (x1 as i32, y1 as i32),
            self.color,
            |p, c, alpha| {
                Rgba([
                    ((1.0 - alpha) * p[0] as f32 + alpha * c[0] as f32) as u8,
                    ((1.0 - alpha) * p[1] as f32 + alpha * c[1] as f32) as u8,
                    ((1.0 - alpha) * p[2] as f32 + alpha * c[2] as f32) as u8,
                    255,
                ])
            },
        );
    }

    fn width(&self) -> f64 {
        self.img.width() as f64
    }

    fn height(&self) -> f64 {
        self.img.height() as f64
    }

    fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        let x0 = x as i32;
        let y0 = y as i32;
        let x1 = (x + w) as i32;
        let y1 = (y + h) as i32;

        for py in y0..y1 {
            for px in x0..x1 {
                if px >= 0
                    && py >= 0
                    && (px as u32) < self.img.width()
                    && (py as u32) < self.img.height()
                {
                    self.img.put_pixel(px as u32, py as u32, self.color);
                }
            }
        }
    }

    fn draw_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, _width: f64) {
        imageproc::drawing::draw_antialiased_line_segment_mut(
            self.img,
            (x0 as i32, y0 as i32),
            (x1 as i32, y1 as i32),
            self.color,
            |p, c, alpha| {
                // alpha-blended pixel
                Rgba([
                    ((1.0 - alpha) * p[0] as f32 + alpha * c[0] as f32) as u8,
                    ((1.0 - alpha) * p[1] as f32 + alpha * c[1] as f32) as u8,
                    ((1.0 - alpha) * p[2] as f32 + alpha * c[2] as f32) as u8,
                    255,
                ])
            },
        );
    }

    fn draw_text(&mut self, x: f64, y: f64, size: f64, text: &str) {
        use ab_glyph::PxScale;
        use imageproc::drawing::draw_text_mut;

        draw_text_mut(
            self.img,
            self.color,
            x as i32,
            y as i32,
            PxScale::from(size as f32),
            &self.font,
            text,
        );
    }
}

#[derive(Debug, Clone)]
pub struct RasterGrid {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<Rgba<u8>>,
    pub valid: Vec<bool>,
}

impl RasterGrid {
    /// Set a pixel from an array
    pub fn set_pixel_array(&mut self, x: u32, y: u32, rgba: [u8; 4]) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.buffer[idx] = Rgba(rgba);
            self.valid[idx] = true;
        }
    }

    /// Set a pixel from `Rgba<u8>`
    pub fn set_pixel(&mut self, x: u32, y: u32, color: Rgba<u8>) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            self.buffer[idx] = color;
            self.valid[idx] = true;
        }
    }

    pub fn set_valid(&mut self, x: u32, y: u32, validity: bool) {
        let idx = (y * self.width + x) as usize;
        self.valid[idx] = validity;
    }

    /// Set a pixel from `Rgba<u8>` without bounds checking
    ///
    /// # Safety
    ///
    /// The caller must ensure that `x < self.width` and `y < self.height`, otherwise
    /// this will access memory out of bounds.
    #[inline]
    pub unsafe fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: Rgba<u8>) {
        let idx = (y * self.width + x) as usize;
        unsafe {
            *self.buffer.get_unchecked_mut(idx) = color;
            *self.valid.get_unchecked_mut(idx) = true;
        }
    }

    /// Set validity without bounds checking
    ///
    /// # Safety
    ///
    /// The caller must ensure that `x < self.width` and `y < self.height`, otherwise
    /// this will access memory out of bounds.
    #[inline]
    pub unsafe fn set_valid_unchecked(&mut self, x: u32, y: u32, validity: bool) {
        let idx = (y * self.width + x) as usize;
        unsafe {
            *self.valid.get_unchecked_mut(idx) = validity;
        }
    }

    #[inline]
    pub fn is_valid(&self, x: u32, y: u32) -> bool {
        let idx = (y * self.width + x) as usize;
        self.valid[idx]
    }

    /// Get pixel and validity in one operation
    #[inline]
    pub fn get_pixel_if_valid(&self, x: u32, y: u32) -> Option<Rgba<u8>> {
        let idx = (y * self.width + x) as usize;
        if self.valid[idx] {
            Some(self.buffer[idx])
        } else {
            None
        }
    }

    /// Get as `Rgba<u8>`
    pub fn get_pixel(&self, x: u32, y: u32) -> Rgba<u8> {
        if x < self.width && y < self.height {
            self.buffer[(y * self.width + x) as usize]
        } else {
            Rgba([0, 0, 0, 0])
        }
    }

    /// Get as `[u8;4]` for PixelSource
    pub fn get_pixel_array(&self, x: u32, y: u32) -> [u8; 4] {
        let Rgba(p) = self.get_pixel(x, y);
        p
    }
    pub fn new(width: u32, height: u32) -> Self {
        let len = (width * height) as usize;
        Self {
            width,
            height,
            buffer: vec![Rgba([0, 0, 0, 0]); len],
            valid: vec![false; len],
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (u32, u32, f64, f64)> + use<> {
        let w = self.width;
        let h = self.height;

        (0..h).flat_map(move |py| {
            (0..w).map(move |px| {
                let u = px as f64 / (w - 1) as f64;
                let v = py as f64 / (h - 1) as f64;
                (px, py, u, v)
            })
        })
    }
    #[inline]
    pub fn norm_x(&self, x: u32) -> f64 {
        x as f64 / (self.width - 1) as f64
    }

    #[inline]
    pub fn norm_y(&self, y: u32) -> f64 {
        y as f64 / (self.height - 1) as f64
    }
}

impl PixelSource for RasterGrid {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
        self.get_pixel_array(x, y)
    }
}
