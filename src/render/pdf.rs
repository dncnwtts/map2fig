use crate::PixelSink;
use crate::colorbar::render_colorbar_gradient;
use crate::colorbar::{ColorbarTicks, format_tick_label_with_units, format_units_label};
use crate::latex_render;
use crate::layout::ColorbarLayout;
use crate::plot::rasterize_to_surface;
use crate::render::RenderBackend;
use crate::render::target::{PixelSource, RenderTarget};
use crate::scale::generate_colorbar_ticks;
use crate::{CairoImageSink, Colormap, Scale};
use cairo::{Context, Format, ImageSurface};
use std::f64::consts::PI;

pub struct PdfBackend<'a> {
    cr: &'a Context,
    width: f64,
    height: f64,
}

impl<'a> PdfBackend<'a> {
    pub fn new(cr: &'a Context, width: f64, height: f64) -> Self {
        Self { cr, width, height }
    }
}

impl<'a> RenderBackend for PdfBackend<'a> {
    fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.cr.set_source_rgba(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            a as f64 / 255.0,
        );
    }

    fn fill_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.cr.rectangle(x, y, w, h);
        self.cr.fill().unwrap();
    }

    fn stroke_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, width: f64) {
        self.cr.set_line_width(width);
        self.cr.move_to(x0, y0);
        self.cr.line_to(x1, y1);
        self.cr.stroke().unwrap();
    }

    fn draw_text(&mut self, x: f64, y: f64, size: f64, text: &str) {
        self.cr.set_font_size(size);

        // Try to use STIX fonts for mathematical text, fall back to default
        self.cr.set_font_face(
            &cairo::FontFace::toy_create(
                "STIXGeneral",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Normal,
            )
            .unwrap_or_else(|_| {
                // Fallback to default font if STIX is not available
                cairo::FontFace::toy_create(
                    "DejaVu Sans",
                    cairo::FontSlant::Normal,
                    cairo::FontWeight::Normal,
                )
                .unwrap()
            }),
        );

        self.cr.move_to(x, y);
        self.cr.show_text(text).unwrap();
    }

    fn width(&self) -> f64 {
        self.width
    }

    fn height(&self) -> f64 {
        self.height
    }

    fn draw_rect(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.cr.rectangle(x, y, w, h);
        let _ = self.cr.fill();
    }

    fn draw_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, width: f64) {
        self.cr.set_line_width(width);
        self.cr.move_to(x0, y0);
        self.cr.line_to(x1, y1);
        let _ = self.cr.stroke();
    }
}

pub fn draw_projection_border_pdf(
    cr: &Context,
    map_x: f64,
    map_y: f64,
    map_width: f64,
    map_height: f64,
    line_width_px: f64,
) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(line_width_px);
    cr.set_antialias(cairo::Antialias::Best);

    let n = 720;

    // Mollweide coordinates:
    // x ∈ [-2, 2]
    // y ∈ [-1, 1]
    let to_px = |x: f64| map_x + (x + 2.0) * 0.25 * map_width;
    let to_py = |y: f64| map_y + (1.0 - y) * 0.5 * map_height;

    for i in 0..=n {
        let t = 2.0 * PI * (i as f64 / n as f64);
        let x = 2.0 * t.cos();
        let y = t.sin();

        let px = to_px(x);
        let py = to_py(y);

        if i == 0 {
            cr.move_to(px, py);
        } else {
            cr.line_to(px, py);
        }
    }

    cr.close_path();
    cr.stroke().unwrap();
}

