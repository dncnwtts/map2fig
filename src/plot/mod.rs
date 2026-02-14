pub mod gnomonic;
pub mod hammer;
pub mod mollweide;

use crate::render::raster::RasterGrid;
use crate::{PixelSink, PixelValue};
use cairo::{Context, Format, ImageSurface};
use image::Rgba;
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale as FontScale};

/// Apply gamma correction with fast-paths for common values
///
/// Uses lookup table (LUT) for frequently-used gamma values to avoid expensive powf() calls.
/// Falls back to general powf() for arbitrary gamma values.
#[inline]
fn apply_gamma(t: f64, gamma: f64) -> f64 {
    // Common values that appear in astronomy and image processing
    match gamma {
        g if (g - 1.0).abs() < 1e-10 => t, // gamma=1.0: no-op (identity)
        g if (g - 2.0).abs() < 1e-10 => t * t, // gamma=2.0: square (brightens)
        g if (g - 0.5).abs() < 1e-10 => t.sqrt(), // gamma=0.5: square root (darkens)
        g if (g - 3.0).abs() < 1e-10 => t * t * t, // gamma=3.0: cube
        g if (g - 0.333).abs() < 1e-6 => t.powf(1.0 / 3.0), // gamma≈0.333: cube root
        _ => t.powf(gamma),                // General fallback
    }
}

// Re-export public APIs from projection modules
pub use gnomonic::{plot_gnomonic_auto, plot_gnomonic_pdf, plot_gnomonic_png};
pub use hammer::{plot_hammer_auto, plot_hammer_pdf, plot_hammer_png};
pub use mollweide::{
    compute_mollweide_scale, plot_mollweide_auto, plot_mollweide_pdf, plot_mollweide_png,
};

/// Rasterize to Cairo image surface
pub fn rasterize_to_surface<F>(width: u32, height: u32, render: F) -> ImageSurface
where
    F: FnOnce(&mut dyn PixelSink),
{
    let surf = ImageSurface::create(Format::ARgb32, width as i32, height as i32).unwrap();

    let cr = Context::new(&surf).unwrap();
    cr.set_operator(cairo::Operator::Source);
    cr.set_antialias(cairo::Antialias::None);

    let mut sink = crate::CairoRasterSink { cr: &cr };
    render(&mut sink);

    surf
}

/// Scale parameters for all mollweide/hammer-like projections
#[derive(Clone, Copy, Debug)]
pub struct MollweideScale {
    pub minv: f64,
    pub maxv: f64,
}

/// Render backend trait for graphics output
pub trait RenderBackend {
    fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8);
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64);
    fn stroke_path(&mut self);
    fn fill_path(&mut self);
    fn draw_text(&mut self, x: f64, y: f64, text: &str, size: f64);
    fn stroke_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, _width: f64);
    fn draw_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, _width: f64);
}

/// Debug overlay configuration for rendering
#[derive(Clone, Copy, Debug)]
pub struct DebugOverlay {
    pub enabled: bool,
    pub show_grid_box: bool,
    pub show_center: bool,
    pub show_background: bool,
}

impl DebugOverlay {
    pub fn off() -> Self {
        Self {
            enabled: false,
            show_grid_box: false,
            show_center: false,
            show_background: false,
        }
    }

    pub fn grid_only() -> Self {
        Self {
            enabled: true,
            show_grid_box: true,
            show_center: true,
            show_background: false,
        }
    }

    pub fn with_background() -> Self {
        Self {
            enabled: true,
            show_grid_box: true,
            show_center: true,
            show_background: true,
        }
    }
}

