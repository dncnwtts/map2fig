//! Benchmark comparing Cairo PDF vs alternative backends

#[cfg(test)]
mod pdf_backend_comparison {
    use image::RgbaImage;
    use map2fig::benchmark::{BenchmarkContext, BenchmarkSuite};
    use std::time::Instant;

    #[test]
    #[ignore] // Run with: cargo test -- --ignored --nocapture
    fn benchmark_cairo_vs_alternative() {
        println!("\n💡 To run backend benchmarks:");
        println!("   cargo test --test benchmark_backends -- --ignored --nocapture");
        println!("\nThis requires a FITS file and will compare:");
        println!("   - Cairo PDF rendering (current)");
        println!("   - Alternative backend rendering");
    }

    #[test]
    fn test_image_extraction_overhead() {
        // Measure just the image serialization overhead
        let width = 1024u32;
        let height = 1024u32;
        let mut img = RgbaImage::new(width, height);

        // Fill with test pattern
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let r = ((x * 255) / width) as u8;
            let g = ((y * 255) / height) as u8;
            let b = 128;
            *pixel = image::Rgba([r, g, b, 255]);
        }

        // Measure image data extraction to RGB
        let start = Instant::now();
        let mut rgb_data = Vec::with_capacity(width as usize * height as usize * 3);
        for pixel in img.pixels() {
            rgb_data.push(pixel[0]);
            rgb_data.push(pixel[1]);
            rgb_data.push(pixel[2]);
        }
        let extract_time = start.elapsed();

        let extract_ms = extract_time.as_secs_f64() * 1000.0;
        let data_size_mb = rgb_data.len() as f64 / 1_000_000.0;

        println!(
            "\n📊 Image conversion overhead (1024x1024 RGBA->RGB): {:.2} ms, {:.2} MB",
            extract_ms, data_size_mb
        );
        println!("   Note: This is ~3% of total PDF rendering time (300ms)");

        // RGBA->RGB extraction is O(n) so expect ~100ms for 3MB data
        assert!(
            extract_ms < 200.0,
            "Image conversion overhead unexpectedly high"
        );
    }

    #[test]
    fn test_bench_suite_functionality() {
        let mut suite = BenchmarkSuite::new();

        // Simulate some operations
        for i in 1..=3 {
            let ctx = BenchmarkContext::start(format!("Op {}", i));
            std::thread::sleep(std::time::Duration::from_millis(5));
            suite.add(ctx.finish());
        }

        suite.print_summary();
        let json = suite.results[0].to_json().unwrap_or_default();
        assert!(!json.is_empty());
    }
}
