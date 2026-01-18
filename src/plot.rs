use image::{RgbaImage, Rgba};
use std::f64::consts::PI;
use crate::colormap::{Colormap};
use crate::colorbar::{format_tick_label, compute_major_tick_values, render_colorbar_gradient, compute_colorbar_tick_positions};
use crate::render::pdf::{draw_projection_border_pdf,draw_colorbar_pdf};
use crate::scale::{Scale, scale_value};
use crate::layout::compute_mollweide_layout;
use crate::healpix::HealpixMeta;
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale as FontScale};
use crate::{PixelValue,NegMode,PixelSink,CairoRasterSink,CairoImageSink,PngSink};
use crate::healpix::{is_seen, ang2pix};
use cairo::{Context, PdfSurface, ImageSurface, Format};
use std::path::Path;

pub struct MollweideScale {
    pub minv: f64,
    pub maxv: f64,
}

pub fn compute_mollweide_scale(
    map: &[f64],
    minv: Option<f64>,
    maxv: Option<f64>,
    scale: Scale,
    gamma: f64,
) -> MollweideScale {
    let mut values: Vec<f64> = map
        .iter()
        .filter(|v| is_seen(**v))
        .copied()
        .collect();

    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let (minv, maxv) = match (minv, maxv) {
        (Some(lo), Some(hi)) => (lo, hi),
        _ => (
            percentile(&values, 5.0),
            percentile(&values, 95.0),
        ),
    };

    if gamma <= 0.0 {
        panic!("Gamma must be > 0");
    }

    if minv > maxv {
        panic!("Invalid color scale: {minv} > {maxv}");
    }

    println!("map min = {}, max = {}", minv, maxv);

    MollweideScale { minv, maxv }
}

pub fn render_mollweide_to_sink<S: PixelSink>(
    map: &[f64],
    map_w: u32,
    map_h: u32,
    scale_params: &MollweideScale,
    cmap: &Colormap,
    gamma: f64,
    scale: Scale,
    neg_mode: NegMode,
    bad_color: Rgba<u8>,
    meta: HealpixMeta,
    sink: &mut S,
) {
    render_mollweide_pixels(
        map,
        map_w,
        map_h,
        scale_params.minv,
        scale_params.maxv,
        cmap,
        gamma,
        scale,
        neg_mode,
        bad_color,
        meta,
        sink,
    );
}



pub fn rasterize_to_surface<F>(
    width: u32,
    height: u32,
    render: F,
) -> ImageSurface
where
    F: FnOnce(&mut dyn PixelSink),
{
    let surf = ImageSurface::create(
        Format::ARgb32,
        width as i32,
        height as i32,
    ).unwrap();

    let cr = Context::new(&surf).unwrap();
    cr.set_operator(cairo::Operator::Source);
    cr.set_antialias(cairo::Antialias::None);

    let mut sink = CairoRasterSink { cr: &cr };
    render(&mut sink);

    surf
}


pub fn render_mollweide_pixels(
    map: &[f64],
    map_width: u32,
    map_height: u32,
    minv: f64,
    maxv: f64,
    cmap: &Colormap,
    gamma: f64,
    scale: Scale,
    neg_mode: NegMode,
    bad_color: Rgba<u8>,
    meta: HealpixMeta,
    sink: &mut dyn PixelSink,
) {
    for py in 0..map_height {
        for px in 0..map_width {
            // Mollweide plane coordinates
            let x = 2.0 - 4.0 * (px as f64 / (map_width - 1) as f64);
            let y = 1.0 - 2.0 * (py as f64 / (map_height - 1) as f64);

            // Outside Mollweide oval
            if x * x / 4.0 + y * y > 1.0 {
                continue;
            }

            // Inverse Mollweide
            let theta_aux = y.asin();
            let sin_lat = (2.0 * theta_aux + (2.0 * theta_aux).sin()) / PI;
            if sin_lat.abs() > 1.0 {
                continue;
            }

            let lat = sin_lat.asin();
            let lon = PI * x / (2.0 * theta_aux.cos());

            let theta = PI / 2.0 - lat;
            if !(0.0..=PI).contains(&theta) {
                continue;
            }

            // HEALPix lookup
            let ipix = ang2pix(meta, theta, lon);
            let val = map[ipix as usize];

            let rgba = match scale_value(val, minv, maxv, scale, neg_mode) {
                PixelValue::Color(t) => {
                    let t = apply_gamma(t, gamma);
                    let c = cmap.sample(t);
                    Rgba([c[0], c[1], c[2], 255])
                }
                PixelValue::Bad => bad_color,
            };

            sink.draw_pixel(px, py, rgba);
        }
    }
}

