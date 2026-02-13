//! Plot execution module - handles projection selection and rendering.
//!
//! This module encapsulates the logic for selecting the appropriate projection
//! and executing the plot generation. It serves as the execution layer that
//! takes prepared parameters and generates output.

use crate::cli::Args;
use crate::cli_builder;
use crate::mask::PixelMask;
use crate::pipeline::ProcessedData;
use crate::rotation::ViewTransform;
use crate::{plot_hammer_auto, plot_gnomonic_auto, plot_mollweide_auto};

/// Configuration bundle for plot execution.
///
/// Contains all the configuration needed to execute a plot,
/// prepared by the setup module.
pub struct ExecutionConfig<'a> {
    /// Parsed command-line arguments
    pub args: &'a Args,
    /// Resolved plot configuration (colors, scales, rendering)
    pub plot_config: &'a crate::cli::PlotConfig,
    /// Loaded and processed HEALPix data
    pub data: &'a ProcessedData,
    /// View transformation (rotation/translation)
    pub view: &'a ViewTransform,
    /// Optional pixel mask for filtering/highlighting
    pub mask: Option<PixelMask>,
}

/// Execute plotting based on the selected projection.
///
/// This function routes to the appropriate projection handler based on
/// the `args.projection` setting. Each projection has its own parameter
/// building function in cli_builder and rendering function.
///
/// # Arguments
///
/// * `config` - Execution configuration with all necessary data and settings
/// * `verbose` - Whether to print progress messages
///
/// # Returns
///
/// Result indicating success or describing the error encountered
///
/// # Errors
///
/// Returns an error if:
/// - Unknown projection is specified
/// - Parameter building fails for the selected projection
/// - Any rendering operation fails
///
/// # Example
///
/// ```ignore
/// let exec_config = ExecutionConfig {
///     args: &args,
///     plot_config: &config,
///     data: &data,
///     view: &view,
///     mask: None,
/// };
/// execute_plot(&exec_config, true)?;
/// ```
pub fn execute_plot(config: &ExecutionConfig, verbose: bool) -> Result<(), String> {
    if verbose {
        let input_file = config.args.fits.as_deref()
            .unwrap_or("<no input>");
        let output_file = config.args.get_output_filename();
        println!("\n{} -> {}", input_file, output_file);
    }

    let projection = config.args.projection.to_lowercase();

    match projection.as_str() {
        "mollweide" => execute_mollweide(config)?,
        "gnomonic" => execute_gnomonic(config)?,
        "hammer" => execute_hammer(config)?,
        proj => {
            return Err(format!(
                "Unknown projection: '{}'. Available projections: 'mollweide', 'gnomonic', 'hammer'",
                proj
            ));
        }
    }

    Ok(())
}

/// Execute Mollweide projection plotting.
fn execute_mollweide(config: &ExecutionConfig) -> Result<(), String> {
    let params = cli_builder::build_mollweide_params(
        config.args,
        config.data,
        config.plot_config,
        config.view,
        config.mask.clone(),
    )?;
    plot_mollweide_auto(params);
    Ok(())
}

/// Execute Gnomonic projection plotting.
fn execute_gnomonic(config: &ExecutionConfig) -> Result<(), String> {
    let params = cli_builder::build_gnomonic_params(
        config.args,
        config.data,
        config.plot_config,
        config.view,
        config.mask.clone(),
    )?;
    plot_gnomonic_auto(params);
    Ok(())
}

/// Execute Hammer projection plotting.
fn execute_hammer(config: &ExecutionConfig) -> Result<(), String> {
    let params = cli_builder::build_hammer_params(
        config.args,
        config.data,
        config.plot_config,
        config.view,
        config.mask.clone(),
    )?;
    plot_hammer_auto(params);
    Ok(())
}

#[cfg(test)]
    mod tests {}
