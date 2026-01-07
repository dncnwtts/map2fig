use image::{GrayImage, RgbaImage, Luma, Rgba};
use std::f64::consts::PI;
use crate::colormap::{Colormap};

#[derive(Clone, Copy)]
pub enum Scale {
    Linear,
    Log,
    Asinh { scale: f64 },
    Symlog { linthresh: f64 },
    PlanckLog { linthresh: f64 },
}

#[derive(Clone, Copy)]
pub enum NegMode {
    Zero,
    Unseen,
}

#[derive(Clone, Copy, Debug)]
pub enum PixelValue {
    Color(f64),
    Bad,
}



pub fn scale_value(
    value: f64,
    min: f64,
    max: f64,
    scale: Scale,
    neg_mode: NegMode,
    gamma: f64,          // <-- apply gamma
) -> PixelValue {
    if min >= max {
        panic!("min must be < max");
    }

    let mut t: f64 = match scale {
        Scale::Linear => {
            if value < min {
                return match neg_mode {
                    NegMode::Zero => PixelValue::Color(0.0),
                    NegMode::Unseen => PixelValue::Bad,
                };
            } else if value > max {
                1.0
            } else {
                (value - min) / (max - min)
            }
        }

        Scale::Log => {
            if value <= 0.0 || value < min {
                return match neg_mode {
                    NegMode::Zero => PixelValue::Color(0.0),
                    NegMode::Unseen => PixelValue::Bad,
                };
            } else if value > max {
                1.0
            } else {
                (value.ln() - min.ln()) / (max.ln() - min.ln())
            }
        }

        Scale::Asinh { scale } => {
            let val = (value / scale).asinh();
            let min_val = (min / scale).asinh();
            let max_val = (max / scale).asinh();

            if val < min_val {
                return match neg_mode {
                    NegMode::Zero => PixelValue::Color(0.0),
                    NegMode::Unseen => PixelValue::Bad,
                };
            } else if val > max_val {
                1.0
            } else {
                (val - min_val) / (max_val - min_val)
            }
        }

        Scale::Symlog { linthresh } => {
            let abs_val = value.abs();
            let scaled = if abs_val < linthresh {
                0.5 + 0.5 * (value / linthresh)
            } else {
                0.5 + 0.5 * value.signum()
                    * (linthresh + (abs_val - linthresh).ln())
                    / (linthresh + (max.abs() - linthresh).ln())
            };

            if value < min {
                return match neg_mode {
                    NegMode::Zero => PixelValue::Color(0.0),
                    NegMode::Unseen => PixelValue::Bad,
                };
            } else if value > max {
                1.0
            } else {
                scaled
            }
        }

        Scale::PlanckLog { linthresh } => {
            if value < min {
                return match neg_mode {
                    NegMode::Zero => PixelValue::Color(0.0),
                    NegMode::Unseen => PixelValue::Bad,
                };
            } else if value > max {
                1.0
            } else {
                if value.abs() < linthresh {
                    0.5 + 0.5 * (value / linthresh)
                } else {
                    0.5 + 0.5 * value.signum()
                        * (linthresh + (value.abs() - linthresh).ln())
                        / (linthresh + (max - linthresh).ln())
                }
            }
        }
    };

    // Apply gamma correction (only if t is in [0,1])
    if t < 0.0 { t = 0.0; }
    if t > 1.0 { t = 1.0; }
    t = t.powf(1.0 / gamma);

    PixelValue::Color(t)
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


use crate::healpix::{is_seen, ang2pix_ring, nside_from_npix};

pub fn plot_mollweide(
    map: &[f64],
    width: u32,
    filename: &str,
    minv: Option<f64>,
    maxv: Option<f64>,
    cmap: &Colormap,
    show_colorbar: bool,
    transparent: bool,
    draw_border: bool,
    gamma: f64,
    scale: Scale,
    neg_mode: NegMode,
    bad_color: Rgba<u8>,
) {

    let map_height = width / 2;
    let colorbar_height = map_height / 20;
    let height = if show_colorbar {
        map_height + colorbar_height
    }
    else {
        map_height
    };
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
    /*
    let data_min = values.first().copied().unwrap();
    let data_max = values.last().copied().unwrap();
    */



    let (minv, maxv) = match (minv, maxv) {
        (Some(lo), Some(hi)) => (lo, hi),
        _ => {
            let lo = percentile(&values, 5.0);
            let hi = percentile(&values, 95.0);
            (lo, hi)
        }
    };

    if gamma <= 0.0 {
        panic!("Gamma must be > 0");
    }
    
    if minv > maxv {
        panic!("Invalid color scale: {minv} > {maxv}");
    }


    println!("map min = {}, max = {}", minv, maxv);
    let bg = if transparent {
        Rgba([0, 0, 0, 0])   // fully transparent
    } else {
        Rgba([255, 255, 255, 255])
    };
    
    let mut img = RgbaImage::from_pixel(width, height, bg);


    let _npix = map.len() as f64;

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

            let px_color = match scale_value(val, minv, maxv, scale, neg_mode, gamma) {
                PixelValue::Color(t) => {
                    let c = cmap.sample(t);
                    Rgba([c[0], c[1], c[2], 255])
                }
                PixelValue::Bad => {
                    bad_color
                }
            };
            
            img.put_pixel(px, py, px_color);
        }
    }
    if show_colorbar {
        for py in map_height..height {
            for px in 0..width {
                let t = px as f64 / (width - 1) as f64;
                let color = cmap.sample(t);
                img.put_pixel(px, py, Rgba([color[0], color[1], color[2], 255]));
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
                        img.put_pixel(x, py, Rgba([0, 0, 0, 255]));
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
                                img.put_pixel(x, py, Rgba([0, 0, 0, 255]));
                            }
                        }
                    }
                }
            }
        }

    }


    if draw_border {
        let border_width_px = (width as f64 * 0.004).max(2.0);
        println!("Border width is {border_width_px}");
        draw_projection_border(
            &mut img,
            map_height,
            Rgba([0, 0, 0, 255]),
            border_width_px,
            |u, v| (u * u) / 4.0 + v * v,
        );
    }




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
    img: &mut RgbaImage,
    map_height: u32,
    border_color: Rgba<u8>,
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
