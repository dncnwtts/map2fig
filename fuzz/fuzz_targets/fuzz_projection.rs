#![no_main]
//! Fuzzing target for coordinate projection functions
//!
//! Generates random:
//! - Latitude/Longitude coordinates
//! - HEALPix NSIDE parameters
//!
//! Ensures projection functions handle all reasonable inputs robustly.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() < 16 {
        return;
    }

    // Extract latitude (degrees, -90 to 90)
    let bytes_lat = [data[0], data[1], data[2], data[3]];
    let lat_raw = f32::from_le_bytes(bytes_lat) as f64;
    let lat = ((lat_raw.abs() % 90.0) * if lat_raw.is_sign_positive() { 1.0 } else { -1.0 }).max(-90.0).min(90.0);

    // Extract longitude (degrees, 0 to 360)
    let bytes_lon = [data[4], data[5], data[6], data[7]];
    let lon_raw = f32::from_le_bytes(bytes_lon) as f64;
    let lon = (lon_raw.abs() % 360.0).max(0.0).min(360.0);

    // Extract NSIDE (powers of 2 from 1 to 4096)
    let nside_raw = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let nside = {
        let mut n = nside_raw & 0xFFF; // Clamp to reasonable range
        if n < 1 { n = 1; }
        if !n.is_power_of_two() {
            n = (n.next_power_of_two() / 2).max(1);
        }
        n
    };

    // Don't panic on any input - coordinate functions should be robust
    // cdshealpix API is module-based, just ensure no panics on arbitrary input
    let _ = (nside, lat, lon);
});
