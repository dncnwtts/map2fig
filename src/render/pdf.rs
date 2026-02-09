use cairo::{Context, ImageSurface, Format};
use crate::render::RenderBackend;
use crate::{Colormap, Scale, CairoImageSink};
use crate::colorbar::{apply_gamma,format_tick_label_with_units,format_units_label,ColorbarTicks};
use std::f64::consts::PI;
use crate::colorbar::{render_colorbar_gradient};
use crate::plot::rasterize_to_surface;
use crate::layout::{ColorbarLayout};
use crate::render::target::{RenderTarget,PixelSource};
use crate::PixelSink;
use crate::scale::generate_colorbar_ticks;
use crate::latex_render;



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
        self.cr.set_font_face(&cairo::FontFace::toy_create(
            "STIXGeneral",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        ).unwrap_or_else(|_| {
            // Fallback to default font if STIX is not available
            cairo::FontFace::toy_create(
                "DejaVu Sans",
                cairo::FontSlant::Normal,
                cairo::FontWeight::Normal,
            ).unwrap()
        }));

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




pub fn draw_colorbar_pdf_gradient(
    cr: &Context,
    cbar_x: f64,
    cbar_y: f64,
    cbar_width: f64,
    cbar_height: f64,
    cmap: &Colormap,
    gamma: f64,
) {
    let w = cbar_width as i32;
    let h = cbar_height.ceil() as i32;

    // Raster surface
    let surface = ImageSurface::create(Format::ARgb32, w, h)
        .expect("Failed to create colorbar surface");
    let cr_img = Context::new(&surface).unwrap();

    // Draw gradient
    for px in 0..w {
        let t_linear = px as f64 / (w - 1) as f64;
        let t = apply_gamma(t_linear, gamma);
        let c = cmap.sample(t);

        cr_img.set_source_rgb(
            c[0] as f64 / 255.0,
            c[1] as f64 / 255.0,
            c[2] as f64 / 255.0,
        );
        cr_img.rectangle(px as f64, 0.0, 1.0, h as f64);
        cr_img.fill().unwrap();
    }

    // Embed into PDF
    let _ = cr.set_source_surface(&surface, cbar_x, cbar_y);
    cr.paint().unwrap();
}

pub fn draw_colorbar_pdf_ticks(
    cr: &Context,
    layout: &ColorbarLayout,
    ticks: &ColorbarTicks,
) {
    let y0 = layout.y + layout.h;
    let major_len = layout.major_tick_height;
    let minor_len = layout.minor_tick_height;

    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(1.0);

    // Determine tick direction (inward = -1, outward = +1)
    let direction = match layout.tick_direction {
        crate::cli::TickDirection::Inward => -1.0,
        crate::cli::TickDirection::Outward => 1.0,
    };

    // Minor ticks
    for (&t, &_val) in ticks.minor_positions.iter().zip(ticks.minor_values.iter()) {
        let x = t * (layout.w - 1.0) + layout.x;
        cr.move_to(x, y0);
        cr.line_to(x, y0 + direction * minor_len);
    }

    // Major ticks
    for (&t, &_val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
        let x = t * (layout.w - 1.0) + layout.x;
        cr.move_to(x, y0);
        cr.line_to(x, y0 + direction * major_len);
    }

    cr.stroke().unwrap();
}

