use cairo::{Context, ImageSurface, Format};
use crate::render::RenderBackend;
use crate::{Colormap, Scale, CairoImageSink};
use crate::colorbar::{apply_gamma,format_tick_label,ColorbarTicks};
use std::f64::consts::PI;
use crate::colorbar::{render_colorbar_gradient};
use crate::plot::rasterize_to_surface;
use crate::layout::{ColorbarLayout};
use crate::render::target::{RenderTarget,PixelSource};
use crate::PixelSink;
use crate::scale::{generate_colorbar_ticks,HistogramScale};



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
    layout: ColorbarLayout,
    ticks: &ColorbarTicks,
) {
    let y0 = layout.y + layout.h;
    let major_len = layout.major_tick_height;
    let minor_len = layout.minor_tick_height;

    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_line_width(1.0);

    // Minor ticks
    for (&t, &_val) in ticks.minor_positions.iter().zip(ticks.minor_values.iter()) {
        let x = t * (layout.w - 1.0) + layout.x;
        cr.move_to(x, y0);
        cr.line_to(x, y0 - minor_len);
    }

    // Major ticks
    for (&t, &_val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
        let x = t * (layout.w - 1.0) + layout.x;
        cr.move_to(x, y0);
        cr.line_to(x, y0 - major_len);
    }

    cr.stroke().unwrap();
}

pub fn draw_colorbar_pdf_labels(
    cr: &Context,
    layout: ColorbarLayout,
    ticks: &ColorbarTicks,
    scale: Scale,
) {
    cr.set_source_rgb(0.0, 0.0, 0.0);
    cr.set_font_size(11.0);

    for (&t, &val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
        let label = format_tick_label(val, scale, Some(t));
        let x = t * layout.w + layout.x;

        // Center text
        let ext = cr.text_extents(&label).unwrap();
        let tx = x - ext.width() / 2.0;

        cr.move_to(tx, layout.tick_label_pad);
        cr.show_text(&label).unwrap();
    }
}


pub fn draw_colorbar_pdf(
    cr: &Context,
    cb_layout: ColorbarLayout,
    cmap: &Colormap,
    minv: f64,
    maxv: f64,
    scale: Scale,
    gamma: f64,
    hist: Option<&HistogramScale>,
) {

        let ticks = generate_colorbar_ticks(
            minv,
            maxv,
            &scale,
            hist,
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
                cmap,
                gamma,
                sink,
            );
        });
        
        let _ = cr.set_source_surface(&surf, cb_layout.x, cb_layout.y);
        cr.paint().unwrap();

        draw_colorbar_pdf_ticks(
            &cr,
            cb_layout,
            &ticks,
        );
    
        draw_colorbar_pdf_labels(
            &cr,
            cb_layout,
            &ticks,
            scale,
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

