use crate::{Scale};
use crate::colormap::Colormap;
use crate::{PixelSink};
use image::Rgba;
use crate::latex::latex_to_unicode;


#[derive(Clone)]
pub struct ColorbarSpec {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,

    pub min: f64,
    pub max: f64,
    pub scale: Scale,
    pub gamma: f64,

    pub cmap: &'static Colormap,

    pub show_ticks: bool,
    pub label_font_size: f64,
}



pub fn apply_gamma(t: f64, gamma: f64) -> f64 {
    if gamma == 1.0 {
        t
    } else {
        t.powf(gamma)
    }
}

pub struct ColorbarTicks {
    pub major_values: Vec<f64>,
    pub major_positions: Vec<f64>,
    pub minor_values: Vec<f64>,
    pub minor_positions: Vec<f64>,
}

/// Format a tick label for display on colorbar
pub fn format_tick_label(value: f64, scale: Scale, pos: Option<f64>, _latex_rendering: bool, _units: Option<&str>) -> String {
    if value.abs() < 1e-12 {
        "0".to_string()
    } else {
        match scale {
            Scale::Histogram => {
                if let Some(p) = pos {
                    if (p - 0.0).abs() < 1e-6 || (p - 1.0).abs() < 1e-6 {
                        println!("Position, value, {}, {}", p, value);
                        format!("{:.3}", value)
                    } else {
                        format!("{:.0}%", p * 100.0)
                    }
                } else {
                    format!("{:.3}", value)
                }
            }
            Scale::Log => {
                let exp = value.abs().log10().floor() as i32;
                let base = 10_f64.powi(exp);
                let coeff = (value / base).round();
                let latex_str = if (coeff - 1.0).abs() < 1e-12 {
                    format!("10^{{{}}}", exp)
                } else {
                    format!("{} \\times 10^{{{}}}", coeff as i64, exp)
                };
                latex_to_unicode(&latex_str)
            }
            _ => {
                if value.abs() < 1000.0 {
                    if value.fract().abs() < 1e-6 {
                        format!("{}", value.round() as i64)
                    } else {
                        format!("{:.3}", value)
                    }
                } else {
                    let exp = value.abs().log10().floor() as i32;
                    let base = 10_f64.powi(exp);
                    let coeff = (value / base).round();
                    let latex_str = if (coeff - 1.0).abs() < 1e-6 {
                        format!("10^{{{}}}", exp)
                    } else {
                        format!("{} \\times 10^{{{}}}", coeff as i64, exp)
                    };
                    latex_to_unicode(&latex_str)
                }
            }
        }
    }
}

/// Format a tick label for display on colorbar with optional LaTeX rendering
/// Note: Units are displayed separately, not appended to labels
pub fn format_tick_label_with_units(value: f64, scale: Scale, pos: Option<f64>, latex_rendering: bool, _units: Option<&str>) -> String {
    let mut label = if value.abs() < 1e-12 {
        "0".to_string()
    } else {
        match scale {
            Scale::Histogram => {
                if let Some(p) = pos {
                    if (p - 0.0).abs() < 1e-6 || (p - 1.0).abs() < 1e-6 {
                        format!("{:.3}", value)
                    } else {
                        format!("{:.0}%", p * 100.0)
                    }
                } else {
                    format!("{:.3}", value)
                }
            }
            Scale::Log => {
                let exp = value.abs().log10().floor() as i32;
                let base = 10_f64.powi(exp);
                let coeff = (value / base).round();
                if (coeff - 1.0).abs() < 1e-12 {
                    format!("10^{{{}}}", exp)
                } else {
                    format!("{} \\times 10^{{{}}}", coeff as i64, exp)
                }
            }
            _ => {
                if value.abs() < 1000.0 {
                    if value.fract().abs() < 1e-6 {
                        format!("{}", value.round() as i64)
                    } else {
                        format!("{:.3}", value)
                    }
                } else {
                    let exp = value.abs().log10().floor() as i32;
                    let base = 10_f64.powi(exp);
                    let coeff = (value / base).round();
                    if (coeff - 1.0).abs() < 1e-12 {
                        format!("10^{{{}}}", exp)
                    } else {
                        format!("{} \\times 10^{{{}}}", coeff as i64, exp)
                    }
                }
            }
        }
    };

    // Apply LaTeX processing if enabled
    if latex_rendering {
        label = latex_to_unicode(&label);
    }

    label
}

