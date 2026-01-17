use cairo::{Context, ImageSurface, Format};
use crate::render::RenderBackend;
use crate::{Colormap, Scale};
use crate::scale::{value_to_t};
use crate::colorbar::{apply_gamma,format_tick_label,ColorbarTicks};
use std::f64::consts::PI;
use crate::colorbar::{compute_colorbar_ticks, render_colorbar_gradient};
use crate::plot::PixelSink;
use image::Rgba;


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
    minv: f64,
    maxv: f64,
    cbar_x: f64,
    cbar_y: f64,
    width: f64,
    cbar_height: f64,
    ticks: &ColorbarTicks,
    scale: Scale,
) {
    let y0 = cbar_y + cbar_height;
    let major_len = cbar_height * 0.4;
    let minor_len = cbar_height * 0.2;

    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(1.0);

    // Minor ticks
    for &val in &ticks.minor {
        if let Some(t) = value_to_t(val, minv, maxv, scale) {
            let x = t * (width - 1.0) + cbar_x;
            cr.move_to(x, y0);
            cr.line_to(x, y0 - minor_len);
        }
    }

    // Major ticks
    for &val in &ticks.major {
        if let Some(t) = value_to_t(val, minv, maxv, scale) {
            let x = t * width + cbar_x;
            cr.move_to(x, y0);
            cr.line_to(x, y0 - major_len);
        }
    }

    cr.stroke().unwrap();
}

pub fn draw_colorbar_pdf_labels(
    cr: &Context,
    cbar_x: f64,
    width: f64,
    label_y: f64,
    ticks: &ColorbarTicks,
    minv: f64,
    maxv: f64,
    scale: Scale,
) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_font_size(11.0);

    for &val in &ticks.major {
        if let Some(t) = value_to_t(val, minv, maxv, scale) {
            let label = format_tick_label(val, scale);
            let x = t * width + cbar_x;

            // Center text
            let ext = cr.text_extents(&label).unwrap();
            let tx = x - ext.width() / 2.0;

            cr.move_to(tx, label_y);
            cr.show_text(&label).unwrap();
        }

    }
}


pub fn draw_colorbar_pdf(
    cr: &Context,
    cbar_x: f64,
    cbar_y: f64,
    cbar_w: f64,
    cbar_h: f64,
    label_y: f64,
    cmap: &Colormap,
    minv: f64,
    maxv: f64,
    scale: Scale,
    gamma: f64,
) {
        let ticks = compute_colorbar_ticks(
            minv,
            maxv,
            scale,
            5,
            5, // minor ticks already handled intelligently
        );

        let surf = ImageSurface::create(
            Format::ARgb32,
            cbar_w as i32,
            cbar_h as i32,
        ).unwrap();
        
        let surf_cr = cairo::Context::new(&surf).unwrap();
        surf_cr.set_operator(cairo::Operator::Source);
        surf_cr.set_antialias(cairo::Antialias::None);


        struct CairoImageSink<'a> {
            cr: &'a Context,
        }
        
        impl<'a> PixelSink for CairoImageSink<'a> {
            fn draw_pixel(&mut self, x: u32, y: u32, rgba: Rgba<u8>) {
                self.cr.set_source_rgba(
                    rgba[0] as f64 / 255.0,
                    rgba[1] as f64 / 255.0,
                    rgba[2] as f64 / 255.0,
                    rgba[3] as f64 / 255.0,
                );
                self.cr.rectangle(x as f64, y as f64, 1.0, 1.0);
                self.cr.fill().unwrap();
            }
        }

        let mut sink = CairoImageSink { cr: &surf_cr };

        render_colorbar_gradient(
            0,
            0,
            cbar_w as u32,
            cbar_h as u32,
            cmap,
            gamma,
            &mut sink,
        );

        cr.set_source_surface(&surf, cbar_x, cbar_y);
        cr.paint().unwrap();

    
        draw_colorbar_pdf_ticks(
            &cr,
            minv,
            maxv,
            cbar_x,
            cbar_y,
            cbar_w,
            cbar_h,
            &ticks,
            scale,
        );
    
        draw_colorbar_pdf_labels(
            &cr,
            cbar_x,
            cbar_w,
            label_y,
            &ticks,
            minv,
            maxv,
            scale,
        );

}
