use image::{Rgba, RgbaImage};
use rusttype::{Font};

use crate::render::RenderBackend;

pub struct RasterBackend<'a> {
    pub img: &'a mut RgbaImage,
    color: Rgba<u8>,
    font: Font<'static>,
}

impl<'a> RasterBackend<'a> {
    pub fn new(img: &'a mut RgbaImage, font: Font<'static>) -> Self {
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
                if px >= 0 && py >= 0 && (px as u32) < self.img.width() && (py as u32) < self.img.height() {
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
        use imageproc::drawing::draw_text_mut;
        use rusttype::Scale;

        draw_text_mut(
            self.img,
            self.color,
            x as i32,
            y as i32,
            Scale::uniform(size as f32),
            &self.font,
            text,
        );
    }
}


