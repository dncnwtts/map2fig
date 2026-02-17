use std::time::Instant;
use std::fs;
use map2fig::read_healpix_column;

fn main() {
    println!("\n╔══════════════════════════════════════════════════════════════════╗");
    println!("║      FITS Data Type Optimization Benchmark - v0.7.0              ║");
    println!("║      Measuring impact of adaptive f32/f64 type preservation      ║");
    println!("╚══════════════════════════════════════════════════════════════════╝\n");

    let test_files = vec![
        ("tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits", "Large f32+sparse (3.1GB)"),
        ("tests/data/npipe6v20_217_map_K.fits", "Medium f32 dense (577MB)"),
        ("tests/data/npipe_nodip.fits", "Small f32 dense (193MB)"),
        ("tests/data/cosmoglobe_DIRBE_06_I_n00512_DR2.fits", "Small f32 dense (73MB)"),
    ];

    let mut results = Vec::new();

    for (filepath, description) in &test_files {
        if !std::path::Path::new(filepath).exists() {
            println!("⊘  {} - FILE NOT FOUND", filepath);
            continue;
        }

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("File: {}", filepath);
        println!("Type: {}", description);
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        match benchmark_fits_read(filepath) {
            Ok((elapsed_ms, data_type, pixels, memory_mb, throughput_mb_s)) => {
                let file_size_mb = fs::metadata(filepath)
                    .map(|m| m.len() as f64 / (1024.0 * 1024.0))
                    .unwrap_or(0.0);

                println!("✓ Data Type:    {}", data_type);
                println!("  Pixels:       {} ({:.0}M)", 
                    format_number(pixels), 
                    pixels as f64 / 1_000_000.0
                );
                println!("  Memory Used:  {:.1} MB ({} bytes)", memory_mb, format_number(memory_mb as usize * 1024 * 1024));
                println!("  Read Time:    {:.3} seconds", elapsed_ms as f64 / 1000.0);
                println!("  Throughput:   {:.1} MB/s", throughput_mb_s);
                println!("  File Size:    {:.1} MB", file_size_mb);
                println!();

                results.push((
                    filepath.to_string(),
                    data_type,
                    elapsed_ms,
                    memory_mb,
                    throughput_mb_s,
                    file_size_mb,
                ));
            }
            Err(e) => {
                println!("✗ Error: {}\n", e);
            }
        }
    }

    // Summary
    if !results.is_empty() {
        println!("\n╔══════════════════════════════════════════════════════════════════╗");
        println!("║                      BENCHMARK SUMMARY                          ║");
        println!("╚══════════════════════════════════════════════════════════════════╝\n");

        println!("{:<50} | {}  | Time(ms) | Memory(MB) | MB/s", "File", "Type");
        println!("{}", "─".repeat(115));

        let mut total_time_ms = 0u128;
        let mut total_size_mb = 0.0;

        for (file, dtype, time_ms, mem_mb, throughput, file_size) in &results {
            let short_file = if file.len() > 50 {
                format!("...{}", &file[file.len()-47..])
            } else {
                file.clone()
            };
            println!("{:<50} | {:5} | {:>8} | {:>10.1} | {:>6.1}",
                short_file,
                dtype,
                time_ms,
                mem_mb,
                throughput
            );
            total_time_ms += time_ms;
            total_size_mb += file_size;
        }

        println!("{}", "─".repeat(115));
        let avg_throughput = total_size_mb * 1000.0 / total_time_ms as f64;
        println!("{:<50} | {:5} | {:>8} | {:>10.1} | {:>6.1}",
            format!("TOTAL ({} files)", results.len()),
            "mixed",
            total_time_ms,
            "",
            avg_throughput
        );
        println!();

        println!("╔══════════════════════════════════════════════════════════════════╗");
        println!("║                    EXPECTED VS ACTUAL                           ║");
        println!("╚══════════════════════════════════════════════════════════════════╝\n");

        // For the largest file (3.1GB), show expected vs actual
        if let Some((_, _, time_ms, _, throughput, size_mb)) = results.first() {
            println!("Largest File Analysis (3.1GB combined_map_95GHz):");
            println!();
            println!("Expected (v0.6.0):  10.9 seconds");
            println!("Actual (v0.7.0):    {:.2} seconds", *time_ms as f64 / 1000.0);
            
            if time_ms < &10900 {
                let improvement = (10900 - *time_ms) as f64 / 10900.0 * 100.0;
                println!("Improvement:        {:.1}%", improvement);
                
                if improvement >= 60.0 {
                    println!("\n✅ OPTIMIZATION SUCCESSFUL - Target 60% reached!");
                } else if improvement >= 40.0 {
                    println!("\n⚠️  PARTIAL SUCCESS - Hit {:.1}%, target was 60%", improvement);
                } else {
                    println!("\n❌ BELOW TARGET - Only {:.1}%, expected 60%", improvement);
                }
            } else {
                println!("\n❌ REGRESSION - Slower than expected!");
            }
            
            println!("\nThroughput: {:.1} MB/s (expected ~285 MB/s)", throughput);
        }

        println!("\n╔══════════════════════════════════════════════════════════════════╗");
        println!("║                         ANALYSIS                               ║");
        println!("╚══════════════════════════════════════════════════════════════════╝\n");

        let f32_count = results.iter().filter(|(_, t, _, _, _, _)| t == "float32").count();
        let f64_count = results.iter().filter(|(_, t, _, _, _, _)| t == "float64").count();

        println!("Data Types Detected:");
        println!("  • f32 (native): {} files", f32_count);
        println!("  • f64 (native): {} files", f64_count);
        println!();

        let avg_memory: f64 = results.iter().map(|(_, _, _, m, _, _)| m).sum::<f64>() / results.len() as f64;
        println!("Memory Efficiency:");
        println!("  • Average memory per file: {:.1} MB", avg_memory);
        println!("  • Expected memory (f32): size * 0.5 (4 bytes per pixel)");
        println!("  • Expected memory (f64): size * 1.0 (8 bytes per pixel)");
        println!();

        let avg_throughput_all: f64 = results.iter().map(|(_, _, _, _, t, _)| t).sum::<f64>() / results.len() as f64;
        let peak_throughput = results.iter().map(|(_, _, _, _, t, _)| t).fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        println!("I/O Performance:");
        println!("  • Average throughput: {:.1} MB/s", avg_throughput_all);
        println!("  • Peak throughput: {:.1} MB/s", peak_throughput);
        println!();

        println!("Conclusion:");
        if avg_throughput_all > 200.0 {
            println!("✅ Consistent high-performance I/O (>200 MB/s)");
        } else {
            println!("⚠️  I/O throughput could be improved");
        }
    }

    println!("\n");
}

fn benchmark_fits_read(filepath: &str) -> Result<(u128, String, usize, f64, f64), String> {
    let file_size = fs::metadata(filepath)
        .map_err(|e| format!("Cannot read file metadata: {}", e))?
        .len() as f64 / (1024.0 * 1024.0);

    let start = Instant::now();
    let data = read_healpix_column(filepath, 0);
    let elapsed_ms = start.elapsed().as_millis();

    let data_type = data.dtype().to_string();
    let pixel_count = data.len();
    let memory_mb = data.memory_size_bytes() as f64 / (1024.0 * 1024.0);

    let throughput = if elapsed_ms > 0 {
        file_size / (elapsed_ms as f64 / 1000.0)
    } else {
        0.0
    };

    Ok((elapsed_ms, data_type, pixel_count, memory_mb, throughput))
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000_000 {
        format!("{:.2}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.2}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.2}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}
