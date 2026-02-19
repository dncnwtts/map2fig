use map2fig::fits::read_healpix_column_cached;
use map2fig::healpix::{
    downgrade_healpix_map, downgrade_healpix_map_checkerboard, read_healpix_meta,
};
/// Compare original vs checkerboard downsampling on real FITS data
///
/// Usage: cargo run --release --example compare_downsampling -- <fits_file> [nside]
/// Example: cargo run --release --example compare_downsampling -- combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits 1024
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <fits_file> [target_nside]", args[0]);
        eprintln!(
            "Example: {} combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits 1024",
            args[0]
        );
        std::process::exit(1);
    }

    let fits_file = &args[1];
    let target_nside: i64 = if args.len() > 2 {
        args[2].parse().expect("target_nside must be a number")
    } else {
        1024
    };

    println!("=== Downsampling Comparison ===");
    println!("FITS file: {}", fits_file);
    println!("Target nside: {}\n", target_nside);

    // Read FITS metadata
    let start = Instant::now();
    let meta = match read_healpix_meta(fits_file) {
        Some(m) => m,
        None => {
            eprintln!("Error reading FITS metadata from {}", fits_file);
            std::process::exit(1);
        }
    };

    // Read the data column
    let data_array = read_healpix_column_cached(fits_file, 0);
    let read_time = start.elapsed();

    // Convert to f64 for this example
    let map = data_array.as_f64_vec().into_owned();

    let source_nside = meta.nside;
    let ordering = meta.ordering;
    let npix = map.len();
    let valid_pixels: usize = map.iter().filter(|&&v| v != -1.6375e30).count();

    println!("File loaded in {:.3}s", read_time.as_secs_f64());
    println!("Source nside: {}", source_nside);
    println!("Ordering: {:?}", ordering);
    println!("Total pixels: {} ({:.2} MB)", npix, npix as f64 * 8.0 / 1e6);
    println!(
        "Valid pixels: {} ({:.1}%)\n",
        valid_pixels,
        valid_pixels as f64 / npix as f64 * 100.0
    );

    // Original downsampling
    println!("Running original downsampling...");
    let start = Instant::now();
    let map_orig = downgrade_healpix_map(&map, source_nside, target_nside, ordering);
    let time_orig = start.elapsed();
    println!("  Completed in {:.3}s", time_orig.as_secs_f64());
    println!("  Output: {} pixels\n", map_orig.len());

    // Checkerboard downsampling
    println!("Running checkerboard downsampling...");
    let start = Instant::now();
    let map_check = downgrade_healpix_map_checkerboard(&map, source_nside, target_nside, ordering);
    let time_check = start.elapsed();
    println!("  Completed in {:.3}s", time_check.as_secs_f64());
    println!("  Output: {} pixels\n", map_check.len());

    // Compare results
    println!("=== Results ===");
    println!("Original time:     {:.3}s", time_orig.as_secs_f64());
    println!("Checkerboard time: {:.3}s", time_check.as_secs_f64());
    let speedup = time_orig.as_secs_f64() / time_check.as_secs_f64();
    let pct_improvement = (1.0 - time_check.as_secs_f64() / time_orig.as_secs_f64()) * 100.0;
    println!(
        "Speedup: {:.2}x ({:.1}% improvement)",
        speedup, pct_improvement
    );

    // Calculate quality metrics
    println!("\n=== Quality Comparison ===");
    let mut rmse: f64 = 0.0;
    let mut unseen_orig: usize = 0;
    let mut unseen_check: usize = 0;
    let mut mismatch: usize = 0;

    for (orig, check) in map_orig.iter().zip(map_check.iter()) {
        if orig.is_nan() || *orig == -1.6375e30 {
            unseen_orig += 1;
        }
        if check.is_nan() || *check == -1.6375e30 {
            unseen_check += 1;
        }
        if (orig - check).abs() > 1e-6 {
            mismatch += 1;
            rmse += (orig - check).powf(2.0);
        }
    }
    rmse = (rmse / map_orig.len() as f64).sqrt();

    println!(
        "Mismatched pixels: {} ({:.1}%)",
        mismatch,
        mismatch as f64 / map_orig.len() as f64 * 100.0
    );
    println!("RMSE: {:.6e}", rmse);
    println!("Unseen (original): {}", unseen_orig);
    println!("Unseen (checkerboard): {}", unseen_check);

    if speedup > 1.5 {
        println!("\n✓ Speedup significant! ({:.2}x)", speedup);
    } else if speedup > 1.1 {
        println!("\n~ Speedup modest ({:.2}x)", speedup);
    } else {
        println!("\n✗ No speedup gained ({:.2}x)", speedup);
    }
}
