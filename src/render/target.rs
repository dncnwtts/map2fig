pub trait RenderTarget {
    /// Draw a fully-rendered raster image at absolute coordinates
    fn blit_raster(
        &mut self,
        raster: &dyn PixelSource,
        x: f64,
        y: f64,
    );
}

pub trait PixelSource {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn get_pixel(&self, x: u32, y: u32) -> [u8; 4];
}
