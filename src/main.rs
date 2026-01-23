use clap::Parser;
use healpix_plotter::cli::Args;
use healpix_plotter::pipeline::load_and_process_data;
use healpix_plotter::plot_mollweide_auto_test;
use std::time::Instant;

fn main() {
    let args = Args::parse();
    let config = args.resolve_config().expect("Failed to resolve configuration");

    println!("Reading HEALPix metadata...");
    let start = Instant::now();
    let data = load_and_process_data(&args.fits, args.col, args.scale, args.width)
        .expect("Failed to load and process data");
    println!("Data processing completed in {:.2}s", start.elapsed().as_secs_f64());

    println!("Starting plot generation...");
    let start = Instant::now();
    println!("DEBUG: Calling plot_mollweide_auto_test");
    plot_mollweide_auto_test();
    println!("Plot generation completed in {:.2}s", start.elapsed().as_secs_f64());
}
