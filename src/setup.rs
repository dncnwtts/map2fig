//! Setup module - handles application initialization.
//!
//! This module encapsulates all the setup steps needed before execution:
//! - Configuration resolution
//! - View transform calculation
//! - Data loading and processing
//!
//! This separation makes the initialization flow clear and testable.

use crate::cli::Args;
use crate::pipeline::load_and_process_data;
use crate::rotation::ViewTransform;

/// Result of successful setup containing all initialized data.
pub struct SetupResult {
    /// Resolved plot configuration
    pub config: crate::cli::PlotConfig,
    /// Calculated view transformation
    pub view: ViewTransform,
}

/// Initialize the application configuration.
///
/// This function performs all necessary setup steps:
/// 1. Resolves plot configuration from CLI arguments
/// 2. Calculates view transformation (rotation)
/// 3. Loads and processes HEALPix data
///
/// # Arguments
///
/// * `args` - Parsed command-line arguments
/// * `verbose` - Whether to print progress messages
///
/// # Returns
///
/// A `SetupResult` containing initialized configuration and view transform,
/// or an error message describing what failed
///
/// # Errors
///
/// Returns an error if:
/// - Configuration resolution fails
/// - View transform calculation fails
/// - Data loading fails
/// - FITS file is invalid or unreadable
///
/// # Example
///
/// ```ignore
/// let setup = setup_initialization(&args, true)?;
/// // setup.config can now be used for plotting
/// // setup.view contains rotation information
/// ```
pub fn setup_initialization(args: &Args, verbose: bool) -> Result<SetupResult, String> {
    // Resolve plot configuration
    let config = args
        .resolve_config()
        .map_err(|e| format!("Failed to resolve configuration: {}", e))?;

    // Resolve view transformation
    let view = args
        .resolve_view_transform()
        .map_err(|e| format!("Failed to resolve rotation: {}", e))?;

    if verbose {
        // Show coordinate transformation if applicable
        if let Some(transform) = args.describe_coord_transform() {
            println!("{}", transform);
        }
    }

    Ok(SetupResult { config, view })
}

/// Load and process HEALPix data.
///
/// Handles loading FITS file data and applying any necessary preprocessing.
/// For gnomonic projections, uses a larger effective width to maintain resolution
/// when sampling a small field of view.
///
/// # Arguments
///
/// * `args` - Command-line arguments (contains FITS filename and column info)
/// * `verbose` - Whether to print progress messages
///
/// # Returns
///
/// Processed HEALPix data or an error message
///
/// # Errors
///
/// Returns an error if the FITS file cannot be read or processed
///
/// # Example
///
/// ```ignore
/// let data = load_data(&args, true)?;
/// println!("Loaded {} pixels", data.map.len());
/// ```
pub fn load_data(args: &Args, _verbose: bool) -> Result<crate::pipeline::ProcessedData, String> {
    // For gnomonic projections, use a larger effective width to avoid map degradation
    // since we're sampling at full resolution in a small field of view
    let effective_width = match args.projection.to_lowercase().as_str() {
        "gnomonic" => 32768, // Force no degradation for gnomonic
        _ => args.width,
    };

    let data = load_and_process_data(
        &args.fits,
        args.col,
        args.scale,
        effective_width,
        args.verbose,
        args.no_downgrade,
        &args.quality,
    )
    .map_err(|e| format!("Failed to load and process data: {}", e))?;

    Ok(data)
}

#[cfg(test)]
mod tests {}
