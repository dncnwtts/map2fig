use std::fs::File;
use std::time::Instant;

fn main() {
    let test_files = vec![
        (
            "Small (7 MB, f32)",
            "tests/data/class_dr1_40GHz_skymap_n128.fits",
        ),
        ("Large (576 MB, f32)", "tests/data/npipe6v20_217_map_K.fits"),
    ];

    println!("F32 Reader Performance Benchmark");
    println!("================================\n");

    for (label, filename) in test_files {
        let f = File::open(filename).expect("Failed to open FITS file");
        let file_size = f.metadata().unwrap().len() as f64 / (1024.0 * 1024.0);

        // Warm up
        let _ = map2fig::read_healpix_column(filename, 0);

        // Measure
        let start = Instant::now();
        let data = map2fig::read_healpix_column(filename, 0);
        let elapsed = start.elapsed();

        let n_pixels = data.len();
        let gbps = (file_size / elapsed.as_secs_f64()) / 1024.0;

        println!("{}", label);
        println!("  File size: {:.1} MB", file_size);
        println!("  Pixels: {} ({})", n_pixels, format_count(n_pixels as u64));
        println!("  Time: {:.3}s", elapsed.as_secs_f64());
        println!("  Throughput: {:.2} GB/s", gbps);
        println!();
    }
}

fn format_count(n: u64) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
    } else {
        format!("{}", n)
    }
}
