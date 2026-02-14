use super::{MollweideScale, render_projection_to_grid};
use crate::PngSink;
use crate::colorbar::{format_tick_label_with_units, render_colorbar_gradient};
use crate::healpix::is_seen;
use crate::layout::compute_gnomonic_layout;
use crate::params::{GnomonicParams, ScaleParams};
use crate::render::raster::RasterGrid;
use crate::rotation::CoordSystem;
use crate::scale::{
    HistogramRange, Scale, build_histogram_scale, generate_colorbar_ticks, unsafe_float_cmp,
};
use cairo::{Context, Format, ImageSurface, PdfSurface};
use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use rusttype::{Font, Scale as FontScale};
use std::path::Path;

/// Compute scale limits for gnomonic projections using only FOV-contained pixels.
/// This ensures the color scale is based on visible data, not the entire map.
pub fn compute_gnomonic_scale(
    map: &[f64],
    scale_params: &ScaleParams,
    projection: &dyn crate::projection::Projection,
    width: u32,
    meta: crate::healpix::HealpixMeta,
    view: &crate::rotation::ViewTransform,
) -> MollweideScale {
    // Collect values only from pixels within the field of view
    let mut values: Vec<f64> = Vec::new();

    for y in 0..width {
        for x in 0..width {
            // Use normalized coordinates [0, 1]
            let u = x as f64 / width as f64;
            let v = y as f64 / width as f64;

            // Check if this pixel projects to a valid sky coordinate
            if let Some((lon, lat)) = projection.inverse(u, v) {
                // Convert from (lon, lat) in radians to (theta, phi) for sample_healpix
                // theta = pi/2 - latitude, phi = longitude
                let theta = std::f64::consts::PI / 2.0 - lat;

                // sample_healpix takes (map, meta, view, theta, lon)
                if let Some(value) = crate::healpix::sample_healpix(map, meta, view, theta, lon)
                    && is_seen(value)
                {
                    values.push(value);
                }
            }
        }
    }

    if values.is_empty() {
        // Fallback to all valid data if no FOV data found
        values = map.iter().filter(|v| is_seen(**v)).copied().collect();

        if values.is_empty() {
            panic!("Map contains no valid HEALPix values");
        }
    }

    values.sort_unstable_by(unsafe_float_cmp);

    let data_min = *values.first().unwrap();
    let data_max = *values.last().unwrap();

    let (minv, maxv) = match scale_params.scale {
        // For histogram scale, use full data range if not explicitly specified
        Scale::Histogram => match (scale_params.minv, scale_params.maxv) {
            (Some(lo), Some(hi)) => (lo, hi),
            _ => (data_min, data_max),
        },

        // For other scales, use percentiles of FOV-contained data
        _ => match (scale_params.minv, scale_params.maxv) {
            (Some(lo), Some(hi)) => (lo, hi),
            _ => (
                super::percentile(&values, 5.0),
                super::percentile(&values, 95.0),
            ),
        },
    };

    if scale_params.gamma <= 0.0 {
        panic!("Gamma must be > 0");
    }

    if minv > maxv {
        panic!("Invalid color scale: {minv} > {maxv}");
    }

    MollweideScale { minv, maxv }
}

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
    let resolution_arcmin = params.resolution_arcmin;
    let show_graticule = params.graticule.show_graticule;
    let grat_dlon = params.graticule.dpar_deg;
    let grat_dlat = params.graticule.dmer_deg;
    let grat_overlay = params.graticule.grat_overlay;
    let roll_deg = params.roll_deg;
    let mask = params.display.mask.as_ref();
    let show_text = params.show_gnomonic_text;
    let overlay_color = params.graticule.overlay_color;

    use crate::gnomonic::GnomonicProjection;

    // Gnomonic tick labels: same size as resolution label (11pt), scales by width/300
    let gnomonic_tick_font_size = 11.0 * (width as f32 / 300.0);
    let (layout, cb_layout) = crate::layout::compute_gnomonic_layout_with_fonts(
        width as f64,
        show_colorbar,
        params.display.tick_direction.clone(),
        show_text,
        params.display.show_title,
        Some(gnomonic_tick_font_size),
    );

    let font_data = include_bytes!("../../assets/fonts/DejaVuSans.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Failed to load font");

    let roll_rad = roll_deg * std::f64::consts::PI / 180.0;
    let proj = GnomonicProjection::with_roll(lon_deg, lat_deg, resolution_arcmin, roll_rad);

    let mut grid = RasterGrid::new(width, width);

    // For gnomonic projections, compute scale limits using only FOV-contained pixels
    let scale_params = compute_gnomonic_scale(map, &params.scale, &proj, width, meta, view);

    let hist_scale = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit {
                min: minv,
                max: maxv,
            },
            _ => HistogramRange::Full,
        };
        Some(build_histogram_scale(map, range, 1024))
    } else {
        None
    };

    // Pre-compute scale cache for fast pixel rendering
    let scale_cache = crate::scale::ScaleCache::new(scale_params.minv, scale_params.maxv, scale);

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
            mask,
            scale_cache: Some(&scale_cache),
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
                crate::params::GraticuleSpacing {
                    dlon_deg: grat_dlon,
                    dlat_deg: grat_dlat,
                },
                crate::params::GraticuleCoordinates {
                    grat_coord: overlay_sys,
                    input_coord: view.input_coord,
                },
                overlay_color,
            );
        }
    }

    let bg = Rgba([
        bg_color[0],
        bg_color[1],
        bg_color[2],
        if transparent { 0 } else { 255 },
    ]);

    let mut img = RgbaImage::from_pixel(layout.width as u32, layout.height as u32, bg);

    // Add title if enabled
    if params.display.show_title {
        let title_text;
        let title_owned;
        if let Some(custom_title) = params.display.title.as_ref() {
            title_text = custom_title.as_str();
        } else {
            // Format default title: (lon, lat) rounded to hundredths place
            let lon_rounded = (lon_deg * 100.0).round() / 100.0;
            let lat_rounded = (lat_deg * 100.0).round() / 100.0;
            title_owned = format!("({:.2}, {:.2})", lon_rounded, lat_rounded);
            title_text = &title_owned;
        }

        let title_font_size = if params.display.scale_text {
            13.0 * (width as f32 / 300.0)
        } else {
            13.0 // constant size when text scaling is disabled
        };

        // Title positioned at top of image with minimal gap
        let title_y = 0_i32;
        let title_text_width_est = (title_text.len() as f32 * title_font_size * 0.35) as i32;
        let image_center = layout.width / 2.0;
        let title_x = (image_center as i32) - (title_text_width_est / 2);

        draw_text_mut(
            &mut img,
            Rgba([0, 0, 0, 255]),
            title_x,
            title_y,
            FontScale::uniform(title_font_size),
            &font,
            title_text,
        );
    }

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

        // For outward ticks, ticks extend downward; for inward, upward
        let (tick_top, tick_bottom_actual) = match &params.display.tick_direction {
            crate::cli::TickDirection::Inward => {
                (tick_bottom.saturating_sub(major_tick_height), tick_bottom)
            }
            crate::cli::TickDirection::Outward => (tick_bottom, tick_bottom + major_tick_height),
        };

        // Draw major ticks and labels
        for (&t, &val) in ticks.major_positions.iter().zip(ticks.major_values.iter()) {
            let px = (layout.cbar_x + (t * layout.cbar_w).round()) as u32;
            for dx in 0..major_tick_width as i32 {
                let x = (px as i32 + dx) as u32;
                if x < layout.width as u32 {
                    for py in tick_top..=tick_bottom_actual {
                        if py < img.height() {
                            img.put_pixel(x, py, Rgba([0, 0, 0, 255]));
                        }
                    }
                }
            }

            // Draw label - position depends on tick direction
            let label =
                format_tick_label_with_units(val, scale, Some(t), latex_rendering, units, false);
            let font_scale = FontScale::uniform(cb_layout.tick_font_size as f32);

            // Measure actual text width using font metrics
            let mut text_width = 0.0;
            for ch in label.chars() {
                let glyph = font.glyph(ch);
                text_width += glyph.scaled(font_scale).h_metrics().advance_width;
            }
            let text_x = px as i32 - (text_width / 2.0) as i32;

            // For outward ticks, shift labels down past the ticks
            let label_y_offset = match &params.display.tick_direction {
                crate::cli::TickDirection::Outward => (major_tick_height * 2) as i32,
                crate::cli::TickDirection::Inward => 0,
            };

            draw_text_mut(
                &mut img,
                Rgba([0, 0, 0, 255]),
                text_x,
                cb_layout.tick_label_pad as i32 + label_y_offset,
                font_scale,
                &font,
                &label,
            );
        }

        // Draw minor ticks
        let minor_tick_height = cb_layout.minor_tick_height as u32;
        let minor_tick_width = cb_layout.minor_tick_width as u32;

        let (minor_tick_top, minor_tick_bottom) = match &params.display.tick_direction {
            crate::cli::TickDirection::Inward => {
                (tick_bottom.saturating_sub(minor_tick_height), tick_bottom)
            }
            crate::cli::TickDirection::Outward => (tick_bottom, tick_bottom + minor_tick_height),
        };

        for (&t, &_val) in ticks.minor_positions.iter().zip(ticks.minor_values.iter()) {
            let px = (layout.cbar_x + (t * layout.cbar_w).round()) as u32;
            for dx in 0..minor_tick_width as i32 {
                let x = (px as i32 + dx) as u32;
                if x < layout.width as u32 {
                    for py in minor_tick_top..=minor_tick_bottom {
                        if py < img.height() {
                            img.put_pixel(x, py, Rgba([0, 0, 0, 255]));
                        }
                    }
                }
            }
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
            // Position units below tick labels with extra spacing
            let spacing_scale = width as f64 / 300.0;
            let outward_offset = match &params.display.tick_direction {
                crate::cli::TickDirection::Outward => (cb_layout.major_tick_height * 2.0) as i32,
                crate::cli::TickDirection::Inward => 0,
            };
            let units_y = (cb_layout.tick_label_pad + 12.0 * spacing_scale) as i32 + outward_offset;

            if latex_rendering {
                let units_font_pt = params
                    .display
                    .units_font_size
                    .unwrap_or_else(|| 28.0 / 1.5 * (width as f32 / 300.0));
                let latex_font_size = units_font_pt.round().clamp(3.0, 24.0) as u32;
                let latex_dpi = 300u32;
                if let Some(rendered) = crate::latex_render::render_latex_to_hires_png(
                    units_str,
                    latex_font_size,
                    latex_dpi,
                ) {
                    let latex_img = image::load_from_memory(&rendered.image_data)
                        .expect("Failed to load rendered LaTeX");
                    let latex_rgba = latex_img.to_rgba8();

                    let x_offset = (layout.cbar_x + layout.cbar_w / 2.0
                        - latex_rgba.width() as f64 / 2.0)
                        as i32;

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
                    let units_label = units_str
                        .strip_prefix('$')
                        .unwrap_or(units_str)
                        .strip_suffix('$')
                        .unwrap_or(units_str);

                    let units_font_size = params
                        .display
                        .units_font_size
                        .unwrap_or_else(|| 28.0 / 1.5 * (width as f32 / 300.0));
                    let text_width_est = (units_label.len() as f32 * units_font_size * 0.6) as i32;
                    let center_x =
                        (layout.cbar_x + layout.cbar_w / 2.0 - text_width_est as f64 / 2.0) as i32;

                    draw_text_mut(
                        &mut img,
                        Rgba([0, 0, 0, 255]),
                        center_x,
                        units_y,
                        FontScale::uniform(units_font_size),
                        &font,
                        units_label,
                    );
                }
            } else if let Some(units_label) =
                crate::colorbar::format_units_label(false, Some(units_str))
            {
                let units_font_size = params
                    .display
                    .units_font_size
                    .unwrap_or_else(|| 28.0 / 1.5 * (width as f32 / 300.0));
                let text_width_est = (units_label.len() as f32 * units_font_size * 0.6) as i32;
                let center_x =
                    (layout.cbar_x + layout.cbar_w / 2.0 - text_width_est as f64 / 2.0) as i32;

                draw_text_mut(
                    &mut img,
                    Rgba([0, 0, 0, 255]),
                    center_x,
                    units_y,
                    FontScale::uniform(units_font_size),
                    &font,
                    &units_label,
                );
            }
        }
    }

    // Add gnomonic text label if requested
    if show_text {
        let text = format!("{:.2} '/pix,   {}x{} pix", resolution_arcmin, width, width);

        let font_size = 11.0 * (layout.map_w / 300.0);
        let surface_width = (200.0 * (layout.map_w / 300.0)).round() as i32;
        let surface_height = (50.0 * (layout.map_w / 300.0)).round() as i32;

        eprintln!(
            "TEXT_RENDER: font_size={:.1}, surface={}x{}, map_w={:.1}",
            font_size, surface_width, surface_height, layout.map_w
        );

        let surface_width_padded = ((200.0 + 10.0) * (layout.map_w / 300.0)).round() as i32;
        let mut text_surface = cairo::ImageSurface::create(
            cairo::Format::ARgb32,
            surface_width_padded,
            surface_height,
        )
        .expect("Failed to create text surface");
        {
            let text_cr =
                cairo::Context::new(&text_surface).expect("Failed to create text context");

            // Set transparent background
            text_cr.set_source_rgba(1.0, 1.0, 1.0, 0.0);
            text_cr.paint().expect("Failed to paint background");

            // Draw text in black
            text_cr.set_source_rgb(0.0, 0.0, 0.0);
            text_cr.set_font_size(font_size);
            let text_x = 2.0 * (layout.map_w / 300.0);
            let text_y = font_size * 0.9 + (50.0 / 10.0) * (layout.map_w / 300.0);
            text_cr.move_to(text_x, text_y);
            text_cr.show_text(&text).ok();
        }

        text_surface.flush();
        let stride = text_surface.stride() as usize;
        let data = text_surface.data().expect("Failed to get surface data");

        let start_y = (layout.map_y + layout.map_h) as i32;

        let mut src_y_min = usize::MAX;
        let mut src_y_max = usize::MIN;
        let mut label_min_x = i32::MAX;
        let mut label_max_x = i32::MIN;

        // Composite text onto main image with 90-degree counterclockwise rotation
        for src_y in 0..surface_height {
            for src_x in 0..surface_width_padded {
                let src_idx = (src_y as usize) * stride + (src_x as usize) * 4;
                if src_idx + 3 < data.len() {
                    let alpha = data[src_idx + 3] as f32 / 255.0;
                    if alpha > 0.1 {
                        src_y_min = src_y_min.min(src_y as usize);
                        src_y_max = src_y_max.max(src_y as usize);

                        let rotated_x = src_y;
                        let rotated_y = start_y - src_x;

                        if rotated_x >= 0
                            && rotated_y >= 0
                            && rotated_x < img.width() as i32
                            && rotated_y < img.height() as i32
                        {
                            label_min_x = label_min_x.min(rotated_x);
                            label_max_x = label_max_x.max(rotated_x);

                            let dst_pixel = img.get_pixel_mut(rotated_x as u32, rotated_y as u32);
                            // Blend with alpha
                            dst_pixel[0] = (data[src_idx + 2] as f32 * alpha
                                + dst_pixel[0] as f32 * (1.0 - alpha))
                                as u8;
                            dst_pixel[1] = (data[src_idx + 1] as f32 * alpha
                                + dst_pixel[1] as f32 * (1.0 - alpha))
                                as u8;
                            dst_pixel[2] = (data[src_idx] as f32 * alpha
                                + dst_pixel[2] as f32 * (1.0 - alpha))
                                as u8;
                        }
                    }
                }
            }
        }

        eprintln!(
            "PNG_SPACE: image_w={}, label_x_range=[{}, {}], src_y_range=[{}, {}], map_x={:.1}, map_w={:.1}, surface_h={}, rotated_text_width={}",
            img.width(),
            label_min_x,
            label_max_x,
            src_y_min,
            src_y_max,
            layout.map_x,
            layout.map_w,
            surface_height,
            label_max_x - label_min_x
        );
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
    let gamma = params.scale.gamma;
    let scale = params.scale.scale;
    let neg_mode = params.scale.neg_mode;
    let bad_color = params.color.bad_color;
    let meta = params.meta;
    let latex_rendering = params.display.latex_rendering;
    let units = params.display.units.as_deref();
    let view = params.view;
    let lon_deg = params.lon_deg;
    let lat_deg = params.lat_deg;
    let resolution_arcmin = params.resolution_arcmin;
    let show_graticule = params.graticule.show_graticule;
    let grat_dlon = params.graticule.dpar_deg;
    let grat_dlat = params.graticule.dmer_deg;
    let grat_overlay = params.graticule.grat_overlay;
    let roll_deg = params.roll_deg;
    let overlay_color = params.graticule.overlay_color;
    let mask = params.display.mask.as_ref();

    use crate::gnomonic::GnomonicProjection;

    let show_text = params.show_gnomonic_text;

    let (layout, _cb_layout) = compute_gnomonic_layout(
        width as f64,
        show_colorbar,
        params.display.tick_direction.clone(),
        show_text,
        params.display.show_title,
    );

    let roll_rad = roll_deg * std::f64::consts::PI / 180.0;
    let proj = GnomonicProjection::with_roll(lon_deg, lat_deg, resolution_arcmin, roll_rad);
    let img_height = width; // Square image for gnomonic

    // Render to grid first (just the map region)
    let mut grid = RasterGrid::new(width, img_height);

    // For gnomonic projections, compute scale limits using only FOV-contained pixels
    let scale_params = compute_gnomonic_scale(map, &params.scale, &proj, width, meta, view);

    let hist_scale = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit {
                min: minv,
                max: maxv,
            },
            _ => HistogramRange::Full,
        };
        Some(build_histogram_scale(map, range, 1024))
    } else {
        None
    };

    // Pre-compute scale cache for fast pixel rendering
    let scale_cache = crate::scale::ScaleCache::new(scale_params.minv, scale_params.maxv, scale);

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
            mask,
            scale_cache: Some(&scale_cache),
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
                crate::params::GraticuleSpacing {
                    dlon_deg: grat_dlon,
                    dlat_deg: grat_dlat,
                },
                crate::params::GraticuleCoordinates {
                    grat_coord: overlay_sys,
                    input_coord: CoordSystem::G, // Map input is Galactic by default
                },
                overlay_color,
            );
        }
    }

    // Create image surface for the full layout (including colorbar)
    let mut img_surface =
        ImageSurface::create(Format::ARgb32, layout.width as i32, layout.height as i32)
            .expect("Failed to create image surface");

    // Fill with white background
    {
        let mut data = img_surface.data().expect("Failed to get surface data");

        for idx in (0..data.len()).step_by(4) {
            data[idx] = 255; // B
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
                            data[idx] = pixel[2]; // B
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

    // Paint the image surface onto the PDF at map position
    cr.set_source_surface(&img_surface, 0.0, 0.0)
        .expect("Failed to set source");
    cr.paint().expect("Failed to paint");

    // Add title if enabled
    if params.display.show_title {
        let title_text;
        let title_owned;
        if let Some(custom_title) = params.display.title.as_ref() {
            title_text = custom_title.as_str();
        } else {
            // Format default title: (lon, lat) rounded to hundredths place
            let lon_rounded = (lon_deg * 100.0).round() / 100.0;
            let lat_rounded = (lat_deg * 100.0).round() / 100.0;
            title_owned = format!("({:.2}, {:.2})", lon_rounded, lat_rounded);
            title_text = &title_owned;
        }

        let title_font_size = if params.display.scale_text {
            13.0 * (width as f64 / 300.0)
        } else {
            13.0 // constant size when text scaling is disabled
        };

        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_font_size(title_font_size);
        cr.select_font_face(
            "sans-serif",
            cairo::FontSlant::Normal,
            cairo::FontWeight::Normal,
        );

        // Title positioned at top of reserved title_pad area
        let extents = cr
            .text_extents(title_text)
            .expect("Failed to get text extents");
        let text_width = extents.width();
        let text_height = extents.height();
        // Center horizontally
        let title_x = (layout.width - text_width) / 2.0;
        // Position baseline at very top
        let title_y = text_height;

        cr.move_to(title_x, title_y);
        cr.show_text(title_text).expect("Failed to show title");
    }

    // Add proper colorbar with ticks and labels if requested
    if show_colorbar {
        use crate::render::pdf::draw_colorbar_pdf;
        // Gnomonic tick labels: 1.5x resolution label (11pt), scales by width/300
        let gnomonic_tick_font_size = 16.5 * (width as f32 / 300.0);
        let cb_layout = crate::layout::compute_gnomonic_layout_with_fonts(
            width as f64,
            show_colorbar,
            params.display.tick_direction.clone(),
            show_text,
            params.display.show_title,
            Some(gnomonic_tick_font_size),
        )
        .1;

        let effective_units_font_size = params
            .display
            .units_font_size
            .or_else(|| Some(28.0 / 1.5 * (width as f32 / 300.0)));

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
                latex_rendering,
                units,
                extend: &params.display.extend,
                units_font_size: effective_units_font_size,
                map_width: Some(width as f64), // Pass map width for DPI scaling
            },
        );
    }

    // Add gnomonic text label if requested
    if show_text {
        let text = format!("{:.2} '/pix,   {}x{} pix", resolution_arcmin, width, width);

        // Position text in left margin with same scaling as PNG version
        let font_size = 11.0 * (width as f64 / 300.0);

        let h_offset = (50.0 / 10.0) * (width as f64 / 300.0);
        let start_x = layout.map_x - h_offset;
        let start_y = layout.map_y + layout.map_h;

        // Draw text with 90-degree clockwise rotation
        let _ = cr.save();
        cr.translate(start_x, start_y);
        cr.rotate(-std::f64::consts::PI / 2.0);
        cr.set_source_rgb(0.0, 0.0, 0.0);
        cr.set_font_size(font_size);
        cr.move_to(0.0, 0.0);
        cr.show_text(&text).ok();
        let _ = cr.restore();
    }

    pdf_surface.finish();
}

/// Plot gnomonic projection with automatic format detection
pub fn plot_gnomonic_auto(params: GnomonicParams) {
    let map = params.plot.map;
    let filename = params.plot.filename;
    let pdf_backend = params.plot.pdf_backend.clone();
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
    let show_text = params.show_gnomonic_text;

    // Compute image width from field of view and resolution
    let width = (fov_arcmin / resolution_arcmin).ceil() as u32;

    let ext = Path::new(filename.as_str())
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
            pdf_backend,
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
            mask: None,
            title: params.display.title.clone(),
            show_title: params.display.show_title,
            scale_text: params.display.scale_text,
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
        show_gnomonic_text: show_text,
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
