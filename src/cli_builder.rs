//! CLI argument builder utilities for constructing plot parameters.
//!
//! This module contains helper functions for building projection-specific parameter
//! structs from command-line arguments and configuration. It extracts common logic
//! from main.rs to reduce duplication and improve maintainability.

use crate::cli::{Args, resolve_color_with_alpha};
use crate::mask::PixelMask;
use crate::params::*;
use crate::pipeline::ProcessedData;
use crate::rotation::{CoordSystem, ViewTransform};
use image::Rgba;
use std::str::FromStr;

/// Create a pixel mask from command-line arguments.
///
/// Supports three mask types:
/// - Value range masks (--mask-below / --mask-above)
/// - FITS file masks (--mask-file)
/// - No mask (returns None)
///
/// # Arguments
///
/// * `args` - Parsed command-line arguments
/// * `data` - Processed HEALPix data (used for nside and metadata)
///
/// # Returns
///
/// `Option<PixelMask>` or error message if mask creation fails
pub fn create_pixel_mask(
    args: &Args,
    data: &ProcessedData,
    verbose: bool,
) -> Result<Option<PixelMask>, String> {
    if let Some(mask_below) = args.mask_below {
        if verbose {
            println!("Creating value-range mask (below: {})", mask_below);
        }
        let fill_color = crate::mask::parse_maskfill_color(&args.maskfill_color);
        let coord = args
            .mask_coord
            .as_ref()
            .and_then(|s| CoordSystem::from_str(s).ok())
            .unwrap_or(data.meta.coord);
        return Ok(Some(PixelMask::from_value_range(
            &data.map,
            args.mask_below,
            args.mask_above,
            data.meta.nside,
            fill_color,
            coord,
        )));
    }

    if let Some(mask_above) = args.mask_above {
        if verbose {
            println!("Creating value-range mask (above: {})", mask_above);
        }
        let fill_color = crate::mask::parse_maskfill_color(&args.maskfill_color);
        let coord = args
            .mask_coord
            .as_ref()
            .and_then(|s| CoordSystem::from_str(s).ok())
            .unwrap_or(data.meta.coord);
        return Ok(Some(PixelMask::from_value_range(
            &data.map,
            args.mask_below,
            args.mask_above,
            data.meta.nside,
            fill_color,
            coord,
        )));
    }

    if let Some(ref mask_file) = args.mask_file {
        if verbose {
            println!("Loading mask from {}", mask_file);
        }
        let fill_color = crate::mask::parse_maskfill_color(&args.maskfill_color);
        let coord = args
            .mask_coord
            .as_ref()
            .and_then(|s| CoordSystem::from_str(s).ok());
        match PixelMask::from_fits_file(mask_file, fill_color, coord) {
            Ok(mask) => {
                if let Some(warning) = mask.warn_coord_mismatch(data.meta.coord, verbose) {
                    eprintln!("{}", warning);
                }
                return Ok(Some(mask));
            }
            Err(e) => {
                eprintln!("Warning: Failed to load mask: {}", e);
                return Ok(None);
            }
        }
    }

    Ok(None)
}

/// Resolve overlay color from command-line arguments.
///
/// Returns the specified overlay color if --grat-coord-overlay is provided,
/// otherwise returns a transparent yellow (0, 0, 0, 0).
pub fn resolve_overlay_color(args: &Args) -> Result<Rgba<u8>, String> {
    if args.grat_coord_overlay.is_some() {
        resolve_color_with_alpha(&args.grat_overlay_color, 200)
            .map_err(|e| format!("Invalid overlay color format: {}", e))
    } else {
        Ok(Rgba([255, 255, 0, 0]))
    }
}

