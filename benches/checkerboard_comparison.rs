/// Benchmark comparing original vs checkerboard downsampling
/// Run with: cargo bench --bench checkerboard_comparison

use std::path::Path;
use std::time::Instant;

fn main() {
    // Try to find a test FITS file
    let test_files = vec![
        "combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits",
        "cosmoglobe_DIRBE_10_I_n00512_DR2_v3.1.fits",
        "npipe_nodip.fits",
    ];

    let mut found_file = None;
    for file in &test_files {
        if Path::new(file).exists() {
            found_file = Some(file.to_string());
            break;
        }
    }

    if found_file.is_none() {
        eprintln!("No test FITS file found in current directory");
        eprintln!("Expected one of: {:?}", test_files);
        return;
    }

    let fits_file = found_file.unwrap();
    println!("Using FITS file: {}", fits_file);

    // This is a manual benchmark runner
    // To make it work, we'd need to expose the internal downsampling functions
    // For now, this shows the test structure
    println!("\nTo properly run this benchmark:");
    println!("1. The downsample functions need to be exposed in lib.rs");
    println!("2. Run via: cargo run --release --example benchmark_checkerboard -- {}", fits_file);
}
