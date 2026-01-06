use image::{GrayImage, RgbImage, Luma, Rgb};
use std::f64::consts::PI;


const FONT_W: u32 = 6;
const FONT_H: u32 = 8;

// Digits 0–9, '.' , '-', 'e'
static FONT: &[(&str, [u8; 8])] = &[
    ("0", [0x3E,0x51,0x49,0x45,0x3E,0,0,0]),
    ("1", [0x00,0x42,0x7F,0x40,0x00,0,0,0]),
    ("2", [0x42,0x61,0x51,0x49,0x46,0,0,0]),
    ("3", [0x21,0x41,0x45,0x4B,0x31,0,0,0]),
    ("4", [0x18,0x14,0x12,0x7F,0x10,0,0,0]),
    ("5", [0x27,0x45,0x45,0x45,0x39,0,0,0]),
    ("6", [0x3C,0x4A,0x49,0x49,0x30,0,0,0]),
    ("7", [0x01,0x71,0x09,0x05,0x03,0,0,0]),
    ("8", [0x36,0x49,0x49,0x49,0x36,0,0,0]),
    ("9", [0x06,0x49,0x49,0x29,0x1E,0,0,0]),
    (".", [0x00,0x40,0x60,0x00,0x00,0,0,0]),
    ("-", [0x08,0x08,0x08,0x08,0x08,0,0,0]),
    ("e", [0x38,0x54,0x54,0x54,0x18,0,0,0]),
];

fn glyph(c: char) -> Option<[u8; 8]> {
    FONT.iter().find(|(k, _)| k.chars().next().unwrap() == c).map(|(_, g)| *g)
}

fn draw_text(
    img: &mut GrayImage,
    x0: u32,
    y0: u32,
    text: &str,
) {
    let mut x = x0;
    for c in text.chars() {
        if let Some(g) = glyph(c) {
            for (row, bits) in g.iter().enumerate() {
                for col in 0..5 {
                    if bits & (1 << (4 - col)) != 0 {
                        let px = x + col;
                        let py = y0 + row as u32;
                        if px < img.width() && py < img.height() {
                            img.put_pixel(px, py, Luma([0u8]));
                        }
                    }
                }
            }
            x += FONT_W;
        }
    }
}

fn format_value(v: f64) -> String {
    if v.abs() >= 100.0 || v.abs() <= 0.01 {
        format!("{:.3e}", v)
    } else {
        format!("{:.4}", v)
    }
}


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

fn percentile(sorted: &[f64], p: f64) -> f64 {
    assert!((0.0..=100.0).contains(&p));
    let n = sorted.len();
    let rank = p / 100.0 * (n - 1) as f64;
    let i = rank.floor() as usize;
    let frac = rank - i as f64;

    if i + 1 < n {
        sorted[i] * (1.0 - frac) + sorted[i + 1] * frac
    } else {
        sorted[i]
    }
}


use crate::healpix::{HPX_UNSEEN, is_seen, ang2pix_ring, nside_from_npix};

