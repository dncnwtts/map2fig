use image::{GrayImage, Luma};
use std::f64::consts::PI;

/// Simple Mollweide projection: lon/lat in radians -> x/y in [-1,1]
fn mollweide(lon: f64, lat: f64) -> (f64, f64) {
    let theta = lat;
    let x = 2.0 * (2.0f64).sqrt() / PI * lon * theta.cos();
    let y = (2.0f64).sqrt() * theta.sin();
    (x, y)
}

/// Render a HEALPix map into a grayscale PNG image.
pub fn plot_mollweide(map: &[f64], nside: usize, filename: &str) {
    // Image dimensions: width = 2*height
    let width = 1024;
    let height = width / 2;
    let mut img = GrayImage::from_pixel(width, height, Luma([255u8])); // white background

    let npix = map.len();
    let scale = width as f64 / (2.0 * PI);

    for ipix in 0..npix {
        // Convert pixel number to lon/lat
        let lon = 2.0 * PI * (ipix as f64 / npix as f64) - PI;
        let lat = PI * ((ipix as f64 / npix as f64) - 0.5); // rough approximation

        let (x, y) = mollweide(lon, lat);

        // Convert to pixel coordinates
        let px = ((x + 2.0_f64.sqrt()) / (4.0_f64.sqrt()) * width as f64) as u32;
        let py = ((1.0 - (y + 1.0_f64.sqrt()) / (2.0_f64.sqrt())) * height as f64) as u32;

        if px < width && py < height {
            // Map value to 0..255
            let val = map[ipix];
            let intensity = ((val + 10.0) / 20.0 * 255.0)
                .clamp(0.0, 255.0) as u8; // example scaling

            img.put_pixel(px, py, Luma([intensity]));
        }
    }

    img.save(filename).expect("Failed to save PNG");
}

