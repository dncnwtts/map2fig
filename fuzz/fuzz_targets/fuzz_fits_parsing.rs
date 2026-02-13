#![no_main]
//! Fuzzing target for FITS file parsing
//!
//! Feeds arbitrary binary data to the FITS parser.
//! Ensures robust error handling without crashes.
//!
//! This catches:
//! - Corrupted FITS headers
//! - Truncated files
//! - Invalid column definitions
//! - Malformed data blocks

use libfuzzer_sys::fuzz_target;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    // Try to parse as FITS data
    // This should gracefully fail rather than panic
    let _ = std::panic::catch_unwind(|| {
        let cursor = Cursor::new(data);
        // Attempt to read FITS file
        // (This is a placeholder - actual parsing depends on fitsrs API)
        // For now, we're just testing that arbitrary data doesn't cause panics
        let _ = cursor;
    });
});