pub fn plot_mollweide_pdf(
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
    meta: HealpixMeta,
) {

    let (layout, cb_layout) = compute_mollweide_layout(width as f64, show_colorbar);

    let mut values: Vec<f64> = map
        .iter()
        .filter(|&v| is_seen(*v))
        .copied()
        .collect();

    
    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());


    let surface_pdf = PdfSurface::new(
        layout.width as f64,
        layout.height as f64,
        filename,
    ).expect("Failed to create PDF surface");
    
    let cr_pdf = Context::new(&surface_pdf).unwrap();
    
    // Background
    if transparent {
        cr_pdf.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    } else {
        cr_pdf.set_source_rgb(1.0, 1.0, 1.0);
    }
    cr_pdf.paint().unwrap();
   
    // -----------------------------
    // 2. Create raster surface
    // -----------------------------
    let surface_img = ImageSurface::create(
        Format::ARgb32,
        layout.map_w as i32,
        layout.map_h as i32,   // IMPORTANT: map height, not full height
    ).expect("Failed to create image surface");
    
    let cr_img = Context::new(&surface_img).unwrap();
    
    // Clear raster background
    if transparent {
        cr_img.set_source_rgba(0.0, 0.0, 0.0, 0.0);
    } else {
        cr_img.set_source_rgb(1.0, 1.0, 1.0);
    }
    cr_img.paint().unwrap();

    let scale_params = compute_mollweide_scale(map, minv, maxv, scale, gamma);
    
    let mut sink = CairoImageSink { cr: &cr_img };
    
    render_mollweide_to_sink(
        map,
        layout.map_w as u32,
        layout.map_h as u32,
        &scale_params,
        cmap,
        gamma,
        scale,
        neg_mode,
        bad_color,
        meta,
        &mut sink,
    );


    // CRITICAL
    surface_img.flush();

    
    // -----------------------------
    // 4. Embed raster into PDF
    // -----------------------------
    let _ = cr_pdf.set_source_surface(
        &surface_img,
        layout.map_x as f64,
        layout.map_y as f64,
    );
    cr_pdf.paint().unwrap();



    // Draw vector border ON TOP
    if draw_border {
        let border_width = (width as f64 * 0.0025).max(1.0);
        draw_projection_border_pdf(
            &cr_pdf,
            layout.map_x as f64,
            layout.map_y as f64,
            layout.map_w as f64,
            layout.map_h as f64,
            border_width,
        );
    }

    if show_colorbar {
        draw_colorbar_pdf(
            &cr_pdf,
            cb_layout,
            &cmap,
            scale_params.minv,
            scale_params.maxv,
            neg_mode,
            scale,
            gamma,
        );
    }


    
    // -----------------------------
    // 5. Finish PDF
    // -----------------------------
    surface_pdf.finish();

}


pub trait RenderBackend {
    fn set_color(&mut self, r: u8, g: u8, b: u8, a: u8);
    fn rect(&mut self, x: f64, y: f64, w: f64, h: f64);
    fn stroke_path(&mut self);
    fn fill_path(&mut self);
    fn draw_text(&mut self, x: f64, y: f64, text: &str, size: f64);
}