fn draw_colorbar_pdf_ticks(cr: &Context, layout: &ColorbarLayout, ticks: &ColorbarTicks) {
    let y0 = layout.y + layout.h;
    let major_len = layout.major_tick_height;
    let minor_len = layout.minor_tick_height;

    cr.set_source_rgb(0.0, 0.0, 0.0);

    // Determine tick direction (inward = -1, outward = +1)
    let direction = match layout.tick_direction {
        crate::cli::TickDirection::Inward => -1.0,
        crate::cli::TickDirection::Outward => 1.0,
    };

    // Minor ticks
    cr.set_line_width(layout.minor_tick_width);
    for (&t, &_val) in ticks.minor_positions.iter().zip(ticks.minor_values.iter()) {
        let x = t * (layout.w - 1.0) + layout.x;
        cr.move_to(x, y0);
        cr.line_to(x, y0 + direction * minor_len);
    }

    // Major ticks
    cr.set_line_width(layout.major_tick_width);
    for (&t, &_val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
        let x = t * (layout.w - 1.0) + layout.x;
        cr.move_to(x, y0);
        cr.line_to(x, y0 + direction * major_len);
    }

    cr.stroke().unwrap();
}

fn draw_colorbar_pdf_labels(
    cr: &Context,
    layout: &ColorbarLayout,
    ticks: &ColorbarTicks,
    scale: Scale,
    latex_rendering: bool,
    units: Option<&str>,
    units_font_size: Option<f32>,
    map_width: Option<f64>,
) {
    cr.set_source_rgb(0.0, 0.0, 0.0);

    // Use serif font for all text to match TeX/astronomy publication standards
    cr.select_font_face(
        "Liberation Serif",
        cairo::FontSlant::Normal,
        cairo::FontWeight::Normal,
    );

    // Tick font size scales with FOV like resolution label
    // Resolution label at FOV 300 is 11pt, tick labels at 3/4 of 1.5x = 12.375pt
    let adjusted_tick_font_size = if let Some(w) = map_width {
        // Scale by (w/300): FOV 300 = 12.375pt, FOV 600 = 24.75pt, etc.
        (16.5 * 0.75) * (w / 300.0)
    } else {
        layout.tick_font_size
    };
    cr.set_font_size(adjusted_tick_font_size);

    // Position labels below ticks, accounting for tick direction
    // For outward ticks, labels need extra space to avoid overlapping with ticks
    let base_label_y = layout.y + layout.h;
    let tick_label_offset = match layout.tick_direction {
        crate::cli::TickDirection::Outward => {
            layout.major_tick_height + 0.5 * layout.major_tick_height
        }
        crate::cli::TickDirection::Inward => 1.0 * layout.major_tick_height,
    };
    let label_y = base_label_y + tick_label_offset;

    // Draw tick labels at the computed position
    for (&t, &val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
        let label = format_tick_label_with_units(val, scale, Some(t), latex_rendering, units, true);
        let x = t * layout.w + layout.x;

        if latex_rendering {
            // Try to render LaTeX label as PNG
            // Wrap in math mode for proper LaTeX rendering
            let math_label = format!("${}", label);
            let math_label = format!("{}$", math_label);
            let font_size_pt = (adjusted_tick_font_size * 1.25) as u32;
            if let Some(rendered) = latex_render::render_latex_to_png(&math_label, font_size_pt) {
                // Embed the rendered LaTeX as an image
                embed_latex_tick_label(cr, &rendered, x, label_y);
            } else {
                // Fallback to plain text
                let ext = cr.text_extents(&label).unwrap();
                let tx = x - ext.width() / 2.0;
                cr.move_to(tx, label_y);
                cr.show_text(&label).unwrap();
            }
        } else {
            // Non-LaTeX rendering: use plain text
            let ext = cr.text_extents(&label).unwrap();
            let tx = x - ext.width() / 2.0;
            cr.move_to(tx, label_y);
            cr.show_text(&label).unwrap();
        }
    }

    // Draw units label below colorbar if specified
    if let Some(units_str) = units {
        // Account for tick direction: when ticks are outward, push units text down by the tick height
        let tick_offset = match layout.tick_direction {
            crate::cli::TickDirection::Outward => layout.major_tick_height,
            crate::cli::TickDirection::Inward => 0.0,
        };

        // Add significant vertical spacing to keep units well clear of tick labels
        // Use map_width-based scale for consistent spacing across FOV sizes
        let spacing_scale = map_width.map(|w| w / 300.0).unwrap_or(1.0);
        let vertical_gap = 12.0 * spacing_scale; // Extra spacing grows with FOV
        let units_y_pos = label_y + tick_offset + vertical_gap; // Position well below tick labels

        // Use a reasonable LaTeX font size for units text
        // Scale both font size and DPI with map width for proper scaling
        let dpi_scale = map_width.map(|w| w / 300.0).unwrap_or(1.0);
        // units_font_size already includes the (width/300) scaling, multiply by 1.25 for desired size
        let latex_font_size = ((units_font_size.unwrap_or(28.0) * 1.25) as u32).max(12);
        let latex_dpi = ((300.0 * dpi_scale) as u32).max(100); // Scale DPI with map width

        // Try LaTeX rendering at scaled DPI for crisp embedding
        if let Some(rendered) =
            latex_render::render_latex_to_hires_png(units_str, latex_font_size, latex_dpi)
        {
            // High resolution PNG - will be embedded at appropriate size for the map dimensions
            embed_latex_png_in_colorbar(cr, &rendered, layout.x, layout.w, units_y_pos);
        } else if let Some(units_label) = format_units_label(true, Some(units_str)) {
            // Fallback to direct Cairo text rendering if LaTeX fails
            // Use serif font to match TeX fonts used in astronomy publications
            cr.select_font_face(
                "Liberation Serif",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Bold,
            );
            // Use provided font size or default 14pt
            let font_size = units_font_size.unwrap_or(14.0) as f64;
            cr.set_font_size(font_size);
            let ext = cr.text_extents(&units_label).unwrap();
            let center_x = layout.x + layout.w / 2.0 - ext.width() / 2.0;
            cr.move_to(center_x, units_y_pos);
            cr.show_text(&units_label).unwrap();
        } else if let Some(units_label) = format_units_label(false, Some(units_str)) {
            // Final fallback to non-LaTeX plain text
            // Use serif font to match TeX fonts used in astronomy publications
            cr.select_font_face(
                "Liberation Serif",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Bold,
            );
            // Use provided font size or default 14pt
            let font_size = units_font_size.unwrap_or(14.0) as f64;
            cr.set_font_size(font_size);
            let ext = cr.text_extents(&units_label).unwrap();
            let center_x = layout.x + layout.w / 2.0 - ext.width() / 2.0;
            cr.move_to(center_x, units_y_pos);
            cr.show_text(&units_label).unwrap();
        }
    }
}