/// Format units label to be displayed below the colorbar
pub fn format_units_label(latex_rendering: bool, units: Option<&str>) -> Option<String> {
    units.map(|unit_str| {
        if latex_rendering {
            latex_to_unicode(unit_str)
        } else {
            unit_str.to_string()
        }
    })
}

/// Convert integer to Unicode superscript string
fn to_superscript(n: i32) -> String {
    let map = [
        ('0', '⁰'), ('1', '¹'), ('2', '²'), ('3', '³'), ('4', '⁴'),
        ('5', '⁵'), ('6', '⁶'), ('7', '⁷'), ('8', '⁸'), ('9', '⁹'),
        ('-', '⁻')
    ].iter().copied().collect::<std::collections::HashMap<_, _>>();

    n.to_string().chars()
        .map(|c| *map.get(&c).unwrap_or(&c))
        .collect()
}

// fn log_minor_ticks(major: &[f64]) -> Vec<f64> {
//     let mut minors = Vec::new();
// 
//     for pair in major.windows(2) {
//         let a = pair[0];
//         let b = pair[1];
// 
//         if a <= 0.0 || b <= 0.0 {
//             continue;
//         }
// 
//         let log_a = a.log10();
// 
//         let base = 10f64.powf(log_a.floor());
// 
//         for mult in [2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0] {
//             let v = base * mult;
//             if v > a && v < b {
//                 minors.push(v);
//             }
//         }
//     }
// 
//     minors
// }


// fn linear_minor_ticks(major: &[f64], n_minor: usize) -> Vec<f64> {
//     let mut minors = Vec::new();
// 
//     for pair in major.windows(2) {
//         let a = pair[0];
//         let b = pair[1];
//         let step = (b - a) / (n_minor + 1) as f64;
// 
//         for i in 1..=n_minor {
//             minors.push(a + step * i as f64);
//         }
//     }
// 
//     minors
// }


// fn compute_minor_tick_values(
//     major_values: &[f64],
//     scale: Scale,
//     n_minor: usize,
// ) -> Vec<f64> {
//     match scale {
//         Scale::Linear
//         | Scale::Asinh { .. }
//         | Scale::Symlog { .. }
//         | Scale::PlanckLog { .. } => {
//             linear_minor_ticks(major_values, n_minor)
//         }
// 
//         Scale::Log => {
//             log_minor_ticks(major_values)
//         }
//     }
// }




pub fn compute_major_tick_values(minv: f64, maxv: f64, scale: Scale, nticks: usize) -> Vec<f64> {
    // Handle the case where all values are the same
    if minv >= maxv {
        return vec![minv; nticks];
    }

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
        Scale::Histogram => todo!()
    }
}

pub fn render_colorbar_gradient(
    x0: u32,
    y0: u32,
    width: u32,
    height: u32,
    cmap: &Colormap,
    gamma: f64,
    sink: &mut dyn PixelSink,
) {
    for py in 0..height {
        for px in 0..width {
            let t_linear = px as f64 / (width - 1) as f64;
            let t = apply_gamma(t_linear, gamma);
            
            let mut c = Rgba([0, 0, 0, 255]);
            let base = cmap.sample(t);
            c[0] = base[0];
            c[1] = base[1];
            c[2] = base[2];
            
            
            sink.draw_pixel(x0 + px, y0 + py, c);

        }
    }
}

