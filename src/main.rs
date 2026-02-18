use clap::Parser;
use map2fig::cli::Args;
use map2fig::cli_builder;
use map2fig::executor::{self, ExecutionConfig};
use map2fig::setup;

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    use std::time::Instant;
    use std::path::Path;

    let mut args = Args::parse();

    // Validate and auto-generate output filename if needed
    if args.fits.is_none() {
        return Err("Usage: map2fig <FITS> [OUTPUT]".to_string());
    }
    
    if args.out.is_none() {
        let fits_path = args.fits.as_ref().unwrap();
        let path = Path::new(fits_path);
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("output");
        args.out = Some(format!("{}.png", stem));
    }

    // Validate PDF backend argument
    let valid_backends = ["cairo"];
    if !valid_backends.contains(&args.pdf_backend.as_str()) {
        return Err(format!(
            "Invalid PDF backend '{}'. Valid options are: {}",
            args.pdf_backend,
            valid_backends.join(", ")
        ));
    }

    let total_start = Instant::now();

    // Setup: Initialize configuration and load data
    let setup_start = Instant::now();
    let setup_result = setup::setup_initialization(&args, args.verbose)?;
    let setup_time = setup_start.elapsed();

    let load_start = Instant::now();
    let data = setup::load_data(&args, args.verbose)?;
    let load_time = load_start.elapsed();

    // Create mask if specified
    let mask = cli_builder::create_pixel_mask(&args, &data, args.verbose)?;

    // Execute: Perform the actual plotting
    let exec_start = Instant::now();
    let exec_config = ExecutionConfig {
        args: &args,
        plot_config: &setup_result.config,
        data: &data,
        view: &setup_result.view,
        mask,
    };

    executor::execute_plot(&exec_config, args.verbose)?;
    let exec_time = exec_start.elapsed();

    let total_time = total_start.elapsed();

    if args.verbose {
        println!("Plot generation completed successfully\n");
        eprintln!("\n=== Performance Breakdown ===");
        eprintln!(
            "Setup time:      {:.3}s ({:.1}%)",
            setup_time.as_secs_f64(),
            100.0 * setup_time.as_secs_f64() / total_time.as_secs_f64()
        );
        eprintln!(
            "Data load time:  {:.3}s ({:.1}%)",
            load_time.as_secs_f64(),
            100.0 * load_time.as_secs_f64() / total_time.as_secs_f64()
        );
        eprintln!(
            "Rendering time:  {:.3}s ({:.1}%)",
            exec_time.as_secs_f64(),
            100.0 * exec_time.as_secs_f64() / total_time.as_secs_f64()
        );
        eprintln!("Total time:      {:.3}s", total_time.as_secs_f64());
    }

    Ok(())
}
