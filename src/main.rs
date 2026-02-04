use clap::Parser;
use map2fig::cli::Args;
use map2fig::pipeline::load_and_process_data;
use map2fig::plot_mollweide_auto;
use std::time::Instant;

fn main() {
    let args = Args::parse();
    let config = args.resolve_config().expect("Failed to resolve configuration");
    let view = args.resolve_view_transform().expect("Failed to resolve rotation");

    if args.verbose {println!("Reading HEALPix metadata...");}
    let start = Instant::now();
    let data = load_and_process_data(&args.fits, args.col, args.scale, args.width, args.verbose)
        .expect("Failed to load and process data");
    if args.verbose {println!("Data processing completed in {:.2}s", start.elapsed().as_secs_f64());}

    if args.verbose {println!("Starting plot generation...");}
    let start = Instant::now();
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
    );
    if args.verbose {println!("Plot generation completed in {:.2}s", start.elapsed().as_secs_f64());}
}
