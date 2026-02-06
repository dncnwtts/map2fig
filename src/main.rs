use clap::Parser;
use map2fig::cli::Args;
use map2fig::pipeline::load_and_process_data;
use map2fig::{plot_mollweide_auto, plot_gnomonic_auto};
use std::time::Instant;

fn main() {
    let args = Args::parse();
    let config = args.resolve_config().expect("Failed to resolve configuration");
    let view = args.resolve_view_transform().expect("Failed to resolve rotation");

    if args.verbose {println!("Reading HEALPix metadata...");}
    let start = Instant::now();
    
    // For gnomonic projections, use a larger effective width to avoid map degradation
    // since we're sampling at full resolution in a small field of view
    let effective_width = match args.projection.to_lowercase().as_str() {
        "gnomonic" => 32768, // Force no degradation for gnomonic
        _ => args.width,
    };
    
    let data = load_and_process_data(&args.fits, args.col, args.scale, effective_width, args.verbose)
        .expect("Failed to load and process data");
    if args.verbose {println!("Data processing completed in {:.2}s", start.elapsed().as_secs_f64());}

    if args.verbose {println!("Starting plot generation...");}
    let start = Instant::now();
    
    match args.projection.to_lowercase().as_str() {
        "mollweide" => {
            let grat_coord = if args.graticule {
                args.grat_coord.as_ref().map(|s| {
                    map2fig::rotation::CoordSystem::from_str(s)
                        .expect("Invalid graticule coordinate system")
                })
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
                use map2fig::cli::parse_hex_color;
                parse_hex_color(&args.grat_overlay_color, 200)
                    .expect("Invalid overlay color format (use #RRGGBB)")
            } else {
                image::Rgba([255, 255, 0, 0])
            };

            plot_mollweide_auto(
                &data.map,
                args.width,
                &args.out,
                args.min,
                args.max,
                config.colormap,
                !args.no_cbar,
                args.transparent,
                !args.no_border,
                args.gamma,
                config.scale,
                config.neg_mode,
                config.bad_color_rgba,
                config.bg_color_rgba,
                data.meta,
                config.latex_rendering,
                config.units.as_deref(),
                &view,
                args.graticule,
                grat_coord,
                grat_overlay,
                overlay_color,
                args.grat_labels,
                args.grat_par,
                args.grat_mer,
            );
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
                use map2fig::cli::parse_hex_color;
                parse_hex_color(&args.grat_overlay_color, 200)
                    .expect("Invalid overlay color format (use #RRGGBB)")
            } else {
                image::Rgba([255, 255, 0, 0])
            };
            
            plot_gnomonic_auto(
                &data.map,
                args.width,
                &args.out,
                args.min,
                args.max,
                config.colormap,
                !args.no_cbar,
                args.transparent,
                args.gamma,
                config.scale,
                config.neg_mode,
                config.bad_color_rgba,
                config.bg_color_rgba,
                data.meta,
                config.latex_rendering,
                config.units.as_deref(),
                &view,
                lon,
                lat,
                fov,
                res,
                args.local_graticule,
                args.local_grat_dlon,
                args.local_grat_dlat,
                grat_overlay,
                overlay_color,
                args.roll,
            );
        }
        _ => {
            panic!(
                "Unknown projection: {}. Use 'mollweide' or 'gnomonic'",
                args.projection
            );
        }
    }
    
    if args.verbose {println!("Plot generation completed in {:.2}s", start.elapsed().as_secs_f64());}
}
