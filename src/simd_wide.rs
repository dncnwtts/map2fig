//! True vector SIMD using the `wide` crate for f64x2 operations
//!
//! This module provides vectorized implementations of mathematical operations using
//! the `wide` crate which provides portable SIMD types that compile on stable Rust.
//!
//! Strategy: Process f64 values in pairs using f64x2 vectors. 8 f64 values = 4 f64x2 vectors.
//! This gives actual hardware vectorization while maintaining portability.

use wide::f64x2;

/// Vectorized sine for 8 f64 values using f64x2
#[inline]
pub fn simd_sin_8_wide(angles: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([angles[0], angles[1]]).sin();
    let v1 = f64x2::new([angles[2], angles[3]]).sin();
    let v2 = f64x2::new([angles[4], angles[5]]).sin();
    let v3 = f64x2::new([angles[6], angles[7]]).sin();

    let a = v0.to_array();
    let b = v1.to_array();
    let c = v2.to_array();
    let d = v3.to_array();
    [a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]]
}

/// Vectorized cosine for 8 f64 values using f64x2
#[inline]
pub fn simd_cos_8_wide(angles: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([angles[0], angles[1]]).cos();
    let v1 = f64x2::new([angles[2], angles[3]]).cos();
    let v2 = f64x2::new([angles[4], angles[5]]).cos();
    let v3 = f64x2::new([angles[6], angles[7]]).cos();

    let a = v0.to_array();
    let b = v1.to_array();
    let c = v2.to_array();
    let d = v3.to_array();
    [a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]]
}

/// Vectorized sin and cos simultaneously using f64x2
#[inline]
pub fn simd_sin_cos_8_wide(angles: [f64; 8]) -> ([f64; 8], [f64; 8]) {
    let (s0, c0) = f64x2::new([angles[0], angles[1]]).sin_cos();
    let (s1, c1) = f64x2::new([angles[2], angles[3]]).sin_cos();
    let (s2, c2) = f64x2::new([angles[4], angles[5]]).sin_cos();
    let (s3, c3) = f64x2::new([angles[6], angles[7]]).sin_cos();

    let sa = s0.to_array();
    let sb = s1.to_array();
    let sc = s2.to_array();
    let sd = s3.to_array();
    let ca = c0.to_array();
    let cb = c1.to_array();
    let cc = c2.to_array();
    let cd = c3.to_array();

    (
        [sa[0], sa[1], sb[0], sb[1], sc[0], sc[1], sd[0], sd[1]],
        [ca[0], ca[1], cb[0], cb[1], cc[0], cc[1], cd[0], cd[1]],
    )
}

/// Vectorized inverse sine using f64x2
#[inline]
pub fn simd_asin_8_wide(x: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([x[0], x[1]]).asin();
    let v1 = f64x2::new([x[2], x[3]]).asin();
    let v2 = f64x2::new([x[4], x[5]]).asin();
    let v3 = f64x2::new([x[6], x[7]]).asin();

    let a = v0.to_array();
    let b = v1.to_array();
    let c = v2.to_array();
    let d = v3.to_array();
    [a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]]
}

/// Vectorized inverse cosine using f64x2
#[inline]
pub fn simd_acos_8_wide(x: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([x[0], x[1]]).acos();
    let v1 = f64x2::new([x[2], x[3]]).acos();
    let v2 = f64x2::new([x[4], x[5]]).acos();
    let v3 = f64x2::new([x[6], x[7]]).acos();

    let a = v0.to_array();
    let b = v1.to_array();
    let c = v2.to_array();
    let d = v3.to_array();
    [a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]]
}

/// Vectorized inverse tangent (atan2) using f64x2
#[inline]
pub fn simd_atan2_8_wide(y: [f64; 8], x: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([y[0], y[1]]).atan2(f64x2::new([x[0], x[1]]));
    let v1 = f64x2::new([y[2], y[3]]).atan2(f64x2::new([x[2], x[3]]));
    let v2 = f64x2::new([y[4], y[5]]).atan2(f64x2::new([x[4], x[5]]));
    let v3 = f64x2::new([y[6], y[7]]).atan2(f64x2::new([x[6], x[7]]));

    let a = v0.to_array();
    let b = v1.to_array();
    let c = v2.to_array();
    let d = v3.to_array();
    [a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]]
}

