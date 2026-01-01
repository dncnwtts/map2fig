use image::{GrayImage, Luma};
use std::f64::consts::PI;

/// Solve 2θ + sin(2θ) = π sin φ for Mollweide projection
fn mollweide_theta(phi: f64) -> f64 {
    let mut theta = phi; // initial guess
    for _ in 0..10 {
        let delta = (2.0 * theta + (2.0 * theta).sin() - PI * phi) / (2.0 + 2.0 * (2.0 * theta).cos());
        theta -= delta;
        if delta.abs() < 1e-10 {
            break;
        }
    }
    theta
}

/// Mollweide projection: lon/lat in radians -> x/y in [-2√2,2√2] x [-√2,√2]
fn mollweide(lon: f64, lat: f64) -> (f64, f64) {
    let theta = mollweide_theta(lat);
    let x = 2.0 * 2f64.sqrt() / PI * lon * theta.cos();
    let y = 2f64.sqrt() * theta.sin();
    (x, y)
}

pub fn plot_mollweide_oval(width: u32, height: u32, filename: &str) {
    let mut img = GrayImage::from_pixel(width, height, Luma([255u8])); // white background

    for py in 0..height {
        for px in 0..width {
            // Normalize to [-1, 1]
            let nx = 4.0 * (px as f64 / (width - 1) as f64) - 2.0;
            let ny = 2.0 * (py as f64 / (height - 1) as f64) - 1.0;

            // Simple ellipse check
            if nx * nx / 4.0 + ny * ny <= 1.0 {
                img.put_pixel(px, py, Luma([0u8]));
            }
        }
    }

    img.save(filename).expect("Failed to save PNG");
}



fn rescale_linear(val: f64, vmin: f64, vmax: f64) -> u8 {
    if val.is_nan() {
        return 0;
    }
    let t = ((val - vmin) / (vmax - vmin)).clamp(0.0, 1.0);
    (t * 255.0) as u8
}

use crate::healpix::{HPX_UNSEEN, is_seen, ang2pix_ring};

pub fn plot_mollweide(
    map: &[f64],
    nside: i64,
    width: u32,
    height: u32,
    filename: &str,
) {

    let mut minv = f64::INFINITY;
    let mut maxv = f64::NEG_INFINITY;
    
    for &v in map {
        if is_seen(v) {
            minv = minv.min(v);
            maxv = maxv.max(v);
        }
    }

    minv = 0f64;
    maxv = 50f64;

    println!("map min = {}, max = {}", minv, maxv);
    let mut img = GrayImage::from_pixel(width, height, Luma([255u8]));
    let npix = map.len() as f64;

    for py in 0..height {
        for px in 0..width {
            // Mollweide plane coordinates
            let x = 4.0 * (px as f64 / (width - 1) as f64) - 2.0;
            let y = 1.0 - 2.0 * (py as f64 / (height - 1) as f64);


            // Outside Mollweide oval
            if x * x / 4.0 + y * y > 1.0 {
                continue;
            }

            // Inverse Mollweide
            let theta_aux = y.asin(); // θ
            let sin_lat = (2.0 * theta_aux + (2.0 * theta_aux).sin()) / PI;

            // Numerical safety
            if sin_lat.abs() > 1.0 {
                continue;
            }

            let lat = sin_lat.asin();
            let lon = PI * x / (2.0 * theta_aux.cos());

            // Convert to HEALPix angles
            let theta = PI / 2.0 - lat; // colatitude

            if !(0.0..=PI).contains(&theta) {
                continue;
            }

            // HEALPix lookup
            let ipix = ang2pix_ring(nside as i64, theta, lon);
            let val = map[ipix as usize];

            // Simple grayscale
            let intensity = rescale_linear(val, minv, maxv);
            img.put_pixel(px, py, Luma([intensity]));
            // let intensity = ((val / npix) * 255.0).clamp(0.0, 255.0) as u8;
            // img.put_pixel(px, py, Luma([intensity]));
        }
    }

    img.save(filename).expect("Failed to save PNG");
}

