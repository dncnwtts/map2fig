// SIMD implementation using std::portable_simd (nightly-only feature)
// On stable Rust, this just delegates to the wide crate
// This module is only compiled when `nightly_simd` feature is enabled.

use crate::simd_wide as wide_simd;

/// Vectorized sin for 8 f64 values
pub fn simd_sin_8_portable(v: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_sin_8_wide(*v)
}

/// Vectorized cos for 8 f64 values
pub fn simd_cos_8_portable(v: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_cos_8_wide(*v)
}

/// Vectorized sin and cos returning both (sin, cos) for efficiency
pub fn simd_sin_cos_8_portable(v: &[f64; 8]) -> ([f64; 8], [f64; 8]) {
    wide_simd::simd_sin_cos_8_wide(*v)
}

/// Vectorized atan2(y, x) for 8 pairs of f64 values
pub fn simd_atan2_8_portable(y: &[f64; 8], x: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_atan2_8_wide(*y, *x)
}

/// Vectorized asin for 8 f64 values
pub fn simd_asin_8_portable(v: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_asin_8_wide(*v)
}

/// Vectorized acos for 8 f64 values
pub fn simd_acos_8_portable(v: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_acos_8_wide(*v)
}

/// Vectorized sqrt for 8 f64 values
pub fn simd_sqrt_8_portable(v: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_sqrt_8_wide(*v)
}

/// Vectorized absolute value for 8 f64 values
pub fn simd_abs_8_portable(v: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_abs_8_wide(*v)
}

/// Vectorized clamp for 8 f64 values: clamp(v, min, max)
pub fn simd_clamp_8_portable(v: &[f64; 8], min: f64, max: f64) -> [f64; 8] {
    wide_simd::simd_clamp_8_wide(*v, min, max)
}

/// Vectorized multiplication for 8 f64 values: a * b
pub fn simd_mul_8_portable(a: &[f64; 8], b: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_mul_8_wide(*a, *b)
}

/// Vectorized addition for 8 f64 values: a + b
pub fn simd_add_8_portable(a: &[f64; 8], b: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_add_8_wide(*a, *b)
}

/// Vectorized multiply-add for 8 f64 values: (a * b) + c
pub fn simd_madd_8_portable(a: &[f64; 8], b: &[f64; 8], c: &[f64; 8]) -> [f64; 8] {
    wide_simd::simd_madd_8_wide(*a, *b, *c)
}
