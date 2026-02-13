use crate::render::target::{PixelSource, RenderTarget};

pub struct PngRenderTarget<'a> {
    pub img: &'a mut image::RgbaImage,
}

impl RenderTarget for PngRenderTarget<'_> {
    fn blit_raster(&mut self, raster: &dyn PixelSource, x: f64, y: f64) {
        let x0 = x as i32;
        let y0 = y as i32;

        for py in 0..raster.height() {
            for px in 0..raster.width() {
                let [r, g, b, a] = raster.get_pixel(px, py);
                self.img.put_pixel(
                    (x0 + px as i32) as u32,
                    (y0 + py as i32) as u32,
                    image::Rgba([r, g, b, a]),
                );
            }
        }
    }
}
