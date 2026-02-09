use image::{RgbaImage, Rgba};
use crate::colormap::{Colormap};
use crate::colorbar::{format_tick_label_with_units, render_colorbar_gradient, apply_gamma};
use crate::render::pdf::{draw_projection_border_pdf,draw_colorbar_pdf};
use crate::scale::{Scale, scale_value,generate_colorbar_ticks,build_histogram_scale,HistogramScale, HistogramRange, unsafe_float_cmp};
use crate::layout::{compute_mollweide_layout, compute_gnomonic_layout, MollweideLayout};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale as FontScale};
use crate::{PixelValue,NegMode,PixelSink,CairoRasterSink,CairoImageSink,PngSink};
use crate::healpix::{is_seen, sample_healpix};
use cairo::{Context, PdfSurface, ImageSurface, Format};
use std::path::Path;
use crate::render::raster::RasterGrid;
use crate::rotation::CoordSystem;
use crate::params::{MollweideParams, GnomonicParams};

pub struct MollweideScale {
    pub minv: f64,
    pub maxv: f64,
}
pub fn compute_mollweide_scale(
    map: &[f64],
    minv: Option<f64>,
    maxv: Option<f64>,
    gamma: f64,
    scale: Scale,
) -> MollweideScale {
    let mut values: Vec<f64> = map
        .iter()
        .filter(|v| is_seen(**v))
        .copied()
        .collect();

    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_unstable_by(unsafe_float_cmp);

    let data_min = *values.first().unwrap();
    let data_max = *values.last().unwrap();

    let (minv, maxv) = match scale {
        // 🔴 Histogram scale overrides percentiles
        Scale::Histogram => match (minv, maxv) {
            (Some(lo), Some(hi)) => (lo, hi),
            _ => (data_min, data_max),
        },

        // 🟢 All other scales keep percentile default
        _ => match (minv, maxv) {
            (Some(lo), Some(hi)) => (lo, hi),
            _ => (
                percentile(&values, 5.0),
                percentile(&values, 95.0),
            ),
        },
    };

    if gamma <= 0.0 {
        panic!("Gamma must be > 0");
    }

    if minv > maxv {
        panic!("Invalid color scale: {minv} > {maxv}");
    }

    MollweideScale { minv, maxv }
}


pub fn render_mollweide_pixels(
    params: crate::params::RenderMollweideParams,
    layout: MollweideLayout,
    sink: &mut dyn PixelSink,
    debug_overlay: Option<DebugOverlay>, 
) {
    use crate::mollweide::MollweideProjection;
    let proj = MollweideProjection;

    let mut grid = RasterGrid::new(layout.map_w as u32, layout.map_h as u32);

    if let Some(overlay) = debug_overlay
        && overlay.show_background {
            fill_grid_background(&mut grid);
        }

    render_projection_to_grid(
        crate::params::RenderGridParams {
            map: params.map,
            proj: &proj,
            scale: params.scale,
            cmap: params.cmap,
            scale_type: params.scale_type,
            neg_mode: params.neg_mode,
            gamma: params.gamma,
            bad_color: params.bad_color,
            meta: params.meta,
            hist_scale: params.hist_scale,
            view: params.view,
        },
        &mut grid,
    );

    // Draw debug overlay only if provided
    if let Some(overlay) = debug_overlay {
        draw_debug_overlay_raster(&mut grid, overlay);
    }

    blit_grid_to_sink(&grid, sink, 0, 0);
}