/// Resolve graticule coordinate system for full-sky projections.
///
/// Returns:
/// - Explicit --grat-coord if provided
/// - Data's coordinate system if graticule is enabled
/// - None if graticule is disabled
pub fn resolve_graticule_coord(args: &Args, data_coord: CoordSystem) -> Option<CoordSystem> {
    if !args.graticule {
        return None;
    }

    if let Some(ref s) = args.grat_coord {
        CoordSystem::from_str(s).ok()
    } else {
        Some(match data_coord {
            CoordSystem::E => CoordSystem::E,
            CoordSystem::G => CoordSystem::G,
            CoordSystem::C => CoordSystem::C,
        })
    }
}

/// Parse overlay graticule coordinate system.
///
/// # Panics
///
/// Panics if the coordinate system string is invalid.
pub fn parse_overlay_coord(overlay_str: &str) -> CoordSystem {
    CoordSystem::from_str(overlay_str)
        .unwrap_or_else(|e| panic!("Invalid overlay coordinate system: {}", e))
}

/// Build MollweideParams from arguments and processed data.
pub fn build_mollweide_params<'a>(
    args: &'a Args,
    data: &'a ProcessedData,
    config: &'a crate::cli::PlotConfig,
    view: &'a ViewTransform,
    mask: Option<PixelMask>,
) -> Result<MollweideParams<'a>, String> {
    let grat_coord = resolve_graticule_coord(args, data.meta.coord);
    let grat_overlay = args
        .grat_coord_overlay
        .as_ref()
        .map(|s| parse_overlay_coord(s));
    let overlay_color = resolve_overlay_color(args)?;

    Ok(MollweideParams {
        plot: PlotData {
            map: &data.map,
            width: args.width,
            filename: args.get_output_filename(),
        },
        scale: ScaleParams {
            minv: args.min,
            maxv: args.max,
            gamma: args.gamma,
            scale: config.scale,
            neg_mode: config.neg_mode,
        },
        color: ColorParams {
            cmap: config.colormap,
            bad_color: config.bad_color_rgba,
            bg_color: config.bg_color_rgba,
        },
        display: DisplayParams {
            show_colorbar: !args.no_cbar && !args.fast_render,
            transparent: args.transparent,
            draw_border: !args.no_border,
            latex_rendering: config.latex_rendering,
            units: config.units.clone(),
            extend: args
                .extend
                .parse()
                .map_err(|_| "Invalid extend option".to_string())?,
            tick_direction: args
                .tick_direction
                .parse()
                .map_err(|_| "Invalid tick direction option".to_string())?,
            tick_font_size: args.tick_font_size,
            units_font_size: args.units_font_size,
            rlabel: args.rlabel.clone(),
            llabel: args.llabel.clone(),
            label_font_size: args.label_font_size,
            mask: mask.clone(),
            title: args.title.clone(),
            show_title: !args.no_title && !args.fast_render,
            scale_text: !args.no_scale_text && !args.no_text && !args.fast_render,
        },
        graticule: GraticuleParams {
            show_graticule: args.graticule && !args.fast_render,
            grat_coord,
            grat_overlay,
            overlay_color,
            show_labels: args.grat_labels,
            dpar_deg: args.grat_par,
            dmer_deg: args.grat_mer,
        },
        meta: data.meta,
        view,
    })
}

