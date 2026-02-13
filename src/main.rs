use clap::Parser;
use map2fig::cli::Args;
use map2fig::cli_builder;
use map2fig::pipeline::load_and_process_data;
use map2fig::{plot_mollweide_auto, plot_gnomonic_auto, plot_hammer_auto};
use std::time::Instant;

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

    if args.verbose {
        println!("Reading HEALPix metadata...");
    }
    let start = Instant::now();

    // For gnomonic projections, use a larger effective width to avoid map degradation
    // since we're sampling at full resolution in a small field of view
    let effective_width = match args.projection.to_lowercase().as_str() {
        "gnomonic" => 32768, // Force no degradation for gnomonic
        _ => args.width,
    };

    let data = load_and_process_data(&args.fits, args.col, args.scale, effective_width, args.verbose)
        .map_err(|e| format!("Failed to load and process data: {}", e))?;
    if args.verbose {
        println!("Data processing completed in {:.2}s", start.elapsed().as_secs_f64());
    }

    // Create mask if specified
    let mask = cli_builder::create_pixel_mask(&args, &data, args.verbose)?;

    if args.verbose {
        println!("Starting plot generation...");
    }
    let start = Instant::now();

    match args.projection.to_lowercase().as_str() {
        "mollweide" => {
            let params = cli_builder::build_mollweide_params(&args, &data, &config, &view, mask)?;
            plot_mollweide_auto(params);
        }
        "gnomonic" => {
            let params = cli_builder::build_gnomonic_params(&args, &data, &config, &view, mask)?;
            plot_gnomonic_auto(params);
        }
        "hammer" => {
            let params = cli_builder::build_hammer_params(&args, &data, &config, &view, mask)?;
            plot_hammer_auto(params);
        }
        proj => {
            return Err(format!(
                "Unknown projection: '{}'. Available projections: 'mollweide', 'gnomonic', 'hammer'",
                proj
            ));
        }
    }

    if args.verbose {
        println!("Plot generation completed in {:.2}s", start.elapsed().as_secs_f64());
    }
    Ok(())
}


