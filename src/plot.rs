use image::{RgbaImage, Rgba};
use crate::colormap::{Colormap};
use crate::colorbar::{format_tick_label, render_colorbar_gradient, compute_colorbar_tick_positions,apply_gamma};
use crate::render::pdf::{draw_projection_border_pdf,draw_colorbar_pdf};
use crate::scale::{Scale, scale_value};
use crate::layout::{compute_mollweide_layout,MollweideLayout};
use crate::healpix::HealpixMeta;
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale as FontScale};
use crate::{PixelValue,NegMode,PixelSink,CairoRasterSink,CairoImageSink,PngSink};
use crate::healpix::{is_seen, sample_healpix};
use cairo::{Context, PdfSurface, ImageSurface, Format};
use std::path::Path;
use crate::projection::Projection;
use crate::render::raster::RasterGrid;


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

pub fn render_mollweide_pixels(
    map: &[f64],
    layout: MollweideLayout,
    scale_params: &MollweideScale,
    cmap: &Colormap,
    gamma: f64,
    scale: Scale,
    neg_mode: NegMode,
    bad_color: Rgba<u8>,
    meta: HealpixMeta,
    sink: &mut dyn PixelSink,
    debug_overlay: Option<DebugOverlay>, // new optional parameter
) {
    use crate::mollweide::MollweideProjection;
    let proj = MollweideProjection;

    let mut grid = RasterGrid::new(layout.map_w as u32, layout.map_h as u32);

    if let Some(overlay) = debug_overlay {
        if overlay.show_background {
            fill_grid_background(&mut grid);
        }
    }


    render_projection_to_grid(
        map,
        &proj,
        &mut grid,
        scale_params,
        cmap,
        scale,
        neg_mode,
        gamma,
        bad_color,
        meta,
    );

    // Draw debug overlay only if provided
    if let Some(overlay) = debug_overlay {
        draw_debug_overlay_raster(&mut grid, overlay);
    }

    blit_grid_to_sink(&grid, sink, 0, 0);
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
    _bg_color: Rgba<u8>,
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
    
    if transparent {
        cr_pdf.set_operator(cairo::Operator::Source);
        cr_pdf.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        cr_pdf.paint().unwrap();
    }

   
    // -----------------------------
    // 2. Create raster surface
    // -----------------------------
    let surface_img = ImageSurface::create(
        Format::ARgb32,
        (layout.map_w + 2.0*layout.map_pad) as i32,
        (layout.map_h + 2.0*layout.map_pad) as i32,
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
    let debug_overlay = if cfg!(feature = "debug_overlay") {
        Some(DebugOverlay::grid_only())
    } else {
        None
    };
    
    render_mollweide_pixels(
        map,
        layout,
        &scale_params,
        cmap,
        gamma,
        scale,
        neg_mode,
        bad_color,
        meta,
        &mut sink,
        debug_overlay, // pass the optional overlay
    );


    // CRITICAL
    surface_img.flush();

    
    // -----------------------------
    // 4. Embed raster into PDF
    // -----------------------------
    let _ = cr_pdf.set_source_surface(
        &surface_img,
        layout.map_x,
        layout.map_y,
    );
    cr_pdf.paint().unwrap();


    // Draw vector border ON TOP
    if draw_border {
        draw_projection_border_pdf(
            &cr_pdf,
            layout.map_x,
            layout.map_y,
            layout.map_w,
            layout.map_h,
            layout.border_width_px,
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
    fn stroke_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, _width: f64);
    fn draw_line(&mut self, x0: f64, y0: f64, x1: f64, y1: f64, _width: f64);
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
    bg_color: Rgba<u8>,
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

    let bg = Rgba([bg_color[0], bg_color[1], bg_color[2], 
        if transparent {0} else {255}]);
    
    let mut img = RgbaImage::from_pixel(layout.width as u32, layout.height as u32, bg);



    let scale_params = compute_mollweide_scale(map, minv, maxv, scale, gamma);
   
    let mut sink = PngSink { 
        img: &mut img, 
        x0: layout.map_x as u32,
        y0: layout.map_y as u32,
    };

    let debug_overlay = if cfg!(feature = "debug_overlay") {
        Some(DebugOverlay::grid_only())
    } else {
        None
    };
    
    render_mollweide_pixels(
        map,
        layout,
        &scale_params,
        cmap,
        gamma,
        scale,
        neg_mode,
        bad_color,
        meta,
        &mut sink,
        debug_overlay, // pass the optional overlay
    );

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
        let data = border_surf.data().unwrap();
   
        for y in 0..surf_h as usize {
            for x in 0..surf_w as usize {
                let idx = y * stride + x * 4;
                let a = data[idx + 3];
                if a == 0 { continue; }
    
                let r = data[idx + 2];
                let g = data[idx + 1];
                let b = data[idx + 0];

                let dst = img.get_pixel(
                    layout.map_x as u32 + x as u32 - pad as u32,
                    layout.map_y as u32 + y as u32 - pad as u32,
                );
                let src = Rgba([r, g, b, a]);
                
                let alpha = a as f32 / 255.0;
                
                let out = Rgba([
                    ((src[0] as f32 + dst[0] as f32 * (1.0 - alpha)) as u8),
                    ((src[1] as f32 + dst[1] as f32 * (1.0 - alpha)) as u8),
                    ((src[2] as f32 + dst[2] as f32 * (1.0 - alpha)) as u8),
                    ((a as f32 + dst[3] as f32 * (1.0 - alpha)) as u8),
                ]);
                
                img.put_pixel(
                    layout.map_x as u32 + x as u32 - pad as u32,
                    layout.map_y as u32 + y as u32 - pad as u32,
                    out,
                );
            }
        }
    }
    
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
        for (&t, &_val) in ticks.minor_positions.iter().zip(ticks.minor_values.iter()) {
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
    bg_color: Rgba<u8>,
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
                bg_color,
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
                bg_color,
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


pub fn map_to_color(
    val: f64,
    minv: f64,
    maxv: f64,
    cmap: &Colormap,
    scale: Scale,
    neg_mode: NegMode,
    gamma: f64,
    bad_color: Rgba<u8>,
) -> Rgba<u8> {
    match scale_value(val, minv, maxv, scale, neg_mode) {
        PixelValue::Color(t) => {
            let t = apply_gamma(t, gamma);
            let c = cmap.sample(t);
            Rgba([c[0], c[1], c[2], 255])
        }
        PixelValue::Bad => {
            bad_color
        }
        PixelValue::Underflow => {
            let c = cmap.sample(0.0);
            Rgba([c[0], c[1], c[2], 255])
        }
        PixelValue::Overflow => {
            let c = cmap.sample(1.0);
            Rgba([c[0], c[1], c[2], 255])
        }
    }
}


pub fn render_projection_to_sink(
    map: &[f64],
    proj: &dyn Projection,
    layout: &MollweideLayout,
    scale_params: &MollweideScale,
    cmap: &Colormap,
    scale: Scale,
    neg_mode: NegMode,
    gamma: f64,
    bad_color: Rgba<u8>,
    meta: HealpixMeta,
    sink: &mut dyn PixelSink,
) {
    let grid = RasterGrid::new(layout.map_w as u32, layout.map_h as u32);


    for py in 0..grid.height {
        for px in 0..grid.width {
            let (theta, lon) = match proj.pixel_to_ang(px, py, &grid) {
                Some(v) => v,
                None => continue,
            };

            let val = match sample_healpix(map, meta, theta, lon) {
                Some(v) => v,
                None => continue,
            };

            let rgba = map_to_color(
                val,
                scale_params.minv,
                scale_params.maxv,
                cmap,
                scale,
                neg_mode,
                gamma,
                bad_color,
            );

            sink.draw_pixel(
                px,
                py,
                rgba,
            );
        }
    }
}


/// Convert a scalar map value to a PixelValue (handles underflow/overflow)
#[inline]
pub fn map_to_pixel_value(
    val: f64,
    minv: f64,
    maxv: f64,
    scale: Scale,
    neg_mode: NegMode,
) -> PixelValue {
    match scale_value(val, minv, maxv, scale, neg_mode) {
        PixelValue::Bad => PixelValue::Bad,
        PixelValue::Underflow => PixelValue::Underflow,
        PixelValue::Overflow => PixelValue::Overflow,
        PixelValue::Color(t) => PixelValue::Color(t),
    }
}
/// Convert a PixelValue into an RGBA color
#[inline]
pub fn pixel_value_to_rgba(
    pv: PixelValue,
    cmap: &Colormap,
    gamma: f64,
    bad_color: Rgba<u8>,
) -> Rgba<u8> {
    match pv {
        PixelValue::Color(t) => {
            let t = apply_gamma(t, gamma);
            let c = cmap.sample(t);
            Rgba([c[0], c[1], c[2], 255])
        }
        PixelValue::Underflow => {
            let c = cmap.sample(0.0);
            Rgba([c[0], c[1], c[2], 255])
        }
        PixelValue::Overflow => {
            let c = cmap.sample(1.0);
            Rgba([c[0], c[1], c[2], 255])
        }
        PixelValue::Bad => bad_color,
    }
}

pub fn render_projection_to_grid(
    map: &[f64],
    proj: &dyn Projection,
    grid: &mut RasterGrid,
    scale_params: &MollweideScale,
    cmap: &Colormap,
    scale: Scale,
    neg_mode: NegMode,
    gamma: f64,
    bad_color: Rgba<u8>,
    meta: HealpixMeta,
) {

    for (px, py, u, v) in grid.iter() {
        let x = 2.0 * u - 1.0;
        let y = 2.0 * v - 1.0;
    
        let inside_oval = (x * x) / 4.0 + (y * y) <= 1.0;
    
        if !inside_oval {
            grid.set_valid(px, py, false);
            continue;
        }
    
        if let Some((lon, lat)) = proj.inverse(u, v) {
            let theta = std::f64::consts::PI / 2.0 - lat;
    
            let pixel_val = match sample_healpix(map, meta, theta, lon) {
                Some(val) => map_to_pixel_value(
                    val,
                    scale_params.minv,
                    scale_params.maxv,
                    scale,
                    neg_mode,
                ),
                None => PixelValue::Bad,
            };
    
            let rgba = pixel_value_to_rgba(pixel_val, cmap, gamma, bad_color);
    
            grid.set_pixel(px, py, rgba);
            grid.set_valid(px, py, true);
        } else {
            grid.set_valid(px, py, false);
        }
    }
}

pub fn blit_grid_to_sink(
    grid: &RasterGrid,
    sink: &mut dyn PixelSink,
    x0: u32,
    y0: u32,
) {
    for y in 0..grid.height {
        for x in 0..grid.width {
            if !grid.is_valid(x, y) {
                continue;
            } else {
                let p = grid.get_pixel(x, y);
                sink.draw_pixel(x0 + x, y0 + y, p);
            }
        }
    }
}


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


pub fn draw_debug_overlay(
    sink: &mut dyn RenderBackend,
    grid: &RasterGrid,
    x0: u32,
    y0: u32,
    overlay: DebugOverlay,
) {
    if !overlay.enabled {
        return;
    }

    let x0 = x0 as f64;
    let y0 = y0 as f64;
    let w = grid.width as f64;
    let h = grid.height as f64;

    sink.set_color(255, 0, 0, 160);

    if overlay.show_grid_box {
        sink.stroke_line(x0, y0, x0 + w, y0, 1.0);
        sink.stroke_line(x0 + w, y0, x0 + w, y0 + h, 1.0);
        sink.stroke_line(x0 + w, y0 + h, x0, y0 + h, 1.0);
        sink.stroke_line(x0, y0 + h, x0, y0, 1.0);
    }

    if overlay.show_center {
        let cx = x0 + 0.5 * w;
        let cy = y0 + 0.5 * h;

        sink.stroke_line(cx - 10.0, cy, cx + 10.0, cy, 1.0);
        sink.stroke_line(cx, cy - 10.0, cx, cy + 10.0, 1.0);
    }
}

pub fn draw_debug_overlay_raster(
    grid: &mut RasterGrid,
    overlay: DebugOverlay,
) {
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

fn fill_grid_background(grid: &mut RasterGrid) {
    let bg = Rgba([220, 220, 220, 255]); // your current gray

    for y in 0..grid.height {
        for x in 0..grid.width {
            grid.set_pixel(x, y, bg);
        }
    }
}
