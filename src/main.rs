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
    let args = Args::parse();

    // Validate PDF backend argument
    let valid_backends = ["cairo", "printpdf"];
    if !valid_backends.contains(&args.pdf_backend.as_str()) {
        return Err(format!(
            "Invalid PDF backend '{}'. Valid options are: {}",
            args.pdf_backend,
            valid_backends.join(", ")
        ));
    }

    // Setup: Initialize configuration and load data
    let setup_result = setup::setup_initialization(&args, args.verbose)?;
    let data = setup::load_data(&args, args.verbose)?;

    // Create mask if specified
    let mask = cli_builder::create_pixel_mask(&args, &data, args.verbose)?;

    // Execute: Perform the actual plotting
    let exec_config = ExecutionConfig {
        args: &args,
        plot_config: &setup_result.config,
        data: &data,
        view: &setup_result.view,
        mask,
    };

    executor::execute_plot(&exec_config, args.verbose)?;

    if args.verbose {
        println!("Plot generation completed successfully\n");
    }

    Ok(())
}