/// Vectorized square root using f64x2
#[inline]
pub fn simd_sqrt_8_wide(x: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([x[0], x[1]]).sqrt();
    let v1 = f64x2::new([x[2], x[3]]).sqrt();
    let v2 = f64x2::new([x[4], x[5]]).sqrt();
    let v3 = f64x2::new([x[6], x[7]]).sqrt();

    let a = v0.to_array();
    let b = v1.to_array();
    let c = v2.to_array();
    let d = v3.to_array();
    [a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]]
}

/// Vectorized reciprocal using f64x2
#[inline]
pub fn simd_recip_8_wide(x: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([1.0 / x[0], 1.0 / x[1]]);
    let v1 = f64x2::new([1.0 / x[2], 1.0 / x[3]]);
    let v2 = f64x2::new([1.0 / x[4], 1.0 / x[5]]);
    let v3 = f64x2::new([1.0 / x[6], 1.0 / x[7]]);

    let a = v0.to_array();
    let b = v1.to_array();
    let c = v2.to_array();
    let d = v3.to_array();
    [a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]]
}

/// Vectorized absolute value using f64x2
#[inline]
pub fn simd_abs_8_wide(x: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([x[0], x[1]]).abs();
    let v1 = f64x2::new([x[2], x[3]]).abs();
    let v2 = f64x2::new([x[4], x[5]]).abs();
    let v3 = f64x2::new([x[6], x[7]]).abs();

    let a = v0.to_array();
    let b = v1.to_array();
    let c = v2.to_array();
    let d = v3.to_array();
    [a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]]
}

/// Vectorized clamp operation using f64x2
#[inline]
pub fn simd_clamp_8_wide(x: [f64; 8], min: f64, max: f64) -> [f64; 8] {
    let min_v = f64x2::new([min, min]);
    let max_v = f64x2::new([max, max]);

    let v0 = f64x2::new([x[0], x[1]]).max(min_v).min(max_v);
    let v1 = f64x2::new([x[2], x[3]]).max(min_v).min(max_v);
    let v2 = f64x2::new([x[4], x[5]]).max(min_v).min(max_v);
    let v3 = f64x2::new([x[6], x[7]]).max(min_v).min(max_v);

    let a = v0.to_array();
    let b = v1.to_array();
    let c = v2.to_array();
    let d = v3.to_array();
    [a[0], a[1], b[0], b[1], c[0], c[1], d[0], d[1]]
}

/// Vectorized element-wise multiplication using f64x2
#[inline]
pub fn simd_mul_8_wide(a: [f64; 8], b: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([a[0], a[1]]) * f64x2::new([b[0], b[1]]);
    let v1 = f64x2::new([a[2], a[3]]) * f64x2::new([b[2], b[3]]);
    let v2 = f64x2::new([a[4], a[5]]) * f64x2::new([b[4], b[5]]);
    let v3 = f64x2::new([a[6], a[7]]) * f64x2::new([b[6], b[7]]);

    let r0 = v0.to_array();
    let r1 = v1.to_array();
    let r2 = v2.to_array();
    let r3 = v3.to_array();
    [r0[0], r0[1], r1[0], r1[1], r2[0], r2[1], r3[0], r3[1]]
}

/// Vectorized element-wise addition using f64x2
#[inline]
pub fn simd_add_8_wide(a: [f64; 8], b: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([a[0], a[1]]) + f64x2::new([b[0], b[1]]);
    let v1 = f64x2::new([a[2], a[3]]) + f64x2::new([b[2], b[3]]);
    let v2 = f64x2::new([a[4], a[5]]) + f64x2::new([b[4], b[5]]);
    let v3 = f64x2::new([a[6], a[7]]) + f64x2::new([b[6], b[7]]);

    let r0 = v0.to_array();
    let r1 = v1.to_array();
    let r2 = v2.to_array();
    let r3 = v3.to_array();
    [r0[0], r0[1], r1[0], r1[1], r2[0], r2[1], r3[0], r3[1]]
}