pub fn plot_mollweide(
    map: &[f64],
    width: u32,
    filename: &str,
    minv: Option<f64>,
    maxv: Option<f64>,
) {

    let map_height = width / 2;
    let colorbar_height = map_height / 20;
    let height = map_height + colorbar_height;
    let npix = map.len();
    let nside = nside_from_npix(npix)
        .expect("Input map is not a valid full-sky HEALPix map");


    let mut values: Vec<f64> = map
        .iter()
        .filter(|&v| is_seen(*v))
        .copied()
        .collect();

    
    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let data_min = values.first().copied().unwrap();
    let data_max = values.last().copied().unwrap();



    let (minv, maxv) = match (minv, maxv) {
        (Some(lo), Some(hi)) => (lo, hi),
        _ => {
            let lo = percentile(&values, 5.0);
            let hi = percentile(&values, 95.0);
            (lo, hi)
        }
    };
    
    if minv >= maxv {
        panic!("Invalid color scale: {minv} >= {maxv}");
    }

    println!("map min = {}, max = {}", minv, maxv);
    let mut img = RgbImage::from_pixel(width, height, Rgb([255, 255, 255]));

    let npix = map.len() as f64;

    for py in 0..map_height {
        for px in 0..width {
            // Mollweide plane coordinates
            let x = 2.0 - 4.0 * (px as f64 / (width - 1) as f64);
            let y = 1.0 - 2.0 * (py as f64 / (map_height - 1) as f64);


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


            let t = ((val - minv) / (maxv - minv)).clamp(0.0, 1.0);
            let color = colormap_gray(t);
            img.put_pixel(px, py, color);
        }
    }
    for py in map_height..height {
        for px in 0..width {
            let t = px as f64 / (width - 1) as f64;
            let color = colormap_gray(t);
            img.put_pixel(px, py, color);
        }
    }

    // ---------------- Colorbar tick marks ----------------
    // ---------------- Tick scaling ----------------
    let nticks = 5;        // major ticks
    let nminor = 5; // minor ticks per major interval
    
    // Scale tick heights relative to colorbar
    let major_tick_height = (colorbar_height as f64 * 0.5).round() as u32;
    let minor_tick_height = (colorbar_height as f64 * 0.3).round() as u32;
    
    let major_tick_height = major_tick_height.max(1);
    let minor_tick_height = minor_tick_height.max(1);
    
    // Scale tick widths relative to image width
    let major_tick_width = ((width as f64) * 0.002).round() as u32; // ~0.2% of width
    let minor_tick_width = ((width as f64) * 0.001).round() as u32; // ~0.1% of width
    
    let major_tick_width = major_tick_width.max(1);
    let minor_tick_width = minor_tick_width.max(1);
    
    let tick_bottom = height - 1;
    
    // Major ticks
    for i in 0..nticks {
        let t = i as f64 / (nticks - 1) as f64;
        let px = (t * (width - 1) as f64).round() as u32;
    
        let tick_top = tick_bottom - major_tick_height;
        for dx in 0..major_tick_width {
            for py in tick_top..=tick_bottom {
                let x = px.saturating_add(dx);
                if x < width {
                    img.put_pixel(x, py, Rgb([0, 0, 0]));
                }
            }
        }
    
        // Minor ticks between this and next major tick
        if i + 1 < nticks {
            let t0 = i as f64 / (nticks - 1) as f64;
            let t1 = (i + 1) as f64 / (nticks - 1) as f64;
    
            for j in 1..nminor {
                let tm = t0 + (t1 - t0) * (j as f64 / nminor as f64);
                let pxm = (tm * (width - 1) as f64).round() as u32;
    
                let tick_top = tick_bottom - minor_tick_height;
                for dx in 0..minor_tick_width {
                    for py in tick_top..=tick_bottom {
                        let x = pxm.saturating_add(dx);
                        if x < width {
                            img.put_pixel(x, py, Rgb([0, 0, 0]));
                        }
                    }
                }
            }
        }
    }

    let border_width_px = (width as f64 * 0.004).max(2.0);

    println!("Border width is {border_width_px}");

    draw_projection_border(
        &mut img,
        map_height,
        Rgb([0, 0, 0]),
        border_width_px,
        |u, v| (u * u) / 4.0 + v * v,
    );




    img.save(filename).expect("Failed to save PNG");
}


pub fn draw_colorbar(
    height: u32,
    width: u32,
    vmin: f64,
    vmax: f64,
    filename: &str,
) {
    let mut img = GrayImage::from_pixel(width, height, Luma([255u8]));

    for y in 0..height {
        // Normalize: top = vmax, bottom = vmin
        let t = 1.0 - (y as f64 / (height - 1) as f64);
        let v = vmin + t * (vmax - vmin);

        // Map to grayscale (replace later with colormap)
        let intensity = ((v - vmin) / (vmax - vmin) * 255.0)
            .clamp(0.0, 255.0) as u8;

        for x in 0..width {
            img.put_pixel(x, y, Luma([intensity]));
        }
    }

    img.save(filename).expect("Failed to save colorbar");
}

pub fn draw_projection_border<F>(
    img: &mut RgbImage,
    map_height: u32,
    border_color: Rgb<u8>,
    line_width_px: f64,
    dist_fn: F,
)
where
    F: Fn(f64, f64) -> f64,
{
    let nx = img.width() as f64;
    let ny = map_height as f64;

    let xc = (nx - 1.0) / 2.0;
    let yc = (ny - 1.0) / 2.0;

    // Normalized pixel size
    let delta = 2.0 / nx;

    // Inflate band for perceptual correctness
    let band = line_width_px * delta * 2.5;

    for py in 0..map_height {
        for px in 0..img.width() {
            let u = 2.0 * (px as f64 - xc) / xc;
            let v = -(py as f64 - yc) / yc;

            let d = dist_fn(u, v);

            if d >= 1.0 - band && d <= 1.0 {
                img.put_pixel(px, py, border_color);
            }
        }
    }
}




fn colormap_viridis(t: f64) -> Rgb<u8> {
    // Clamp t to [0,1]
    let t = t.clamp(0.0, 1.0);

    // Viridis-like approximation
    // You could replace this with exact values or use a crate like 'palette'
    let r = (0.280 * (1.0 - t) + 0.993 * t) * 255.0;
    let g = (0.0   * (1.0 - t) + 0.906 * t) * 255.0;
    let b = (0.509 * (1.0 - t) + 0.143 * t) * 255.0;

    Rgb([r as u8, g as u8, b as u8])
}

fn colormap_gray(t: f64) -> Rgb<u8> {
    let v = (t.clamp(0.0, 1.0) * 255.0) as u8;
    Rgb([v, v, v])
}

