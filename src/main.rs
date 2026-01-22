use clap::Parser;
use healpix_plotter::{
    Args, NegMode, plot_mollweide_auto, get_colormap, read_healpix_column,
    validate_scale_config, resolve_input_color, InputColor,
};
use healpix_plotter::scale::Scale;
use healpix_plotter::healpix::read_healpix_meta;



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

    let meta = read_healpix_meta(&args.fits)
        .expect("Could not determine HEALPix ordering / NSIDE");

    // println!("Reading fits file");
    let mut map = read_healpix_column(&args.fits, args.col);
    for v in &mut map {
        *v *= args.scale;
    }


    let neg_mode = match args.neg_mode.as_str() {
        "zero" => NegMode::Zero,
        "unseen" => NegMode::Unseen,
        _ => panic!("--neg-mode must be 'zero' or 'unseen'"),
    };

    let bad_color_rgba = resolve_input_color(Some(args.bad_color.unwrap_or(InputColor::Gray)), &cmap, args.transparent);

    let bg_color_rgba = resolve_input_color(Some(args.bg_color.unwrap_or(InputColor::Transparent)), &cmap, args.transparent);

    // println!("Making Mollweide figure.");
    plot_mollweide_auto(
        &map,
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
        meta,
    );



}