/// Embed a LaTeX PNG image in the colorbar with proper positioning and padding
/// The image is embedded at full resolution but displayed at scaled size (0.5x)
/// This preserves quality when zoomed in the PDF viewer
fn embed_latex_png_in_colorbar(
    cr: &Context,
    rendered: &latex_render::RenderedLatex,
    colorbar_x: f64,
    colorbar_width: f64,
    y_pos: f64,
) {
    // Convert PNG to Cairo surface
    if let Ok(img_buf) =
        image::load_from_memory_with_format(&rendered.image_data, image::ImageFormat::Png)
    {
        let rgba = img_buf.to_rgba8();

        // Create Cairo surface from the image
        if let Ok(mut surf) =
            ImageSurface::create(Format::ARgb32, rgba.width() as i32, rgba.height() as i32)
        {
            let surf_stride = surf.stride() as usize;
            {
                let mut surf_data = surf.data().expect("Failed to get surface data");

                // Copy image data with proper RGBA -> ARGB conversion
                let raw_data = rgba.as_raw();
                for (i, chunk) in raw_data.chunks_exact(4).enumerate() {
                    let y = i / rgba.width() as usize;
                    let x = i % rgba.width() as usize;
                    let dst_idx = y * surf_stride + x * 4;

                    if dst_idx + 3 < surf_data.len() {
                        // Copy as RGBA (Red, Green, Blue, Alpha)
                        surf_data[dst_idx] = chunk[0]; // R
                        surf_data[dst_idx + 1] = chunk[1]; // G
                        surf_data[dst_idx + 2] = chunk[2]; // B
                        surf_data[dst_idx + 3] = chunk[3]; // A
                    }
                }
            }
            surf.flush();

            // Find bounding box of non-transparent pixels for cropping info
            // but render the full image with transparency to preserve quality
            let mut min_y = rgba.height();
            let mut max_y = 0u32;

            for (_x, y, pixel) in rgba.enumerate_pixels() {
                if pixel[3] > 10 {
                    // Not fully transparent
                    min_y = min_y.min(y);
                    max_y = max_y.max(y);
                }
            }

            // Scale factor: 0.25 = quarter size for compact display (keep full 300 DPI quality)
            let scale_factor = 0.25;
            let scaled_width = (rgba.width() as f64) * scale_factor;
            let scaled_height = (rgba.height() as f64) * scale_factor;

            // Calculate centered position
            let center_x = colorbar_x + colorbar_width / 2.0 - scaled_width / 2.0;

            // Position text: place it below y_pos to avoid covering the colorbar
            // Use only the actual content height range to position properly
            let content_height = if min_y < max_y {
                ((max_y - min_y + 1) as f64) * scale_factor
            } else {
                scaled_height
            };

            let adjusted_y = y_pos - content_height * 0.75; // Move up slightly, leaving a small gap

            // Apply scale transform before drawing
            cr.save().unwrap();
            cr.translate(center_x, adjusted_y);
            cr.scale(scale_factor, scale_factor);
            cr.set_source_surface(&surf, 0.0, 0.0).unwrap();
            cr.paint().unwrap();
            cr.restore().unwrap();
        }
    }
}