/// Build GnomonicParams from arguments and processed data.
pub fn build_gnomonic_params<'a>(
    args: &'a Args,
    data: &'a ProcessedData,
    config: &'a crate::cli::PlotConfig,
    view: &'a ViewTransform,
    mask: Option<PixelMask>,
) -> Result<GnomonicParams<'a>, String> {
    let lon = args.lon.unwrap_or(0.0);
    let lat = args.lat.unwrap_or(0.0);
    let fov = args.fov;
    let res = args.res;

    let grat_overlay = args
        .grat_coord_overlay
        .as_ref()
        .map(|s| parse_overlay_coord(s));
    let overlay_color = resolve_overlay_color(args)?;

    Ok(GnomonicParams {
        plot: PlotData {
            map: &data.map,
            width: args.width,
            filename: args.get_output_filename(),
        },
        scale: ScaleParams {
            minv: args.min,
            maxv: args.max,
            gamma: args.gamma,
            scale: config.scale,
            neg_mode: config.neg_mode,
        },
        color: ColorParams {
            cmap: config.colormap,
            bad_color: config.bad_color_rgba,
            bg_color: config.bg_color_rgba,
        },
        display: DisplayParams {
            show_colorbar: !args.no_cbar && !args.fast_render,
            transparent: args.transparent,
            draw_border: false,
            latex_rendering: config.latex_rendering,
            units: config.units.clone(),
            extend: args
                .extend
                .parse()
                .map_err(|_| "Invalid extend option".to_string())?,
            tick_direction: args
                .tick_direction
                .parse()
                .map_err(|_| "Invalid tick direction option".to_string())?,
            tick_font_size: args.tick_font_size,
            units_font_size: args.units_font_size,
            rlabel: args.rlabel.clone(),
            llabel: args.llabel.clone(),
            label_font_size: args.label_font_size,
            mask: mask.clone(),
            title: args.title.clone(),
            show_title: !args.no_title && !args.no_text && !args.fast_render,
            scale_text: !args.no_scale_text && !args.no_text && !args.fast_render,
        },
        graticule: GraticuleParams {
            show_graticule: args.local_graticule && !args.fast_render,
            grat_coord: None,
            grat_overlay,
            overlay_color,
            show_labels: false,
            dpar_deg: args.local_grat_dlon,
            dmer_deg: args.local_grat_dlat,
        },
        meta: data.meta,
        view,
        lon_deg: lon,
        lat_deg: lat,
        fov_arcmin: fov,
        resolution_arcmin: res,
        roll_deg: args.roll,
        grat_line_width: args.grat_line_width,
        show_gnomonic_text: !args.no_text,
    })
}

/// Build HammerParams from arguments and processed data.
pub fn build_hammer_params<'a>(
    args: &'a Args,
    data: &'a ProcessedData,
    config: &'a crate::cli::PlotConfig,
    view: &'a ViewTransform,
    mask: Option<PixelMask>,
) -> Result<HammerParams<'a>, String> {
    let grat_coord = resolve_graticule_coord(args, data.meta.coord);
    let grat_overlay = args
        .grat_coord_overlay
        .as_ref()
        .map(|s| parse_overlay_coord(s));
    let overlay_color = resolve_overlay_color(args)?;

    Ok(HammerParams {
        plot: PlotData {
            map: &data.map,
            width: args.width,
            filename: args.get_output_filename(),
        },
        scale: ScaleParams {
            minv: args.min,
            maxv: args.max,
            gamma: args.gamma,
            scale: config.scale,
            neg_mode: config.neg_mode,
        },
        color: ColorParams {
            cmap: config.colormap,
            bad_color: config.bad_color_rgba,
            bg_color: config.bg_color_rgba,
        },
        display: DisplayParams {
            show_colorbar: !args.no_cbar && !args.fast_render,
            transparent: args.transparent,
            draw_border: !args.no_border,
            latex_rendering: config.latex_rendering,
            units: config.units.clone(),
            extend: args
                .extend
                .parse()
                .map_err(|_| "Invalid extend option".to_string())?,
            tick_direction: args
                .tick_direction
                .parse()
                .map_err(|_| "Invalid tick direction option".to_string())?,
            tick_font_size: args.tick_font_size,
            units_font_size: args.units_font_size,
            rlabel: args.rlabel.clone(),
            llabel: args.llabel.clone(),
            label_font_size: args.label_font_size,
            mask: mask.clone(),
            title: args.title.clone(),
            show_title: !args.no_title && !args.no_text && !args.fast_render,
            scale_text: !args.no_scale_text && !args.no_text && !args.fast_render,
        },
        graticule: GraticuleParams {
            show_graticule: args.graticule && !args.fast_render,
            grat_coord,
            grat_overlay,
            overlay_color,
            show_labels: args.grat_labels,
            dpar_deg: args.grat_par,
            dmer_deg: args.grat_mer,
        },
        meta: data.meta,
        view,
    })
}
