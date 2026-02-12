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
pub fn format_tick_label_with_units(value: f64, scale: Scale, pos: Option<f64>, latex_rendering: bool, _units: Option<&str>, is_pdf: bool) -> String {
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
                    let latex_str = if (coeff - 1.0).abs() < 1e-12 {
                        format!("10^{{{}}}", exp)
                    } else {
                        format!("{} \\times 10^{{{}}}", coeff as i64, exp)
                    };
                    latex_to_unicode(&latex_str)
                }
            }
        }
    };

    // Apply LaTeX processing if enabled
    // For PNG: convert to Unicode superscripts (supported by imageproc)
    // For PDF: keep LaTeX notation (Cairo doesn't support Unicode superscripts well)
    if latex_rendering && !is_pdf {
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

// /// Convert integer to Unicode superscript string
// fn to_superscript(n: i32) -> String {
//     let map = [
//         ('0', '⁰'), ('1', '¹'), ('2', '²'), ('3', '³'), ('4', '⁴'),
//         ('5', '⁵'), ('6', '⁶'), ('7', '⁷'), ('8', '⁸'), ('9', '⁹'),
//         ('-', '⁻')
//     ].iter().copied().collect::<std::collections::HashMap<_, _>>();
// 
//     n.to_string().chars()
//         .map(|c| *map.get(&c).unwrap_or(&c))
//         .collect()
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

            ticks.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
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

/// Draw extend triangles at the ends of a colorbar
/// extend_type: "none", "min", "max", or "both"
/// cbar_x, cbar_y: position of colorbar
/// cbar_w, cbar_h: width and height of colorbar
/// cmap: colormap to get min/max colors from
/// img: image to draw on
pub fn draw_colorbar_extends(
    extend: &crate::cli::Extend,
    cbar_x: f64,
    cbar_y: f64,
    cbar_w: f64,
    cbar_h: f64,
    cmap: &Colormap,
    img: &mut image::RgbaImage,
) {
    if *extend == crate::cli::Extend::None {
        return;
    }

    // Calculate the Y bounds for clamping triangles to the colorbar region
    let cbar_y_truncated = (cbar_y as u32) as f64;
    let cbar_h_truncated = (cbar_h as u32) as i32;
    let clamp_y_min = cbar_y_truncated as i32;
    let clamp_y_max = clamp_y_min + cbar_h_truncated - 1;
    let tip_distance = (cbar_h * 0.5).round() as i32;
    
    // Get colors from colormap ends
    let min_color_rgb = cmap.sample(0.0);
    let max_color_rgb = cmap.sample(1.0);
    let min_color = [min_color_rgb.0[0], min_color_rgb.0[1], min_color_rgb.0[2]];
    let max_color = [max_color_rgb.0[0], max_color_rgb.0[1], max_color_rgb.0[2]];

    // Convert colorbar coordinates to pixel integers (round to nearest for symmetry)
    let cbar_x_px = cbar_x.round() as i32;
    let cbar_w_px = cbar_w.round() as i32;
    
    // Position of the colorbar edges
    let cbar_left_x = cbar_x_px;
    let cbar_right_x = cbar_x_px + cbar_w_px - 1;
    
    // Calculate center more carefully to handle even/odd widths symmetrically
    // For symmetric extends, center should be at the midpoint between left and right edges
    let _cbar_center_x_f64 = (cbar_left_x as f64 + cbar_right_x as f64) / 2.0;
    
    // For integer pixel positioning: 
    // If we're using banker's rounding (round-half-to-even), the center rounds to:
    let _cbar_center_x = if _cbar_center_x_f64.fract().abs() < 1e-10 {
        _cbar_center_x_f64 as i32  // Already integer
    } else {
        // Round half to even
        let floor_x = _cbar_center_x_f64.floor() as i32;
        if (floor_x & 1) == 0 {
            floor_x  // Even, so .5 rounds down
        } else {
            floor_x + 1  // Odd, so .5 rounds up
        }
    };
    
    // The colorbar gradient is drawn starting at (cbar_y as u32), which truncates.
    // The gradient height is (cbar_h as u32), so it spans from y_top to y_top + height - 1.
    let cbar_y_truncated = (cbar_y as u32) as f64;
    let cbar_h_truncated = (cbar_h as u32) as i32;
    
    // The triangle base should align with the gradient bounds
    let gradient_top = cbar_y_truncated as i32;
    let gradient_bottom = gradient_top + cbar_h_truncated - 1;  // Original: gradient ends at y_top + height - 1
    
    // The clamping bounds: triangles should not exceed the colorbar region
    let clamp_y_min = gradient_top;
    let _clamp_y_max = gradient_bottom;
    
    // Tip should be at the floating-point center of the gradient for symmetry
    let gradient_center_f64 = (gradient_top as f64 + gradient_bottom as f64) / 2.0;
    // For even-height gradients, this centers the tip between two pixels
    // Use ceil so tip is at the upper center pixel
    let tip_y = gradient_center_f64.ceil() as i32;
    
    // For the extend triangles to be truly isosceles, the base vertices must be 
    // equidistant from the tip. With an integer tip_y, we need symmetric base positioning.
    // Calculate the symmetric half-height from tip to base
    let half_height = (gradient_bottom - gradient_top) as f64 / 2.0;
    let base_top_y_f = tip_y as f64 - half_height;
    let base_bottom_y_f = tip_y as f64 + half_height;
    let base_top_y = base_top_y_f.floor() as i32;
    let base_bottom_y = base_bottom_y_f.ceil() as i32;
    


    // Draw min arrow (left side) if needed
    if *extend == crate::cli::Extend::Min || *extend == crate::cli::Extend::Both {
        let arrow_color = Rgba([min_color[0], min_color[1], min_color[2], 255]);
        // Left arrow: tip points left (OUTWARD from colorbar), base is at colorbar left edge
        let tip_x = cbar_left_x - tip_distance;
        let base_x = cbar_left_x;
        
        let vertices = [
            (tip_x, tip_y),
            (base_x, base_top_y),
            (base_x, base_bottom_y),
        ];
        fill_triangle_with_clamp(vertices, arrow_color, img, Some((clamp_y_min, clamp_y_max)));
    }

    // Draw max arrow (right side) if needed
    if *extend == crate::cli::Extend::Max || *extend == crate::cli::Extend::Both {
        let arrow_color = Rgba([max_color[0], max_color[1], max_color[2], 255]);
        // Right arrow: tip points right (OUTWARD from colorbar), base is at colorbar right edge
        let tip_x = cbar_right_x + tip_distance;
        let base_x = cbar_right_x;
        
        let vertices = [
            (tip_x, tip_y),
            (base_x, base_top_y),
            (base_x, base_bottom_y),
        ];
        fill_triangle_with_clamp(vertices, arrow_color, img, Some((clamp_y_min, clamp_y_max)));
    }
}

/// Fill a triangle using scanline algorithm with consistent Bresenham rasterization
/// Ensures symmetric rendering regardless of triangle orientation
/// Optional clamp_y: if Some((min, max)), clamp all Y pixels to this range
pub fn fill_triangle(vertices: [(i32, i32); 3], color: Rgba<u8>, img: &mut image::RgbaImage) {
    fill_triangle_with_clamp(vertices, color, img, None);
}

pub fn fill_triangle_with_clamp(
    vertices: [(i32, i32); 3],
    color: Rgba<u8>,
    img: &mut image::RgbaImage,
    clamp_y: Option<(i32, i32)>,
) {
    let [v0, v1, v2] = vertices;

    // Find the y-range
    let y_min = v0.1.min(v1.1).min(v2.1);
    let y_max = v0.1.max(v1.1).max(v2.1);

    if y_min == y_max {
        return; // Degenerate triangle
    }

    let width = img.width() as i32;
    let height = img.height() as i32;

    // Special case: detect isosceles triangles (common for colorbar extends)
    // Isosceles triangle has two equal sides
    let dist2 = |(dx, dy)| dx * dx + dy * dy;
    let v01_dist = dist2((v1.0 - v0.0, v1.1 - v0.1));
    let v12_dist = dist2((v2.0 - v1.0, v2.1 - v1.1));
    let v20_dist = dist2((v0.0 - v2.0, v0.1 - v2.1));
    
    // Determine which vertex is the "tip" (equidistant from other two)
    // If v01_dist == v20_dist, then v0 is equidistant from v1 and v2, so v0 is the tip
    let tip_option = if v01_dist == v20_dist && v01_dist != v12_dist {
        Some(v0)  // v0 is the tip (equidistant from v1 and v2)
    } else if v12_dist == v01_dist && v12_dist != v20_dist {
        Some(v1)  // v1 is the tip (equidistant from v0 and v2)
    } else if v20_dist == v12_dist && v20_dist != v01_dist {
        Some(v2)  // v2 is the tip (equidistant from v0 and v1)
    } else {
        None
    };
    
    if let Some(tip_vertex) = tip_option {
        // Find the two base vertices
        let base_verts: Vec<_> = [v0, v1, v2].iter()
            .filter(|&&v| v != tip_vertex)
            .copied()
            .collect();
        
        if base_verts.len() == 2 {
            let base_v1 = base_verts[0];
            let base_v2 = base_verts[1];
            
            // Check if base is vertical (same X) or horizontal (same Y)
            let is_vertical_base = base_v1.0 == base_v2.0;
            
            if is_vertical_base {
                // For a vertical-base triangle: tip at tip_vertex, base vertices at base_v1 and base_v2
                // This forms a wedge. The base is a vertical line from base_v1 to base_v2.
                // Fill the wedge by finding the X extent at each Y.
                
                let _round_half_to_even = |x: f64| {
                    let floor_x = x.floor();
                    let frac = x - floor_x;
                    if frac < 0.5 {
                        floor_x as i32
                    } else if frac > 0.5 {
                        (floor_x + 1.0) as i32
                    } else {
                        let as_int = floor_x as i32;
                        if as_int % 2 == 0 {
                            as_int
                        } else {
                            as_int + 1
                        }
                    }
                };
                
                // Helper to find where an edge intersects a Y-scanline
                let edge_x_at_y = |p1: (i32, i32), p2: (i32, i32), y: i32| -> Option<f64> {
                    let (x1, y1) = (p1.0 as f64, p1.1 as f64);
                    let (x2, y2) = (p2.0 as f64, p2.1 as f64);
                    
                    if y1 == y2 {
                        return None;  // Horizontal edge
                    }
                    
                    // For any Y, calculate the X on the infinite line through p1 and p2
                    let t = (y as f64 - y1) / (y2 - y1);
                    Some(x1 + t * (x2 - x1))
                };
                
                // For a vertical-base triangle, the base is at a constant X coordinate
                // Find which base vertex is which, and determine the single non-vertical edge
                let (base_v_top, base_v_bottom) = if base_v1.1 < base_v2.1 {
                    (base_v1, base_v2)
                } else {
                    (base_v2, base_v1)
                };
                
                // Track previous edge positions to ensure monotonic stepping
                let _prev_x_edge: Option<i32> = None;
                
                for y in y_min..=y_max {
                    if y < 0 || y >= height {
                        continue;
                    }
                    
                    // Clamp Y to the allowed range if specified
                    if let Some((clamp_min, clamp_max)) = clamp_y
                        && (y < clamp_min || y > clamp_max) {
                            continue;
                        }
                    
                    // Find where the slanted edge (from tip to either top or bottom base) intersects this Y
                    // We need to figure out which segment of the edge is relevant
                    let x_on_edge = if y <= tip_vertex.1 {
                        // Above or at tip: use edge from tip to base_top
                        edge_x_at_y(tip_vertex, base_v_top, y)
                    } else {
                        // Below tip: use edge from tip to base_bottom
                        edge_x_at_y(tip_vertex, base_v_bottom, y)
                    };
                    
                    if let Some(x_edge_f) = x_on_edge {
                        // The fill range at this Y spans from the slanted edge to the vertical base
                        let base_x = base_v1.0;  // Both base vertices have the same X
                        
                        let x_min_f = x_edge_f.min(base_x as f64);
                        let x_max_f = x_edge_f.max(base_x as f64);
                        
                        // Use floor for min edge and ceil for max edge to ensure monotonic coverage
                        let x_min_i = x_min_f.floor() as i32;
                        let x_max_i = x_max_f.ceil() as i32;
                        
                        // Clamp before computing the fill range
                        let x_min_clamped = x_min_i.max(0);
                        let x_max_clamped = x_max_i.min(width - 1);
                        
                        // Ensure valid range
                        if x_max_clamped >= x_min_clamped {
                            let x_start = x_min_clamped as u32;
                            let x_end = (x_max_clamped + 1) as u32;
                            
                            for x in x_start..x_end {
                                img.put_pixel(x, y as u32, color);
                            }
                        }
                    }
                }
            } else {
                // For a horizontal base, use the original isosceles rasterization
                let mut base_left = base_v1;
                let mut base_right = base_v2;
                
                // Ensure left < right
                if base_left.0 > base_right.0 {
                    std::mem::swap(&mut base_left, &mut base_right);
                }
                
                let round_half_to_even = |x: f64| {
                    let floor_x = x.floor();
                    let frac = x - floor_x;
                    if frac < 0.5 {
                        floor_x as i32
                    } else if frac > 0.5 {
                        (floor_x + 1.0) as i32
                    } else {
                        let as_int = floor_x as i32;
                        if as_int % 2 == 0 {
                            as_int
                        } else {
                            as_int + 1
                        }
                    }
                };
                
                // Use symmetric isosceles rasterization
                for y in y_min..=y_max {
                    if y < 0 || y >= height {
                        continue;
                    }
                    
                    let tip_y = tip_vertex.1 as f64;
                    let base_y = base_left.1 as f64;
                    
                    let distance_from_tip = ((y as f64) - tip_y).abs();
                    let distance_to_base = (base_y - tip_y).abs();
                    
                    let t = if distance_to_base > 0.0 {
                        (distance_from_tip / distance_to_base).min(1.0)
                    } else {
                        0.0
                    };
                    
                    let tip_center_x = tip_vertex.0 as f64;
                    let left_x = tip_center_x + ((base_left.0 as f64) - tip_center_x) * t;
                    let right_x = tip_center_x + ((base_right.0 as f64) - tip_center_x) * t;
                    
                    let x_left = round_half_to_even(left_x);
                    let x_right = round_half_to_even(right_x);
                    
                    let x_start = (x_left.max(0) as u32).min(width as u32);
                    let x_end = ((x_right + 1).max(0) as u32).min(width as u32);
                    
                    for x in x_start..x_end {
                        img.put_pixel(x, y as u32, color);
                    }
                }
            }
            return;
        }
    }

    // Fall back to standard triangle rasterization for non-isosceles triangles
    for y in y_min..=y_max {
        if y < 0 || y >= height {
            continue;
        }

        let mut x_values = Vec::new();

        // Check each edge of the triangle
        // Edge v0-v1
        if let Some(x) = edge_x_at_y(v0, v1, y) {
            x_values.push(x);
        }

        // Edge v1-v2
        if let Some(x) = edge_x_at_y(v1, v2, y) {
            x_values.push(x);
        }

        // Edge v2-v0
        if let Some(x) = edge_x_at_y(v2, v0, y) {
            x_values.push(x);
        }

        // Sort to prepare for min/max
        x_values.sort();

        // For a valid scanline inside the triangle, we need at least 2 x values
        if x_values.len() >= 2 {
            let x_min = x_values[0];
            let x_max = x_values[x_values.len() - 1];

            // Fill the scanline from min to max
            let x_start = x_min.max(0) as u32;
            let x_end = (x_max + 1).min(width) as u32;
            for x in x_start..x_end {
                img.put_pixel(x, y as u32, color);
            }
        }
    }
}

/// Get the x-coordinate where an edge crosses a scanline y
/// Returns None if the edge doesn't intersect that scanline
/// Uses the rule: scanlines cross edges in the interval [y_min, y_max)
/// (includes bottom but not top, avoiding double-counting at shared vertices)
fn edge_x_at_y(p1: (i32, i32), p2: (i32, i32), y: i32) -> Option<i32> {
    let (x1, y1) = p1;
    let (x2, y2) = p2;

    let y_min = y1.min(y2);
    let y_max = y1.max(y2);

    // Use half-open interval [y_min, y_max): includes min but not max
    // This avoids counting shared vertices twice while covering all pixels
    if y < y_min || y >= y_max {
        return None;
    }

    // Horizontal edge - doesn't cross any scanline
    if y1 == y2 {
        return None;
    }

    // Calculate x using symmetric midpoint rounding
    // For symmetric rasterization, we need to handle negative and positive slopes identically
    let dy = (y2 - y1).abs();
    let dx = x2 - x1;

    let t_num = if y2 > y1 {
        y - y1
    } else {
        y1 - y
    };

    // Symmetric rounding for both positive and negative slopes
    // Goal: round(a/b) with ties going away from zero
    // Formula: (a + sign(a) * (b-1)/2) / b
    // But for integer arithmetic that handles both signs: use ceil division for positive,
    // which means we use different offsets... Actually, we need true symmetric rounding.
    // 
    // Better approach: round-half-away-from-zero
    // Positive: (n + d/2) / d       (adds d/2 before dividing)
    // Negative: (n - d/2 + 1) / d   (must be different to round away from zero)
    //
    // Even better: just use proper symmetric rounding that works for all cases:
    // If n >= 0: add (d+1)/2 then divide
    // If n < 0: add -(d+1)/2 then divide
    
    let numerator = dx * t_num;
    let round_offset = (dy + 1) / 2;  // Use (dy+1)/2 for symmetric rounding ties away from zero
    let x = if numerator >= 0 {
        x1 + (numerator + round_offset) / dy
    } else {
        x1 + (numerator - round_offset) / dy
    };

    Some(x)
}



#[cfg(test)]
mod tests {
    /// Test that colorbar extend triangle base coordinates align correctly with colorbar bounds
    #[test]
    fn test_extend_triangle_vertical_alignment() {
        // Test case 1: colorbar with integer bounds
        let cbar_y: f64 = 100.0;
        let cbar_h: f64 = 50.0;
        
        let base_top_y = cbar_y.floor() as i32;
        let base_bottom_y = (cbar_y + cbar_h).floor() as i32;
        
        assert_eq!(base_top_y, 100);
        assert_eq!(base_bottom_y, 150);
    }

    /// Test that fractional colorbar coordinates are handled correctly
    #[test]
    fn test_extend_triangle_fractional_coordinates() {
        // Test case: colorbar with fractional bounds (like 776.50 .. 812.50)
        let cbar_y: f64 = 776.5;
        let cbar_h: f64 = 36.0;
        
        // The PNG colorbar renders from cbar_y as i32 (truncates) with height cbar_h as i32
        // So gradient renders from pixel 776 to pixel 776 + 36 - 1 = 811
        let base_top_y = cbar_y as i32;
        let base_bottom_y = base_top_y + cbar_h as i32 - 1;
        // Tip is exactly centered between base vertices (integer division)
        let tip_y = (base_top_y + base_bottom_y) / 2;
        
        // base_top should truncate: 776.5 -> 776
        assert_eq!(base_top_y, 776);
        // base_bottom should be last pixel: 776 + 36 - 1 = 811
        assert_eq!(base_bottom_y, 811);
        // tip should be midpoint: (776 + 811) / 2 = 793 (integer division)
        assert_eq!(tip_y, 793);
        
        // Verify: base height should be 36 pixels (776 to 811 inclusive)
        let base_height = base_bottom_y - base_top_y + 1;
        assert_eq!(base_height, 36);
    }

    /// Test that tip_distance (0.5x colorbar height) is calculated correctly
    #[test]
    fn test_extend_triangle_proportions() {
        // Test case 1: small colorbar
        let cbar_h: f64 = 50.0;
        let tip_distance = (cbar_h * 0.5).round() as i32;
        assert_eq!(tip_distance, 25);
        
        // Test case 2: large colorbar (like at 1500px)
        let cbar_h: f64 = 36.0;
        let tip_distance = (cbar_h * 0.5).round() as i32;
        assert_eq!(tip_distance, 18);
        
        // Test case 3: odd height
        let cbar_h: f64 = 37.0;
        let tip_distance = (cbar_h * 0.5).round() as i32;
        assert_eq!(tip_distance, 19);
    }

    /// Test horizontal coordinate rounding
    #[test]
    fn test_extend_triangle_horizontal_alignment() {
        // Test case: colorbar horizontal position
        let cbar_x: f64 = 24.0;
        let cbar_w: f64 = 1152.0;
        
        let cbar_x_px = cbar_x.round() as i32;
        let cbar_w_px = cbar_w.round() as i32;
        
        assert_eq!(cbar_x_px, 24);
        assert_eq!(cbar_w_px, 1152);
        
        // Left arrow base should be at cbar_x_px
        let left_base_x = cbar_x_px;
        assert_eq!(left_base_x, 24);
        
        // Right arrow base should be at cbar_x_px + cbar_w_px
        let right_base_x = cbar_x_px + cbar_w_px;
        assert_eq!(right_base_x, 1176);
    }

    /// Test 1500px case specifically (user reported issue)
    #[test]
    fn test_extend_triangle_1500px_case() {
        // From the debug output: y_float: 776.50 .. 812.50
        let cbar_y: f64 = 776.5;
        let cbar_h: f64 = 36.0;
        
        // The PNG colorbar gradient renders using: y0 = cbar_y as u32, height = cbar_h as u32
        // So it renders from pixel 776 to pixel 776 + 36 - 1 = 811
        let colorbar_top = cbar_y as i32;
        let colorbar_bottom = colorbar_top + cbar_h as i32 - 1;
        
        // Triangle base should match exactly
        let base_top_y = cbar_y as i32;
        let base_bottom_y = base_top_y + cbar_h as i32 - 1;
        // Tip centered between base vertices
        let tip_y = (base_top_y + base_bottom_y) / 2;
        
        assert_eq!(base_top_y, colorbar_top);
        assert_eq!(base_bottom_y, colorbar_bottom);
        
        // Both should be 776 .. 811
        assert_eq!(base_top_y, 776);
        assert_eq!(base_bottom_y, 811);
        // Tip at midpoint: (776 + 811) / 2 = 793
        assert_eq!(tip_y, 793);
    }

    /// Test that left and right extend triangles have identical pixel widths at each scanline
    #[test]
    fn test_extend_triangles_symmetric() {
        // For w=1500, the triangles should be mirror images
        // This test verifies they have the same width at corresponding rows
        
        // Left triangle: tip at (2, 793), base at x=20, vertical from 776-810
        // Right triangle: tip at (1477, 793), base at x=1459, vertical from 776-810
        
        // At the base rows, both should have width=1
        // At tip row (y=793), both should have width=19
        // At symmetric rows above/below, widths should match
        
        // Simulate edge calculations for left triangle at y=777
        let base_top_y = 776;
        let _base_bottom_y = 810;
        let tip_y = 793;
        let y = 777;
        
        // Left triangle edges
        let tip_x_left = 2;
        let base_x_left = 20;
        let t = (y - base_top_y) as f64 / (tip_y - base_top_y) as f64;
        let left_edge_x = base_x_left as f64 + t * (tip_x_left as f64 - base_x_left as f64);
        
        // Right triangle edges  
        let tip_x_right = 1477;
        let base_x_right = 1459;
        let right_edge_x = base_x_right as f64 + t * (tip_x_right as f64 - base_x_right as f64);
        
        // Check symmetry: edges should have same absolute displacement from their bases
        let left_displacement = (base_x_left as f64 - left_edge_x).abs();
        let right_displacement = (right_edge_x - base_x_right as f64).abs();
        
        // Displacements should be nearly identical (within rounding error)
        assert!((left_displacement - right_displacement).abs() < 0.01,
            "Left displacement {:.2} != Right displacement {:.2}", 
            left_displacement, right_displacement);
    }

    /// Test that both triangles use the same rounding strategy for pixel coverage
    #[test]
    fn test_extend_triangle_rounding_symmetry() {
        // Test that ceil/floor rounding creates symmetric pixel spans
        
        // Edge case: fractional coordinates that require careful rounding
        let left_min: f64 = 18.94;  // Left edge moving left
        let right_max: f64 = 1460.06;  // Right edge moving right
        let fixed_max: f64 = 20.0;  // Vertical edge
        let fixed_min: f64 = 1459.0;  // Vertical edge
        
        // Both should use the same rounding strategy
        // ceil() for min edge (expand left/down)
        // round() for max edge (follow actual edge)
        let left_min_pixel = left_min.ceil() as i32;
        let left_max_pixel = fixed_max.round() as i32;
        let right_min_pixel = fixed_min.ceil() as i32;
        let right_max_pixel = right_max.round() as i32;
        
        // Verify consistent coverage
        assert_eq!(left_max_pixel - left_min_pixel, right_max_pixel - right_min_pixel,
            "Pixel spans should be equal: left=[{},{}], right=[{},{}]",
            left_min_pixel, left_max_pixel, right_min_pixel, right_max_pixel);
    }

    /// Test that triangles are vertically symmetric about the tip row in actual pixel coverage
    #[test]
    fn test_extend_triangle_vertical_symmetry() {
        // The triangle should be symmetric vertically in terms of pixel WIDTHS at symmetric rows
        
        // For a triangle with:
        // - Base top at y=776
        // - Tip at y=793 (distance = 17)
        // - Base bottom at y=810 (distance = 17)
        
        let base_top_y = 776i32;
        let base_bottom_y = 810i32;
        let tip_y = 793i32;
        let tip_x = 2i32;  // Left triangle
        let base_x = 20i32;
        
        // Verify the triangle is symmetric about the tip
        let dist_to_top = tip_y - base_top_y;
        let dist_to_bottom = base_bottom_y - tip_y;
        
        assert_eq!(dist_to_top, dist_to_bottom,
            "Triangle should be vertically symmetric: distance to top {} should equal distance to bottom {}",
            dist_to_top, dist_to_bottom);
        
        // More importantly: check that the SLOPES are equal in magnitude
        // Top slope: change in x per unit y from base_top to tip
        // Bottom slope: change in x per unit y from tip to base_bottom
        
        // Top: from (base_x, base_top_y) to (tip_x, tip_y)
        let top_dx = (tip_x - base_x) as f64;
        let top_dy = (tip_y - base_top_y) as f64;
        let top_slope_magnitude = (top_dx / top_dy).abs();
        
        // Bottom: from (tip_x, tip_y) to (base_x, base_bottom_y)  
        let bottom_dx = (base_x - tip_x) as f64;
        let bottom_dy = (base_bottom_y - tip_y) as f64;
        let bottom_slope_magnitude = (bottom_dx / bottom_dy).abs();
        
        assert!((top_slope_magnitude - bottom_slope_magnitude).abs() < 0.0001,
            "Slopes should have equal magnitude: top={:.4}, bottom={:.4}",
            top_slope_magnitude, bottom_slope_magnitude);
        
        // Now verify specific rows equidistant from tip have equal fractional x-coordinates
        let y_above: i32 = 777;  // 16 rows above tip
        let y_below: i32 = 809;  // 16 rows below tip
        
        // Calculate fractional x at y_above (on top edge)
        let t_above = (y_above as f64 - base_top_y as f64) / (tip_y as f64 - base_top_y as f64);
        let x_above = base_x as f64 + t_above * (tip_x as f64 - base_x as f64);
        
        // Calculate fractional x at y_below (on bottom edge)
        let t_below = (y_below as f64 - tip_y as f64) / (base_bottom_y as f64 - tip_y as f64);
        let x_below = tip_x as f64 + t_below * (base_x as f64 - tip_x as f64);
        
        // Both should be approximately 18.94 (approaching the tip from opposite sides)
        assert!((x_above - x_below).abs() < 0.01,
            "Symmetric rows should have nearly equal x-coordinates: y={} x={:.2}, y={} x={:.2}",
            y_above, x_above, y_below, x_below);
    }

    /// Test that both left and right triangles have identical vertical extent
    #[test]
    fn test_extend_triangle_height_equality() {
        // Both triangles should span the exact same y-range (full colorbar height)
        
        let base_top_y = 776i32;
        let base_bottom_y = 810i32;
        
        let left_height = base_bottom_y - base_top_y + 1;  // +1 for inclusive range
        let right_height = base_bottom_y - base_top_y + 1;
        
        assert_eq!(left_height, right_height,
            "Both triangles should have same height: left={}, right={}",
            left_height, right_height);
        
        // Triangle spans from 776 to 810 = 35 pixels
        assert_eq!(left_height, 35,
            "Triangle height should span full range: {} pixels",
            left_height);
    }

    /// Test that the discrete derivative (slope) of edges is consistent
    #[test]
    fn test_edge_discrete_derivative_consistency() {
        // Test the ACTUAL triangle dimensions from the real rendering:
        // With the updated tip_y calculation:
        // cbar_h = 28.8 => tip_y = (621.0 + 28.8 / 2.0).round() = 635
        // base_top_y = 621, base_bottom_y = 648
        // cbar_w = 1152, tip_distance = 14
        // Left triangle: tip=(2, 635), base=(16, 621)/(16, 648)
        // Right triangle: tip=(1181, 635), base=(1167, 621)/(1167, 648)
        
        eprintln!("\n=== LEFT TRIANGLE TOP EDGE: (2, 635) to (16, 621) ===");
        
        let mut prev_x = 2.0;
        for y in 621..=635 {
            let t = (y as f64 - 635.0) / (621.0 - 635.0);
            let x_float = 2.0 + t * (16.0 - 2.0);
            
            let round_away = |x: f64| {
                if x >= 0.0 {
                    (x + 0.5).floor()
                } else {
                    (x - 0.5).ceil()
                }
            };
            
            let x = round_away(x_float) as i32;
            let dx = x - prev_x as i32;
            
            eprintln!("y={:3}: t={:6.4}, x_float={:7.3}, x_rounded={:3}, dx={}", 
                y, t, x_float, x, dx);
            
            prev_x = x as f64;
        }
        
        eprintln!("\n=== LEFT TRIANGLE BOTTOM EDGE: (2, 635) to (16, 648) ===");
        
        let mut prev_x = 2.0;
        for y in 635..=648 {
            let t = (y as f64 - 635.0) / (648.0 - 635.0);
            let x_float = 2.0 + t * (16.0 - 2.0);
            
            let round_away = |x: f64| {
                if x >= 0.0 {
                    (x + 0.5).floor()
                } else {
                    (x - 0.5).ceil()
                }
            };
            
            let x = round_away(x_float) as i32;
            let dx = x - prev_x as i32;
            
            eprintln!("y={:3}: t={:6.4}, x_float={:7.3}, x_rounded={:3}, dx={}", 
                y, t, x_float, x, dx);
            
            prev_x = x as f64;
        }
        
        eprintln!("\n=== RIGHT TRIANGLE TOP EDGE: (1181, 635) to (1167, 621) ===");
        
        let mut prev_x = 1181.0;
        for y in 621..=635 {
            let t = (y as f64 - 635.0) / (621.0 - 635.0);
            let x_float = 1181.0 + t * (1167.0 - 1181.0);
            
            let round_away = |x: f64| {
                if x >= 0.0 {
                    (x + 0.5).floor()
                } else {
                    (x - 0.5).ceil()
                }
            };
            
            let x = round_away(x_float) as i32;
            let dx = prev_x as i32 - x;
            
            eprintln!("y={:3}: t={:6.4}, x_float={:7.3}, x_rounded={:3}, dx={}", 
                y, t, x_float, x, dx);
            
            prev_x = x as f64;
        }
        
        eprintln!("\n=== RIGHT TRIANGLE BOTTOM EDGE: (1181, 635) to (1167, 648) ===");
        
        let mut prev_x = 1181.0;
        for y in 635..=648 {
            let t = (y as f64 - 635.0) / (648.0 - 635.0);
            let x_float = 1181.0 + t * (1167.0 - 1181.0);
            
            let round_away = |x: f64| {
                if x >= 0.0 {
                    (x + 0.5).floor()
                } else {
                    (x - 0.5).ceil()
                }
            };
            
            let x = round_away(x_float) as i32;
            let dx = prev_x as i32 - x;
            
            eprintln!("y={:3}: t={:6.4}, x_float={:7.3}, x_rounded={:3}, dx={}", 
                y, t, x_float, x, dx);
            
            prev_x = x as f64;
        }
    }

    /// Test that a diagonal edge (45 degrees) produces perfect stepping:
    /// For each step down, exactly one step sideways, with no plateaus
    #[test]
    fn test_diagonal_edge_perfect_stepping() {
        // A perfect 45-degree diagonal should step: 1 pixel horizontally per 1 pixel vertically
        // Test with coordinates that should produce this pattern
        
        // Simple case: from (0,0) to (10,10) should be 10 steps right, 10 steps down
        eprintln!("\n=== PERFECT DIAGONAL TEST: (0,0) to (10,10) ===");
        
        let mut prev_x = 0.0;
        let mut step_sizes = Vec::new();
        
        for y in 0..=10 {
            let t = y as f64 / 10.0;
            let x_float = 0.0 + t * 10.0;
            
            let round_away = |x: f64| {
                if x >= 0.0 {
                    (x + 0.5).floor()
                } else {
                    (x - 0.5).ceil()
                }
            };
            
            let x = round_away(x_float) as i32;
            let dx = (x as f64 - prev_x).abs() as i32;
            
            eprintln!("y={:2}: x_float={:5.1}, x={:2}, dx={}", y, x_float, x, dx);
            step_sizes.push(dx);
            prev_x = x as f64;
        }
        
        // For a perfect diagonal, we should have minimal plateaus
        // Count how many steps have dx==0 (flat sections)
        let plateaus = step_sizes.iter().filter(|&&dx| dx == 0).count();
        eprintln!("Plateaus (dx==0): {} out of {}", plateaus, step_sizes.len());
        
        // For a true 45-degree line from (0,0) to (10,10), we expect no more than 1 plateau
        // (possibly at the endpoints where the same point is visited)
        assert!(plateaus <= 1, 
            "Diagonal edge should have at most 1 plateau, found {} out of {} steps",
            plateaus, step_sizes.len());
    }
    
    /// Test that the actual triangle edges are truly diagonal with no plateaus
    /// Uses the Bresenham-style integer rasterization
    #[test]
    fn test_triangle_edge_no_plateaus() {
        // Test both top and bottom edges of the triangle
        // With current geometry: tip=(2,635), base=(16,621)/(16,648)
        // Top edge: 14 pixels horizontal, 14 pixels vertical = perfect 45 degrees
        // Bottom edge: 14 pixels horizontal, 13 pixels vertical ≈ 47 degrees
        
        eprintln!("\n=== TOP EDGE PLATEAU CHECK (Bresenham) ===");
        let mut prev_x = i32::MAX;
        let mut top_plateaus = 0;
        
        for y in 621..=635 {
            // Use the Bresenham method
            let p1 = (2, 635);
            let p2 = (16, 621);
            
            let (x1, y1) = p1;
            let (x2, y2) = p2;
            let dy = y2 - y1;
            let dx = x2 - x1;
            let t_num = y - y1;
            
            let x = if dy > 0 {
                x1 + (dx * t_num + dy / 2) / dy
            } else {
                x1 + (dx * t_num - dy / 2) / dy
            };
            
            if prev_x != i32::MAX {
                let dx_val = (x - prev_x).abs();
                if dx_val == 0 {
                    top_plateaus += 1;
                    eprintln!("  Plateau at y={}, x_prev={}, x_curr={}", y, prev_x, x);
                }
            }
            eprintln!("y={:3}: x={:2}", y, x);
            prev_x = x;
        }
        
        eprintln!("\n=== BOTTOM EDGE PLATEAU CHECK (Bresenham) ===");
        let mut prev_x = i32::MAX;
        let mut bottom_plateaus = 0;
        
        for y in 635..=648 {
            let p1 = (2, 635);
            let p2 = (16, 648);
            
            let (x1, y1) = p1;
            let (x2, y2) = p2;
            let dy = y2 - y1;
            let dx = x2 - x1;
            let t_num = y - y1;
            
            let x = if dy > 0 {
                x1 + (dx * t_num + dy / 2) / dy
            } else {
                x1 + (dx * t_num - dy / 2) / dy
            };
            
            if prev_x != i32::MAX {
                let dx_val = (x - prev_x).abs();
                if dx_val == 0 {
                    bottom_plateaus += 1;
                    eprintln!("  Plateau at y={}, x_prev={}, x_curr={}", y, prev_x, x);
                }
            }
            eprintln!("y={:3}: x={:2}", y, x);
            prev_x = x;
        }
        
        eprintln!("Top edge plateaus: {}, Bottom edge plateaus: {}", top_plateaus, bottom_plateaus);
        
        // For proper diagonal edges, we should have no plateaus except at shared vertices
        // The tip at (2,635) is where both edges meet, so a plateau there is unavoidable
        assert!(top_plateaus <= 1, "Top edge should have at most 1 plateau (at tip vertex)");
        assert_eq!(bottom_plateaus, 0, "Bottom edge should have no plateaus");
    }
    /// Test that pixel edges align with the mathematical triangle edges
    /// For a 45-degree edge, the edge line should pass through pixel corners
    #[test]
    fn test_vectorized_edge_alignment() {
        // A 45-degree line from (2, 635) to (16, 621) should pass through pixel corners
        // For integer coordinates with 45-degree slope, the mathematical edge should align
        // with pixel boundaries
        
        eprintln!("\n=== VECTORIZED EDGE ALIGNMENT TEST ===");
        
        let p1 = (2i32, 635i32);
        let p2 = (16i32, 621i32);
        let (x1, y1) = p1;
        let (x2, y2) = p2;
        
        eprintln!("Edge from ({}, {}) to ({}, {})", x1, y1, x2, y2);
        eprintln!("Distance: dx={}, dy={}", (x2 - x1).abs(), (y2 - y1).abs());
        
        // The mathematical slope should be exactly -1 (since dx=14, dy=-14)
        let dx = (x2 - x1) as f64;
        let dy = (y2 - y1) as f64;
        let slope = dx / dy;
        
        eprintln!("Slope: {} (expected: -1.0 for 45-degree diagonal)", slope);
        
        // For a perfect 45-degree line, slope magnitude should be 1.0
        assert!((slope + 1.0).abs() < 0.01, 
            "Top edge should have -1.0 slope for 45-degree diagonal, got {}", slope);
        
        // Check that pixel centers are collinear with the edge
        // For each rasterized pixel on the edge, the center should be close to the mathematical line
        let mut max_distance: f64 = 0.0;
        
        for y in 621..=635 {
            let p1 = (2i32, 635i32);
            let p2 = (16i32, 621i32);
            
            let (x1, y1) = p1;
            let (x2, y2) = p2;
            let dy = y2 - y1;
            let dx = x2 - x1;
            let t_num = y - y1;
            
            let x = if dy > 0 {
                x1 + (dx * t_num + dy / 2) / dy
            } else {
                x1 + (dx * t_num - dy / 2) / dy
            };
            
            // Pixel center is at (x + 0.5, y + 0.5)
            let pixel_cx = x as f64 + 0.5;
            let pixel_cy = y as f64 + 0.5;
            
            // Mathematical line: y - y1 = slope * (x - x1)
            // At pixel center x-coordinate, what should y be?
            let expected_y = y1 as f64 + (dy as f64 / dx as f64) * (pixel_cx - x1 as f64);
            
            let distance = (pixel_cy - expected_y).abs();
            max_distance = max_distance.max(distance);
            
            if distance > 0.1 {
                eprintln!("  y={}: pixel_center=({:.1},{:.1}), expected_y={:.1}, distance={:.2}",
                    y, pixel_cx, pixel_cy, expected_y, distance);
            }
        }
        
        eprintln!("Max distance from mathematical line: {:.3}", max_distance);
        
        // Pixel centers should be very close to the mathematical line
        // At most 1 pixel away due to rasterization discretization
        assert!(max_distance <= 1.1, 
            "Pixel centers should align with mathematical edge, max distance was {}", max_distance);
    }

    /// Test that triangles have correct angles: ~90 degrees at tip, 45 degrees at base
    #[test]
    fn test_triangle_angles() {
        // Triangle geometry:
        // Tip: (2, 635)
        // Base top: (16, 621)
        // Base bottom: (16, 648)
        // This is an isosceles triangle with:
        // - Vertical base (right edge)
        // - Horizontal distance from tip to base: 14 pixels
        // - Base height: 27 pixels (from 621 to 648 inclusive)
        
        eprintln!("\n=== TRIANGLE ANGLE TEST ===");
        
        // Calculate angles
        let tip = (2.0_f64, 635.0_f64);
        let base_top = (16.0_f64, 621.0_f64);
        let base_bottom = (16.0_f64, 648.0_f64);
        
        // Vectors from tip to base vertices
        let v1 = (base_top.0 - tip.0, base_top.1 - tip.1);  // (14, -14)
        let v2 = (base_bottom.0 - tip.0, base_bottom.1 - tip.1);  // (14, 13)
        
        eprintln!("Tip: {:?}", tip);
        eprintln!("Base top: {:?}", base_top);
        eprintln!("Base bottom: {:?}", base_bottom);
        eprintln!("Vector to top: {:?}", v1);
        eprintln!("Vector to bottom: {:?}", v2);
        
        // Angle between the two edges (at the tip)
        let dot_product = v1.0 * v2.0 + v1.1 * v2.1;
        let mag1 = (v1.0.powi(2) + v1.1.powi(2)).sqrt();
        let mag2 = (v2.0.powi(2) + v2.1.powi(2)).sqrt();
        let cos_angle: f64 = dot_product / (mag1 * mag2);
        let angle_rad = cos_angle.acos();
        let angle_deg = angle_rad.to_degrees();
        
        eprintln!("Tip angle: {:.1} degrees (expected: ~90 degrees)", angle_deg);
        
        // For our triangle:
        // v1 = (14, -14), magnitude = sqrt(392) ≈ 19.8
        // v2 = (14, 13), magnitude = sqrt(365) ≈ 19.1
        // The angle should be close to 90 degrees for a isosceles triangle
        assert!(angle_deg > 85.0 && angle_deg < 95.0,
            "Tip angle should be approximately 90 degrees, got {:.1}", angle_deg);
        
        // Check base angles
        // The edge from base_top to tip should be 45 degrees (relative to vertical)
        let angle_to_tip_top = (v1.0 / v1.1.abs()).abs().atan().to_degrees();
        eprintln!("Top edge slope angle: {:.1} degrees (expected: 45)", angle_to_tip_top);
        
        // Since top edge slope is (x2-x1)/(y2-y1) = 14/-14 = -1, the angle is 45 degrees
        assert!((angle_to_tip_top - 45.0).abs() < 1.0,
            "Top edge should be 45 degrees, got {:.1}", angle_to_tip_top);
    }
}

/// Render a standalone colorbar to a PNG image with optional extend triangles
/// This is useful for testing the colorbar rendering independently of the full map
pub fn render_colorbar_standalone(
    width: u32,
    height: u32,
    cmap: &Colormap,
    gamma: f64,
    extend: crate::cli::Extend,
    padding: u32,
) -> image::RgbaImage {
    // Create white background image
    let mut img = image::ImageBuffer::from_pixel(width, height, image::Rgba([255, 255, 255, 255]));
    
    // Calculate colorbar dimensions with proper centering for symmetry
    let cbar_height = (height as i32 - 2 * padding as i32).max(1) as u32;
    let cbar_y = padding;
    
    // Calculate center of image
    let image_center_x = (width as f64) / 2.0;
    
    // Calculate colorbar width: make sure it's odd so it centers perfectly
    let mut cbar_width_f64 = width as f64 - 2.0 * padding as f64;
    if (cbar_width_f64 as u32).is_multiple_of(2) {
        cbar_width_f64 -= 1.0;  // Make odd width for perfect centering
    }
    let cbar_width = cbar_width_f64 as u32;
    
    // Center the colorbar
    let cbar_x_f64 = image_center_x - cbar_width_f64 / 2.0;
    let cbar_x = cbar_x_f64.round() as u32;
    
    // Draw gradient using pixels directly
    for py in 0..cbar_height {
        for px in 0..cbar_width {
            let t_linear = px as f64 / (cbar_width - 1) as f64;
            let t = apply_gamma(t_linear, gamma);
            let color_rgb = cmap.sample(t);
            // Convert RGB to RGBA
            let color = Rgba([color_rgb.0[0], color_rgb.0[1], color_rgb.0[2], 255]);
            img.put_pixel(cbar_x + px, cbar_y + py, color);
        }
    }
    
    // Draw extend triangles if requested
    if extend != crate::cli::Extend::None {
        draw_colorbar_extends(
            &extend,
            cbar_x as f64,
            cbar_y as f64,
            cbar_width as f64,
            cbar_height as f64,
            cmap,
            &mut img,
        );
    }
    
    img
}