/// Compute percentile of a sorted array
pub fn percentile(sorted: &[f64], p: f64) -> f64 {
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

/// Draw figure labels (rlabel, llabel) on PNG image
/// Supports LaTeX rendering when latex_rendering is true
pub fn draw_figure_labels_png(
    img: &mut image::RgbaImage,
    width: u32,
    height: u32,
    rlabel: &Option<String>,
    llabel: &Option<String>,
    latex_rendering: bool,
    label_font_size: Option<f32>,
) {
    // Calculate font size for labels
    let scale = width as f64 / 800.0;
    let font_size = if let Some(size) = label_font_size {
        size * scale as f32
    } else {
        (14.0 * scale as f32 + 2.0).max(6.0)
    };
    let font_size_pt = font_size as u32;
    let font_scale = FontScale::uniform(font_size);

    let font_data = include_bytes!("../../assets/fonts/DejaVuSans.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Failed to load font");

    let text_color = Rgba([0, 0, 0, 255]); // Black text

    // Position labels with larger padding to prevent clipping at top
    let padding_x = 20.0 * scale;
    let x_left = padding_x as i32;
    let x_right = (width as f64 - padding_x) as i32;
    let y_label = (padding_x + (height as f64 * 0.095)) as i32;

    // Draw left label (llabel) - top left, left-aligned
    if let Some(text) = llabel {
        if latex_rendering {
            // Try to render as LaTeX
            if let Some(rendered) = crate::latex_render::render_latex_to_png(text, font_size_pt) {
                // Composite the rendered LaTeX PNG onto the main image
                let latex_img = image::load_from_memory(&rendered.image_data)
                    .expect("Failed to load rendered LaTeX");
                let latex_rgba = latex_img.to_rgba8();

                // Composite with alpha blending (left-aligned)
                for (lx, ly, pixel) in latex_rgba.enumerate_pixels() {
                    let img_x = x_left + lx as i32;
                    let img_y = y_label + ly as i32;

                    if img_x >= 0 && img_x < width as i32 && img_y >= 0 && img_y < height as i32 {
                        let alpha = pixel[3] as f32 / 255.0;
                        if alpha > 0.01 {
                            let existing = img.get_pixel(img_x as u32, img_y as u32);
                            let blended = Rgba([
                                ((pixel[0] as f32 * alpha + existing[0] as f32 * (1.0 - alpha))
                                    as u8),
                                ((pixel[1] as f32 * alpha + existing[1] as f32 * (1.0 - alpha))
                                    as u8),
                                ((pixel[2] as f32 * alpha + existing[2] as f32 * (1.0 - alpha))
                                    as u8),
                                255,
                            ]);
                            img.put_pixel(img_x as u32, img_y as u32, blended);
                        }
                    }
                }
            } else {
                // Fallback to plain text
                draw_text_mut(img, text_color, x_left, y_label, font_scale, &font, text);
            }
        } else {
            draw_text_mut(img, text_color, x_left, y_label, font_scale, &font, text);
        }
    }

    // Draw right label (rlabel) - top right, right-aligned
    if let Some(text) = rlabel {
        if latex_rendering {
            // Try to render as LaTeX
            if let Some(rendered) = crate::latex_render::render_latex_to_png(text, font_size_pt) {
                // Composite the rendered LaTeX PNG onto the main image (right-aligned)
                let latex_img = image::load_from_memory(&rendered.image_data)
                    .expect("Failed to load rendered LaTeX");
                let latex_rgba = latex_img.to_rgba8();

                // Composite with alpha blending (right-aligned)
                let latex_width = latex_rgba.width() as i32;
                for (lx, ly, pixel) in latex_rgba.enumerate_pixels() {
                    let img_x = x_right - latex_width + lx as i32;
                    let img_y = y_label + ly as i32;

                    if img_x >= 0 && img_x < width as i32 && img_y >= 0 && img_y < height as i32 {
                        let alpha = pixel[3] as f32 / 255.0;
                        if alpha > 0.01 {
                            let existing = img.get_pixel(img_x as u32, img_y as u32);
                            let blended = Rgba([
                                ((pixel[0] as f32 * alpha + existing[0] as f32 * (1.0 - alpha))
                                    as u8),
                                ((pixel[1] as f32 * alpha + existing[1] as f32 * (1.0 - alpha))
                                    as u8),
                                ((pixel[2] as f32 * alpha + existing[2] as f32 * (1.0 - alpha))
                                    as u8),
                                255,
                            ]);
                            img.put_pixel(img_x as u32, img_y as u32, blended);
                        }
                    }
                }
            } else {
                // Fallback to plain text (right-aligned)
                let text_width = (text.len() as f32 * (font_size / 2.0)) as i32;
                let x = (x_right - text_width).max(0);
                draw_text_mut(img, text_color, x, y_label, font_scale, &font, text);
            }
        } else {
            let text_width = (text.len() as f32 * (font_size / 2.0)) as i32;
            let x = (x_right - text_width).max(0);
            draw_text_mut(img, text_color, x, y_label, font_scale, &font, text);
        }
    }
}

/// Render projection to grid
pub fn render_projection_to_grid(params: crate::params::RenderGridParams, grid: &mut RasterGrid) {
    let width = grid.width;
    let height = grid.height;

    // Precompute gamma value to avoid repeated checks
    let gamma_inv = if (params.gamma - 1.0).abs() < f64::EPSILON {
        1.0
    } else {
        params.gamma
    };

    // Process rows using batch operations for vectorization opportunity
    // Tier 5: Process 16 pixels at a time for improved cache locality and throughput
    for py in 0..height {
        let mut px: u32 = 0;

        // Tier 5 16-pixel batch loop: process 16 pixels at a time
        while px + 16 <= width {
            // First batch of 8 pixels
            let mut px_array_lo = [0u32; 8];
            let py_array_lo = [py; 8];
            for (i, item) in px_array_lo.iter_mut().enumerate() {
                *item = px + i as u32;
            }

            let (lons_lo, lats_lo, proj_mask_lo) =
                params
                    .proj
                    .pixel_to_ang_batch(&px_array_lo, &py_array_lo, grid);

            // Convert latitudes to theta values
            let mut thetas_lo = [0.0_f64; 8];
            for i in 0..8 {
                if proj_mask_lo[i] {
                    thetas_lo[i] = std::f64::consts::PI / 2.0 - lats_lo[i];
                }
            }

            // Sample HEALPix for first batch
            let (healpix_values_lo, healpix_mask_lo) = crate::healpix::sample_healpix_batch_simd(
                params.map,
                params.meta,
                params.view,
                &thetas_lo,
                &lons_lo,
            );

            // Combine masks for first batch
            let validity_mask_lo: [bool; 8] = [
                proj_mask_lo[0] && healpix_mask_lo[0],
                proj_mask_lo[1] && healpix_mask_lo[1],
                proj_mask_lo[2] && healpix_mask_lo[2],
                proj_mask_lo[3] && healpix_mask_lo[3],
                proj_mask_lo[4] && healpix_mask_lo[4],
                proj_mask_lo[5] && healpix_mask_lo[5],
                proj_mask_lo[6] && healpix_mask_lo[6],
                proj_mask_lo[7] && healpix_mask_lo[7],
            ];

            // Second batch of 8 pixels
            let mut px_array_hi = [0u32; 8];
            let py_array_hi = [py; 8];
            for (i, item) in px_array_hi.iter_mut().enumerate() {
                *item = px + 8 + i as u32;
            }

            let (lons_hi, lats_hi, proj_mask_hi) =
                params
                    .proj
                    .pixel_to_ang_batch(&px_array_hi, &py_array_hi, grid);

            // Convert latitudes to theta values
            let mut thetas_hi = [0.0_f64; 8];
            for i in 0..8 {
                if proj_mask_hi[i] {
                    thetas_hi[i] = std::f64::consts::PI / 2.0 - lats_hi[i];
                }
            }

            // Sample HEALPix for second batch
            let (healpix_values_hi, healpix_mask_hi) = crate::healpix::sample_healpix_batch_simd(
                params.map,
                params.meta,
                params.view,
                &thetas_hi,
                &lons_hi,
            );

            // Combine masks for second batch
            let validity_mask_hi: [bool; 8] = [
                proj_mask_hi[0] && healpix_mask_hi[0],
                proj_mask_hi[1] && healpix_mask_hi[1],
                proj_mask_hi[2] && healpix_mask_hi[2],
                proj_mask_hi[3] && healpix_mask_hi[3],
                proj_mask_hi[4] && healpix_mask_hi[4],
                proj_mask_hi[5] && healpix_mask_hi[5],
                proj_mask_hi[6] && healpix_mask_hi[6],
                proj_mask_hi[7] && healpix_mask_hi[7],
            ];

            // Merge 16 elements for scaling
            let mut healpix_values_16 = [0.0; 16];
            let mut validity_mask_16 = [false; 16];
            let mut thetas_16 = [0.0; 16];
            let mut lons_16 = [0.0; 16];
            healpix_values_16[..8].copy_from_slice(&healpix_values_lo);
            validity_mask_16[..8].copy_from_slice(&validity_mask_lo);
            thetas_16[..8].copy_from_slice(&thetas_lo);
            lons_16[..8].copy_from_slice(&lons_lo);
            healpix_values_16[8..16].copy_from_slice(&healpix_values_hi);
            validity_mask_16[8..16].copy_from_slice(&validity_mask_hi);
            thetas_16[8..16].copy_from_slice(&thetas_hi);
            lons_16[8..16].copy_from_slice(&lons_hi);

            // Tier 5: Use 16-element SIMD scaling for improved throughput
            let pixel_values: [PixelValue; 16] = if matches!(
                params.scale_type,
                crate::scale::Scale::Linear | crate::scale::Scale::Log
            ) {
                let use_log = matches!(params.scale_type, crate::scale::Scale::Log);
                let log_cache = if use_log && params.scale_cache.is_some() {
                    let cache = params.scale_cache.as_ref().unwrap();
                    Some((cache.log_min, cache.log_range))
                } else {
                    None
                };

                // 16-element batch scaling
                let (scaled_values, out_mask) = crate::simd::simd_batch_scale_16(
                    healpix_values_16,
                    params.scale.minv,
                    params.scale.maxv,
                    use_log,
                    log_cache,
                    validity_mask_16,
                );

                // Convert to PixelValue array
                crate::simd::simd_to_pixel_values_16(scaled_values, out_mask)
            } else {
                // Fallback to scalar path for non-linear/log scales
                let mut result = [PixelValue::Bad; 16];
                for i in 0..16 {
                    result[i] = if validity_mask_16[i] {
                        crate::scale::scale_value(
                            healpix_values_16[i],
                            params.scale.minv,
                            params.scale.maxv,
                            params.scale_type,
                            params.neg_mode,
                            params.hist_scale,
                            params.scale_cache,
                        )
                    } else {
                        PixelValue::Bad
                    };
                }
                result
            };

            // Process 16 pixels in parallel
            for i in 0..16 {
                let pixel_x = px + i as u32;
                let pixel_valid = validity_mask_16[i];
                let pixel_val = pixel_values[i];

                // Convert to RGBA
                let mut rgba = match pixel_val {
                    PixelValue::Color(t) => {
                        let t = apply_gamma(t, gamma_inv);
                        let c = params.cmap.sample(t);
                        Rgba([c[0], c[1], c[2], 255])
                    }
                    PixelValue::Underflow => {
                        let c = params.cmap.sample(0.0);
                        Rgba([c[0], c[1], c[2], 255])
                    }
                    PixelValue::Overflow => {
                        let c = params.cmap.sample(1.0);
                        Rgba([c[0], c[1], c[2], 255])
                    }
                    PixelValue::Bad => params.bad_color,
                };

                // Apply mask if present
                if let Some(mask) = params.mask {
                    let healpix_idx = crate::healpix::sample_healpix_index(
                        params.map,
                        params.meta,
                        params.view,
                        thetas_16[i],
                        lons_16[i],
                    );
                    if let Some(idx) = healpix_idx
                        && !mask.is_valid(idx)
                        && let Some(fill_color) = mask.fill_color
                    {
                        rgba = fill_color;
                    }
                }

                // Draw pixel
                if pixel_valid {
                    unsafe {
                        grid.set_pixel_unchecked(pixel_x, py, rgba);
                    }
                } else {
                    grid.set_valid(pixel_x, py, false);
                }
            }

            px += 16;
        }

        // Tier 5: Handle remainder with 8-pixel batch (pixels 16-23 if width > 16)
        while px + 8 <= width {
            // Prepare 8-pixel batch
            let mut px_array = [0u32; 8];
            let py_array = [py; 8];
            for (i, item) in px_array.iter_mut().enumerate() {
                *item = px + i as u32;
            }

            // Batch projection
            let (lons, lats, proj_mask) =
                params.proj.pixel_to_ang_batch(&px_array, &py_array, grid);

            // Convert to theta
            let mut thetas = [0.0_f64; 8];
            for i in 0..8 {
                if proj_mask[i] {
                    thetas[i] = std::f64::consts::PI / 2.0 - lats[i];
                }
            }

            // HEALPix sampling
            let (healpix_values, healpix_mask) = crate::healpix::sample_healpix_batch_simd(
                params.map,
                params.meta,
                params.view,
                &thetas,
                &lons,
            );

            // Combine masks
            let validity_mask: [bool; 8] = [
                proj_mask[0] && healpix_mask[0],
                proj_mask[1] && healpix_mask[1],
                proj_mask[2] && healpix_mask[2],
                proj_mask[3] && healpix_mask[3],
                proj_mask[4] && healpix_mask[4],
                proj_mask[5] && healpix_mask[5],
                proj_mask[6] && healpix_mask[6],
                proj_mask[7] && healpix_mask[7],
            ];

            // Scaling
            let pixel_values: [PixelValue; 8] = if matches!(
                params.scale_type,
                crate::scale::Scale::Linear | crate::scale::Scale::Log
            ) {
                let use_log = matches!(params.scale_type, crate::scale::Scale::Log);
                let log_cache = if use_log && params.scale_cache.is_some() {
                    let cache = params.scale_cache.as_ref().unwrap();
                    Some((cache.log_min, cache.log_range))
                } else {
                    None
                };

                let (scaled_values, out_mask) = crate::simd::simd_batch_scale_8(
                    healpix_values,
                    params.scale.minv,
                    params.scale.maxv,
                    use_log,
                    log_cache,
                    validity_mask,
                );

                crate::simd::simd_to_pixel_values(scaled_values, out_mask)
            } else {
                let mut result = [PixelValue::Bad; 8];
                for i in 0..8 {
                    result[i] = if validity_mask[i] {
                        crate::scale::scale_value(
                            healpix_values[i],
                            params.scale.minv,
                            params.scale.maxv,
                            params.scale_type,
                            params.neg_mode,
                            params.hist_scale,
                            params.scale_cache,
                        )
                    } else {
                        PixelValue::Bad
                    };
                }
                result
            };

            // Process 8 pixels
            for i in 0..8 {
                let pixel_x = px + i as u32;
                let pixel_valid = validity_mask[i];
                let pixel_val = pixel_values[i];

                let mut rgba = match pixel_val {
                    PixelValue::Color(t) => {
                        let t = apply_gamma(t, gamma_inv);
                        let c = params.cmap.sample(t);
                        Rgba([c[0], c[1], c[2], 255])
                    }
                    PixelValue::Underflow => {
                        let c = params.cmap.sample(0.0);
                        Rgba([c[0], c[1], c[2], 255])
                    }
                    PixelValue::Overflow => {
                        let c = params.cmap.sample(1.0);
                        Rgba([c[0], c[1], c[2], 255])
                    }
                    PixelValue::Bad => params.bad_color,
                };

                if let Some(mask) = params.mask {
                    let healpix_idx = crate::healpix::sample_healpix_index(
                        params.map,
                        params.meta,
                        params.view,
                        thetas[i],
                        lons[i],
                    );
                    if let Some(idx) = healpix_idx
                        && !mask.is_valid(idx)
                        && let Some(fill_color) = mask.fill_color
                    {
                        rgba = fill_color;
                    }
                }

                if pixel_valid {
                    unsafe {
                        grid.set_pixel_unchecked(pixel_x, py, rgba);
                    }
                } else {
                    grid.set_valid(pixel_x, py, false);
                }
            }

            px += 8;
        }

        // Scalar fallback: process remaining pixels (0-7 pixels)
        while px < width {
            if let Some((lon, lat)) = params.proj.pixel_to_ang(px, py, grid) {
                let theta = std::f64::consts::PI / 2.0 - lat;

                let pixel_val = match crate::healpix::sample_healpix(
                    params.map,
                    params.meta,
                    params.view,
                    theta,
                    lon,
                ) {
                    Some(val) => crate::scale::scale_value(
                        val,
                        params.scale.minv,
                        params.scale.maxv,
                        params.scale_type,
                        params.neg_mode,
                        params.hist_scale,
                        params.scale_cache,
                    ),
                    None => PixelValue::Bad,
                };

                let mut rgba = match pixel_val {
                    PixelValue::Color(t) => {
                        let t = apply_gamma(t, gamma_inv);
                        let c = params.cmap.sample(t);
                        Rgba([c[0], c[1], c[2], 255])
                    }
                    PixelValue::Underflow => {
                        let c = params.cmap.sample(0.0);
                        Rgba([c[0], c[1], c[2], 255])
                    }
                    PixelValue::Overflow => {
                        let c = params.cmap.sample(1.0);
                        Rgba([c[0], c[1], c[2], 255])
                    }
                    PixelValue::Bad => params.bad_color,
                };

                if let Some(mask) = params.mask {
                    let healpix_idx = crate::healpix::sample_healpix_index(
                        params.map,
                        params.meta,
                        params.view,
                        theta,
                        lon,
                    );
                    if let Some(idx) = healpix_idx
                        && !mask.is_valid(idx)
                        && let Some(fill_color) = mask.fill_color
                    {
                        rgba = fill_color;
                    }
                }

                unsafe {
                    grid.set_pixel_unchecked(px, py, rgba);
                }
            } else {
                grid.set_valid(px, py, false);
            }
            px += 1;
        }
    }
}

/// Blit grid to sink
pub fn blit_grid_to_sink(grid: &RasterGrid, sink: &mut dyn PixelSink, x0: u32, y0: u32) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            if let Some(p) = grid.get_pixel_if_valid(x, y) {
                sink.draw_pixel(x0 + x, y0 + y, p);
            }
        }
    }
}