/// Vectorized fused multiply-add using f64x2
#[inline]
pub fn simd_madd_8_wide(a: [f64; 8], b: [f64; 8], c: [f64; 8]) -> [f64; 8] {
    let v0 = f64x2::new([a[0], a[1]]).mul_add(f64x2::new([b[0], b[1]]), f64x2::new([c[0], c[1]]));
    let v1 = f64x2::new([a[2], a[3]]).mul_add(f64x2::new([b[2], b[3]]), f64x2::new([c[2], c[3]]));
    let v2 = f64x2::new([a[4], a[5]]).mul_add(f64x2::new([b[4], b[5]]), f64x2::new([c[4], c[5]]));
    let v3 = f64x2::new([a[6], a[7]]).mul_add(f64x2::new([b[6], b[7]]), f64x2::new([c[6], c[7]]));

    let r0 = v0.to_array();
    let r1 = v1.to_array();
    let r2 = v2.to_array();
    let r3 = v3.to_array();
    [r0[0], r0[1], r1[0], r1[1], r2[0], r2[1], r3[0], r3[1]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-14;

    #[test]
    fn test_simd_sin_8_wide() {
        let angles = [
            0.0,
            PI / 6.0,
            PI / 4.0,
            PI / 3.0,
            PI / 2.0,
            PI,
            -PI / 6.0,
            -PI / 2.0,
        ];
        let result = simd_sin_8_wide(angles);

        for i in 0..8 {
            let expected = angles[i].sin();
            assert!(
                (result[i] - expected).abs() < EPSILON,
                "sin mismatch at index {}: {} vs {}",
                i,
                result[i],
                expected
            );
        }
    }

    #[test]
    fn test_simd_cos_8_wide() {
        let angles = [
            0.0,
            PI / 6.0,
            PI / 4.0,
            PI / 3.0,
            PI / 2.0,
            PI,
            -PI / 6.0,
            -PI / 2.0,
        ];
        let result = simd_cos_8_wide(angles);

        for i in 0..8 {
            let expected = angles[i].cos();
            assert!(
                (result[i] - expected).abs() < EPSILON,
                "cos mismatch at index {}: {} vs {}",
                i,
                result[i],
                expected
            );
        }
    }

    #[test]
    fn test_simd_sin_cos_8_wide() {
        let angles = [
            0.0,
            PI / 6.0,
            PI / 4.0,
            PI / 3.0,
            PI / 2.0,
            PI,
            -PI / 6.0,
            -PI / 2.0,
        ];
        let (sines, cosines) = simd_sin_cos_8_wide(angles);

        for i in 0..8 {
            let (expected_sin, expected_cos) = angles[i].sin_cos();
            assert!(
                (sines[i] - expected_sin).abs() < EPSILON,
                "sin mismatch at index {}",
                i
            );
            assert!(
                (cosines[i] - expected_cos).abs() < EPSILON,
                "cos mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_simd_atan2_8_wide() {
        let y = [1.0, 1.0, 0.0, -1.0, -1.0, -1.0, 0.0, 1.0];
        let x = [1.0, 0.0, 1.0, 1.0, 0.0, -1.0, -1.0, -1.0];
        let result = simd_atan2_8_wide(y, x);

        for i in 0..8 {
            let expected = y[i].atan2(x[i]);
            assert!(
                (result[i] - expected).abs() < EPSILON,
                "atan2 mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_simd_mul_8_wide() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = [2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let result = simd_mul_8_wide(a, b);

        for i in 0..8 {
            let expected = a[i] * b[i];
            assert!(
                (result[i] - expected).abs() < EPSILON,
                "mul mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_simd_add_8_wide() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let result = simd_add_8_wide(a, b);

        for i in 0..8 {
            let expected = a[i] + b[i];
            assert!(
                (result[i] - expected).abs() < EPSILON,
                "add mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_simd_madd_8_wide() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let b = [2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0];
        let c = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let result = simd_madd_8_wide(a, b, c);

        for i in 0..8 {
            let expected = a[i].mul_add(b[i], c[i]);
            assert!(
                (result[i] - expected).abs() < EPSILON,
                "madd mismatch at index {}",
                i
            );
        }
    }
}
