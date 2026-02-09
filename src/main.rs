use clap::Parser;
use map2fig::cli::Args;
use map2fig::pipeline::load_and_process_data;
use map2fig::{plot_mollweide_auto, plot_gnomonic_auto, plot_hammer_auto};
use map2fig::params::{PlotData, ScaleParams, ColorParams, DisplayParams, GraticuleParams, MollweideParams, GnomonicParams, HammerParams};
use map2fig::mask::{PixelMask, parse_maskfill_color};
use std::time::Instant;
use std::str::FromStr;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse();
    let config = args.resolve_config().map_err(|e| format!("Failed to resolve configuration: {}", e))?;
    let view = args.resolve_view_transform().map_err(|e| format!("Failed to resolve rotation: {}", e))?;

    if args.verbose {println!("Reading HEALPix metadata...");}
    let start = Instant::now();
    
    // For gnomonic projections, use a larger effective width to avoid map degradation
    // since we're sampling at full resolution in a small field of view
    let effective_width = match args.projection.to_lowercase().as_str() {
        "gnomonic" => 32768, // Force no degradation for gnomonic
        _ => args.width,
    };
    
    let data = load_and_process_data(&args.fits, args.col, args.scale, effective_width, args.verbose)
        .map_err(|e| format!("Failed to load and process data: {}", e))?;
    if args.verbose {println!("Data processing completed in {:.2}s", start.elapsed().as_secs_f64());}

    // Create mask if specified
    let mask = if let Some(mask_below) = args.mask_below {
        if args.verbose {println!("Creating value-range mask (below: {})", mask_below);}
        let fill_color = parse_maskfill_color(&args.maskfill_color);
        let coord = args.mask_coord.as_ref()
            .and_then(|s| map2fig::rotation::CoordSystem::from_str(s).ok())
            .unwrap_or(data.meta.coord);
        Some(PixelMask::from_value_range(
            &data.map,
            args.mask_below,
            args.mask_above,
            data.meta.nside,
            fill_color,
            coord,
        ))
    } else if let Some(mask_above) = args.mask_above {
        if args.verbose {println!("Creating value-range mask (above: {})", mask_above);}
        let fill_color = parse_maskfill_color(&args.maskfill_color);
        let coord = args.mask_coord.as_ref()
            .and_then(|s| map2fig::rotation::CoordSystem::from_str(s).ok())
            .unwrap_or(data.meta.coord);
        Some(PixelMask::from_value_range(
            &data.map,
            args.mask_below,
            args.mask_above,
            data.meta.nside,
            fill_color,
            coord,
        ))
    } else if let Some(ref mask_file) = args.mask_file {
        if args.verbose {println!("Loading mask from {}", mask_file);}
        let fill_color = parse_maskfill_color(&args.maskfill_color);
        let coord = args.mask_coord.as_ref()
            .and_then(|s| map2fig::rotation::CoordSystem::from_str(s).ok());
        match PixelMask::from_fits_file(mask_file, fill_color, coord) {
            Ok(mask) => {
                if let Some(warning) = mask.warn_coord_mismatch(data.meta.coord, args.verbose) {
                    eprintln!("{}", warning);
                }
                Some(mask)
            }
            Err(e) => {
                eprintln!("Warning: Failed to load mask: {}", e);
                None
            }
        }
    } else {
        None
    };

    if args.verbose {println!("Starting plot generation...");}
    let start = Instant::now();
    
    match args.projection.to_lowercase().as_str() {
        "mollweide" => {
            let grat_coord = if args.graticule {
                if let Some(ref s) = args.grat_coord {
                    // Explicit --grat-coord provided
                    Some(map2fig::rotation::CoordSystem::from_str(s)
                        .expect("Invalid graticule coordinate system"))
                } else {
                    // Use header coordinate system if available, otherwise default to Galactic
                    match data.meta.coord {
                        map2fig::rotation::CoordSystem::E => Some(map2fig::rotation::CoordSystem::E),
                        map2fig::rotation::CoordSystem::G => Some(map2fig::rotation::CoordSystem::G),
                        map2fig::rotation::CoordSystem::C => Some(map2fig::rotation::CoordSystem::C),
                    }
                }
            } else {
                None
            };

            let grat_overlay = if let Some(ref overlay_str) = args.grat_coord_overlay {
                match map2fig::rotation::CoordSystem::from_str(overlay_str) {
                    Ok(coord) => Some(coord),
                    Err(e) => panic!("Invalid overlay coordinate system: {}", e),
                }
            } else {
                None
            };

            let overlay_color = if args.grat_coord_overlay.is_some() {
                use map2fig::cli::resolve_color_with_alpha;
                resolve_color_with_alpha(&args.grat_overlay_color, 200)
                    .expect("Invalid overlay color format")
            } else {
                image::Rgba([255, 255, 0, 0])
            };

            let params = MollweideParams {
                plot: PlotData {
                    map: &data.map,
                    width: args.width,
                    filename: &args.out,
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
                    show_colorbar: !args.no_cbar,
                    transparent: args.transparent,
                    draw_border: !args.no_border,
                    latex_rendering: config.latex_rendering,
                    units: config.units,
                    extend: args.extend.parse().expect("Invalid extend option"),
                    tick_direction: args.tick_direction.parse().expect("Invalid tick direction option"),
                    tick_font_size: args.tick_font_size,
                    units_font_size: args.units_font_size,
                    rlabel: args.rlabel.clone(),
                    llabel: args.llabel.clone(),
                    label_font_size: args.label_font_size,
                    mask: mask.clone(),
                },
                graticule: GraticuleParams {
                    show_graticule: args.graticule,
                    grat_coord,
                    grat_overlay,
                    overlay_color,
                    show_labels: args.grat_labels,
                    dpar_deg: args.grat_par,
                    dmer_deg: args.grat_mer,
                },
                meta: data.meta,
                view: &view,
            };

            plot_mollweide_auto(params);
        }
        "gnomonic" => {
            let lon = args.lon.unwrap_or(0.0);
            let lat = args.lat.unwrap_or(0.0);
            let fov = args.fov;
            let res = args.res;

            let grat_overlay = if let Some(ref overlay_str) = args.grat_coord_overlay {
                match map2fig::rotation::CoordSystem::from_str(overlay_str) {
                    Ok(coord) => Some(coord),
                    Err(e) => panic!("Invalid overlay coordinate system: {}", e),
                }
            } else {
                None
            };

            let overlay_color = if args.grat_coord_overlay.is_some() {
                use map2fig::cli::resolve_color_with_alpha;
                resolve_color_with_alpha(&args.grat_overlay_color, 200)
                    .expect("Invalid overlay color format")
            } else {
                image::Rgba([255, 255, 0, 0])
            };
            
            let params = GnomonicParams {
                plot: PlotData {
                    map: &data.map,
                    width: args.width,
                    filename: &args.out,
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
                    show_colorbar: !args.no_cbar,
                    transparent: args.transparent,
                    draw_border: false,
                    latex_rendering: config.latex_rendering,
                    units: config.units,
                    extend: args.extend.parse().expect("Invalid extend option"),
                    tick_direction: args.tick_direction.parse().expect("Invalid tick direction option"),
                    tick_font_size: args.tick_font_size,
                    units_font_size: args.units_font_size,
                    rlabel: args.rlabel.clone(),
                    llabel: args.llabel.clone(),
                    label_font_size: args.label_font_size,
                    mask: mask.clone(),
                },
                graticule: GraticuleParams {
                    show_graticule: args.local_graticule,
                    grat_coord: None,
                    grat_overlay,
                    overlay_color,
                    show_labels: false,
                    dpar_deg: args.local_grat_dlon,
                    dmer_deg: args.local_grat_dlat,
                },
                meta: data.meta,
                view: &view,
                lon_deg: lon,
                lat_deg: lat,
                fov_arcmin: fov,
                resolution_arcmin: res,
                roll_deg: args.roll,
                grat_line_width: args.grat_line_width,
                show_gnomonic_text: !args.no_text,
            };

            plot_gnomonic_auto(params);
        }
        "hammer" => {
            let grat_coord = if args.graticule {
                if let Some(ref s) = args.grat_coord {
                    // Explicit --grat-coord provided
                    Some(map2fig::rotation::CoordSystem::from_str(s)
                        .expect("Invalid graticule coordinate system"))
                } else {
                    // Use header coordinate system if available, otherwise default to Galactic
                    match data.meta.coord {
                        map2fig::rotation::CoordSystem::E => Some(map2fig::rotation::CoordSystem::E),
                        map2fig::rotation::CoordSystem::G => Some(map2fig::rotation::CoordSystem::G),
                        map2fig::rotation::CoordSystem::C => Some(map2fig::rotation::CoordSystem::C),
                    }
                }
            } else {
                None
            };

            let grat_overlay = if let Some(ref overlay_str) = args.grat_coord_overlay {
                match map2fig::rotation::CoordSystem::from_str(overlay_str) {
                    Ok(coord) => Some(coord),
                    Err(e) => panic!("Invalid overlay coordinate system: {}", e),
                }
            } else {
                None
            };

            let overlay_color = if args.grat_coord_overlay.is_some() {
                use map2fig::cli::resolve_color_with_alpha;
                resolve_color_with_alpha(&args.grat_overlay_color, 200)
                    .expect("Invalid overlay color format")
            } else {
                image::Rgba([255, 255, 0, 0])
            };

            let params = HammerParams {
                plot: PlotData {
                    map: &data.map,
                    width: args.width,
                    filename: &args.out,
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
                    show_colorbar: !args.no_cbar,
                    transparent: args.transparent,
                    draw_border: !args.no_border,
                    latex_rendering: config.latex_rendering,
                    units: config.units,
                    extend: args.extend.parse().expect("Invalid extend option"),
                    tick_direction: args.tick_direction.parse().expect("Invalid tick direction option"),
                    tick_font_size: args.tick_font_size,
                    units_font_size: args.units_font_size,
                    rlabel: args.rlabel.clone(),
                    llabel: args.llabel.clone(),
                    label_font_size: args.label_font_size,
                    mask: mask.clone(),
                },
                graticule: GraticuleParams {
                    show_graticule: args.graticule,
                    grat_coord,
                    grat_overlay,
                    overlay_color,
                    show_labels: args.grat_labels,
                    dpar_deg: args.grat_par,
                    dmer_deg: args.grat_mer,
                },
                meta: data.meta,
                view: &view,
            };

            plot_hammer_auto(params);
        }
        proj => {
            return Err(format!(
                "Unknown projection: '{}'. Available projections: 'mollweide', 'gnomonic', 'hammer'",
                proj
            ));
        }
    }
    
    if args.verbose {println!("Plot generation completed in {:.2}s", start.elapsed().as_secs_f64());}
    Ok(())
}