/// Draw debug overlay on raster grid
pub fn draw_debug_overlay_raster(grid: &mut RasterGrid, overlay: DebugOverlay) {
    if !overlay.enabled {
        return;
    }

    let w = grid.width;
    let h = grid.height;
    let red = [255, 0, 0, 160];

    if overlay.show_grid_box {
        for x in 0..w {
            grid.set_pixel_array(x, 0, red);
            grid.set_pixel_array(x, h - 1, red);
        }
        for y in 0..h {
            grid.set_pixel_array(0, y, red);
            grid.set_pixel_array(w - 1, y, red);
        }
    }

    if overlay.show_center {
        let cx = w / 2;
        let cy = h / 2;

        for dx in cx.saturating_sub(10)..=(cx + 10).min(w - 1) {
            grid.set_pixel_array(dx, cy, red);
        }
        for dy in cy.saturating_sub(10)..=(cy + 10).min(h - 1) {
            grid.set_pixel_array(cx, dy, red);
        }
    }
}

/// Fill grid with background color
pub fn fill_grid_background(grid: &mut RasterGrid) {
    let bg = Rgba([220, 220, 220, 255]); // Light gray

    for y in 0..grid.height {
        for x in 0..grid.width {
            grid.set_pixel(x, y, bg);
        }
    }
}
