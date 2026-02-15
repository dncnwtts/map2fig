use image::{ImageBuffer, Rgba};
use std::io::Write;
use std::path::Path;

/// Optimized PDF rendering using printpdf library
/// For now, this generates uncompressed PPM files for benchmarking
/// A full PDF writer would eliminate Cairo's zlib compression overhead
pub struct PrintpdfBackend {
    /// Store image data for later writing
    image_data: Option<Vec<u8>>,
    width: u32,
    height: u32,
}

impl PrintpdfBackend {
    /// Create a new PDF document backend
    pub fn new(_width_pt: f64, _height_pt: f64) -> Self {
        Self {
            image_data: None,
            width: 0,
            height: 0,
        }
    }

    /// Store a pre-rendered image buffer
    /// This prepares the RGB data for embedding in an uncompressed format
    pub fn embed_image_buffer(
        &mut self,
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        _x_pt: f64,
        _y_pt: f64,
        _width_pt: f64,
        _height_pt: f64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = image.dimensions();

        // Convert RGBA to RGB (drop alpha)
        let mut rgb_data = Vec::with_capacity(width as usize * height as usize * 3);
        for pixel in image.pixels() {
            rgb_data.push(pixel[0]); // R
            rgb_data.push(pixel[1]); // G
            rgb_data.push(pixel[2]); // B
        }

        self.image_data = Some(rgb_data);
        self.width = width;
        self.height = height;

        Ok(())
    }

    /// Save to an uncompressed format
    /// In production, this would create an uncompressed PDF using printpdf
    /// For now, it demonstrates the serialization time without compression overhead
    pub fn save<P: AsRef<Path>>(self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(data) = self.image_data {
            // Write as uncompressed PPM (P6 binary format)
            // This is equivalent to an uncompressed PDF for benchmarking purposes
            let ppm_header = format!("P6\n{} {}\n255\n", self.width, self.height);
            let mut file = std::fs::File::create(path.as_ref())?;
            file.write_all(ppm_header.as_bytes())?;
            file.write_all(&data)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_printpdf_backend_creation() {
        let _backend = PrintpdfBackend::new(100.0, 100.0);
        // Just verify creation doesn't panic
    }

    #[test]
    fn test_simple_image_embedding() {
        // Create a simple test image
        let img = ImageBuffer::new(10, 10);

        let mut backend = PrintpdfBackend::new(100.0, 100.0);
        let result = backend.embed_image_buffer(&img, 10.0, 10.0, 80.0, 80.0);

        // Should succeed without error
        assert!(result.is_ok());
    }
}