/// Embed a LaTeX PNG image for a tick label, centered horizontally
fn embed_latex_tick_label(
    cr: &Context,
    rendered: &latex_render::RenderedLatex,
    x_pos: f64,
    y_pos: f64,
) {
    // Convert PNG to Cairo surface
    if let Ok(img_buf) =
        image::load_from_memory_with_format(&rendered.image_data, image::ImageFormat::Png)
    {
        let rgba = img_buf.to_rgba8();

        // Create surface with proper dimensions
        if let Ok(surf) = ImageSurface::create_for_data(
            rgba.clone().into_raw(),
            Format::ARgb32,
            rendered.width as i32,
            rendered.height as i32,
            rendered.width as i32 * 4,
        ) {
            // Scale for display (approximate pixel size)
            let scale_factor = 1.0;
            let display_width = rendered.width as f64 * scale_factor;
            let display_height = rendered.height as f64 * scale_factor;

            // Position: center horizontally and vertically
            let x = x_pos - display_width / 2.0;
            let y = y_pos - display_height / 2.0;

            // Translate and scale to draw the LaTeX image
            cr.save().unwrap();
            cr.translate(x, y);
            cr.scale(scale_factor, scale_factor);

            cr.set_source_surface(&surf, 0.0, 0.0).unwrap();
            cr.paint().unwrap();

            cr.restore().unwrap();
        }
    }
}

pub fn draw_colorbar_pdf(
    cr: &Context,
    cb_layout: ColorbarLayout,
    params: crate::params::ColorbarParams,
) {
    let ticks = generate_colorbar_ticks(
        params.minv,
        params.maxv,
        &params.scale_type,
        params.hist_scale,
    );

    let surf =
        ImageSurface::create(Format::ARgb32, cb_layout.w as i32, cb_layout.h as i32).unwrap();

    let surf_cr = cairo::Context::new(&surf).unwrap();
    surf_cr.set_operator(cairo::Operator::Source);
    surf_cr.set_antialias(cairo::Antialias::None);

    let surf = rasterize_to_surface(cb_layout.w as u32, cb_layout.h as u32, |sink| {
        render_colorbar_gradient(
            0,
            0,
            cb_layout.w as u32,
            cb_layout.h as u32,
            params.cmap,
            params.gamma,
            sink,
        );
    });

    let _ = cr.set_source_surface(&surf, cb_layout.x, cb_layout.y);
    cr.paint().unwrap();

    draw_colorbar_pdf_ticks(cr, &cb_layout, &ticks);

    draw_colorbar_pdf_labels(
        cr,
        &cb_layout,
        &ticks,
        params.scale_type,
        params.latex_rendering,
        params.units,
        params.units_font_size,
        params.map_width, // Pass map_width for DPI scaling
    );

    draw_colorbar_pdf_extends(cr, &cb_layout, params.extend, params.cmap);
}

