use image::{GrayImage, RgbaImage, Luma, Rgba};
use std::f64::consts::PI;
use crate::colormap::{Colormap};

fn load_default_font() -> Font<'static> {
    static FONT_DATA: &[u8] = include_bytes!(
        "../assets/fonts/DejaVuSans.ttf"
    );

    Font::try_from_bytes(FONT_DATA)
        .expect("Failed to load embedded font")
}

fn compute_major_tick_values(minv: f64, maxv: f64, scale: Scale, nticks: usize) -> Vec<f64> {
    match scale {
        Scale::Linear => {
            let mut ticks = Vec::with_capacity(nticks);
            let step = (maxv - minv) / (nticks - 1) as f64;
            for i in 0..nticks {
                ticks.push(minv + i as f64 * step);
            }
            ticks
        }
        Scale::Log => {
            // Find log10 range
            let log_min = minv.log10();
            let log_max = maxv.log10();
            let mut ticks = Vec::new();

            // Pick integer powers of 10 first
            let min_pow = log_min.floor() as i32;
            let max_pow = log_max.ceil() as i32;

            for p in min_pow..=max_pow {
                let base = 10f64.powi(p);
                for mult in &[1.0, 2.0, 5.0] {
                    let val = base * mult;
                    if val >= minv && val <= maxv {
                        ticks.push(val);
                    }
                }
            }

            ticks.sort_by(|a, b| a.partial_cmp(b).unwrap());
            ticks
        }
        Scale::Asinh { scale: _ } |
        Scale::Symlog { linthresh: _ } |
        Scale::PlanckLog { linthresh: _ } => {
            // Fall back to linear-style ticks for now
            let mut ticks = Vec::with_capacity(nticks);
            let step = (maxv - minv) / (nticks - 1) as f64;
            for i in 0..nticks {
                ticks.push(minv + i as f64 * step);
            }
            ticks
        }
    }
}




pub struct ColorbarTick {
    pub t: f64,      // normalized position [0,1]
    pub value: f64,  // data value
}


fn format_tick_label(value: f64, scale: Scale) -> String {
    match scale {
        Scale::Linear | Scale::Asinh { .. } | Scale::Symlog { .. } => {
            if value.abs() >= 1e4 || value.abs() < 1e-3 {
                format!("{:.1e}", value)
            } else {
                format!("{:.3}", value)
            }
        }
        Scale::Log | Scale::PlanckLog { .. } => {
            // Log scales: show powers of 10 when possible
            let exp = value.log10().round();
            if (10f64.powf(exp) - value).abs() / value < 1e-6 {
                format!("10^{:.0}", exp)
            } else {
                format!("{:.2}", value)
            }
        }
    }
}

fn scale_t_to_value(
    t: f64,
    min: f64,
    max: f64,
    scale: Scale,
) -> f64 {
    match scale {
        Scale::Linear => {
            min + t * (max - min)
        }

        Scale::Log => {
            let lmin = min.ln();
            let lmax = max.ln();
            (lmin + t * (lmax - lmin)).exp()
        }

        Scale::Asinh { scale } => {
            let amin = (min / scale).asinh();
            let amax = (max / scale).asinh();
            scale * (amin + t * (amax - amin)).sinh()
        }

        Scale::Symlog { linthresh } => {
            let sign = if t < 0.5 { -1.0 } else { 1.0 };
            let tt = (t - 0.5).abs() * 2.0;

            if tt * max.abs() < linthresh {
                sign * linthresh * tt
            } else {
                sign * (linthresh + ((tt * max.abs()) - linthresh).exp())
            }
        }

        Scale::PlanckLog { linthresh } => {
            if t * max < linthresh {
                t * linthresh
            } else {
                linthresh + ((t * max) - linthresh).exp()
            }
        }
    }
}


use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale as FontScale};

