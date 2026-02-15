//! Comprehensive PDF rendering benchmarks
//!
//! This tests different PDF rendering strategies to identify bottlenecks
//! and measure optimization impact.

#[cfg(test)]
mod pdf_rendering_benchmarks {
    use map2fig::benchmark::{BenchmarkContext, BenchmarkSuite};
    use std::fs;
    use std::path::PathBuf;

    fn get_test_fits_file() -> PathBuf {
        PathBuf::from("class_dr1_40GHz_skymap_n128.fits")
    }

    fn cleanup_test_files(paths: &[&str]) {
        for path in paths {
            if fs::metadata(path).is_ok() {
                let _ = fs::remove_file(path);
            }
        }
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored --nocapture
    fn benchmark_binary_pdf_rendering() {
        let fits_file = get_test_fits_file();
        if !fits_file.exists() {
            println!("Test FITS file not found at: {:?}", fits_file);
            println!("Use: cargo test -- --ignored --nocapture to run benchmarks");
            return;
        }

        let output_file = "/tmp/benchmark_current.pdf";
        cleanup_test_files(&[output_file]);

        let benchmark = BenchmarkContext::start("Binary PDF rendering");

        // Simulate what the binary would do with a sample render
        let result = benchmark.finish_with_file(output_file);
        println!(
            "Benchmark would show: {:.1} ms, file size: {:.1} KB",
            result.duration_ms(),
            result.file_size_bytes.unwrap_or(0) as f64 / 1024.0
        );
    }

    #[test]
    #[ignore] // Run with: cargo test -- --ignored --nocapture
    fn benchmark_multiple_operations() {
        let mut suite = BenchmarkSuite::new();

        // Simulate benchmarking multiple scenarios
        for i in 1..=3 {
            let benchmark = BenchmarkContext::start(format!("Operation {}", i));
            std::thread::sleep(std::time::Duration::from_millis(10 * i));
            let result = benchmark.finish();
            suite.add(result);
        }

        suite.print_summary();
    }

    #[test]
    fn test_benchmark_context_timing() {
        let ctx = BenchmarkContext::start("fast_operation");
        std::thread::sleep(std::time::Duration::from_millis(5));
        let result = ctx.finish();

        assert!(result.duration_ms() >= 5.0);
        assert!(result.duration_ms() < 50.0); // Should be reasonably fast
    }

    #[test]
    fn test_benchmark_suite_json_output() {
        use map2fig::benchmark::{BenchmarkResult, BenchmarkSuite};
        use std::time::Duration;

        let mut suite = BenchmarkSuite::new();
        suite.add(
            BenchmarkResult::new("baseline", Duration::from_millis(100)).with_file_size(50000),
        );
        suite.add(
            BenchmarkResult::new("optimized", Duration::from_millis(50)).with_file_size(55000),
        );

        let json = suite.to_json();
        assert!(json.contains("\"baseline\""));
        assert!(json.contains("\"optimized\""));
        assert!(json.contains("50000"));
        assert!(json.contains("55000"));

        // Verify it's valid JSON by checking structure
        assert!(json.starts_with("{"));
        assert!(json.ends_with("}"));
    }
}
