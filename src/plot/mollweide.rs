use super::{
    DebugOverlay, MollweideScale, blit_grid_to_sink, draw_debug_overlay_raster,
    draw_figure_labels_png, fill_grid_background, percentile, render_projection_to_grid,
};
use crate::colorbar::{format_tick_label_with_units, render_colorbar_gradient};
use crate::healpix::is_seen;
use crate::layout::{MollweideLayout, compute_mollweide_layout};
use crate::params::MollweideParams;
use crate::render::pdf::{draw_colorbar_pdf, draw_projection_border_pdf};
use crate::render::raster::RasterGrid;
use crate::rotation::CoordSystem;
use crate::scale::{
    HistogramRange, Scale, build_histogram_scale, generate_colorbar_ticks, unsafe_float_cmp,
};
use crate::{PixelSink, PngSink};
use ab_glyph::{FontRef, PxScale};
use cairo::{Context, Format, ImageSurface, PdfSurface};
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use std::path::Path;

/// Compute percentile memory-efficiently without allocating full copy for large maps
/// Uses sampling for maps > 50M pixels to avoid 12+ GB allocations
fn compute_percentile_from_map(map: &[f64], percentile_pct: f64, max_sample_size: usize) -> f64 {
    use std::cmp::Ordering;

    // For very large maps, sample instead of allocating full vector
    let skip_rate = if map.len() > max_sample_size {
        (map.len() as f64 / max_sample_size as f64).ceil() as usize
    } else {
        1
    };

    // Collect samples (avoids allocating full 6.4 GB vector for 806M pixel maps)
    let estimated_samples = map.len().div_ceil(skip_rate);
    let mut samples = Vec::with_capacity(estimated_samples.min(max_sample_size * 2));

    for (i, &val) in map.iter().enumerate() {
        if is_seen(val) && (i % skip_rate == 0 || skip_rate == 1) {
            samples.push(val);
        }
    }

    if samples.is_empty() {
        return 0.0;
    }

    // Sort only the sample (much faster than sorting 806M pixels)
    samples.sort_unstable_by(|a, b| {
        if a < b {
            Ordering::Less
        } else if a > b {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });

    // Compute percentile of samples
    let n = samples.len();
    let rank = (percentile_pct / 100.0) * (n - 1) as f64;
    let idx = rank.floor() as usize;
    let frac = rank - idx as f64;

    if idx + 1 < n {
        samples[idx] * (1.0 - frac) + samples[idx + 1] * frac
    } else {
        samples[idx]
    }
}

pub fn compute_mollweide_scale(
    map: &[f64],
    minv: Option<f64>,
    maxv: Option<f64>,
    gamma: f64,
    scale: Scale,
) -> MollweideScale {
    const MAX_PERCENTILE_SAMPLE_SIZE: usize = 10_000_000; // 10M samples = 80 MB, not 6.4 GB

    // **Tier 1.2: Memory optimization for huge maps**
    // For maps > 50M pixels, use sampling instead of allocating full vector
    // Reduces 806M pixel map memory from 6.4 GB to 80 MB
    let (data_min, data_max, p5, p95) = if map.len() > 50_000_000 {
        // Large map: use efficient streaming computation
        let mut min = f64::INFINITY;
        let mut max = f64::NEG_INFINITY;

        for &val in map.iter() {
            if is_seen(val) {
                if val < min {
                    min = val;
                }
                if val > max {
                    max = val;
                }
            }
        }

        if min.is_infinite() {
            panic!("Map contains no valid HEALPix values");
        }

        let p5 = compute_percentile_from_map(map, 5.0, MAX_PERCENTILE_SAMPLE_SIZE);
        let p95 = compute_percentile_from_map(map, 95.0, MAX_PERCENTILE_SAMPLE_SIZE);
        (min, max, p5, p95)
    } else {
        // Small map: use original accurate method
        let mut values: Vec<f64> = map.iter().filter(|v| is_seen(**v)).copied().collect();

        if values.is_empty() {
            panic!("Map contains no valid HEALPix values");
        }

        values.sort_unstable_by(unsafe_float_cmp);

        let data_min = *values.first().unwrap();
        let data_max = *values.last().unwrap();
        let p5 = percentile(&values, 5.0);
        let p95 = percentile(&values, 95.0);
        (data_min, data_max, p5, p95)
    };

    let (minv, maxv) = match scale {
        // 🔴 Histogram scale overrides percentiles
        Scale::Histogram => match (minv, maxv) {
            (Some(lo), Some(hi)) => (lo, hi),
            _ => (data_min, data_max),
        },

        // 🟢 All other scales keep percentile default
        _ => match (minv, maxv) {
            (Some(lo), Some(hi)) => (lo, hi),
            _ => (p5, p95),
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
        && overlay.show_background
    {
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
            mask: params.mask,
            scale_cache: params.scale_cache,
            underflow: (255, 0, 0),
            overflow: (0, 0, 255),
        },
        &mut grid,
    );

    // Draw debug overlay only if provided
    if let Some(overlay) = debug_overlay {
        draw_debug_overlay_raster(&mut grid, overlay);
    }

    blit_grid_to_sink(&grid, sink, 0, 0);
}

// Non-generic storage for PDF rendering setup
#[allow(dead_code)]
struct MollweidePdfSetup<'a> {
    surface_pdf: PdfSurface,
    cr_pdf: Context,
    layout: MollweideLayout,
    cb_layout: crate::layout::ColorbarLayout,
    map: &'a [f64],
    map_w_int: u32,
    map_h_int: u32,
    pixel_buffer: RgbaImage,
    scale_params: MollweideScale,
    hist_scale_opt: Option<crate::scale::HistogramScale>,
    scale_cache: crate::scale::ScaleCache,
    show_colorbar: bool,
    draw_border: bool,
    show_graticule: bool,
    latex_rendering: bool,
    transparent: bool,
    // Graticule params
    grat_coord: Option<CoordSystem>,
    grat_overlay: Option<CoordSystem>,
    overlay_color: Rgba<u8>,
    dpar_deg: f64,
    dmer_deg: f64,
    // Display params
    rlabel: Option<String>,
    llabel: Option<String>,
    label_font_size: Option<f32>,
    extend: crate::cli::Extend,
    units_font_size: Option<f32>,
    // Colorbar params
    cmap: &'a crate::colormap::Colormap,
    scale_type: Scale,
    units: Option<String>,
    gamma: f64,
    // Other params
    view: &'a crate::rotation::ViewTransform,
    filename: String,
    neg_mode: crate::NegMode,
    bad_color: image::Rgba<u8>,
    meta: crate::healpix::HealpixMeta,
}

/// Non-generic setup for PDF rendering - only generic call is the pixel_renderer
fn _mollweide_pdf_setup_impl<'a>(params: &'a MollweideParams<'a>) -> MollweidePdfSetup<'a> {
    let map = params.plot.map;
    let width = params.plot.width;
    let filename = params.plot.filename.to_string();
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
    let meta = params.meta;
    let latex_rendering = params.display.latex_rendering;
    let units = params.display.units.clone();
    let view = params.view;
    let show_graticule = params.graticule.show_graticule;
    let grat_coord = params.graticule.grat_coord;
    let grat_overlay = params.graticule.grat_overlay;
    let overlay_color = params.graticule.overlay_color;
    let dpar_deg = params.graticule.dpar_deg;
    let dmer_deg = params.graticule.dmer_deg;

    let (layout, cb_layout) = compute_mollweide_layout(
        width as f64,
        show_colorbar,
        params.display.tick_direction.clone(),
    );

    let surface_pdf = PdfSurface::new(layout.width, layout.height, &filename)
        .expect("Failed to create PDF surface");

    let cr_pdf = Context::new(&surface_pdf).unwrap();

    if transparent {
        cr_pdf.set_operator(cairo::Operator::Source);
        cr_pdf.set_source_rgba(0.0, 0.0, 0.0, 0.0);
        cr_pdf.paint().unwrap();
    }

    // Create raster surface (not used for rendering but kept for compatibility)
    let _surface_img = ImageSurface::create(
        Format::ARgb32,
        (layout.map_w + 2.0 * layout.map_pad) as i32,
        (layout.map_h + 2.0 * layout.map_pad) as i32,
    )
    .expect("Failed to create image surface");

    let scale_params = compute_mollweide_scale(map, minv, maxv, gamma, scale);

    let hist_scale_opt = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit {
                min: minv,
                max: maxv,
            },
            _ => HistogramRange::Full,
        };

        Some(build_histogram_scale(
            map, range, 1024, // number of bins
        ))
    } else {
        None
    };

    // Pre-compute scale cache for fast pixel rendering
    let scale_cache = crate::scale::ScaleCache::new(scale_params.minv, scale_params.maxv, scale);

    // Create in-memory pixel buffer
    let map_w_int = (layout.map_w + 2.0 * layout.map_pad) as u32;
    let map_h_int = (layout.map_h + 2.0 * layout.map_pad) as u32;

    let mut pixel_buffer = crate::render::create_image_buffer_uninitialized(map_w_int, map_h_int);

    // Clear buffer background
    let bg_color = if transparent {
        image::Rgba([0, 0, 0, 0])
    } else {
        image::Rgba([255, 255, 255, 255])
    };
    for pixel in pixel_buffer.pixels_mut() {
        *pixel = bg_color;
    }

    MollweidePdfSetup {
        surface_pdf,
        cr_pdf,
        layout,
        cb_layout,
        map,
        map_w_int,
        map_h_int,
        pixel_buffer,
        scale_params,
        hist_scale_opt,
        scale_cache,
        show_colorbar,
        draw_border,
        show_graticule,
        latex_rendering,
        transparent,
        grat_coord,
        grat_overlay,
        overlay_color,
        dpar_deg,
        dmer_deg,
        rlabel: params.display.rlabel.clone(),
        llabel: params.display.llabel.clone(),
        label_font_size: params.display.label_font_size,
        extend: params.display.extend.clone().clone().clone(),
        units_font_size: params.display.units_font_size,
        cmap,
        scale_type: scale,
        units,
        gamma,
        view,
        filename,
        neg_mode,
        bad_color,
        meta,
    }
}

