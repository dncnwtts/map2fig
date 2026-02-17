/// Benchmark suite for FITS data type optimization (Tier Adaptive)
/// 
/// This benchmark measures the impact of preserving f32/f64 types instead of
/// converting everything to f64 upfront.
///
/// Run with: cargo bench --bench fits_type_optimization
/// Or single test: cargo test --release fit_read_benchmark -- --nocapture --test-threads=1

#[cfg(test)]
mod benchmarks {
    use std::time::Instant;
    use std::fs;

    // Test files available in workspace
    const TEST_FILES: &[(&str, &str, &str)] = &[
        (
            "tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits",
            "3.1GB",
            "Large f32+sparse"
        ),
        (
            "tests/data/npipe6v20_217_map_K.fits",
            "577MB",
            "Medium f32 dense"
        ),
        (
            "tests/data/npipe_nodip.fits",
            "193MB",
            "Small f32 dense"
        ),
        (
            "tests/data/cosmoglobe_DIRBE_06_I_n00512_DR2.fits",
            "73MB",
            "Small f32 dense"
        ),
    ];

    #[test]
    fn fits_read_benchmark_with_timing() {
        println!("\n=== FITS Data Type Optimization Benchmark ===");
        println!("Testing adaptive f32/f64 preservation...\n");

        for (filepath, size_label, description) in TEST_FILES {
            // Skip files that don't exist
            if !std::path::Path::new(filepath).exists() {
                println!("⊘ {}: {} (file not found)", filepath, description);
                continue;
            }

            let file_metadata = match fs::metadata(filepath) {
                Ok(m) => m,
                Err(e) => {
                    println!("✗ {}: Error reading metadata - {}", filepath, e);
                    continue;
                }
            };

            let file_size_mb = file_metadata.len() as f64 / (1024.0 * 1024.0);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("File: {}", filepath);
            println!("Size: {} ({:.1} MB actual)", size_label, file_size_mb);
            println!("Type: {}", description);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

            // Benchmark with timing
            match benchmark_read_column(filepath, 0) {
                Ok((elapsed_ms, data_type, pixel_count, mem_bytes)) => {
                    let throughput_mb_s = file_size_mb / (elapsed_ms as f64 / 1000.0);
                    let mem_mb = mem_bytes as f64 / (1024.0 * 1024.0);
                    
                    println!("✓ Data type: {}", data_type);
                    println!("  Pixels: {}", pixel_count);
                    println!("  Memory: {:.1} MB", mem_mb);
                    println!("  Time: {:.3} s", elapsed_ms as f64 / 1000.0);
                    println!("  Throughput: {:.1} MB/s", throughput_mb_s);
                    println!();
                }
                Err(e) => {
                    println!("✗ Error: {}\n", e);
                }
            }
        }

        println!("=== Benchmark Complete ===\n");
    }

    fn benchmark_read_column(
        filepath: &str,
        col_idx: usize,
    ) -> Result<(u128, String, usize, usize), String> {
        use map2fig::{read_healpix_column, DataArray};

        let start = Instant::now();
        let data = read_healpix_column(filepath, col_idx);
        let elapsed = start.elapsed();

        let data_type = data.dtype().to_string();
        let pixel_count = data.len();
        let mem_bytes = data.memory_size_bytes();

        Ok((elapsed.as_millis(), data_type, pixel_count, mem_bytes))
    }

    #[test]
    fn verify_data_types() {
        println!("\n=== Data Type Verification ===");
        println!("Checking that adaptive types are working...\n");

        use map2fig::read_healpix_column;

        for (filepath, _size, description) in TEST_FILES {
            if !std::path::Path::new(filepath).exists() {
                continue;
            }

            match read_healpix_column(filepath, 0) {
                data => {
                    println!("✓ {} ({})", filepath, description);
                    println!("  Type: {}", data.dtype());
                    println!("  Size: {} pixels ({} MB)", 
                        data.len(), 
                        data.memory_size_bytes() / (1024 * 1024)
                    );
                    
                    // Verify we can get statistics without full conversion
                    let min = data.min_value();
                    let max = data.max_value();
                    println!("  Range: {:.6e} to {:.6e}", min, max);
                    println!();
                }
            }
        }
    }

    #[test]
    fn benchmark_with_system_time() {
        println!("\n=== Memory & CPU Usage Benchmark ===");
        println!("Format: file → type | read_time | memory_used | throughput\n");

        use map2fig::read_healpix_column;
        
        let mut total_time_ms = 0u128;
        let mut total_size_mb = 0.0;

        for (filepath, size_label, _desc) in TEST_FILES {
            if !std::path::Path::new(filepath).exists() {
                continue;
            }

            let file_size = fs::metadata(filepath)
                .map(|m| m.len() as f64 / (1024.0 * 1024.0))
                .unwrap_or(0.0);

            let start = Instant::now();
            let data = read_healpix_column(filepath, 0);
            let elapsed_ms = start.elapsed().as_millis();

            let throughput = if elapsed_ms > 0 {
                file_size / (elapsed_ms as f64 / 1000.0)
            } else {
                0.0
            };

            println!("{:50} | {} | {:7}ms | {:7}MB | {:.1} MB/s",
                &filepath[..std::cmp::min(50, filepath.len())],
                format!("{:7}", data.dtype()),
                elapsed_ms,
                data.memory_size_bytes() / (1024 * 1024),
                throughput
            );

            total_time_ms += elapsed_ms;
            total_size_mb += file_size;
        }

        if total_time_ms > 0 {
            let avg_throughput = total_size_mb / (total_time_ms as f64 / 1000.0);
            println!("\n{}", "─".repeat(120));
            println!("TOTAL: {:.1} MB read in {:.2}s at {:.1} MB/s avg",
                total_size_mb,
                total_time_ms as f64 / 1000.0,
                avg_throughput
            );
        }
        println!();
    }
}

fn main() {
    println!("Run benchmarks with: cargo test --release fit_read_benchmark -- --nocapture");
}
