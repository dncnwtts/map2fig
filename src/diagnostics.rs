//! Performance diagnostics and profiling module.
//!
//! Provides instrumentation for measuring:
//! - Cache hit/miss rates
//! - I/O time vs. computation time
//! - Bottleneck identification

use std::time::{Duration, Instant};

/// Global profiling state - tracks cache hits/misses and timing
pub struct ProfileStats {
    /// Number of successful cache lookups
    pub cache_hits: usize,
    /// Number of cache misses (required FITS parsing)
    pub cache_misses: usize,
    /// Total time spent in FITS parsing
    pub fits_parse_time: Duration,
    /// Total time spent in column extraction
    pub column_extract_time: Duration,
    /// Number of FITS files processed
    pub files_processed: usize,
}

impl Default for ProfileStats {
    fn default() -> Self {
        Self {
            cache_hits: 0,
            cache_misses: 0,
            fits_parse_time: Duration::ZERO,
            column_extract_time: Duration::ZERO,
            files_processed: 0,
        }
    }
}

impl ProfileStats {
    pub fn print_summary(&self) {
        let total_cache_ops = self.cache_hits + self.cache_misses;
        let hit_rate = if total_cache_ops > 0 {
            100.0 * self.cache_hits as f64 / total_cache_ops as f64
        } else {
            0.0
        };

        eprintln!("\n=== I/O Profiling Summary ===");
        eprintln!("Files processed:      {}", self.files_processed);
        eprintln!("Cache operations:     {} ({} hits, {} misses)", 
                  total_cache_ops, 
                  self.cache_hits, 
                  self.cache_misses);
        eprintln!("Cache hit rate:       {:.1}%", hit_rate);
        eprintln!("FITS parse time:      {:.3}s ({:.1}% of total)",
                  self.fits_parse_time.as_secs_f64(),
                  100.0 * self.fits_parse_time.as_secs_f64() / 
                      (self.fits_parse_time.as_secs_f64() + self.column_extract_time.as_secs_f64()));
        eprintln!("Column extract time:  {:.3}s", self.column_extract_time.as_secs_f64());
        eprintln!("Total I/O time:       {:.3}s",
                  (self.fits_parse_time + self.column_extract_time).as_secs_f64());
    }
}

/// Thread-local profiling state
pub struct ScopedTimer {
    start: Instant,
    recorded_time: Option<Duration>,
}

impl ScopedTimer {
    /// Create a new timer starting now
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
            recorded_time: None,
        }
    }

    /// Stop the timer and return the elapsed duration
    pub fn elapsed(&mut self) -> Duration {
        let elapsed = self.start.elapsed();
        self.recorded_time = Some(elapsed);
        elapsed
    }

    /// Get the recorded time without stopping
    pub fn get_elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Default for ScopedTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ScopedTimer {
    fn drop(&mut self) {
        if let Some(elapsed) = self.recorded_time {
            // Optional: log on drop if desired
            let _ = elapsed;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer() {
        let mut timer = ScopedTimer::new();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_profile_stats() {
        let mut stats = ProfileStats::default();
        stats.cache_hits = 5;
        stats.cache_misses = 1;
        stats.fits_parse_time = Duration::from_millis(100);
        stats.column_extract_time = Duration::from_millis(200);
        
        let total = stats.cache_hits + stats.cache_misses;
        assert_eq!(total, 6);
        
        // Hit rate should be 5/6 ≈ 83.3%
        let hit_rate = 100.0 * stats.cache_hits as f64 / total as f64;
        assert!(hit_rate > 83.0 && hit_rate < 84.0);
    }
}