/// Non-generic finalization of PDF rendering
fn _mollweide_pdf_finalize_impl<'a>(
    setup: MollweidePdfSetup<'a>,
    _debug_overlay: Option<DebugOverlay>,
) {
    // Convert pre-rendered image to Cairo surface and paint onto PDF surface
    let mut argb_buffer = Vec::with_capacity(setup.pixel_buffer.len() * 4);
    for pixel in setup.pixel_buffer.pixels() {
        argb_buffer.push(pixel[2]); // B
        argb_buffer.push(pixel[1]); // G
        argb_buffer.push(pixel[0]); // R
        argb_buffer.push(pixel[3]); // A
    }

    if let Ok(pixel_surface) = cairo::ImageSurface::create_for_data(
        argb_buffer,
        cairo::Format::ARgb32,
        setup.map_w_int as i32,
        setup.map_h_int as i32,
        setup.map_w_int as i32 * 4,
    ) {
        let _ =
            setup
                .cr_pdf
                .set_source_surface(&pixel_surface, setup.layout.map_x, setup.layout.map_y);
        setup.cr_pdf.paint().unwrap();
    }

    // Draw graticule BEFORE border
    if setup.show_graticule {
        use crate::graticule::{
            render_graticule_cairo, render_graticule_cairo_with_color,
            render_graticule_mollweide_vectorized,
        };

        let grat_coord_sys = setup.grat_coord.unwrap_or(CoordSystem::E);

        let graticule = render_graticule_mollweide_vectorized(
            setup.view,
            setup.dpar_deg,
            setup.dmer_deg,
            grat_coord_sys,
            setup.view.input_coord,
        );

        render_graticule_cairo(
            &graticule,
            &setup.cr_pdf,
            setup.layout.map_x,
            setup.layout.map_y,
            setup.layout.map_w,
            setup.layout.map_h,
        );

        // Render secondary graticule overlay if specified
        if let Some(overlay_sys) = setup.grat_overlay {
            let overlay_graticule = render_graticule_mollweide_vectorized(
                setup.view,
                setup.dpar_deg,
                setup.dmer_deg,
                overlay_sys,
                setup.view.input_coord,
            );

            let r = setup.overlay_color[0] as f64 / 255.0;
            let g = setup.overlay_color[1] as f64 / 255.0;
            let b = setup.overlay_color[2] as f64 / 255.0;

            render_graticule_cairo_with_color(
                &overlay_graticule,
                &setup.cr_pdf,
                crate::params::GeometryRect {
                    x: setup.layout.map_x,
                    y: setup.layout.map_y,
                    w: setup.layout.map_w,
                    h: setup.layout.map_h,
                },
                (r, g, b),
            );
        }
    }

    // Draw vector border ON TOP
    if setup.draw_border {
        draw_projection_border_pdf(
            &setup.cr_pdf,
            setup.layout.map_x,
            setup.layout.map_y,
            setup.layout.map_w,
            setup.layout.map_h,
            setup.layout.border_width_px,
        );
    }

    if setup.show_colorbar {
        draw_colorbar_pdf(
            &setup.cr_pdf,
            setup.cb_layout,
            crate::params::ColorbarParams {
                cmap: setup.cmap,
                minv: setup.scale_params.minv,
                maxv: setup.scale_params.maxv,
                scale_type: setup.scale_type,
                gamma: setup.gamma,
                hist_scale: setup.hist_scale_opt.as_ref(),
                latex_rendering: setup.latex_rendering,
                units: setup.units.as_deref(),
                extend: &setup.extend,
                units_font_size: setup.units_font_size,
                map_width: None,
            },
        );
    }

    // Draw figure labels
    crate::render::pdf::draw_figure_labels_pdf(
        &setup.cr_pdf,
        setup.layout.width,
        setup.layout.height,
        &setup.rlabel,
        &setup.llabel,
        setup.latex_rendering,
        setup.label_font_size,
    );

    setup.surface_pdf.finish();
}

