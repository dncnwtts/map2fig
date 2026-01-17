use image::{RgbaImage, Rgba};
use std::f64::consts::PI;
use crate::colormap::{Colormap};
use crate::colorbar::{compute_colorbar_ticks,format_tick_label, compute_major_tick_values, render_colorbar_gradient};
use crate::render::pdf::{draw_projection_border_pdf,draw_colorbar_pdf};
use crate::scale::{Scale, scale_value};
use crate::layout::compute_mollweide_layout;
use crate::healpix::HealpixMeta;
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale as FontScale};
use crate::{PixelValue,NegMode,PixelSink,CairoRasterSink};
use crate::healpix::{is_seen, ang2pix, nside_from_npix};
use cairo::{Context, PdfSurface, ImageSurface, Format};


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



pub fn draw_map_pdf_pixels(
    cr: &Context,
    map: &[f64],
    width: u32,
    height: u32,
    minv: f64,
    maxv: f64,
    cmap: &Colormap,
    gamma: f64,
    scale: Scale,
    neg_mode: NegMode,
    bad_color: Rgba<u8>,
    meta: HealpixMeta,
) {
    struct PdfPixelSink<'a> {
        cr: &'a Context,
    }
    
    impl<'a> PixelSink for PdfPixelSink<'a> {
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

    let map_height = height;

    // -----------------------------
    // HEALPix setup
    // -----------------------------
    let npix = map.len();
    let nside = nside_from_npix(npix)
        .expect("Input map is not a valid full-sky HEALPix map");
    println!("Nside is {nside}");

    // -----------------------------
    // Determine color scale limits
    // -----------------------------
    let mut values: Vec<f64> = map
        .iter()
        .filter(|&v| is_seen(*v))
        .copied()
        .collect();

    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());


    if gamma <= 0.0 {
        panic!("Gamma must be > 0");
    }

    if minv > maxv {
        panic!("Invalid color scale: {minv} > {maxv}");
    }
    
    let mut sink = PdfPixelSink { cr };
    cr.set_antialias(cairo::Antialias::None);
    cr.set_operator(cairo::Operator::Source);

    
    render_mollweide_pixels(
        map,
        width,
        map_height,
        minv,
        maxv,
        cmap,
        gamma,
        scale,
        neg_mode,
        bad_color,
        meta,
        &mut sink,
    );


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

    let layout = compute_mollweide_layout(width as f64, show_colorbar);

    let font_data = include_bytes!("../assets/fonts/DejaVuSans.ttf");
    let _font = Font::try_from_bytes(font_data as &[u8])
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
    let _bg = if transparent {
        Rgba([0, 0, 0, 0])   // fully transparent
    } else {
        Rgba([255, 255, 255, 255])
    };
    
    

    use cairo::{Context, ImageSurface, Format};
    
    let surface_pdf = PdfSurface::new(
        layout.width as f64,
        layout.height as f64,
        filename,
    ).expect("Failed to create PDF surface");
    
    let cr_pdf = Context::new(&surface_pdf).unwrap();
    
    // Optional background
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
    
    // -----------------------------
    // 3. Draw map pixels
    // -----------------------------
    draw_map_pdf_pixels(
        &cr_img,
        map,
        layout.map_w as u32,
        layout.map_h as u32,
        minv,
        maxv,
        cmap,
        gamma,
        scale,
        neg_mode,
        bad_color,
        meta,
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
            layout.cbar_x,
            layout.cbar_y,
            layout.cbar_w,
            layout.cbar_h,
            layout.label_y,
            &cmap,
            minv,
            maxv,
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
    meta: HealpixMeta,
) {

    struct PngSink<'a> {
        img: &'a mut RgbaImage,
    }
    
    impl<'a> PixelSink for PngSink<'a> {
        fn draw_pixel(&mut self, x: u32, y: u32, rgba: Rgba<u8>) {
            self.img.put_pixel(x, y, rgba);
        }
    }

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


    let _height = if show_colorbar {
        map_height + colorbar_height + label_font_size as u32 + label_padding
    }
    else {
        map_height
    };
    let extra_label_space = if show_colorbar { label_font_size as u32 + 4 } else { 0 };
    let height = map_height + colorbar_height + extra_label_space;


    let mut values: Vec<f64> = map
        .iter()
        .filter(|&v| is_seen(*v))
        .copied()
        .collect();

    
    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_by(|a, b| a.partial_cmp(b).unwrap());



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

    if draw_border {
        let border_width_px = (width as f64 * 0.01).max(2.0);
        println!("Border width is {border_width_px}");
        draw_projection_border(
            &mut img,
            map_height,
            Rgba([0, 0, 0, 255]),
            border_width_px,
            |u, v| (u * u) / 4.0 + v * v,
        );
    }

    let mut sink = PngSink { img: &mut img };
    
    render_mollweide_pixels(
        map,
        width,
        map_height,
        minv,
        maxv,
        cmap,
        gamma,
        scale,
        neg_mode,
        bad_color,
        meta,
        &mut sink,
    );



    if show_colorbar {
        let mut sink = PngSink { img: &mut img };
        
        render_colorbar_gradient(
            cbar_pad,
            map_height,
            width - 2 * cbar_pad,
            colorbar_height,
            cmap,
            gamma,
            &mut sink,
        );


        // ---------------- Colorbar tick marks (scale-aware) ----------------
        
        let nticks = 5;   // major ticks
        let nminor = 5;   // minor ticks per interval
        
        let _ticks = compute_colorbar_ticks(
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
                        if (dx <= -1) | (dx >= major_tick_width as i32) | (py == tick_top) {
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
