use clap::Parser;
use healpix_plotter::{Args, BadColor, NegMode, plot_mollweide, get_colormap, Colormap, read_healpix_column};
use image::Rgba;

fn main() {
    // Parse CLI arguments
    let args = Args::parse();

    // Determine colormap
    let cmap = get_colormap(&args.cmap);

    // Determine how to handle negative or invalid pixels
    let neg_mode = match args.neg_mode.as_str() {
        "zero" => NegMode::Zero,
        "unseen" => NegMode::Unseen,
        other => panic!("--neg-mode must be 'zero' or 'unseen', got '{other}'"),
    };

    // Determine RGBA color for "bad" pixels
    let bad_color_rgba = healpix_plotter::resolve_bad_color(args.bad_color, cmap);

    // Read HEALPix column from FITS file
    println!("Reading HEALPix column {} from {}", args.col, args.fits);
    let map = read_healpix_column(&args.fits, args.col);

    // Call the plotting routine
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
        args.log,
        args.symlog,
        args.asinh,
        args.linthresh,
        args.asinh_scale,
        neg_mode,
        bad_color_rgba,
    );

    println!("Saved Mollweide projection to {}", args.out);
}