pub fn plot_mollweide_pdf(params: MollweideParams) {
    _plot_mollweide_pdf_impl(params, render_mollweide_pixels);
}

pub fn _plot_mollweide_pdf_impl<'a, F>(params: MollweideParams<'a>, pixel_renderer: F)
where
    F: Fn(
        crate::params::RenderMollweideParams,
        MollweideLayout,
        &mut dyn PixelSink,
        Option<DebugOverlay>,
    ),
{
    // Non-generic setup - not duplicated per monomorphization
    let mut setup = _mollweide_pdf_setup_impl(&params);

    // Construct render params
    let debug_overlay = if cfg!(feature = "debug_overlay") {
        Some(DebugOverlay::grid_only())
    } else {
        None
    };

    let render_params = crate::params::RenderMollweideParams {
        map: setup.map,
        scale: &setup.scale_params,
        cmap: setup.cmap,
        gamma: setup.gamma,
        scale_type: setup.scale_type,
        neg_mode: setup.neg_mode,
        bad_color: setup.bad_color,
        meta: setup.meta,
        hist_scale: setup.hist_scale_opt.as_ref(),
        view: setup.view,
        mask: None,
        scale_cache: Some(&setup.scale_cache),
    };

    // Generic pixel rendering - only this call is monomorphized
    let mut sink = PngSink {
        img: &mut setup.pixel_buffer,
        x0: 0,
        y0: 0,
    };

    pixel_renderer(render_params, setup.layout, &mut sink, debug_overlay);

    // Non-generic finalization - not duplicated per monomorphization
    _mollweide_pdf_finalize_impl(setup, debug_overlay);
}