pub fn plot_mollweide_png(
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
    meta: HealpixMeta,
) {

    let (layout, cb_layout) = compute_mollweide_layout(width as f64, show_colorbar);


    let font_data = include_bytes!("../assets/fonts/DejaVuSans.ttf");
    let font = Font::try_from_bytes(font_data as &[u8])
        .expect("Failed to load font");


    let mut values: Vec<f64> = map
        .iter()
        .filter(|&v| is_seen(*v))
        .copied()
        .collect();

    
    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let bg = if transparent {
        Rgba([0, 0, 0, 0])   // fully transparent
    } else {
        Rgba([255, 255, 255, 255])
    };
    
    let mut img = RgbaImage::from_pixel(layout.width as u32, layout.height as u32, bg);

    if draw_border {
        use cairo::{ImageSurface, Context, Format};

        // Creating a padded surface
        let pad = layout.border_width_px.ceil() as i32; 
        let surf_w = layout.map_w as i32 + 2 * pad;
        let surf_h = layout.map_h as i32 + 2 * pad;
    
        let mut border_surf = ImageSurface::create(
            Format::ARgb32,
            surf_w,
            surf_h,
        ).unwrap();
    
        {
            let border_cr = Context::new(&border_surf).unwrap();
    
            border_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            border_cr.paint().unwrap();
    
            draw_projection_border_pdf(
                &border_cr,
                pad as f64,
                pad as f64,
                layout.map_w,
                layout.map_h,
                layout.border_width_px,
            );
            // border_cr dropped here
        }
    
        border_surf.flush();
    
        let stride = border_surf.stride() as usize;
        let mut data = border_surf.data().unwrap();
   
        for y in 0..surf_h as usize {
            for x in 0..surf_w as usize {
                let idx = y * stride + x * 4;
                let a = data[idx + 3];
                if a == 0 { continue; }
    
                let r = data[idx + 2];
                let g = data[idx + 1];
                let b = data[idx + 0];
    
                img.put_pixel(
                    layout.map_x as u32 + x as u32 - pad as u32,
                    layout.map_y as u32 + y as u32 - pad as u32,
                    Rgba([r, g, b, a]),
                );
            }
        }
    }


    let scale_params = compute_mollweide_scale(map, minv, maxv, scale, gamma);
   
    let mut sink = PngSink { 
        img: &mut img, 
        x0: layout.map_x as u32,
        y0: layout.map_y as u32,
    };
    
    render_mollweide_to_sink(
        map,
        layout.map_w as u32,
        layout.map_h as u32,
        &scale_params,
        cmap,
        gamma,
        scale,
        neg_mode,
        bad_color,
        meta,
        &mut sink,
    );

    if show_colorbar {
        let mut sink = PngSink { 
            img: &mut img,
            x0: layout.cbar_pad as u32,
            y0: layout.cbar_y as u32,
        };
        
        render_colorbar_gradient(
            0,
            0,
            layout.cbar_w as u32,
            layout.cbar_h as u32,
            cmap,
            gamma,
            &mut sink,
        );

        let ticks = compute_colorbar_tick_positions(
            scale_params.minv,
            scale_params.maxv,
            scale,
            neg_mode,
            5,
            5,
        );

        // Scale tick heights relative to colorbar
        let major_tick_height = cb_layout.major_tick_height as u32;
        let minor_tick_height = cb_layout.minor_tick_height as u32;
        
        // Scale tick widths relative to image width
        let major_tick_width = cb_layout.major_tick_width as u32;
        let minor_tick_width = cb_layout.minor_tick_width as u32;
        
        let tick_bottom = cb_layout.tick_bottom as u32;


        // ---------------- Major ticks + labels ----------------
        let tick_top = tick_bottom.saturating_sub(major_tick_height);
        for (&t, &val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
            let px = (layout.cbar_pad + (t * layout.cbar_w).round()) as u32;
            for dx in 0..major_tick_width as i32 {
                let x = (px as i32 + dx) as u32;
                if x < layout.width as u32 {
                    for py in tick_top-1..=tick_bottom {
                        img.put_pixel(x, py, Rgba([0,0,0,255]));
                    }
                }
            }
        
            // Draw label
            let label = format_tick_label(val, scale);
            let text_width_est = (label.len() as f32 * 
                cb_layout.tick_font_size as f32 * 0.6) as i32;
            let text_x = px as i32 - text_width_est / 2;
        
            draw_text_mut(
                &mut img,
                Rgba([0, 0, 0, 255]),
                text_x,
                // The two label calculations make a significant difference here.
                // Different units maybe?
                cb_layout.tick_label_pad as i32,
                FontScale::uniform(cb_layout.tick_font_size as f32),
                &font,
                &label,
            );
        }




 
        // ---------------- Minor ticks ----------------
        let tick_top = tick_bottom.saturating_sub(minor_tick_height);
        for (&t, &val) in ticks.minor_positions.iter().zip(ticks.minor_values.iter()) {
            let px = (layout.cbar_pad + (t * layout.cbar_w).round()) as u32;
        
            for dx in 0..minor_tick_width as i32 {
                let x = (px as i32 + dx) as u32;
                if x < width {
                    for py in tick_top-1..=tick_bottom {
                        img.put_pixel(x, py, Rgba([0,0,0,255]));
                    }
                }
            }
        }
    }



    img.save(filename).expect("Failed to save PNG");
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

    // Normalized pixel size in "projection space"
    let delta = 2.0 / nx;

    // Convert pixel width → normalized distance
    let half_width = 0.5 * line_width_px * delta;

    for py in 0..map_height {
        for px in 0..img.width() {
            // Normalized coordinates (same as your existing code)
            let u = 2.0 * (px as f64 - xc) / xc;
            let v = -(py as f64 - yc) / yc;

            let d = dist_fn(u, v);

            // Signed distance from the ideal boundary (d = 1)
            let dist = (d - 1.0).abs();

            if dist <= half_width {
                // Linear coverage (anti-alias)
                let mut alpha = 1.0 - dist / half_width;

                // Optional perceptual tweak (comment out if unwanted)
                alpha = alpha.powf(0.8);

                let a = (alpha * 255.0).round() as u8;

                if a > 0 {
                    let mut c = border_color;
                    c[3] = a;
                    img.put_pixel(px, py, c);
                }
            }
        }
    }
}

#[inline]
fn apply_gamma(t: f64, gamma: f64) -> f64 {
    if gamma == 1.0 {
        t
    } else {
        t.clamp(0.0, 1.0).powf(1.0 / gamma)
    }
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



pub fn plot_mollweide_auto(
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
    meta: HealpixMeta,
) {
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "png" => {
            plot_mollweide_png(
                map,
                width,
                filename,
                minv,
                maxv,
                cmap,
                show_colorbar,
                transparent,
                draw_border,
                gamma,
                scale,
                neg_mode,
                bad_color,
                meta,
            );
        }
        "pdf" => {
            plot_mollweide_pdf(
                map,
                width,
                filename,
                minv,
                maxv,
                cmap,
                show_colorbar,
                transparent,
                draw_border,
                gamma,
                scale,
                neg_mode,
                bad_color,
                meta,
            );
        }
        _ => {
            panic!(
                "Unsupported output format: .{} (expected .png or .pdf)",
                ext
            );
        }
    }
}

