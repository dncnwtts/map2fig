use clap::Parser;
use std::time::Instant;
use healpix_plotter::{
    Args, NegMode, plot_mollweide_auto, get_colormap, read_healpix_column,
    validate_scale_config, resolve_input_color, InputColor,
};
use healpix_plotter::scale::Scale;
use healpix_plotter::healpix::{read_healpix_meta, HealpixMeta, target_nside_for_resolution, downgrade_healpix_map};



fn main() {
    let args = Args::parse();

    // -----------------------------
    // Resolve scale + colormap
    // -----------------------------
    let (scale, cmap_name) = if args.planck_log {
        (
            Scale::PlanckLog {
                linthresh: args.linthresh.unwrap_or(300.0),
            },
            "planck-log",
        )
    } else {
        let scale = if args.symlog {
            Scale::Symlog {
                linthresh: args.linthresh.unwrap_or(1.0),
            }
        } else if args.asinh {
            Scale::Asinh {
                scale: args.linthresh.unwrap_or(1.0),
            }
        } else if args.log {
            Scale::Log
        } else if args.hist {
            Scale::Histogram
        } else {
            Scale::Linear
        };

        (scale, args.cmap.as_str())
    };

    validate_scale_config(&scale, args.min, args.max);

    let cmap = get_colormap(cmap_name);

    println!("Reading HEALPix metadata...");
    let start = Instant::now();
    let meta = read_healpix_meta(&args.fits)
        .expect("Could not determine HEALPix ordering / NSIDE");
    println!("Metadata read in {:.2}s", start.elapsed().as_secs_f64());

    println!("Reading FITS data ({} pixels)...", 12 * meta.nside * meta.nside);
    let start = Instant::now();
    let mut map = read_healpix_column(&args.fits, args.col);
    for v in &mut map {
        *v *= args.scale;
    }
    println!("Data read and scaled in {:.2}s", start.elapsed().as_secs_f64());

    let neg_mode = match args.neg_mode.as_str() {
        "zero" => NegMode::Zero,
        "unseen" => NegMode::Unseen,
        _ => panic!("--neg-mode must be 'zero' or 'unseen'"),
    };

    let bad_color_rgba = resolve_input_color(Some(args.bad_color.unwrap_or(InputColor::Gray)), &cmap, args.transparent);

    let bg_color_rgba = resolve_input_color(Some(args.bg_color.unwrap_or(InputColor::Transparent)), &cmap, args.transparent);

    // For very high resolution maps, downgrade to improve performance
    // nside=8192 has 805M pixels, which causes cache misses during rendering
    let (final_map, final_meta) = if meta.nside > 1024 {
        println!("High-resolution map detected (nside={}), downgrading for performance", meta.nside);
        let target_nside = target_nside_for_resolution(args.width as usize, (args.width / 2) as usize);
        println!("Downgrading from nside={} to nside={} for {}x{} output", 
                meta.nside, target_nside, args.width, args.width / 2);
        let start = Instant::now();
        let downgraded_map = downgrade_healpix_map(&map, meta.nside, target_nside, meta.ordering);
        println!("Downgrade completed in {:.2}s", start.elapsed().as_secs_f64());
        (downgraded_map, HealpixMeta { nside: target_nside, ordering: meta.ordering })
    } else {
        (map, meta)
    };

    println!("Starting plot generation...");
    let start = Instant::now();
    plot_mollweide_auto(
        &final_map,
        args.width,
        &args.out,
        args.min,
        args.max,
        &cmap,
        !args.no_cbar,
        args.transparent,
        !args.no_border,
        args.gamma,
        scale,
        neg_mode,
        bad_color_rgba,
        bg_color_rgba,
        final_meta,
    );
    println!("Plot generation completed in {:.2}s", start.elapsed().as_secs_f64());



}