pub fn plot_mollweide_png(params: MollweideParams) {
    _plot_mollweide_png_impl_projected(params, render_mollweide_pixels, ProjectionType::Mollweide);
}

#[derive(Clone, Copy, Debug)]
pub enum ProjectionType {
    Mollweide,
    Hammer,
}

pub fn _plot_mollweide_png_impl_projected<F>(
    params: MollweideParams,
    pixel_renderer: F,
    projection: ProjectionType,
) where
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
    let dpar_deg = params.graticule.dpar_deg;
    let dmer_deg = params.graticule.dmer_deg;
    let mask = params.display.mask.as_ref();

    let (layout, cb_layout) = compute_mollweide_layout(
        width as f64,
        show_colorbar,
        params.display.tick_direction.clone(),
    );

    let font_data = include_bytes!("../../assets/fonts/DejaVuSans.ttf");
    let font = FontRef::try_from_slice(font_data).expect("Failed to load font");

    let mut values: Vec<f64> = map.iter().filter(|&v| is_seen(*v)).copied().collect();

    if values.is_empty() {
        // Diagnostic: check what we actually have
        let n_maps = map.len();
        let n_finite = map.iter().filter(|v| v.is_finite()).count();
        let n_gt_neg1e30 = map.iter().filter(|v| **v > -1e30).count();
        let n_inf = map.iter().filter(|v| v.is_infinite()).count();
        let n_nan = map.iter().filter(|v| v.is_nan()).count();

        eprintln!("\n=== DEBUG: No valid HEALPix values found ===");
        eprintln!("Total pixels in map: {}", n_maps);
        eprintln!("Finite values: {}", n_finite);
        eprintln!("Values > -1e30: {}", n_gt_neg1e30);
        eprintln!("Infinite values: {}", n_inf);
        eprintln!("NaN values: {}", n_nan);
        if !map.is_empty() {
            eprintln!(
                "Min: {:.6e}, Max: {:.6e}",
                map.iter().copied().fold(f64::INFINITY, f64::min),
                map.iter().copied().fold(f64::NEG_INFINITY, f64::max)
            );
            eprintln!("First 5 values: {:?}", &map[..5.min(map.len())]);
        }
        panic!("Map contains no valid HEALPix values");
    }

    values.sort_unstable_by(unsafe_float_cmp);

    let bg = Rgba([
        bg_color[0],
        bg_color[1],
        bg_color[2],
        if transparent { 0 } else { 255 },
    ]);

    let mut img = RgbaImage::from_pixel(layout.width as u32, layout.height as u32, bg);

    let scale_params = compute_mollweide_scale(map, minv, maxv, gamma, scale);

    let hist_scale = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit {
                min: minv,
                max: maxv,
            },
            _ => HistogramRange::Full,
        };

        Some(build_histogram_scale(
            map, range, 1024, // number of bins
        ))
    } else {
        None
    };

    // Pre-compute scale cache for fast pixel rendering
    let scale_cache = crate::scale::ScaleCache::new(scale_params.minv, scale_params.maxv, scale);

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
            hist_scale: hist_scale.as_ref(),
            view,
            mask,
            scale_cache: Some(&scale_cache),
        },
        layout,
        &mut sink,
        debug_overlay,
    );

    if draw_border || show_graticule {
        use cairo::{Context, Format, ImageSurface};

        // Creating a padded surface (shared for both border and graticule)
        let pad = layout.border_width_px.ceil() as i32;
        let surf_w = layout.map_w as i32 + 2 * pad;
        let surf_h = layout.map_h as i32 + 2 * pad;

        let mut border_surf = ImageSurface::create(Format::ARgb32, surf_w, surf_h).unwrap();

        {
            let border_cr = Context::new(&border_surf).unwrap();

            border_cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            border_cr.paint().unwrap();

            // Draw graticule using Cairo (anti-aliased) before border
            if show_graticule {
                use crate::graticule::{
                    render_graticule_cairo, render_graticule_cairo_with_color,
                    render_graticule_hammer_vectorized, render_graticule_mollweide_vectorized,
                };

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
                        crate::params::GeometryRect {
                            x: pad as f64,
                            y: pad as f64,
                            w: layout.map_w,
                            h: layout.map_h,
                        },
                        (r, g, b),
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
            hist_scale.as_ref(),
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

        // Major ticks + labels
        let tick_top = tick_bottom.saturating_sub(major_tick_height);
        for (&t, &val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
            let px = (layout.cbar_pad + (t * layout.cbar_w).round()) as u32;
            for dx in 0..major_tick_width as i32 {
                let x = (px as i32 + dx) as u32;
                if x < layout.width as u32 {
                    for py in tick_top - 1..=tick_bottom {
                        img.put_pixel(x, py, Rgba([0, 0, 0, 255]));
                    }
                }
            }

            // Draw label
            let label =
                format_tick_label_with_units(val, scale, Some(t), latex_rendering, units, false);
            let font_scale = PxScale::from(cb_layout.tick_font_size as f32);

            // Estimate text width: approximately 0.6 * font_size per character
            let text_width = label.len() as f32 * cb_layout.tick_font_size as f32 * 0.6;
            let text_x = px as i32 - (text_width / 2.0) as i32;

            draw_text_mut(
                &mut img,
                Rgba([0, 0, 0, 255]),
                text_x,
                cb_layout.tick_label_pad as i32,
                font_scale,
                &font,
                &label,
            );
        }

        // Minor ticks
        let tick_top = tick_bottom.saturating_sub(minor_tick_height);
        for (&t, &_val) in ticks.minor_positions.iter().zip(ticks.minor_values.iter()) {
            let px = (layout.cbar_pad + (t * layout.cbar_w).round()) as u32;

            for dx in 0..minor_tick_width as i32 {
                let x = (px as i32 + dx) as u32;
                if x < width {
                    for py in tick_top - 1..=tick_bottom {
                        img.put_pixel(x, py, Rgba([0, 0, 0, 255]));
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
    if show_colorbar && let Some(units_str) = units {
        let scale = layout.width / 1200.0;
        let units_y = (cb_layout.tick_label_pad + 30.0 * scale) as i32;

        if latex_rendering {
            // Scale LaTeX font size with width (reduced for appropriate sizing)
            let latex_font_size =
                (cb_layout.tick_font_size * 0.467).round().clamp(3.0, 24.0) as u32;
            // Try to render LaTeX and composite onto image
            if let Some(rendered) =
                crate::latex_render::render_latex_to_png(units_str, latex_font_size)
            {
                // Composite the rendered LaTeX PNG onto the main image
                let latex_img = image::load_from_memory(&rendered.image_data)
                    .expect("Failed to load rendered LaTeX");
                let latex_rgba = latex_img.to_rgba8();

                // Center horizontally
                let x_offset = (layout.cbar_pad + layout.cbar_w / 2.0
                    - latex_rgba.width() as f64 / 2.0) as i32;

                // Composite with alpha blending
                for (lx, ly, pixel) in latex_rgba.enumerate_pixels() {
                    let img_x = x_offset + lx as i32;
                    let img_y = units_y + ly as i32;

                    if img_x >= 0
                        && img_x < layout.width as i32
                        && img_y >= 0
                        && img_y < layout.height as i32
                    {
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
                // Fallback to stripped LaTeX text if rendering fails
                let units_label = units_str
                    .strip_prefix('$')
                    .unwrap_or(units_str)
                    .strip_suffix('$')
                    .unwrap_or(units_str);

                let text_width_est =
                    (units_label.len() as f32 * cb_layout.tick_font_size as f32 * 0.6) as i32;
                let center_x =
                    (layout.cbar_pad + layout.cbar_w / 2.0 - text_width_est as f64 / 2.0) as i32;

                draw_text_mut(
                    &mut img,
                    Rgba([0, 0, 0, 255]),
                    center_x,
                    units_y,
                    PxScale::from(cb_layout.tick_font_size as f32),
                    &font,
                    units_label,
                );
            }
        } else {
            // Non-LaTeX: render as plain text
            if let Some(units_label) = crate::colorbar::format_units_label(false, Some(units_str)) {
                let text_width_est =
                    (units_label.len() as f32 * cb_layout.tick_font_size as f32 * 0.6) as i32;
                let center_x =
                    (layout.cbar_pad + layout.cbar_w / 2.0 - text_width_est as f64 / 2.0) as i32;

                draw_text_mut(
                    &mut img,
                    Rgba([0, 0, 0, 255]),
                    center_x,
                    units_y,
                    PxScale::from(cb_layout.tick_font_size as f32),
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

pub fn plot_mollweide_auto(params: MollweideParams) {
    let ext = Path::new(params.plot.filename.as_str())
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