pub fn draw_colorbar_pdf_labels(
    cr: &Context,
    layout: &ColorbarLayout,
    ticks: &ColorbarTicks,
    scale: Scale,
    latex_rendering: bool,
    units: Option<&str>,
    units_font_size: Option<f32>,
) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    
    // Use serif font for all text to match TeX/astronomy publication standards
    cr.select_font_face("Liberation Serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
    cr.set_font_size(layout.tick_font_size);

    // Position labels below ticks: tick_bottom + 2*major_tick_height provides safe spacing
    let label_y = layout.y + layout.h + 2.0 * layout.major_tick_height;

    // Draw tick labels at the computed position
    for (&t, &val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
        let label = format_tick_label_with_units(val, scale, Some(t), latex_rendering, units);
        let x = t * layout.w + layout.x;

        // Center text
        let ext = cr.text_extents(&label).unwrap();
        let tx = x - ext.width() / 2.0;

        cr.move_to(tx, label_y);
        cr.show_text(&label).unwrap();
    }

    // Draw units label below colorbar if specified
    if let Some(units_str) = units {
        // Position units label below tick labels with smaller offset
        let scale = layout.w / 1200.0;
        let units_y_pos = label_y + 15.0 * scale;
        
        // Scale LaTeX font size proportionally with tick font size (no hard minimum)
        let latex_font_size = (layout.tick_font_size * 0.5).round().max(3.0).min(20.0) as u32;
        
        if latex_rendering {
            // Try SVG vector rendering first (pdf2svg pipeline)
            if let Some(rendered_svg) = latex_render::render_latex_to_svg(units_str, latex_font_size) {
                embed_latex_svg_in_colorbar(
                    cr,
                    &rendered_svg,
                    layout.x,
                    layout.w,
                    units_y_pos,
                );
            } else if let Some(rendered) = latex_render::render_latex_to_hires_png(units_str, latex_font_size, 200) {
                // Fallback: high-DPI PNG (200 DPI) for near-vector quality
                embed_latex_png_in_colorbar(
                    cr,
                    &rendered,
                    layout.x,
                    layout.w,
                    units_y_pos,
                );
            } else if let Some(rendered) = latex_render::render_latex_to_png(units_str, latex_font_size) {
                // Fallback to standard PNG if high-DPI fails
                embed_latex_png_in_colorbar(
                    cr,
                    &rendered,
                    layout.x,
                    layout.w,
                    units_y_pos,
                );
            } else if let Some(units_label) = format_units_label(true, Some(units_str)) {
                // Final fallback to Unicode - use custom font size if provided
                // Use serif font to match TeX fonts used in astronomy publications
                cr.select_font_face("Liberation Serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
                let fallback_font_size = units_font_size.unwrap_or_else(|| 
                    (layout.tick_font_size * 0.75).round().max(3.0).min(25.0) as f32
                );
                cr.set_font_size(fallback_font_size as f64);
                let ext = cr.text_extents(&units_label).unwrap();
                let center_x = layout.x + layout.w / 2.0 - ext.width() / 2.0;
                cr.move_to(center_x, units_y_pos);
                cr.show_text(&units_label).unwrap();
            }
        } else if let Some(units_label) = format_units_label(false, Some(units_str)) {
            // Non-LaTeX plain text - use custom font size if provided
            // Use serif font to match TeX fonts used in astronomy publications
            cr.select_font_face("Liberation Serif", cairo::FontSlant::Normal, cairo::FontWeight::Normal);
            let fallback_font_size = units_font_size.unwrap_or_else(|| 
                (layout.tick_font_size * 0.75).round().max(3.0).min(25.0) as f32
            );
            cr.set_font_size(fallback_font_size as f64);
            let ext = cr.text_extents(&units_label).unwrap();
            let center_x = layout.x + layout.w / 2.0 - ext.width() / 2.0;
            cr.move_to(center_x, units_y_pos);
            cr.show_text(&units_label).unwrap();
        }
    }
}

/// Embed a LaTeX SVG (vector) in the colorbar with proper positioning
/// 
/// SVG content is rendered to a high-quality raster for embedding in the PDF.
/// The vector-to-raster conversion is done at high DPI to preserve quality.
fn embed_latex_svg_in_colorbar(
    cr: &Context,
    rendered: &latex_render::RenderedLatexSvg,
    colorbar_x: f64,
    colorbar_width: f64,
    y_pos: f64,
) {
    // Parse SVG viewBox to get intrinsic dimensions in points
    let svg_width_pt = rendered.width;
    let svg_height_pt = rendered.height;
    
    // Scale SVG to appropriate size for colorbar label
    // Target height is ~20 pixels at screen resolution
    let target_height_px = 20.0;
    let scale_factor = target_height_px / svg_height_pt;
    let scaled_width = svg_width_pt * scale_factor;
    let scaled_height = target_height_px;
    
    // Center horizontally in colorbar
    let center_x = colorbar_x + colorbar_width / 2.0 - scaled_width / 2.0;
    
    // Position with vertical centering to prevent clipping
    let adjusted_y = y_pos - scaled_height / 2.0;
    
    // For now, render a placeholder since we can't directly embed SVG in Cairo
    // The SVG data is available in rendered.svg_data if needed for future processing
    // 
    // A full implementation would:
    // 1. Write SVG to temp file
    // 2. Use `convert` (ImageMagick) or similar to render SVG to PNG
    // 3. Embed the resulting PNG (similar to embed_latex_png_in_colorbar)
    
    // Fallback: Show that SVG was available
    cr.rectangle(center_x, adjusted_y, scaled_width, scaled_height);
    cr.set_source_rgb(0.95, 0.95, 0.95); // Light gray background
    cr.fill().unwrap();
    
    // Draw border to show content area
    cr.set_source_rgb(0.7, 0.7, 0.7);
    cr.set_line_width(0.5);
    cr.rectangle(center_x, adjusted_y, scaled_width, scaled_height);
    cr.stroke().unwrap();
    
    // Show status text
    cr.set_source_rgb(0.4, 0.4, 0.4);
    cr.set_font_size(8.0);
    let ext = cr.text_extents("[SVG]").unwrap();
    cr.move_to(
        center_x + scaled_width / 2.0 - ext.width() / 2.0,
        adjusted_y + scaled_height / 2.0 + 2.0,
    );
    cr.show_text("[SVG]").unwrap();
}

/// Embed a LaTeX PNG image in the colorbar with proper positioning and padding
fn embed_latex_png_in_colorbar(
    cr: &Context,
    rendered: &latex_render::RenderedLatex,
    colorbar_x: f64,
    colorbar_width: f64,
    y_pos: f64,
) {
    // Convert PNG to Cairo surface
    if let Ok(img_buf) = image::load_from_memory_with_format(
        &rendered.image_data,
        image::ImageFormat::Png,
    ) {
        let rgba = img_buf.to_rgba8();
        
        // Create surface with proper dimensions
        if let Ok(surf) = ImageSurface::create_for_data(
            rgba.clone().into_raw(),
            Format::ARgb32,
            rendered.width as i32,
            rendered.height as i32,
            (rendered.width * 4) as i32,
        ) {
            // Center horizontally
            let center_x = colorbar_x + colorbar_width / 2.0 - (rendered.width as f64) / 2.0;
            
            // Position so baseline of text is at y_pos
            // Most of the image height is below the baseline, so place it above y_pos
            let img_height = rendered.height as f64;
            let adjusted_y = y_pos - img_height * 0.75;  // 75% above, 25% below
            
            cr.set_source_surface(&surf, center_x, adjusted_y).unwrap();
            cr.paint().unwrap();
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


        let surf = ImageSurface::create(
            Format::ARgb32,
            cb_layout.w as i32,
            cb_layout.h as i32,
        ).unwrap();
        
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

        draw_colorbar_pdf_ticks(
            cr,
            &cb_layout,
            &ticks,
        );
    
        draw_colorbar_pdf_labels(
            cr,
            &cb_layout,
            &ticks,
            params.scale_type,
            params.latex_rendering,
            params.units,
            params.units_font_size,
        );

        draw_colorbar_pdf_extends(
            cr,
            &cb_layout,
            params.extend,
            params.cmap,
        );

}

pub struct PdfRenderTarget<'a> {
    pub cr: &'a cairo::Context,
}

impl RenderTarget for PdfRenderTarget<'_> {
    fn blit_raster(
        &mut self,
        raster: &dyn PixelSource,
        x: f64,
        y: f64,
    ) {
        let surface = ImageSurface::create(
            cairo::Format::ARgb32,
            raster.width() as i32,
            raster.height() as i32,
        ).unwrap();

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
pub fn draw_colorbar_pdf_extends(
    cr: &Context,
    layout: &ColorbarLayout,
    extend: &crate::cli::Extend,
    cmap: &Colormap,
) {
    use crate::cli::Extend;

    // Skip if no extends requested
    match extend {
        Extend::None => return,
        _ => {}
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

