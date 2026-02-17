// This example temporarily tests the fallback path performance
// by using the internal fitsrs reader directly

use std::time::Instant;
use std::fs::File;
use std::io::Cursor;
use fitsrs::Fits;
use fitsrs::hdu::HDU;
use fitsrs::hdu::data::bintable::ColumnId;

fn main() {
    println!("Comparing F32 Native Reader vs Fallback (fitsrs) Path");
    println!("====================================================\n");

    let filename = "tests/data/npipe6v20_217_map_K.fits";
    let file = File::open(filename).expect("Failed to open file");
    let file_size = file.metadata().unwrap().len() as f64 / (1024.0 * 1024.0);

    // Test 1: F32 native reader
    println!("1. F32 Native Reader (optimized direct binary reading):");
    let start = Instant::now();
    for _ in 0..3 {
        let _ = map2fig::read_healpix_column(filename, 0);
    }
    let native_time = start.elapsed();
    let native_avg = native_time.as_secs_f64() / 3.0;
    println!("   3 runs: {:.3}s (avg: {:.3}s)", native_time.as_secs_f64(), native_avg);
    println!("   Throughput: {:.2} GB/s\n", file_size / native_avg / 1024.0);

    // Test 2: Fallback fitsrs path (simulated by using fitsrs only)
    println!("2. Fallback Path (fitsrs DataValue enum conversion):");
    let start = Instant::now();
    for _ in 0..3 {
        read_via_fitsrs(filename);
    }
    let fallback_time = start.elapsed();
    let fallback_avg = fallback_time.as_secs_f64() / 3.0;
    println!("   3 runs: {:.3}s (avg: {:.3}s)", fallback_time.as_secs_f64(), fallback_avg);
    println!("   Throughput: {:.2} GB/s\n", file_size / fallback_avg / 1024.0);

    // Summary
    let speedup = fallback_avg / native_avg;
    println!("Performance Improvement:");
    println!("  Speedup: {:.2}x faster", speedup);
    println!("  Time saved per read: {:.0}ms", (fallback_avg - native_avg) * 1000.0);
    println!("  Percentage improvement: {:.1}%", (1.0 - native_avg / fallback_avg) * 100.0);
}

// Simulate reading via fitsrs (the old slow path)
fn read_via_fitsrs(filename: &str) -> usize {
    use std::fs;
    let data = fs::read(filename).expect("Failed to read file");
    let cursor = Cursor::new(data);
    let mut fits = Fits::from_reader(cursor);
    let mut count = 0;

    while let Some(Ok(hdu)) = fits.next() {
        if let HDU::XBinaryTable(hdu) = hdu {
            let data = fits.get_data(&hdu);
            let mut table = data.table_data();
            
            // Just iterate through all values (equivalent to what the old code did)
            for _ in table.select_fields(&[ColumnId::Index(0)]) {
                count += 1;
            }
        }
    }
    count
}
