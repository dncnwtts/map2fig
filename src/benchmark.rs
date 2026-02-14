//! PDF Rendering Benchmarks and Performance Analysis
//!
//! This module provides comprehensive benchmarking tools for evaluating
//! different PDF rendering strategies and identifying optimization opportunities.

use std::path::Path;
use std::time::{Duration, Instant};

/// Results from a single benchmark run
#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub name: String,
    pub duration: Duration,
    pub file_size_bytes: Option<usize>,
    pub memory_peak_mb: Option<f64>,
}

impl BenchmarkResult {
    pub fn new(name: impl Into<String>, duration: Duration) -> Self {
        Self {
            name: name.into(),
            duration,
            file_size_bytes: None,
            memory_peak_mb: None,
        }
    }

    pub fn with_file_size(mut self, size: usize) -> Self {
        self.file_size_bytes = Some(size);
        self
    }

    pub fn with_memory(mut self, peak_mb: f64) -> Self {
        self.memory_peak_mb = Some(peak_mb);
        self
    }

    pub fn duration_ms(&self) -> f64 {
        self.duration.as_secs_f64() * 1000.0
    }

    pub fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        let mut json = format!(
            r#"{{"name": "{}", "duration_ms": {:.2}"#,
            self.name,
            self.duration_ms()
        );

        if let Some(size) = self.file_size_bytes {
            json.push_str(&format!(r#", "file_size_bytes": {}"#, size));
        }

        if let Some(mem) = self.memory_peak_mb {
            json.push_str(&format!(r#", "memory_peak_mb": {:.2}"#, mem));
        }

        json.push('}');
        Ok(json)
    }
}

/// A collection of benchmark results for comparison
#[derive(Debug, Default)]
pub struct BenchmarkSuite {
    pub results: Vec<BenchmarkResult>,
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }

    /// Print a formatted comparison table
    pub fn print_summary(&self) {
        if self.results.is_empty() {
            println!("No results to display");
            return;
        }

        // Find the baseline (first result)
        let baseline_time = self.results[0].duration_ms();

        println!(
            "\n{:<40} {:>12} {:>12} {:>12}",
            "Benchmark", "Time (ms)", "Size (KB)", "Speedup"
        );
        println!("{}", "=".repeat(80));

        for result in &self.results {
            let time_ms = result.duration_ms();
            let speedup = baseline_time / time_ms;
            let speedup_str = if speedup == 1.0 {
                "baseline".to_string()
            } else if speedup > 1.0 {
                format!("{:.2}x faster", speedup)
            } else {
                format!("{:.2}x slower", 1.0 / speedup)
            };

            let size_str = result
                .file_size_bytes
                .map(|b| format!("{:.1}", b as f64 / 1024.0))
                .unwrap_or_else(|| "?".to_string());

            println!(
                "{:<40} {:>12.1} {:>12} {:>12}",
                result.name, time_ms, size_str, speedup_str
            );
        }
        println!();
    }

    /// Export results as JSON
    pub fn to_json(&self) -> String {
        let mut json = String::from("{\n  \"benchmarks\": [\n");
        for (i, result) in self.results.iter().enumerate() {
            json.push_str(&format!(
                "    {{\n      \"name\": \"{}\",\n      \"duration_ms\": {:.2},\n      \"file_size_bytes\": {}",
                result.name,
                result.duration_ms(),
                result
                    .file_size_bytes
                    .map(|b| b.to_string())
                    .unwrap_or_else(|| "null".to_string())
            ));

            if let Some(mem) = result.memory_peak_mb {
                json.push_str(&format!(",\n      \"memory_peak_mb\": {:.2}", mem));
            }

            json.push_str("\n    }");
            if i < self.results.len() - 1 {
                json.push(',');
            }
            json.push('\n');
        }
        json.push_str("  ]\n}");
        json
    }
}

/// Benchmark context for measuring specific operations
pub struct BenchmarkContext {
    start: Instant,
    name: String,
}

impl BenchmarkContext {
    pub fn start(name: impl Into<String>) -> Self {
        Self {
            start: Instant::now(),
            name: name.into(),
        }
    }

    pub fn finish(self) -> BenchmarkResult {
        let duration = self.start.elapsed();
        BenchmarkResult::new(self.name, duration)
    }

    pub fn finish_with_file<P: AsRef<Path>>(self, path: P) -> BenchmarkResult {
        let duration = self.start.elapsed();
        let file_size = std::fs::metadata(path).ok().map(|m| m.len() as usize);

        let mut result = BenchmarkResult::new(self.name, duration);
        if let Some(size) = file_size {
            result = result.with_file_size(size);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_result_creation() {
        let result = BenchmarkResult::new("test", Duration::from_millis(100));
        assert_eq!(result.name, "test");
        assert_eq!(result.duration_ms(), 100.0);
    }

    #[test]
    fn test_benchmark_suite_speedup_calculation() {
        let mut suite = BenchmarkSuite::new();
        suite.add(BenchmarkResult::new("baseline", Duration::from_millis(100)));
        suite.add(BenchmarkResult::new("optimized", Duration::from_millis(50)));

        // Speedup should be 2x
        let baseline_time = suite.results[0].duration_ms();
        let optimized_time = suite.results[1].duration_ms();
        let speedup = baseline_time / optimized_time;

        assert!((speedup - 2.0).abs() < 0.01);
    }

    #[test]
    fn test_benchmark_json_export() {
        let mut suite = BenchmarkSuite::new();
        suite.add(BenchmarkResult::new("test1", Duration::from_millis(100)).with_file_size(50000));
        suite.add(BenchmarkResult::new("test2", Duration::from_millis(200)).with_file_size(100000));

        let json = suite.to_json();
        assert!(json.contains("\"name\": \"test1\""));
        assert!(json.contains("\"duration_ms\""));
        assert!(json.contains("\"file_size_bytes\": 50000"));
    }

    #[test]
    fn test_benchmark_context() {
        let ctx = BenchmarkContext::start("test_op");
        std::thread::sleep(Duration::from_millis(10));
        let result = ctx.finish();

        assert!(result.duration_ms() >= 10.0);
        assert_eq!(result.name, "test_op");
    }
}
