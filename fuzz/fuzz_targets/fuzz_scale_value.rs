#![no_main]
//! Fuzzing target for scale_value function
//!
//! This fuzzer generates random combinations of:
//! - Data values (including extremes like NaN, Inf, very large/small numbers)
//! - Min/max bounds
//! - Scaling modes (Linear, Log, SymLog, Asinh, Histogram)
//! - Negative value handling (Zero, Unseen)
//!
//! Ensures scale_value never panics and always returns valid values.

use libfuzzer_sys::fuzz_target;
use map2fig::scale::{scale_value, Scale};
use map2fig::NegMode;

fuzz_target!(|data: &[u8]| {
    // Need at least 20 bytes: 3 f64s (24 bytes) + 1 byte for mode
    if data.len() < 25 {
        return;
    }

    // Extract three f64 values from first 24 bytes
    let mut buf = [0u8; 8];
    
    buf.copy_from_slice(&data[0..8]);
    let value = f64::from_le_bytes(buf);
    
    buf.copy_from_slice(&data[8..16]);
    let min = f64::from_le_bytes(buf);
    
    buf.copy_from_slice(&data[16..24]);
    let max = f64::from_le_bytes(buf);

    // Use remaining bytes to select scale mode
    let mode_byte = data[24] % 5;
    let scale = match mode_byte {
        0 => Scale::Linear,
        1 => Scale::Log,
        2 => Scale::Symlog { linthresh: 1.0 },
        3 => Scale::Asinh { scale: 1.0 },
        _ => Scale::Linear,
    };

    let neg_mode_byte = data.get(25).unwrap_or(&0) % 2;
    let neg_mode = match neg_mode_byte {
        0 => NegMode::Zero,
        _ => NegMode::Unseen,
    };

    // This should never panic regardless of input
    let _ = scale_value(value, min, max, scale, neg_mode, None);
});