fn draw_centered_text(
    img: &mut RgbaImage,
    font: &Font,
    text: &str,
    center_x: i32,
    top_y: i32,
    size: f32,
) {
    let scale = FontScale::uniform(size);

    // Estimate text width (rusttype has no layout API)
    let width_estimate = (text.len() as f32 * size * 0.6) as i32;
    let x = center_x - width_estimate / 2;

    draw_text_mut(
        img,
        Rgba([0, 0, 0, 255]),
        x,
        top_y,
        scale,
        font,
        text,
    );
}



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
) -> PixelValue {
    if min >= max {
        panic!("min must be < max");
    }

    let t: f64 = match scale {
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
    let colorbar_height = if show_colorbar {
        map_height / 20
    }
    else
    {
        0
    };


    let cbar_pad = if show_colorbar {
        width / 25
    }
    else {
        0
    };
    let label_padding = if show_colorbar {
        map_height
    }
    else {
        0
    };

    let font_data = include_bytes!("../assets/fonts/DejaVuSans.ttf");
    let font = Font::try_from_bytes(font_data as &[u8])
        .expect("Failed to load font");
    
    let label_font_size = (colorbar_height as f32 * 0.35).max(10.0) as f32;
    //let label_y = (map_height + label_padding) as i32;
    let label_y = (map_height + colorbar_height + 2) as i32; // 2 px padding


    let height = if show_colorbar {
        map_height + colorbar_height + label_font_size as u32 + label_padding
    }
    else {
        map_height
    };
    let extra_label_space = if show_colorbar { label_font_size as u32 + 4 } else { 0 };
    let height = map_height + colorbar_height + extra_label_space;

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

            /*
            let px_color = match scale_value(val, minv, maxv, scale, neg_mode, gamma) {
                PixelValue::Color(t) => {
                    let c = cmap.sample(t);
                    Rgba([c[0], c[1], c[2], 255])
                }
                PixelValue::Bad => {
                    bad_color
                }
            };
            */
            let px_color = match scale_value(val, minv, maxv, scale, neg_mode) {
                PixelValue::Color(t) => {
                    let t = apply_gamma(t, gamma);
                    let c = cmap.sample(t);
                    Rgba([c[0], c[1], c[2], 255])
                }
                PixelValue::Bad => bad_color,
            };

            
            img.put_pixel(px, py, px_color);
        }
    }
    if show_colorbar {
        for py in map_height..(map_height + colorbar_height) {
            for px in cbar_pad..width-cbar_pad {
                let t_linear = px as f64 / (width - 1 - 2*cbar_pad) as f64;
                let t_gamma  = apply_gamma(t_linear, gamma);
                let color    = cmap.sample(t_gamma);
                img.put_pixel(px, py, Rgba([color[0], color[1], color[2], 255]));
            }
        }


        // ---------------- Colorbar tick marks (scale-aware) ----------------
        
        let nticks = 5;   // major ticks
        let nminor = 5;   // minor ticks per interval
        
        let ticks = compute_colorbar_ticks(
            minv,
            maxv,
            scale,
            nticks,
            nminor,
        );
        let major_values = compute_major_tick_values(minv, maxv, scale, nticks);
        
        let major_positions: Vec<f64> = major_values.iter()
            .map(|&v| scale_value(v, minv, maxv, scale, neg_mode))
            .filter_map(|pv| {
                if let PixelValue::Color(t) = pv { Some(t) } else { None }
            })
            .collect();

        
        // Scale tick heights relative to colorbar
        let major_tick_height = ((colorbar_height as f64) * 0.5).round().max(1.0) as u32;
        let minor_tick_height = ((colorbar_height as f64) * 0.3).round().max(1.0) as u32;
        
        // Scale tick widths relative to image width
        let major_tick_width = ((width as f64) * 0.002).round().max(1.0) as u32;
        let minor_tick_width = ((width as f64) * 0.001).round().max(1.0) as u32;
        
        let tick_bottom = map_height + colorbar_height - 1;
        

        // ---------------- Major ticks + labels ----------------
        for (&t, &val) in major_positions.iter().zip(major_values.iter()) {
            let px = cbar_pad + (t * (width - 1 - 2*cbar_pad) as f64).round() as u32;
            let tick_top = tick_bottom.saturating_sub(major_tick_height);
        
            // Draw major tick
            for dx in -1..major_tick_width as i32 +2 {
                let x = (px as i32 + dx) as u32;
                if x < width {
                    for py in tick_top-1..=tick_bottom {
                        if (dx <= -1) | (dx >= major_tick_width as i32 +1) | (py == tick_top-1) {
                            img.put_pixel(x, py, Rgba([255,255,255,255]));
                        }
                        else{
                            img.put_pixel(x, py, Rgba([0,0,0,255]));
                        }
                    }
                }
            }
        
            // Draw label
            let label = format_tick_label(val, scale);
            let text_width_est = (label.len() as f32 * label_font_size * 0.6) as i32;
            let text_x = px as i32 - text_width_est / 2;
        
            draw_text_mut(
                &mut img,
                Rgba([0, 0, 0, 255]),
                text_x,
                label_y,
                FontScale::uniform(label_font_size),
                &font,
                &label,
            );
        }



        /*        
        // ---------------- Minor ticks ----------------
        for &t in &ticks.minor {
            let px = (t * (width - 1) as f64).round() as u32;
            let px = cbar_pad  + (t * (width - 1 - 2*cbar_pad) as f64).round() as u32;
            let tick_top = tick_bottom.saturating_sub(minor_tick_height);
        
            for dx in 0..minor_tick_width {
                let x = px.saturating_add(dx);
                if x >= width {
                    continue;
                }
        
                for py in tick_top..=tick_bottom {
                    img.put_pixel(x, py, Rgba([0, 0, 0, 255]));
                }
            }
        }
*/

        // ---------------- Minor ticks ----------------
        // Interpolate between major ticks
        for i in 0..major_positions.len() - 1 {
            let t0 = major_positions[i];
            let t1 = major_positions[i + 1];
        
            for j in 1..nminor {
                let frac = j as f64 / nminor as f64;
                let tm = t0 + (t1 - t0) * frac;
                let pxm = cbar_pad + (tm * (width - 1 - 2*cbar_pad) as f64).round() as u32;
        
                let tick_top = tick_bottom.saturating_sub(minor_tick_height);
                for dx in -1..minor_tick_width as i32 + 2 {
                    let x = (pxm as i32 + dx) as u32;
                    if x >= width {
                        continue;
                    }
        
                    for py in tick_top-1..=tick_bottom {
                        if (dx <= -1) | (dx >= minor_tick_width as i32 +1) | (py == tick_top-1) {
                            img.put_pixel(x, py, Rgba([255, 255, 255, 255]));
                        }
                        else {
                            img.put_pixel(x, py, Rgba([0, 0, 0, 255]));
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

#[derive(Debug)]
pub struct ColorbarTicks {
    pub major: Vec<f64>, // normalized [0,1]
    pub minor: Vec<f64>, // normalized [0,1]
}


fn normalize_value(
    value: f64,
    min: f64,
    max: f64,
    scale: Scale,
) -> Option<f64> {
    match scale_value(value, min, max, scale, NegMode::Unseen) {
        PixelValue::Color(t) => Some(t),
        PixelValue::Bad => None,
    }
}

pub fn compute_colorbar_ticks(
    min: f64,
    max: f64,
    scale: Scale,
    nticks: usize,
    nminor: usize,
) -> ColorbarTicks {
    let mut major_vals: Vec<f64> = Vec::new();
    let mut minor_vals: Vec<f64> = Vec::new();

    match scale {
        /* ------------------------------------------------------------ */
        /* Linear                                                       */
        /* ------------------------------------------------------------ */
        Scale::Linear => {
            for i in 0..nticks {
                let t = i as f64 / (nticks - 1) as f64;
                major_vals.push(min + t * (max - min));
            }

            for w in major_vals.windows(2) {
                let (a, b) = (w[0], w[1]);
                for j in 1..nminor {
                    let t = j as f64 / nminor as f64;
                    minor_vals.push(a + t * (b - a));
                }
            }
        }

        /* ------------------------------------------------------------ */
        /* Log                                                          */
        /* ------------------------------------------------------------ */
        Scale::Log => {
            let log_min = min.log10().ceil() as i32;
            let log_max = max.log10().floor() as i32;

            for p in log_min..=log_max {
                let v = 10f64.powi(p);
                if v >= min && v <= max {
                    major_vals.push(v);
                }

                for m in 2..10 {
                    let vm = v * m as f64;
                    if vm > min && vm < max {
                        minor_vals.push(vm);
                    }
                }
            }
        }

        /* ------------------------------------------------------------ */
        /* Asinh                                                        */
        /* ------------------------------------------------------------ */
        Scale::Asinh { scale } => {
            let lin = scale;

            let anchors = [
                min,
                -10.0 * lin,
                -lin,
                0.0,
                lin,
                10.0 * lin,
                max,
            ];

            for &v in &anchors {
                if v >= min && v <= max {
                    major_vals.push(v);
                }
            }

            major_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

            for w in major_vals.windows(2) {
                let (a, b) = (w[0], w[1]);
                for j in 1..nminor {
                    let t = j as f64 / nminor as f64;
                    minor_vals.push(a + t * (b - a));
                }
            }
        }

        /* ------------------------------------------------------------ */
        /* Symlog                                                       */
        /* ------------------------------------------------------------ */
        Scale::Symlog { linthresh } => {
            let mut pos = Vec::new();
            let mut neg = Vec::new();

            let log_max = max.abs().log10().floor() as i32;

            for p in 0..=log_max {
                let v = 10f64.powi(p);
                if v >= linthresh {
                    pos.push(v);
                    neg.push(-v);
                }
            }

            major_vals.extend(neg.iter().rev());
            major_vals.push(0.0);
            major_vals.extend(pos.iter());

            major_vals.retain(|&v| v >= min && v <= max);

            for w in major_vals.windows(2) {
                let (a, b) = (w[0], w[1]);
                for j in 1..nminor {
                    let t = j as f64 / nminor as f64;
                    minor_vals.push(a + t * (b - a));
                }
            }
        }

        /* ------------------------------------------------------------ */
        /* PlanckLog                                                    */
        /* ------------------------------------------------------------ */
        Scale::PlanckLog { linthresh: _ } => {
            let anchors = [
                min,
                -300.0,
                -100.0,
                -30.0,
                0.0,
                30.0,
                100.0,
                300.0,
                max,
            ];

            for &v in &anchors {
                if v >= min && v <= max {
                    major_vals.push(v);
                }
            }

            major_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

            for w in major_vals.windows(2) {
                let (a, b) = (w[0], w[1]);
                for j in 1..nminor {
                    let t = j as f64 / nminor as f64;
                    minor_vals.push(a + t * (b - a));
                }
            }
        }
    }

    /* ------------------------------------------------------------ */
    /* Normalize + filter                                           */
    /* ------------------------------------------------------------ */
    let major = major_vals
        .into_iter()
        .filter_map(|v| normalize_value(v, min, max, scale))
        .filter(|&t| t >= 0.0 && t <= 1.0)
        .collect::<Vec<_>>();

    let minor = minor_vals
        .into_iter()
        .filter_map(|v| normalize_value(v, min, max, scale))
        .filter(|&t| t > 0.0 && t < 1.0)
        .collect::<Vec<_>>();

    ColorbarTicks { major, minor }
}


#[inline]
fn apply_gamma(t: f64, gamma: f64) -> f64 {
    if gamma == 1.0 {
        t
    } else {
        t.clamp(0.0, 1.0).powf(1.0 / gamma)
    }
}