pub struct PdfRenderTarget<'a> {
    pub cr: &'a cairo::Context,
}

impl RenderTarget for PdfRenderTarget<'_> {
    fn blit_raster(&mut self, raster: &dyn PixelSource, x: f64, y: f64) {
        let surface = ImageSurface::create(
            cairo::Format::ARgb32,
            raster.width() as i32,
            raster.height() as i32,
        )
        .unwrap();

        {
            let cr = cairo::Context::new(&surface).unwrap();
            let mut sink = CairoImageSink { cr: &cr };

            for py in 0..raster.height() {
                for px in 0..raster.width() {
                    let [r, g, b, a] = raster.get_pixel(px, py);
                    sink.draw_pixel(px, py, image::Rgba([r, g, b, a]));
                }
            }
        }

        surface.flush();
        let _ = self.cr.set_source_surface(&surface, x, y);
        self.cr.paint().unwrap();
    }
}

/// Draw colorbar extend arrows for PDF
fn draw_colorbar_pdf_extends(
    cr: &Context,
    layout: &ColorbarLayout,
    extend: &crate::cli::Extend,
    cmap: &Colormap,
) {
    use crate::cli::Extend;

    // Skip if no extends requested
    if extend == &Extend::None {
        return;
    }

    // Get colors for extend arrows from colormap endpoints
    let min_color_rgb = cmap.sample(0.0);
    let max_color_rgb = cmap.sample(1.0);

    // Isosceles triangles: tip distance is about half the colorbar height
    let tip_distance = layout.h * 0.5;

    // Line width for triangle outline
    cr.set_line_width(0.5);

    // Draw left arrow for min extend
    if matches!(extend, Extend::Min | Extend::Both) {
        // Left arrow: tip points left, base is at colorbar left edge
        let tip_x = layout.x - tip_distance;
        let base_x = layout.x;
        let base_top_y = layout.y;
        let base_bottom_y = layout.y + layout.h;
        let tip_y = layout.y + layout.h / 2.0;

        // Fill triangle with min color (no outline)
        cr.set_source_rgb(
            min_color_rgb.0[0] as f64 / 255.0,
            min_color_rgb.0[1] as f64 / 255.0,
            min_color_rgb.0[2] as f64 / 255.0,
        );
        cr.move_to(tip_x, tip_y);
        cr.line_to(base_x, base_top_y);
        cr.line_to(base_x, base_bottom_y);
        cr.close_path();
        cr.fill().unwrap();
    }

    // Draw right arrow for max extend
    if matches!(extend, Extend::Max | Extend::Both) {
        // Right arrow: tip points right, base is at colorbar right edge
        let tip_x = layout.x + layout.w + tip_distance;
        let base_x = layout.x + layout.w;
        let base_top_y = layout.y;
        let base_bottom_y = layout.y + layout.h;
        let tip_y = layout.y + layout.h / 2.0;

        // Fill triangle with max color (no outline)
        cr.set_source_rgb(
            max_color_rgb.0[0] as f64 / 255.0,
            max_color_rgb.0[1] as f64 / 255.0,
            max_color_rgb.0[2] as f64 / 255.0,
        );
        cr.move_to(tip_x, tip_y);
        cr.line_to(base_x, base_top_y);
        cr.line_to(base_x, base_bottom_y);
        cr.close_path();
        cr.fill().unwrap();
    }
}