pub fn render_hammer_pixels(
    params: crate::params::RenderMollweideParams,
    layout: MollweideLayout,
    sink: &mut dyn PixelSink,
    debug_overlay: Option<DebugOverlay>, 
) {
    use crate::hammer::HammerProjection;
    let proj = HammerProjection::new();

    let mut grid = RasterGrid::new(layout.map_w as u32, layout.map_h as u32);

    if let Some(overlay) = debug_overlay
        && overlay.show_background {
            fill_grid_background(&mut grid);
        }

    render_projection_to_grid(
        crate::params::RenderGridParams {
            map: params.map,
            proj: &proj,
            scale: params.scale,
            cmap: params.cmap,
            scale_type: params.scale_type,
            neg_mode: params.neg_mode,
            gamma: params.gamma,
            bad_color: params.bad_color,
            meta: params.meta,
            hist_scale: params.hist_scale,
            view: params.view,
        },
        &mut grid,
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


pub fn plot_mollweide_pdf(params: MollweideParams) {
    _plot_mollweide_pdf_impl(params, render_mollweide_pixels);
}

fn _plot_mollweide_pdf_impl<F>(params: MollweideParams, pixel_renderer: F) 
where
    F: Fn(
        crate::params::RenderMollweideParams,
        MollweideLayout,
        &mut dyn PixelSink,
        Option<DebugOverlay>,
    ),
{
    let map = params.plot.map;
    let width = params.plot.width;
    let filename = params.plot.filename;
    let minv = params.scale.minv;
    let maxv = params.scale.maxv;
    let cmap = params.color.cmap;
    let show_colorbar = params.display.show_colorbar;
    let transparent = params.display.transparent;
    let draw_border = params.display.draw_border;
    let gamma = params.scale.gamma;
    let scale = params.scale.scale;
    let neg_mode = params.scale.neg_mode;
    let bad_color = params.color.bad_color;
    let _bg_color = params.color.bg_color;
    let meta = params.meta;
    let latex_rendering = params.display.latex_rendering;
    let units = params.display.units.as_deref();
    let view = params.view;
    let show_graticule = params.graticule.show_graticule;
    let grat_coord = params.graticule.grat_coord;
    let grat_overlay = params.graticule.grat_overlay;
    let overlay_color = params.graticule.overlay_color;
    let _show_labels = params.graticule.show_labels;
    let dpar_deg = params.graticule.dpar_deg;
    let dmer_deg = params.graticule.dmer_deg;

    let (layout, cb_layout) = compute_mollweide_layout(width as f64, show_colorbar, params.display.tick_direction.clone());

    let mut values: Vec<f64> = map
        .iter()
        .filter(|&v| is_seen(*v))
        .copied()
        .collect();

    
    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_unstable_by(unsafe_float_cmp);


    let surface_pdf = PdfSurface::new(
        layout.width,
        layout.height,
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

    let scale_params = compute_mollweide_scale(map, minv, maxv, gamma, scale);

    let hist_scale = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit { min: minv, max: maxv },
            _ => HistogramRange::Full,
        };
    
        Some(
            &build_histogram_scale(
                map,
                range,
                1024, // number of bins
            )
        )
    } else {
        None
    };

    
    let mut sink = CairoImageSink { cr: &cr_img };
    let debug_overlay = if cfg!(feature = "debug_overlay") {
        Some(DebugOverlay::grid_only())
    } else {
        None
    };
    
    pixel_renderer(
        crate::params::RenderMollweideParams {
            map,
            scale: &scale_params,
            cmap,
            gamma,
            scale_type: scale,
            neg_mode,
            bad_color,
            meta,
            hist_scale,
            view,
        },
        layout,
        &mut sink,
        debug_overlay,
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

    // Draw graticule BEFORE border (so border appears on top)
    if show_graticule {
        use crate::graticule::{render_graticule_mollweide_vectorized, render_graticule_cairo, render_graticule_cairo_with_color};
        
        let grat_coord_sys = grat_coord.unwrap_or(CoordSystem::E);
        
        let graticule = render_graticule_mollweide_vectorized(
            view,
            dpar_deg,
            dmer_deg,
            grat_coord_sys,
            view.input_coord,
        );
        
        // Render primary graticule in black
        render_graticule_cairo(
            &graticule,
            &cr_pdf,
            layout.map_x,
            layout.map_y,
            layout.map_w,
            layout.map_h,
        );

        // Render secondary graticule overlay if specified
        if let Some(overlay_sys) = grat_overlay {
            let overlay_graticule = render_graticule_mollweide_vectorized(
                view,
                dpar_deg,
                dmer_deg,
                overlay_sys,
                view.input_coord,
            );
            
            // Convert RGBA color to normalized RGB for Cairo
            let r = overlay_color[0] as f64 / 255.0;
            let g = overlay_color[1] as f64 / 255.0;
            let b = overlay_color[2] as f64 / 255.0;
            
            render_graticule_cairo_with_color(
                &overlay_graticule,
                &cr_pdf,
                layout.map_x,
                layout.map_y,
                layout.map_w,
                layout.map_h,
                r,
                g,
                b,
            );
        }
    }

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
            crate::params::ColorbarParams {
                cmap,
                minv: scale_params.minv,
                maxv: scale_params.maxv,
                scale_type: scale,
                gamma,
                hist_scale,
                latex_rendering,
                units,
                extend: &params.display.extend,
                units_font_size: params.display.units_font_size,
            },
        );
    }

    // Draw figure labels (rlabel, llabel)
    crate::render::pdf::draw_figure_labels_pdf(
        &cr_pdf,
        layout.width,
        layout.height,
        &params.display.rlabel,
        &params.display.llabel,
        latex_rendering,
        params.display.label_font_size,
    );

    
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


pub fn plot_mollweide_png(params: MollweideParams) {
    _plot_mollweide_png_impl_projected(params, render_mollweide_pixels, ProjectionType::Mollweide);
}

fn _plot_mollweide_png_impl_projected<F>(params: MollweideParams, pixel_renderer: F, projection: ProjectionType) 
where
    F: Fn(
        crate::params::RenderMollweideParams,
        MollweideLayout,
        &mut dyn PixelSink,
        Option<DebugOverlay>,
    ),
{
    let map = params.plot.map;
    let width = params.plot.width;
    let filename = params.plot.filename;
    let minv = params.scale.minv;
    let maxv = params.scale.maxv;
    let cmap = params.color.cmap;
    let show_colorbar = params.display.show_colorbar;
    let transparent = params.display.transparent;
    let draw_border = params.display.draw_border;
    let gamma = params.scale.gamma;
    let scale = params.scale.scale;
    let neg_mode = params.scale.neg_mode;
    let bad_color = params.color.bad_color;
    let bg_color = params.color.bg_color;
    let meta = params.meta;
    let latex_rendering = params.display.latex_rendering;
    let units = params.display.units.as_deref();
    let view = params.view;
    let show_graticule = params.graticule.show_graticule;
    let grat_coord = params.graticule.grat_coord;
    let grat_overlay = params.graticule.grat_overlay;
    let overlay_color = params.graticule.overlay_color;
    let _show_labels = params.graticule.show_labels;
    let dpar_deg = params.graticule.dpar_deg;
    let dmer_deg = params.graticule.dmer_deg;

    let (layout, cb_layout) = compute_mollweide_layout(width as f64, show_colorbar, params.display.tick_direction.clone());


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

    values.sort_unstable_by(unsafe_float_cmp);

    let bg = Rgba([bg_color[0], bg_color[1], bg_color[2], 
        if transparent {0} else {255}]);
    
    let mut img = RgbaImage::from_pixel(layout.width as u32, layout.height as u32, bg);

    let scale_params = compute_mollweide_scale(map, minv, maxv, gamma, scale);

    let hist_scale = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit { min: minv, max: maxv },
            _ => HistogramRange::Full,
        };
    
        Some(
            &build_histogram_scale(
                map,
                range,
                1024, // number of bins
            )
        )
    } else {
        None
    };
   
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
    
    pixel_renderer(
        crate::params::RenderMollweideParams {
            map,
            scale: &scale_params,
            cmap,
            gamma,
            scale_type: scale,
            neg_mode,
            bad_color,
            meta,
            hist_scale,
            view,
        },
        layout,
        &mut sink,
        debug_overlay,
    );

    if draw_border || show_graticule {
        use cairo::{ImageSurface, Context, Format};

        // Creating a padded surface (shared for both border and graticule)
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

            // Draw graticule using Cairo (anti-aliased) before border
            if show_graticule {
                use crate::graticule::{render_graticule_mollweide_vectorized, render_graticule_hammer_vectorized, render_graticule_cairo, render_graticule_cairo_with_color};
                
                let grat_coord_sys = grat_coord.unwrap_or(CoordSystem::E);
                
                let graticule = match projection {
                    ProjectionType::Mollweide => render_graticule_mollweide_vectorized(
                        view,
                        dpar_deg,
                        dmer_deg,
                        grat_coord_sys,
                        view.input_coord,
                    ),
                    ProjectionType::Hammer => render_graticule_hammer_vectorized(
                        view,
                        dpar_deg,
                        dmer_deg,
                        grat_coord_sys,
                        view.input_coord,
                    ),
                };
                
                // Render primary graticule on Cairo surface (anti-aliased)
                render_graticule_cairo(
                    &graticule,
                    &border_cr,
                    pad as f64,
                    pad as f64,
                    layout.map_w,
                    layout.map_h,
                );

                // Render secondary graticule overlay if specified
                if let Some(overlay_sys) = grat_overlay {
                    let overlay_graticule = match projection {
                        ProjectionType::Mollweide => render_graticule_mollweide_vectorized(
                            view,
                            dpar_deg,
                            dmer_deg,
                            overlay_sys,
                            view.input_coord,
                        ),
                        ProjectionType::Hammer => render_graticule_hammer_vectorized(
                            view,
                            dpar_deg,
                            dmer_deg,
                            overlay_sys,
                            view.input_coord,
                        ),
                    };
                    
                    // Convert RGBA color to normalized RGB for Cairo
                    let r = overlay_color[0] as f64 / 255.0;
                    let g = overlay_color[1] as f64 / 255.0;
                    let b = overlay_color[2] as f64 / 255.0;
                    
                    render_graticule_cairo_with_color(
                        &overlay_graticule,
                        &border_cr,
                        pad as f64,
                        pad as f64,
                        layout.map_w,
                        layout.map_h,
                        r,
                        g,
                        b,
                    );
                }
            }

            if draw_border {
                draw_projection_border_pdf(
                    &border_cr,
                    pad as f64,
                    pad as f64,
                    layout.map_w,
                    layout.map_h,
                    layout.border_width_px,
                );
            }
            // border_cr dropped here
        }
    
        border_surf.flush();
    
        let stride = border_surf.stride() as usize;
        let data = border_surf.data().unwrap();
   
        for y in 0..surf_h {
            for x in 0..surf_w {
                let idx = (y as usize) * stride + (x as usize) * 4;
                let a = data[idx + 3];
                if a == 0 {
                    continue;
                }
        
                let r = data[idx + 2];
                let g = data[idx + 1];
                let b = data[idx];
        
                let dst_x = layout.map_x as i32 + x - pad;
                let dst_y = layout.map_y as i32 + y - pad;
        
                if dst_x < 0 || dst_y < 0 {
                    continue;
                }
        
                let dst_x = dst_x as u32;
                let dst_y = dst_y as u32;
        
                if dst_x >= img.width() || dst_y >= img.height() {
                    continue;
                }
        
                let dst = img.get_pixel(dst_x, dst_y);
                let src = Rgba([r, g, b, a]);
        
                let alpha = a as f32 / 255.0;
        
                let out = Rgba([
                    (src[0] as f32 + dst[0] as f32 * (1.0 - alpha)) as u8,
                    (src[1] as f32 + dst[1] as f32 * (1.0 - alpha)) as u8,
                    (src[2] as f32 + dst[2] as f32 * (1.0 - alpha)) as u8,
                    (a as f32 + dst[3] as f32 * (1.0 - alpha)) as u8,
                ]);
        
                img.put_pixel(dst_x, dst_y, out);
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

        let ticks = generate_colorbar_ticks(
            scale_params.minv,
            scale_params.maxv,
            &scale,
            hist_scale,
        );

        // Draw extend arrows first so ticks render on top
        crate::colorbar::draw_colorbar_extends(
            &params.display.extend,
            layout.cbar_pad,
            layout.cbar_y,
            layout.cbar_w,
            layout.cbar_h,
            cmap,
            &mut img,
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
            let label = format_tick_label_with_units(val, scale, Some(t), latex_rendering, units);
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

    // Render graticule if requested
    if show_graticule {
        // Get the input coordinate system from the view transform
        let grat_coord_sys = grat_coord.unwrap_or(CoordSystem::E);

        // Create a temporary RasterGrid wrapper for the mollweide map region
        let mut grid = RasterGrid {
            width: layout.map_w as u32,
            height: layout.map_h as u32,
            buffer: vec![Rgba([0, 0, 0, 0]); (layout.map_w * layout.map_h) as usize],
            valid: vec![true; (layout.map_w * layout.map_h) as usize],
        };

        // Copy the map region from the image
        for y in 0..layout.map_h as u32 {
            for x in 0..layout.map_w as u32 {
                let src_x = layout.map_x as u32 + x;
                let src_y = layout.map_y as u32 + y;
                if src_x < img.width() && src_y < img.height() {
                    let pixel = img.get_pixel(src_x, src_y);
                    let idx = (y * grid.width + x) as usize;
                    grid.buffer[idx] = *pixel;
                }
            }
        }

        // Render the graticule on the grid
        crate::graticule::render_graticule_mollweide(
            &mut grid,
            view,
            dpar_deg,
            dmer_deg,
            grat_coord_sys,
            view.input_coord,
        );

        // Copy the grid back to the image
        for y in 0..layout.map_h as u32 {
            for x in 0..layout.map_w as u32 {
                let dst_x = layout.map_x as u32 + x;
                let dst_y = layout.map_y as u32 + y;
                if dst_x < img.width() && dst_y < img.height() {
                    let idx = (y * grid.width + x) as usize;
                    img.put_pixel(dst_x, dst_y, grid.buffer[idx]);
                }
            }
        }
    }

    // Draw units label below colorbar
    if show_colorbar
        && let Some(units_str) = units {
            let scale = layout.width / 1200.0;
            let units_y = (cb_layout.tick_label_pad + 30.0 * scale) as i32;
            
            if latex_rendering {
                // Scale LaTeX font size with width (proportional to tick font, no hard minimum)
                let latex_font_size = (cb_layout.tick_font_size * 0.5).round().clamp(3.0, 20.0) as u32;
                // Try to render LaTeX and composite onto image
                if let Some(rendered) = crate::latex_render::render_latex_to_png(units_str, latex_font_size) {
                    // Composite the rendered LaTeX PNG onto the main image
                    let latex_img = image::load_from_memory(&rendered.image_data)
                        .expect("Failed to load rendered LaTeX");
                    let latex_rgba = latex_img.to_rgba8();
                    
                    // Center horizontally
                    let x_offset = (layout.cbar_pad + layout.cbar_w / 2.0 - latex_rgba.width() as f64 / 2.0) as i32;
                    
                    // Composite with alpha blending
                    for (lx, ly, pixel) in latex_rgba.enumerate_pixels() {
                        let img_x = x_offset + lx as i32;
                        let img_y = units_y + ly as i32;
                        
                        if img_x >= 0 && img_x < layout.width as i32 && 
                           img_y >= 0 && img_y < layout.height as i32 {
                            let alpha = pixel[3] as f32 / 255.0;
                            if alpha > 0.01 {
                                let existing = img.get_pixel(img_x as u32, img_y as u32);
                                let blended = Rgba([
                                    ((pixel[0] as f32 * alpha + existing[0] as f32 * (1.0 - alpha)) as u8),
                                    ((pixel[1] as f32 * alpha + existing[1] as f32 * (1.0 - alpha)) as u8),
                                    ((pixel[2] as f32 * alpha + existing[2] as f32 * (1.0 - alpha)) as u8),
                                    255,
                                ]);
                                img.put_pixel(img_x as u32, img_y as u32, blended);
                            }
                        }
                    }
                } else {
                    // Fallback to stripped LaTeX text if rendering fails
                    let units_label = units_str
                        .strip_prefix('$').unwrap_or(units_str)
                        .strip_suffix('$').unwrap_or(units_str);
                    
                    let text_width_est = (units_label.len() as f32 * 
                        cb_layout.tick_font_size as f32 * 0.6) as i32;
                    let center_x = (layout.cbar_pad + layout.cbar_w / 2.0 - text_width_est as f64 / 2.0) as i32;
                    
                    draw_text_mut(
                        &mut img,
                        Rgba([0, 0, 0, 255]),
                        center_x,
                        units_y,
                        FontScale::uniform(cb_layout.tick_font_size as f32),
                        &font,
                        units_label,
                    );
                }
            } else {
                // Non-LaTeX: render as plain text
                if let Some(units_label) = crate::colorbar::format_units_label(false, Some(units_str)) {
                    let text_width_est = (units_label.len() as f32 * 
                        cb_layout.tick_font_size as f32 * 0.6) as i32;
                    let center_x = (layout.cbar_pad + layout.cbar_w / 2.0 - text_width_est as f64 / 2.0) as i32;
                    
                    draw_text_mut(
                        &mut img,
                        Rgba([0, 0, 0, 255]),
                        center_x,
                        units_y,
                        FontScale::uniform(cb_layout.tick_font_size as f32),
                        &font,
                        &units_label,
                    );
                }
            }
        }

    // Draw figure labels (rlabel, llabel)
    draw_figure_labels_png(
        &mut img,
        layout.width as u32,
        layout.height as u32,
        &params.display.rlabel,
        &params.display.llabel,
        latex_rendering,
        params.display.label_font_size,
    );

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



pub fn plot_mollweide_auto(params: MollweideParams) {
    let ext = Path::new(params.plot.filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match ext.as_str() {
        "png" => plot_mollweide_png(params),
        "pdf" => plot_mollweide_pdf(params),
        _ => {
            panic!(
                "Unsupported output format: .{} (expected .png or .pdf)",
                ext
            );
        }
    }
}

/// Draw figure labels on PNG image
/// Supports LaTeX rendering when latex_rendering is true
pub fn draw_figure_labels_png(
    img: &mut RgbaImage,
    width: u32,
    height: u32,
    rlabel: &Option<String>,
    llabel: &Option<String>,
    latex_rendering: bool,
    label_font_size: Option<f32>,
) {
    // Calculate font size for labels
    // Default: 2pt larger than units label (which is 14pt)
    let scale = width as f64 / 800.0;
    let font_size = if let Some(size) = label_font_size {
        size * scale as f32 
    } else {
        // Default: 2pt larger than standard units label (14pt * scale + 2pt)
        (14.0 * scale as f32 + 2.0).max(6.0)
    };
    let font_size_pt = font_size as u32;
    let font_scale = FontScale::uniform(font_size);
    
    let font_data = include_bytes!("../assets/fonts/DejaVuSans.ttf");
    let font = Font::try_from_bytes(font_data as &[u8])
        .expect("Failed to load font");
    
    let text_color = Rgba([0, 0, 0, 255]); // Black text
    
    // Position labels with larger padding to prevent clipping at top
    // Position at the average of ellipse-relative and figure-relative positions
    let padding_x = 20.0 * scale;     // Horizontal padding from edges
    let x_left = padding_x as i32;
    let x_right = (width as f64 - padding_x) as i32;
    let y_label = (padding_x + (height as f64 * 0.095)) as i32;  // Average of two positions
    
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
                    
                    if img_x >= 0 && img_x < width as i32 && 
                       img_y >= 0 && img_y < height as i32 {
                        let alpha = pixel[3] as f32 / 255.0;
                        if alpha > 0.01 {
                            let existing = img.get_pixel(img_x as u32, img_y as u32);
                            let blended = Rgba([
                                ((pixel[0] as f32 * alpha + existing[0] as f32 * (1.0 - alpha)) as u8),
                                ((pixel[1] as f32 * alpha + existing[1] as f32 * (1.0 - alpha)) as u8),
                                ((pixel[2] as f32 * alpha + existing[2] as f32 * (1.0 - alpha)) as u8),
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
                    
                    if img_x >= 0 && img_x < width as i32 && 
                       img_y >= 0 && img_y < height as i32 {
                        let alpha = pixel[3] as f32 / 255.0;
                        if alpha > 0.01 {
                            let existing = img.get_pixel(img_x as u32, img_y as u32);
                            let blended = Rgba([
                                ((pixel[0] as f32 * alpha + existing[0] as f32 * (1.0 - alpha)) as u8),
                                ((pixel[1] as f32 * alpha + existing[1] as f32 * (1.0 - alpha)) as u8),
                                ((pixel[2] as f32 * alpha + existing[2] as f32 * (1.0 - alpha)) as u8),
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



/// Convert a scalar map value to a PixelValue (handles underflow/overflow)
#[inline]
pub fn map_to_pixel_value(
    val: f64,
    minv: f64,
    maxv: f64,
    scale: Scale,
    neg_mode: NegMode,
    hist_scale: Option<&HistogramScale>,
) -> PixelValue {
    scale_value(val, minv, maxv, scale, neg_mode, hist_scale)
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
    params: crate::params::RenderGridParams,
    grid: &mut RasterGrid,
) {
    let width = grid.width;
    let height = grid.height;

    // Precompute gamma value to avoid repeated checks
    let gamma_inv = if (params.gamma - 1.0).abs() < f64::EPSILON {
        1.0
    } else {
        params.gamma
    };

    for py in 0..height {
        for px in 0..width {
            // Use pixel_to_ang for all projections (handles each type correctly)
            if let Some((lon, lat)) = params.proj.pixel_to_ang(px, py, grid) {
                let theta = std::f64::consts::PI / 2.0 - lat;

                let pixel_val = match sample_healpix(params.map, params.meta, params.view, theta, lon) {
                    Some(val) => scale_value(
                        val,
                        params.scale.minv,
                        params.scale.maxv,
                        params.scale_type,
                        params.neg_mode,
                        params.hist_scale,
                    ),
                    None => PixelValue::Bad,
                };

                // Inline pixel_value_to_rgba for better performance
                let rgba = match pixel_val {
                    PixelValue::Color(t) => {
                        let t = if gamma_inv == 1.0 { t } else { t.powf(gamma_inv) };
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

                // Use unchecked access for hot path (bounds guaranteed by loop)
                unsafe {
                    grid.set_pixel_unchecked(px, py, rgba);
                }
            } else {
                grid.set_valid(px, py, false);
            }
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
            if let Some(p) = grid.get_pixel_if_valid(x, y) {
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
// ============================================================================
// GNOMONIC PROJECTION PLOTTING
// ============================================================================

/// Plot a gnomonic projection to PNG
pub fn plot_gnomonic_png(params: GnomonicParams) {
    let map = params.plot.map;
    let width = params.plot.width;
    let filename = params.plot.filename;
    let minv = params.scale.minv;
    let maxv = params.scale.maxv;
    let cmap = params.color.cmap;
    let show_colorbar = params.display.show_colorbar;
    let transparent = params.display.transparent;
    let gamma = params.scale.gamma;
    let scale = params.scale.scale;
    let neg_mode = params.scale.neg_mode;
    let bad_color = params.color.bad_color;
    let bg_color = params.color.bg_color;
    let meta = params.meta;
    let latex_rendering = params.display.latex_rendering;
    let units = params.display.units.as_deref();
    let view = params.view;
    let lon_deg = params.lon_deg;
    let lat_deg = params.lat_deg;
    let _fov_arcmin = params.fov_arcmin;
    let resolution_arcmin = params.resolution_arcmin;
    let show_graticule = params.graticule.show_graticule;
    let grat_dlon = params.graticule.dpar_deg;
    let grat_dlat = params.graticule.dmer_deg;
    let grat_overlay = params.graticule.grat_overlay;
    let _overlay_color = params.graticule.overlay_color;
    let roll_deg = params.roll_deg;

    use crate::gnomonic::GnomonicProjection;

    let (layout, cb_layout) = compute_gnomonic_layout(width as f64, show_colorbar, params.display.tick_direction.clone());

    let font_data = include_bytes!("../assets/fonts/DejaVuSans.ttf");
    let font = Font::try_from_bytes(font_data as &[u8])
        .expect("Failed to load font");

    let roll_rad = roll_deg * std::f64::consts::PI / 180.0;
    let proj = GnomonicProjection::with_roll(lon_deg, lat_deg, resolution_arcmin, roll_rad);
    
    let mut grid = RasterGrid::new(width, width);
    
    let scale_params = compute_mollweide_scale(map, minv, maxv, gamma, scale);
    
    let hist_scale = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit { min: minv, max: maxv },
            _ => HistogramRange::Full,
        };
        Some(build_histogram_scale(map, range, 1024))
    } else {
        None
    };

    render_projection_to_grid(
        crate::params::RenderGridParams {
            map,
            proj: &proj,
            scale: &scale_params,
            cmap,
            scale_type: scale,
            neg_mode,
            gamma,
            bad_color,
            meta,
            hist_scale: hist_scale.as_ref(),
            view,
        },
        &mut grid,
    );

    // Add graticule if requested
    if show_graticule {
        use crate::gnomonic_graticule::{render_gnomonic_local_grid, render_gnomonic_sky_overlay};
        
        // Always render local graticule in black
        render_gnomonic_local_grid(&mut grid, &proj, grat_dlon, grat_dlat);
        
        // Render overlay if specified
        if let Some(overlay_sys) = grat_overlay {
            render_gnomonic_sky_overlay(
                &mut grid,
                &proj,
                view,
                grat_dlon,
                grat_dlat,
                overlay_sys,
                view.input_coord,
                _overlay_color,
            );
        }
    }

    let bg = Rgba([bg_color[0], bg_color[1], bg_color[2], 
        if transparent {0} else {255}]);
    
    let mut img = RgbaImage::from_pixel(layout.width as u32, layout.height as u32, bg);

    // Blit grid to image at map position
    for y in 0..width {
        for x in 0..width {
            if let Some(pixel) = grid.get_pixel_if_valid(x, y) {
                let img_x = layout.map_x as u32 + x;
                let img_y = layout.map_y as u32 + y;
                if img_x < img.width() && img_y < img.height() {
                    img.put_pixel(img_x, img_y, pixel);
                }
            }
        }
    }

    // Add colorbar if requested
    if show_colorbar {
        let mut sink = PngSink {
            img: &mut img,
            x0: layout.cbar_x as u32,
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
        
        // Add tick marks and labels
        let ticks = generate_colorbar_ticks(
            scale_params.minv,
            scale_params.maxv,
            &scale,
            hist_scale.as_ref(),
        );

        let major_tick_height = cb_layout.major_tick_height as u32;
        let major_tick_width = cb_layout.major_tick_width as u32;
        let tick_bottom = cb_layout.tick_bottom as u32;
        let tick_top = tick_bottom.saturating_sub(major_tick_height);

        // Draw major ticks and labels
        for (&t, &val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
            let px = (layout.cbar_x + (t * layout.cbar_w).round()) as u32;
            for dx in 0..major_tick_width as i32 {
                let x = (px as i32 + dx) as u32;
                if x < layout.width as u32 {
                    for py in tick_top..=tick_bottom {
                        if py < img.height() {
                            img.put_pixel(x, py, Rgba([0, 0, 0, 255]));
                        }
                    }
                }
            }
            
            // Draw label
            let label = format_tick_label_with_units(val, scale, Some(t), latex_rendering, units);
            let text_width_est = (label.len() as f32 * 
                cb_layout.tick_font_size as f32 * 0.6) as i32;
            let text_x = px as i32 - text_width_est / 2;
        
            draw_text_mut(
                &mut img,
                Rgba([0, 0, 0, 255]),
                text_x,
                cb_layout.tick_label_pad as i32,
                FontScale::uniform(cb_layout.tick_font_size as f32),
                &font,
                &label,
            );
        }

        // Draw colorbar extend arrows if needed
        crate::colorbar::draw_colorbar_extends(
            &params.display.extend,
            layout.cbar_x,
            layout.cbar_y,
            layout.cbar_w,
            layout.cbar_h,
            cmap,
            &mut img,
        );

        // Draw units label below colorbar
        if let Some(units_str) = units {
            let scale = layout.width / 1200.0;
            let units_y = (cb_layout.tick_label_pad + 25.0 * scale) as i32;
            
            if latex_rendering {
                // Scale LaTeX font size with width (proportional to tick font, no hard minimum)
                let latex_font_size = (cb_layout.tick_font_size * 0.5).round().clamp(3.0, 20.0) as u32;
                // Try to render LaTeX and composite onto image
                if let Some(rendered) = crate::latex_render::render_latex_to_png(units_str, latex_font_size) {
                    // Composite the rendered LaTeX PNG onto the main image
                    let latex_img = image::load_from_memory(&rendered.image_data)
                        .expect("Failed to load rendered LaTeX");
                    let latex_rgba = latex_img.to_rgba8();
                    
                    // Center horizontally
                    let x_offset = (layout.cbar_x + layout.cbar_w / 2.0 - latex_rgba.width() as f64 / 2.0) as i32;
                    
                    // Composite with alpha blending
                    for (lx, ly, pixel) in latex_rgba.enumerate_pixels() {
                        let img_x = x_offset + lx as i32;
                        let img_y = units_y + ly as i32;
                        
                        if img_x >= 0 && img_x < layout.width as i32 && 
                           img_y >= 0 && img_y < layout.height as i32 {
                            let alpha = pixel[3] as f32 / 255.0;
                            if alpha > 0.01 {
                                let existing = img.get_pixel(img_x as u32, img_y as u32);
                                let blended = Rgba([
                                    ((pixel[0] as f32 * alpha + existing[0] as f32 * (1.0 - alpha)) as u8),
                                    ((pixel[1] as f32 * alpha + existing[1] as f32 * (1.0 - alpha)) as u8),
                                    ((pixel[2] as f32 * alpha + existing[2] as f32 * (1.0 - alpha)) as u8),
                                    255,
                                ]);
                                img.put_pixel(img_x as u32, img_y as u32, blended);
                            }
                        }
                    }
                } else {
                    // Fallback to stripped LaTeX text if rendering fails
                    let units_label = units_str
                        .strip_prefix('$').unwrap_or(units_str)
                        .strip_suffix('$').unwrap_or(units_str);
                    
                    let text_width_est = (units_label.len() as f32 * 
                        cb_layout.tick_font_size as f32 * 0.6) as i32;
                    let center_x = (layout.cbar_x + layout.cbar_w / 2.0 - text_width_est as f64 / 2.0) as i32;
                    
                    draw_text_mut(
                        &mut img,
                        Rgba([0, 0, 0, 255]),
                        center_x,
                        units_y,
                        FontScale::uniform(cb_layout.tick_font_size as f32),
                        &font,
                        units_label,
                    );
                }
            } else {
                // Non-LaTeX: render as plain text
                if let Some(units_label) = crate::colorbar::format_units_label(false, Some(units_str)) {
                    let text_width_est = (units_label.len() as f32 * 
                        cb_layout.tick_font_size as f32 * 0.6) as i32;
                    let center_x = (layout.cbar_x + layout.cbar_w / 2.0 - text_width_est as f64 / 2.0) as i32;
                    
                    draw_text_mut(
                        &mut img,
                        Rgba([0, 0, 0, 255]),
                        center_x,
                        units_y,
                        FontScale::uniform(cb_layout.tick_font_size as f32),
                        &font,
                        &units_label,
                    );
                }
            }
        }
    }

    // Save PNG
    img.save(filename).expect("Failed to save PNG");
}

/// Plot a gnomonic projection to PDF
pub fn plot_gnomonic_pdf(params: GnomonicParams) {
    let map = params.plot.map;
    let width = params.plot.width;
    let filename = params.plot.filename;
    let minv = params.scale.minv;
    let maxv = params.scale.maxv;
    let cmap = params.color.cmap;
    let show_colorbar = params.display.show_colorbar;
    let _transparent = params.display.transparent;
    let gamma = params.scale.gamma;
    let scale = params.scale.scale;
    let neg_mode = params.scale.neg_mode;
    let bad_color = params.color.bad_color;
    let _bg_color = params.color.bg_color;
    let meta = params.meta;
    let _latex_rendering = params.display.latex_rendering;
    let _units = params.display.units.as_deref();
    let view = params.view;
    let lon_deg = params.lon_deg;
    let lat_deg = params.lat_deg;
    let _fov_arcmin = params.fov_arcmin;
    let resolution_arcmin = params.resolution_arcmin;
    let show_graticule = params.graticule.show_graticule;
    let grat_dlon = params.graticule.dpar_deg;
    let grat_dlat = params.graticule.dmer_deg;
    let grat_overlay = params.graticule.grat_overlay;
    let _overlay_color = params.graticule.overlay_color;
    let roll_deg = params.roll_deg;

    use crate::gnomonic::GnomonicProjection;

    let (layout, _cb_layout) = compute_gnomonic_layout(width as f64, show_colorbar, params.display.tick_direction.clone());
    
    let roll_rad = roll_deg * std::f64::consts::PI / 180.0;
    let proj = GnomonicProjection::with_roll(lon_deg, lat_deg, resolution_arcmin, roll_rad);
    let img_height = width; // Square image for gnomonic

    // Render to grid first (just the map region)
    let mut grid = RasterGrid::new(width, img_height);
    
    let scale_params = compute_mollweide_scale(map, minv, maxv, gamma, scale);
    
    let hist_scale = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit { min: minv, max: maxv },
            _ => HistogramRange::Full,
        };
        Some(build_histogram_scale(map, range, 1024))
    } else {
        None
    };

    render_projection_to_grid(
        crate::params::RenderGridParams {
            map,
            proj: &proj,
            scale: &scale_params,
            cmap,
            scale_type: scale,
            neg_mode,
            gamma,
            bad_color,
            meta,
            hist_scale: hist_scale.as_ref(),
            view,
        },
        &mut grid,
    );

    // Add graticule if requested
    if show_graticule {
        use crate::gnomonic_graticule::{render_gnomonic_local_grid, render_gnomonic_sky_overlay};
        
        // Always render local graticule in black
        render_gnomonic_local_grid(&mut grid, &proj, grat_dlon, grat_dlat);
        
        // Render overlay if specified
        if let Some(overlay_sys) = grat_overlay {
            render_gnomonic_sky_overlay(
                &mut grid,
                &proj,
                view,
                grat_dlon,
                grat_dlat,
                overlay_sys,
                CoordSystem::G,  // Map input is Galactic by default
                _overlay_color,
            );
        }
    }

    // Create image surface for the full layout (including colorbar)
    let mut img_surface = ImageSurface::create(Format::ARgb32, layout.width as i32, layout.height as i32)
        .expect("Failed to create image surface");
    
    // Fill with white background
    {
        let _stride = img_surface.stride() as usize;
        let mut data = img_surface.data().expect("Failed to get surface data");
        
        for idx in (0..data.len()).step_by(4) {
            data[idx] = 255;     // B
            data[idx + 1] = 255; // G
            data[idx + 2] = 255; // R
            data[idx + 3] = 255; // A
        }
    }

    // Blit grid to image at map position
    {
        let stride = img_surface.stride() as usize;
        let mut data = img_surface.data().expect("Failed to get surface data");
        
        for y in 0..img_height as usize {
            for x in 0..width as usize {
                if let Some(pixel) = grid.get_pixel_if_valid(x as u32, y as u32) {
                    let img_x = layout.map_x as usize + x;
                    let img_y = layout.map_y as usize + y;
                    
                    if img_x < layout.width as usize && img_y < layout.height as usize {
                        let idx = img_y * stride + img_x * 4;
                        if idx + 3 < data.len() {
                            data[idx] = pixel[2];     // B
                            data[idx + 1] = pixel[1]; // G
                            data[idx + 2] = pixel[0]; // R
                            data[idx + 3] = pixel[3]; // A
                        }
                    }
                }
            }
        }
    }

    // Flush to ensure data is written
    img_surface.flush();

    // Create PDF surface with matching layout dimensions
    let pdf_surface = PdfSurface::new(layout.width, layout.height, filename)
        .expect("Failed to create PDF surface");
    let cr = Context::new(&pdf_surface).expect("Failed to create Cairo context");

    // Paint the image surface onto the PDF at 1:1 scale
    cr.set_source_surface(&img_surface, 0.0, 0.0).expect("Failed to set source");
    cr.paint().expect("Failed to paint");

    // Add proper colorbar with ticks and labels if requested
    if show_colorbar {
        use crate::render::pdf::draw_colorbar_pdf;
        let cb_layout = compute_gnomonic_layout(width as f64, show_colorbar, params.display.tick_direction.clone()).1;
        
        draw_colorbar_pdf(
            &cr,
            cb_layout,
            crate::params::ColorbarParams {
                cmap,
                minv: minv.unwrap_or(0.0),
                maxv: maxv.unwrap_or(1.0),
                scale_type: scale,
                gamma,
                hist_scale: hist_scale.as_ref(),
                latex_rendering: false,
                units: Some("μK_CMB"),
                extend: &params.display.extend,
                units_font_size: params.display.units_font_size,
            },
        );
    }

    pdf_surface.finish();
}

/// Plot gnomonic projection with automatic format detection
pub fn plot_gnomonic_auto(params: GnomonicParams) {
    let map = params.plot.map;
    let _width = params.plot.width;
    let filename = params.plot.filename;
    let minv = params.scale.minv;
    let maxv = params.scale.maxv;
    let cmap = params.color.cmap;
    let show_colorbar = params.display.show_colorbar;
    let transparent = params.display.transparent;
    let gamma = params.scale.gamma;
    let scale = params.scale.scale;
    let neg_mode = params.scale.neg_mode;
    let bad_color = params.color.bad_color;
    let bg_color = params.color.bg_color;
    let meta = params.meta;
    let latex_rendering = params.display.latex_rendering;
    let units = params.display.units.as_deref();
    let view = params.view;
    let lon_deg = params.lon_deg;
    let lat_deg = params.lat_deg;
    let fov_arcmin = params.fov_arcmin;
    let resolution_arcmin = params.resolution_arcmin;
    let show_graticule = params.graticule.show_graticule;
    let grat_dlon = params.graticule.dpar_deg;
    let grat_dlat = params.graticule.dmer_deg;
    let grat_overlay = params.graticule.grat_overlay;
    let overlay_color = params.graticule.overlay_color;
    let roll_deg = params.roll_deg;

    // Compute image width from field of view and resolution
    // This ensures the FOV parameter is actually respected
    let width = (fov_arcmin / resolution_arcmin).ceil() as u32;
    
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    // Reconstruct bundled params for dispatch
    let bundled_params = GnomonicParams {
        plot: crate::params::PlotData {
            map,
            width,
            filename,
        },
        scale: crate::params::ScaleParams {
            minv,
            maxv,
            gamma,
            scale,
            neg_mode,
        },
        color: crate::params::ColorParams {
            cmap,
            bad_color,
            bg_color,
        },
        display: crate::params::DisplayParams {
            show_colorbar,
            transparent,
            draw_border: false,
            latex_rendering,
            units: units.map(|s| s.to_string()),
            extend: params.display.extend.clone(),
            tick_direction: params.display.tick_direction.clone(),
            tick_font_size: params.display.tick_font_size,
            units_font_size: params.display.units_font_size,
            rlabel: params.display.rlabel.clone(),
            llabel: params.display.llabel.clone(),
            label_font_size: params.display.label_font_size,
        },
        graticule: crate::params::GraticuleParams {
            show_graticule,
            grat_coord: None,
            grat_overlay,
            overlay_color,
            show_labels: false,
            dpar_deg: grat_dlon,
            dmer_deg: grat_dlat,
        },
        meta,
        view,
        lon_deg,
        lat_deg,
        fov_arcmin,
        resolution_arcmin,
        roll_deg,
        grat_line_width: 1,
    };

    match ext.as_str() {
        "png" => plot_gnomonic_png(bundled_params),
        "pdf" => plot_gnomonic_pdf(bundled_params),
        _ => {
            panic!(
                "Unsupported output format: .{} (expected .png or .pdf)",
                ext
            );
        }
    }
}

/// Projection types for conditional rendering
#[derive(Clone, Copy, Debug)]
enum ProjectionType {
    Mollweide,
    Hammer,
}

/// Plot a Hammer projection map as PNG.
pub fn plot_hammer_png(params: crate::params::HammerParams) {
    let mollweide_params = crate::params::MollweideParams {
        plot: params.plot,
        scale: params.scale,
        color: params.color,
        display: params.display,
        graticule: params.graticule,
        meta: params.meta,
        view: params.view,
    };
    _plot_mollweide_png_impl_projected(mollweide_params, render_hammer_pixels, ProjectionType::Hammer);
}

/// Plot a Hammer projection map as PDF.
pub fn plot_hammer_pdf(params: crate::params::HammerParams) {
    let mollweide_params = crate::params::MollweideParams {
        plot: params.plot,
        scale: params.scale,
        color: params.color,
        display: params.display,
        graticule: params.graticule,
        meta: params.meta,
        view: params.view,
    };
    _plot_hammer_pdf_impl(mollweide_params, render_hammer_pixels);
}

/// Internal Hammer PDF plotting implementation
fn _plot_hammer_pdf_impl<F>(params: MollweideParams, pixel_renderer: F) 
where
    F: Fn(
        crate::params::RenderMollweideParams,
        MollweideLayout,
        &mut dyn PixelSink,
        Option<DebugOverlay>,
    ),
{
    let map = params.plot.map;
    let width = params.plot.width;
    let filename = params.plot.filename;
    let minv = params.scale.minv;
    let maxv = params.scale.maxv;
    let cmap = params.color.cmap;
    let show_colorbar = params.display.show_colorbar;
    let transparent = params.display.transparent;
    let draw_border = params.display.draw_border;
    let gamma = params.scale.gamma;
    let scale = params.scale.scale;
    let neg_mode = params.scale.neg_mode;
    let bad_color = params.color.bad_color;
    let _bg_color = params.color.bg_color;
    let meta = params.meta;
    let latex_rendering = params.display.latex_rendering;
    let units = params.display.units.as_deref();
    let view = params.view;
    let show_graticule = params.graticule.show_graticule;
    let grat_coord = params.graticule.grat_coord;
    let grat_overlay = params.graticule.grat_overlay;
    let overlay_color = params.graticule.overlay_color;
    let _show_labels = params.graticule.show_labels;
    let dpar_deg = params.graticule.dpar_deg;
    let dmer_deg = params.graticule.dmer_deg;

    let (layout, cb_layout) = compute_mollweide_layout(width as f64, show_colorbar, params.display.tick_direction.clone());

    let mut values: Vec<f64> = map
        .iter()
        .filter(|&v| is_seen(*v))
        .copied()
        .collect();

    
    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_unstable_by(unsafe_float_cmp);


    let surface_pdf = PdfSurface::new(
        layout.width,
        layout.height,
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

    let scale_params = compute_mollweide_scale(map, minv, maxv, gamma, scale);

    let hist_scale = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit { min: minv, max: maxv },
            _ => HistogramRange::Full,
        };
    
        Some(
            &build_histogram_scale(
                map,
                range,
                1024, // number of bins
            )
        )
    } else {
        None
    };

    
    let mut sink = CairoImageSink { cr: &cr_img };
    let debug_overlay = if cfg!(feature = "debug_overlay") {
        Some(DebugOverlay::grid_only())
    } else {
        None
    };
    
    pixel_renderer(
        crate::params::RenderMollweideParams {
            map,
            scale: &scale_params,
            cmap,
            gamma,
            scale_type: scale,
            neg_mode,
            bad_color,
            meta,
            hist_scale,
            view,
        },
        layout,
        &mut sink,
        debug_overlay,
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

    // Draw graticule BEFORE border (so border appears on top)
    if show_graticule {
        use crate::graticule::{render_graticule_hammer_vectorized, render_graticule_cairo, render_graticule_cairo_with_color};
        
        let grat_coord_sys = grat_coord.unwrap_or(CoordSystem::E);
        
        let graticule = render_graticule_hammer_vectorized(
            view,
            dpar_deg,
            dmer_deg,
            grat_coord_sys,
            view.input_coord,
        );
        
        // Render primary graticule in black
        render_graticule_cairo(
            &graticule,
            &cr_pdf,
            layout.map_x,
            layout.map_y,
            layout.map_w,
            layout.map_h,
        );

        // Render secondary graticule overlay if specified
        if let Some(overlay_sys) = grat_overlay {
            let overlay_graticule = render_graticule_hammer_vectorized(
                view,
                dpar_deg,
                dmer_deg,
                overlay_sys,
                view.input_coord,
            );
            
            // Convert RGBA color to normalized RGB for Cairo
            let r = overlay_color[0] as f64 / 255.0;
            let g = overlay_color[1] as f64 / 255.0;
            let b = overlay_color[2] as f64 / 255.0;
            
            render_graticule_cairo_with_color(
                &overlay_graticule,
                &cr_pdf,
                layout.map_x,
                layout.map_y,
                layout.map_w,
                layout.map_h,
                r,
                g,
                b,
            );
        }
    }

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
            crate::params::ColorbarParams {
                cmap,
                minv: scale_params.minv,
                maxv: scale_params.maxv,
                scale_type: scale,
                gamma,
                hist_scale,
                latex_rendering,
                units,
                extend: &params.display.extend,
                units_font_size: params.display.units_font_size,
            },
        );
    }

    surface_pdf.finish();
}

/// Automatically choose PNG or PDF based on file extension for Hammer projection.
pub fn plot_hammer_auto(params: crate::params::HammerParams) {
    let filename = params.plot.filename;
    let ext = std::path::Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "png" => plot_hammer_png(params),
        "pdf" => plot_hammer_pdf(params),
        _ => {
            panic!(
                "Unsupported output format: .{} (expected .png or .pdf)",
                ext
            );
        }
    }
}