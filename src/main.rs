use clap::Parser;
use healpix_plotter::{
    Args, NegMode, plot_mollweide, plot_mollweide_pdf, get_colormap, read_healpix_column,
};
use healpix_plotter::scale::Scale;
use healpix_plotter::{validate_scale_config, resolve_bad_color, BadColor};

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
        } else {
            Scale::Linear
        };

        (scale, args.cmap.as_str())
    };

    validate_scale_config(&scale, args.min, args.max);

    let cmap = get_colormap(cmap_name);
    let map = read_healpix_column(&args.fits, args.col);

    let neg_mode = match args.neg_mode.as_str() {
        "zero" => NegMode::Zero,
        "unseen" => NegMode::Unseen,
        _ => panic!("--neg-mode must be 'zero' or 'unseen'"),
    };


    let bad_color_rgba = resolve_bad_color(Some(args.bad_color.unwrap_or(BadColor::Auto)), &cmap, args.transparent);

    /*
    plot_mollweide(
        &map,
        args.width,
        &args.out,
        args.min,
        args.max,
        cmap,
        !args.no_cbar,
        args.transparent,
        !args.no_border,
        args.gamma,
        scale,
        neg_mode,
        bad_color_rgba,
    );
    */

    plot_mollweide_pdf(
        &map,
        args.width,
        &args.out,
        args.min,
        args.max,
        cmap,
        !args.no_cbar,
        args.transparent,
        !args.no_border,
        args.gamma,
        scale,
        neg_mode,
        bad_color_rgba,
    );

}