/// Draw figure labels (rlabel, llabel) on PDF
/// Supports LaTeX rendering when latex_rendering is true
pub fn draw_figure_labels_pdf(
    cr: &Context,
    width: f64,
    height: f64,
    rlabel: &Option<String>,
    llabel: &Option<String>,
    latex_rendering: bool,
    label_font_size: Option<f32>,
) {
    // Calculate font size for labels
    // Default: 2pt larger than units label (which is 14pt)
    // Units label: 14pt * 0.5 ≈ 7pt, so labels default to ~16pt
    let scale = width / 800.0;
    let font_size = if let Some(size) = label_font_size {
        size as f64 * scale
    } else {
        // Default: 2pt larger than standard units label (14pt * scale + 2pt)
        14.0 * scale + 2.0
    };
    let font_size_pt = if let Some(size) = label_font_size {
        (size as u32).max(6)
    } else {
        ((14.0 * scale as f32 + 2.0) as u32).max(6)
    };

    // Set color to black
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_font_size(font_size);

    // Set up font for plain text
    cr.set_font_face(
        &cairo::FontFace::toy_create(
            "STIXGeneral",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        )
        .unwrap_or_else(|_| {
            cairo::FontFace::toy_create(
                "DejaVu Sans",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Normal,
            )
            .unwrap()
        }),
    );

    // Position labels with larger padding to prevent clipping at top
    // Position at the average of ellipse-relative and figure-relative positions
    let padding_x = 20.0 * scale; // Horizontal padding from edges
    let x_left = padding_x;
    let x_right = width - padding_x;
    let y_label = padding_x + (height * 0.095); // Average of two positions

    // Draw left label (llabel)
    if let Some(text) = llabel {
        if latex_rendering {
            // Try to render as LaTeX
            if let Some(rendered) = latex_render::render_latex_to_hires_png(text, font_size_pt, 150)
            {
                embed_latex_png_in_label(cr, &rendered, x_left, y_label, false);
            } else if let Some(rendered) = latex_render::render_latex_to_png(text, font_size_pt) {
                embed_latex_png_in_label(cr, &rendered, x_left, y_label, false);
            } else {
                // Fallback to plain text
                cr.move_to(x_left, y_label);
                cr.show_text(text).unwrap();
            }
        } else {
            cr.move_to(x_left, y_label);
            cr.show_text(text).unwrap();
        }
    }

    // Draw right label (rlabel)
    if let Some(text) = rlabel {
        if latex_rendering {
            // Try to render as LaTeX
            if let Some(rendered) = latex_render::render_latex_to_hires_png(text, font_size_pt, 150)
            {
                embed_latex_png_in_label(cr, &rendered, x_right, y_label, true);
            } else if let Some(rendered) = latex_render::render_latex_to_png(text, font_size_pt) {
                embed_latex_png_in_label(cr, &rendered, x_right, y_label, true);
            } else {
                // Fallback to plain text, right-aligned
                let extents = cr.text_extents(text).unwrap();
                let x = x_right - extents.x_advance();
                cr.move_to(x, y_label);
                cr.show_text(text).unwrap();
            }
        } else {
            let extents = cr.text_extents(text).unwrap();
            let x = x_right - extents.x_advance();
            cr.move_to(x, y_label);
            cr.show_text(text).unwrap();
        }
    }
}

/// Embed a LaTeX PNG image in a figure label with proper positioning
fn embed_latex_png_in_label(
    cr: &Context,
    rendered: &latex_render::RenderedLatex,
    x_pos: f64,
    y_pos: f64,
    right_aligned: bool,
) {
    // Convert PNG to Cairo surface
    if let Ok(img_buf) =
        image::load_from_memory_with_format(&rendered.image_data, image::ImageFormat::Png)
    {
        let rgba = img_buf.to_rgba8();

        // Create surface with proper dimensions
        if let Ok(surf) = ImageSurface::create_for_data(
            rgba.clone().into_raw(),
            Format::ARgb32,
            rendered.width as i32,
            rendered.height as i32,
            rendered.width as i32 * 4,
        ) {
            // Scale for display (approximate pixel size)
            let scale_factor = 1.0;
            let display_width = rendered.width as f64 * scale_factor;
            let display_height = rendered.height as f64 * scale_factor;

            // Position: adjust for right-alignment if needed
            let x = if right_aligned {
                x_pos - display_width
            } else {
                x_pos
            };

            // Translate and scale to draw the LaTeX image
            cr.save().unwrap();
            cr.translate(x, y_pos - display_height / 2.0);
            cr.scale(scale_factor, scale_factor);

            cr.set_source_surface(&surf, 0.0, 0.0).unwrap();
            cr.paint().unwrap();

            cr.restore().unwrap();
        }
    }
}
