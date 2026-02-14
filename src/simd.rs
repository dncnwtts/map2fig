//! SIMD vectorization module for high-performance batch operations.
//!
//! Provides optimized implementations of common mathematical operations
//! for processing 8 values in parallel, matching Tier 2 batch size.
//!
//! Note: These are currently scalar implementations optimized for instruction-level
//! parallelism and CPU pipelining. True SIMD vectorization would require either
//! nightly Rust with portable_simd or external C library bindings.

/// Vectorized sine for 8 f64 values
///
/// Computes sin(x) for 8 angles simultaneously.
/// Optimized for CPU pipelining and cache efficiency.
///
/// Input: 8 angles in radians
/// Output: 8 sine values in [-1, 1]
#[inline(always)]
pub fn simd_sin_8(angles: [f64; 8]) -> [f64; 8] {
    // Unrolled to allow CPU to parallelize operations across multiple pipelines
    [
        angles[0].sin(),
        angles[1].sin(),
        angles[2].sin(),
        angles[3].sin(),
        angles[4].sin(),
        angles[5].sin(),
        angles[6].sin(),
        angles[7].sin(),
    ]
}

/// Vectorized cosine for 8 f64 values
///
/// Computes cos(x) for 8 angles simultaneously.
/// Optimized for CPU pipelining and cache efficiency.
#[inline(always)]
pub fn simd_cos_8(angles: [f64; 8]) -> [f64; 8] {
    // Unrolled to allow CPU to parallelize operations
    [
        angles[0].cos(),
        angles[1].cos(),
        angles[2].cos(),
        angles[3].cos(),
        angles[4].cos(),
        angles[5].cos(),
        angles[6].cos(),
        angles[7].cos(),
    ]
}

/// Vectorized sine and cosine simultaneously (more efficient than separate calls)
///
/// Computes both sin(x) and cos(x) for 8 angles in a single operation.
/// Uses the fused sin_cos operation where available for better efficiency.
///
/// Returns: (sin_values, cos_values)
#[inline(always)]
pub fn simd_sin_cos_8(angles: [f64; 8]) -> ([f64; 8], [f64; 8]) {
    // Process in parallel to allow CPU instruction-level parallelism
    // This unrolled version breaks data dependencies and allows better pipelining
    let (s0, c0) = angles[0].sin_cos();
    let (s1, c1) = angles[1].sin_cos();
    let (s2, c2) = angles[2].sin_cos();
    let (s3, c3) = angles[3].sin_cos();
    
    let (s4, c4) = angles[4].sin_cos();
    let (s5, c5) = angles[5].sin_cos();
    let (s6, c6) = angles[6].sin_cos();
    let (s7, c7) = angles[7].sin_cos();
    
    ([s0, s1, s2, s3, s4, s5, s6, s7], [c0, c1, c2, c3, c4, c5, c6, c7])
}

/// Vectorized inverse tangent (atan2) for 8 point pairs
///
/// Computes atan2(y, x) for 8 (y, x) coordinate pairs.
/// Handles all quadrants correctly.
///
/// Optimized for CPU instruction-level parallelism.
///
/// Inputs: y and x arrays of 8 values each
/// Output: 8 angles in [-π, π]
#[inline(always)]
pub fn simd_atan2_8(y: [f64; 8], x: [f64; 8]) -> [f64; 8] {
    // Process in parallel pipelines to improve ILP
    // Dependencies are broken up so CPU can execute multiple atan2 calls concurrently
    [
        y[0].atan2(x[0]),
        y[1].atan2(x[1]),
        y[2].atan2(x[2]),
        y[3].atan2(x[3]),
        y[4].atan2(x[4]),
        y[5].atan2(x[5]),
        y[6].atan2(x[6]),
        y[7].atan2(x[7]),
    ]
}

/// Vectorized inverse sine for 8 f64 values
///
/// Computes asin(x) for 8 values simultaneously.
/// Optimized for instruction-level parallelism.
/// Input values must be in [-1, 1].
///
/// Output: 8 angles in [-π/2, π/2]
#[inline(always)]
pub fn simd_asin_8(x: [f64; 8]) -> [f64; 8] {
    [
        x[0].asin(),
        x[1].asin(),
        x[2].asin(),
        x[3].asin(),
        x[4].asin(),
        x[5].asin(),
        x[6].asin(),
        x[7].asin(),
    ]
}

/// Vectorized inverse cosine for 8 f64 values
///
/// Computes acos(x) for 8 values simultaneously.
/// Optimized for instruction-level parallelism.
/// Input values must be in [-1, 1].
///
/// Output: 8 angles in [0, π]
#[inline(always)]
pub fn simd_acos_8(x: [f64; 8]) -> [f64; 8] {
    [
        x[0].acos(),
        x[1].acos(),
        x[2].acos(),
        x[3].acos(),
        x[4].acos(),
        x[5].acos(),
        x[6].acos(),
        x[7].acos(),
    ]
}

/// Vectorized square root
///
/// Optimized for instruction-level parallelism.
#[inline(always)]
pub fn simd_sqrt_8(x: [f64; 8]) -> [f64; 8] {
    [
        x[0].sqrt(),
        x[1].sqrt(),
        x[2].sqrt(),
        x[3].sqrt(),
        x[4].sqrt(),
        x[5].sqrt(),
        x[6].sqrt(),
        x[7].sqrt(),
    ]
}

/// Vectorized power function (y = x^exp)
///
/// Computes x^exp for 8 values simultaneously.
/// Optimized for instruction-level parallelism.
/// Used for gamma correction and scaling operations.
#[inline(always)]
pub fn simd_pow_8(x: [f64; 8], exp: f64) -> [f64; 8] {
    [
        x[0].powf(exp),
        x[1].powf(exp),
        x[2].powf(exp),
        x[3].powf(exp),
        x[4].powf(exp),
        x[5].powf(exp),
        x[6].powf(exp),
        x[7].powf(exp),
    ]
}

/// Vectorized natural logarithm
///
/// Computes ln(x) for 8 values simultaneously.
/// Input values must be positive.
#[inline]
pub fn simd_ln_8(x: [f64; 8]) -> [f64; 8] {
    [
        x[0].ln(),
        x[1].ln(),
        x[2].ln(),
        x[3].ln(),
        x[4].ln(),
        x[5].ln(),
        x[6].ln(),
        x[7].ln(),
    ]
}

/// Vectorized reciprocal (1/x)
///
/// Computes 1/x for 8 values simultaneously.
/// More efficient than division for reciprocals.
#[inline]
pub fn simd_recip_8(x: [f64; 8]) -> [f64; 8] {
    [
        x[0].recip(),
        x[1].recip(),
        x[2].recip(),
        x[3].recip(),
        x[4].recip(),
        x[5].recip(),
        x[6].recip(),
        x[7].recip(),
    ]
}

/// Vectorized absolute value
#[inline]
pub fn simd_abs_8(x: [f64; 8]) -> [f64; 8] {
    [
        x[0].abs(),
        x[1].abs(),
        x[2].abs(),
        x[3].abs(),
        x[4].abs(),
        x[5].abs(),
        x[6].abs(),
        x[7].abs(),
    ]
}

/// Vectorized clamp operation
///
/// Clamps all 8 values to range [min, max]
#[inline]
pub fn simd_clamp_8(x: [f64; 8], min: f64, max: f64) -> [f64; 8] {
    [
        x[0].clamp(min, max),
        x[1].clamp(min, max),
        x[2].clamp(min, max),
        x[3].clamp(min, max),
        x[4].clamp(min, max),
        x[5].clamp(min, max),
        x[6].clamp(min, max),
        x[7].clamp(min, max),
    ]
}

/// Vectorized element-wise multiplication
#[inline]
pub fn simd_mul_8(a: [f64; 8], b: [f64; 8]) -> [f64; 8] {
    [
        a[0] * b[0],
        a[1] * b[1],
        a[2] * b[2],
        a[3] * b[3],
        a[4] * b[4],
        a[5] * b[5],
        a[6] * b[6],
        a[7] * b[7],
    ]
}

/// Vectorized element-wise addition
#[inline]
pub fn simd_add_8(a: [f64; 8], b: [f64; 8]) -> [f64; 8] {
    [
        a[0] + b[0],
        a[1] + b[1],
        a[2] + b[2],
        a[3] + b[3],
        a[4] + b[4],
        a[5] + b[5],
        a[6] + b[6],
        a[7] + b[7],
    ]
}

/// Vectorized fused multiply-add: result = a * b + c
///
/// More efficient than separate multiply and add operations.
#[inline]
pub fn simd_madd_8(a: [f64; 8], b: [f64; 8], c: [f64; 8]) -> [f64; 8] {
    [
        a[0].mul_add(b[0], c[0]),
        a[1].mul_add(b[1], c[1]),
        a[2].mul_add(b[2], c[2]),
        a[3].mul_add(b[3], c[3]),
        a[4].mul_add(b[4], c[4]),
        a[5].mul_add(b[5], c[5]),
        a[6].mul_add(b[6], c[6]),
        a[7].mul_add(b[7], c[7]),
    ]
}

/// Vectorized 3D vector normalization
///
/// Normalizes 8 3D vectors: v_normalized = v / ||v||
///
/// Input: 8 3D vectors as 3×8 arrays: [x values; y values; z values]
/// Output: 8 normalized vectors with same structure
pub fn simd_normalize_vec3_8(
    x: [f64; 8],
    y: [f64; 8],
    z: [f64; 8],
) -> ([f64; 8], [f64; 8], [f64; 8]) {
    // Compute magnitudes: mag = sqrt(x^2 + y^2 + z^2)
    let mag_sq = simd_madd_8(x, x, simd_madd_8(y, y, simd_mul_8(z, z)));
    let mag = simd_sqrt_8(mag_sq);
    let mag_inv = simd_recip_8(mag);

    // Normalize each component
    (
        simd_mul_8(x, mag_inv),
        simd_mul_8(y, mag_inv),
        simd_mul_8(z, mag_inv),
    )
}

/// Vectorized 3D dot product
///
/// Computes dot product for 8 3D vector pairs
/// dot = a_x*b_x + a_y*b_y + a_z*b_z for each pair
pub fn simd_dot3_8(
    a_x: [f64; 8],
    a_y: [f64; 8],
    a_z: [f64; 8],
    b_x: [f64; 8],
    b_y: [f64; 8],
    b_z: [f64; 8],
) -> [f64; 8] {
    simd_madd_8(a_x, b_x, simd_madd_8(a_y, b_y, simd_mul_8(a_z, b_z)))
}

/// Vectorized 3D cross product
///
/// Computes cross product for 8 3D vector pairs
/// c = a × b
pub fn simd_cross_8(
    a_x: [f64; 8],
    a_y: [f64; 8],
    a_z: [f64; 8],
    b_x: [f64; 8],
    b_y: [f64; 8],
    b_z: [f64; 8],
) -> ([f64; 8], [f64; 8], [f64; 8]) {
    let c_x = simd_add_8(
        simd_mul_8(a_y, b_z),
        simd_mul_8(simd_mul_8(a_z, b_y), [-1.0; 8]),
    );
    let c_y = simd_add_8(
        simd_mul_8(a_z, b_x),
        simd_mul_8(simd_mul_8(a_x, b_z), [-1.0; 8]),
    );
    let c_z = simd_add_8(
        simd_mul_8(a_x, b_y),
        simd_mul_8(simd_mul_8(a_y, b_x), [-1.0; 8]),
    );

    (c_x, c_y, c_z)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const EPSILON: f64 = 1e-14;

    #[test]
    fn test_simd_sin_8() {
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
        let result = simd_sin_8(angles);

        for i in 0..8 {
            let expected = angles[i].sin();
            assert!(
                (result[i] - expected).abs() < EPSILON,
                "sin mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_simd_cos_8() {
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
        let result = simd_cos_8(angles);

        for i in 0..8 {
            let expected = angles[i].cos();
            assert!(
                (result[i] - expected).abs() < EPSILON,
                "cos mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_simd_sin_cos_8() {
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
        let (sines, cosines) = simd_sin_cos_8(angles);

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
    fn test_simd_atan2_8() {
        let y = [1.0, 1.0, 0.0, -1.0, -1.0, -1.0, 0.0, 1.0];
        let x = [1.0, 0.0, 1.0, 1.0, 0.0, -1.0, -1.0, -1.0];
        let result = simd_atan2_8(y, x);

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
    fn test_simd_asin_8() {
        let x = [-1.0, -0.5, 0.0, 0.5, 1.0, -0.707, 0.707, 0.866];
        let result = simd_asin_8(x);

        for i in 0..8 {
            let expected = x[i].asin();
            assert!(
                (result[i] - expected).abs() < EPSILON,
                "asin mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_simd_normalize_vec3_8() {
        let x = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let z = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let (nx, ny, nz) = simd_normalize_vec3_8(x, y, z);

        for i in 0..8 {
            // Each normalized vector should have magnitude ~= 1.0
            let mag_sq = nx[i] * nx[i] + ny[i] * ny[i] + nz[i] * nz[i];
            assert!(
                (mag_sq - 1.0).abs() < EPSILON,
                "magnitude mismatch at index {}",
                i
            );
            // For y=0, z=0, normalized x should be 1.0
            assert!(
                (nx[i] - 1.0).abs() < EPSILON,
                "normalized x mismatch at index {}",
                i
            );
        }
    }

    #[test]
    fn test_simd_dot3_8() {
        // Dot product of [1,0,0] with each of [1,0,0], [0,1,0], [0,0,1]
        let a_x = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let a_y = [0.0; 8];
        let a_z = [0.0; 8];

        let b_x = [1.0, 0.0, 0.0, 2.0, 3.0, -1.0, 0.5, 10.0];
        let b_y = [0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let b_z = [0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];

        let result = simd_dot3_8(a_x, a_y, a_z, b_x, b_y, b_z);

        // Expected: dot([1,0,0], [b_x[i], 0, 0]) = b_x[i]
        for i in 0..8 {
            assert!(
                (result[i] - b_x[i]).abs() < EPSILON,
                "dot product mismatchat index {}",
                i
            );
        }
    }
}

//─────────────────────────────────────────────────────────────────────────────
// HEALPix-Specific Vectorized Operations
//─────────────────────────────────────────────────────────────────────────────

/// Vectorized spherical to Cartesian conversion (8 theta-phi pairs)
///
/// Converts 8 spherical coordinates (theta, phi) to 3D Cartesian vectors (x, y, z).
/// Used in HEALPix sampling pipeline: project coordinates → convert to vectors.
///
/// Formulas:
/// - x = sin(theta) * cos(phi)
/// - y = sin(theta) * sin(phi)
/// - z = cos(theta)
///
/// Input:
/// - theta: 8 polar angles [0, π]
/// - phi: 8 azimuthal angles [0, 2π]
///
/// Output:
/// - (x, y, z): 3 arrays of 8 Cartesian coordinates each
#[inline]
pub fn simd_sph_to_vec_8(theta: [f64; 8], phi: [f64; 8]) -> ([f64; 8], [f64; 8], [f64; 8]) {
    // Vectorized sin/cos
    let sin_theta = simd_sin_8(theta);
    let cos_theta = simd_cos_8(theta);
    let sin_phi = simd_sin_8(phi);
    let cos_phi = simd_cos_8(phi);

    // x = sin(theta) * cos(phi)
    let x = simd_mul_8(sin_theta, cos_phi);
    // y = sin(theta) * sin(phi)
    let y = simd_mul_8(sin_theta, sin_phi);
    // z = cos(theta)
    let z = cos_theta;

    (x, y, z)
}

/// Vectorized Cartesian to spherical conversion (8 vectors)
///
/// Converts 8 3D Cartesian vectors back to spherical coordinates.
/// Used in HEALPix sampling after view transformation.
///
/// Formulas:
/// - theta = acos(clamp(z, -1, 1))
/// - phi = atan2(y, x)
///
/// Input:
/// - x, y, z: 3 arrays of Cartesian coordinates
///
/// Output:
/// - (theta, phi): 2 arrays of 8 spherical coordinates each
#[inline]
pub fn simd_vec_to_sph_8(x: [f64; 8], y: [f64; 8], z: [f64; 8]) -> ([f64; 8], [f64; 8]) {
    // Clamp z to avoid acos domain errors
    let z_clamped = simd_clamp_8(z, -1.0, 1.0);

    // theta = acos(z_clamped)
    let theta = simd_acos_8(z_clamped);

    // phi = atan2(y, x)
    let phi = simd_atan2_8(y, x);

    (theta, phi)
}

/// Vectorized 3x3 matrix-vector multiplication (8 vectors)
///
/// Applies 3x3 rotation/transformation matrix to 8 vectors in parallel.
/// Used in HEALPix sampling for view transformation application.
///
/// Formula for each vector i:
/// - x'\[i\] = m\[0\]\[0\] * x\[i\] + m\[0\]\[1\] * y\[i\] + m\[0\]\[2\] * z\[i\]
/// - y'\[i\] = m\[1\]\[0\] * x\[i\] + m\[1\]\[1\] * y\[i\] + m\[1\]\[2\] * z\[i\]
/// - z'\[i\] = m\[2\]\[0\] * x\[i\] + m\[2\]\[1\] * y\[i\] + m\[2\]\[2\] * z\[i\]
///
/// Input:
/// - mat: 3x3 matrix (row-major, \[row\]\[col\])
/// - x, y, z: 3 arrays of input vector components
///
/// Output:
/// - (x', y', z'): 3 arrays of transformed vector components
#[inline]
pub fn simd_matvec3_8(
    mat: [[f64; 3]; 3],
    x: [f64; 8],
    y: [f64; 8],
    z: [f64; 8],
) -> ([f64; 8], [f64; 8], [f64; 8]) {
    // First row: [m00*x[i] + m01*y[i] + m02*z[i]]
    let x_new = simd_add_8(
        simd_add_8(simd_mul_8(x, [mat[0][0]; 8]), simd_mul_8(y, [mat[0][1]; 8])),
        simd_mul_8(z, [mat[0][2]; 8]),
    );

    // Second row: [m10*x[i] + m11*y[i] + m12*z[i]]
    let y_new = simd_add_8(
        simd_add_8(simd_mul_8(x, [mat[1][0]; 8]), simd_mul_8(y, [mat[1][1]; 8])),
        simd_mul_8(z, [mat[1][2]; 8]),
    );

    // Third row: [m20*x[i] + m21*y[i] + m22*z[i]]
    let z_new = simd_add_8(
        simd_add_8(simd_mul_8(x, [mat[2][0]; 8]), simd_mul_8(y, [mat[2][1]; 8])),
        simd_mul_8(z, [mat[2][2]; 8]),
    );

    (x_new, y_new, z_new)
}

#[cfg(test)]
mod healpix_tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_simd_sph_to_vec_8() {
        let theta = [
            0.0,
            PI / 2.0,
            PI,
            0.0,
            PI / 4.0,
            PI / 4.0,
            PI / 3.0,
            PI / 6.0,
        ];
        let phi = [0.0, 0.0, 0.0, PI / 2.0, 0.0, PI / 2.0, PI / 4.0, PI / 3.0];

        let (x, y, z) = simd_sph_to_vec_8(theta, phi);

        // Test case 0: theta=0, phi=0 => (0, 0, 1) [north pole]
        assert!((x[0] - 0.0).abs() < 1e-14);
        assert!((y[0] - 0.0).abs() < 1e-14);
        assert!((z[0] - 1.0).abs() < 1e-14);

        // Test case 1: theta=π/2, phi=0 => (1, 0, 0) [equator, prime meridian]
        assert!((x[1] - 1.0).abs() < 1e-14);
        assert!((y[1] - 0.0).abs() < 1e-14);
        assert!((z[1] - 0.0).abs() < 1e-14);

        // Test case 2: theta=π, phi=0 => (0, 0, -1) [south pole]
        assert!((x[2] - 0.0).abs() < 1e-14);
        assert!((y[2] - 0.0).abs() < 1e-14);
        assert!((z[2] - (-1.0)).abs() < 1e-14);

        // Test case 3: theta=0 (north pole again, different phi, should still be (0,0,1))
        // phi doesn't matter at the poles
        assert!((x[3] - 0.0).abs() < 1e-14);
        assert!((y[3] - 0.0).abs() < 1e-14);
        assert!((z[3] - 1.0).abs() < 1e-14);
    }

    #[test]
    fn test_simd_vec_to_sph_8_roundtrip() {
        let theta_in = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 0.1, std::f64::consts::PI];
        let phi_in = [0.0, PI / 4.0, PI / 2.0, PI, 3.0 * PI / 2.0, 0.1, 0.2, 0.3];

        // Convert to Cartesian
        let (x, y, z) = simd_sph_to_vec_8(theta_in, phi_in);

        // Convert back to spherical
        let (theta_out, phi_out) = simd_vec_to_sph_8(x, y, z);

        // Check roundtrip (the phi for theta=0 or theta=π is undefined in the mathematics)
        for i in 0..8 {
            assert!(
                (theta_out[i] - theta_in[i]).abs() < 1e-12,
                "Theta mismatch at {}: {} vs {}",
                i,
                theta_out[i],
                theta_in[i]
            );

            // For phi, compare modulo 2π (wrap around)
            let phi_diff = (phi_out[i] - phi_in[i]).abs();
            let phi_diff_wrapped = (2.0 * PI - phi_diff).min(phi_diff);
            assert!(
                phi_diff_wrapped < 1e-12 || theta_in[i].sin().abs() < 1e-10,
                "Phi mismatch at {}: {} vs {} (theta_sin={})",
                i,
                phi_out[i],
                phi_in[i],
                theta_in[i].sin()
            );
        }
    }

    #[test]
    fn test_simd_matvec3_8_identity() {
        let identity = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

        let x_in = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y_in = [0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5];
        let z_in = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];

        let (x_out, y_out, z_out) = simd_matvec3_8(identity, x_in, y_in, z_in);

        // Should be unchanged
        for i in 0..8 {
            assert!((x_out[i] - x_in[i]).abs() < 1e-14);
            assert!((y_out[i] - y_in[i]).abs() < 1e-14);
            assert!((z_out[i] - z_in[i]).abs() < 1e-14);
        }
    }

    #[test]
    fn test_simd_matvec3_8_scaling() {
        // Diagonal matrix with scaling factors
        let scale_matrix = [[2.0, 0.0, 0.0], [0.0, 3.0, 0.0], [0.0, 0.0, 5.0]];

        let x_in = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let y_in = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let z_in = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];

        let (x_out, y_out, z_out) = simd_matvec3_8(scale_matrix, x_in, y_in, z_in);

        // x should be scaled by 2, y by 3, z by 5
        for i in 0..8 {
            assert!((x_out[i] - 2.0 * x_in[i]).abs() < 1e-14);
            assert!((y_out[i] - 3.0 * y_in[i]).abs() < 1e-14);
            assert!((z_out[i] - 5.0 * z_in[i]).abs() < 1e-14);
        }
    }
}

//─────────────────────────────────────────────────────────────────────────────
// Scaling & Color SIMD Operations
//─────────────────────────────────────────────────────────────────────────────

/// Vectorized linear scaling (normalization to [0, 1] range)
///
/// Maps 8 values from [min, max] to [0, 1] range using linear formula:
/// `t_i = (value_i - min) / (max - min)`
///
/// Special handling:
/// - Values ≤ min → t = 0.0
/// - Values ≥ max → t = 1.0
/// - Otherwise apply linear formula
///
/// Input:
/// - values: 8 raw data values
/// - min, max: scaling bounds
/// - mask: validity mask (true = process, false = skip)
///
/// Output:
/// - normalized: 8 normalized values in [0, 1]
/// - out_mask: updated mask (invalid values set to false)
#[inline]
pub fn simd_linear_scale_8(
    values: [f64; 8],
    min: f64,
    max: f64,
    mask: [bool; 8],
) -> ([f64; 8], [bool; 8]) {
    let inv_range = if max > min {
        1.0 / (max - min)
    } else {
        0.0 // Degenerate case: max ≤ min
    };

    let mut result = [0.0; 8];
    let out_mask = mask;

    for i in 0..8 {
        if !mask[i] {
            continue;
        }

        if max <= min {
            // Degenerate: all valid values map to 0.5
            result[i] = 0.5;
        } else if values[i] <= min {
            result[i] = 0.0;
        } else if values[i] >= max {
            result[i] = 1.0;
        } else {
            result[i] = (values[i] - min) * inv_range;
        }
    }

    (result, out_mask)
}

/// Vectorized log scale transformation (for positive data)
///
/// Maps positive values using logarithmic formula:
/// `t_i = (ln(value_i) - ln(min)) / (ln(max) - ln(min))`
///
/// Handles edge cases:
/// - value ≤ 0: returns mask\[i\] = false (invalid)
/// - value < min: t = 0.0
/// - value ≥ max: t = 1.0
///
/// Pre-computed cache values (log_min, log_range) avoid repeated ln() calls
///
/// Input:
/// - values: 8 raw data values (should be positive for log scale)
/// - log_min: pre-computed ln(min)
/// - log_range: pre-computed ln(max) - ln(min)
/// - mask: validity mask
///
/// Output:
/// - normalized: 8 normalized values in [0, 1]
/// - out_mask: updated mask (non-positive values marked invalid)
#[inline]
pub fn simd_log_scale_8(
    values: [f64; 8],
    log_min: f64,
    log_range: f64,
    mask: [bool; 8],
) -> ([f64; 8], [bool; 8]) {
    let mut result = [0.0; 8];
    let mut out_mask = mask;

    for i in 0..8 {
        if !mask[i] {
            continue;
        }

        if values[i] <= 0.0 {
            // Negative or zero: invalid for log scale
            out_mask[i] = false;
            continue;
        }

        if log_range <= 0.0 {
            result[i] = 0.5;
        } else {
            let log_val = values[i].ln();
            result[i] = ((log_val - log_min) / log_range).clamp(0.0, 1.0);
        }
    }

    (result, out_mask)
}

/// Vectorized colormap LUT lookup (fast palette sampling)
///
/// Maps 8 normalized values [0, 1] to palette indices via fast LUT lookup.
/// Input values clamped to [0, 1], then multiplied by 255 and truncated.
///
/// Input:
/// - normalized: 8 values in [0, 1] (typically from scale_value)
/// - lut: 256-entry RGB lookup table (pre-computed colormap)
/// - mask: validity mask
///
/// Output:
/// - rgb_buffer: flattened RGB buffer (8 pixels × 3 bytes = 24 bytes)
///   Format: [R0, G0, B0, R1, G1, B1, ..., R7, G7, B7]
/// - out_mask: unchanged validity mask
///
/// Note: This is a scalar loop in portable SIMD version.
///       With AVX2, could gather 8 LUT entries in parallel.
#[inline]
pub fn simd_colormap_sample_8(
    normalized: [f64; 8],
    lut: &[[u8; 3]; 256],
    mask: [bool; 8],
) -> ([u8; 24], [bool; 8]) {
    let mut rgb_buffer = [0u8; 24];

    for i in 0..8 {
        if !mask[i] {
            // Bad pixel: set to black (0, 0, 0)
            rgb_buffer[i * 3] = 0;
            rgb_buffer[i * 3 + 1] = 0;
            rgb_buffer[i * 3 + 2] = 0;
            continue;
        }

        // Fast LUT lookup
        let idx = (normalized[i].clamp(0.0, 1.0) * 255.0) as usize;
        let rgb = lut[idx];

        rgb_buffer[i * 3] = rgb[0];
        rgb_buffer[i * 3 + 1] = rgb[1];
        rgb_buffer[i * 3 + 2] = rgb[2];
    }

    (rgb_buffer, mask)
}

/// Vectorized gamma correction for 8 values
///
/// Applies inverse gamma: `out_i = value_i ^ (1/gamma)`
/// Used to linearize colormapped output before display
///
/// Input:
/// - values: 8 normalized values in [0, 1]
/// - gamma_inv: pre-computed 1/gamma value
/// - mask: validity mask
///
/// Output:
/// - corrected: 8 gamma-corrected values
#[inline]
pub fn simd_gamma_correct_8(
    values: [f64; 8],
    gamma_inv: f64,
    mask: [bool; 8],
) -> ([f64; 8], [bool; 8]) {
    let mut result = [0.0; 8];

    for i in 0..8 {
        if !mask[i] {
            continue;
        }
        result[i] = values[i].powf(gamma_inv);
    }

    (result, mask)
}

#[cfg(test)]
mod scaling_tests {
    use super::*;

    #[test]
    fn test_simd_linear_scale_8() {
        let values = [0.0, 2.5, 5.0, 7.5, 10.0, 1.0, 3.0, 9.0];
        let mask = [true; 8];
        let (result, out_mask) = simd_linear_scale_8(values, 0.0, 10.0, mask);

        // Expected: values are just divided by 10
        let expected = [0.0, 0.25, 0.5, 0.75, 1.0, 0.1, 0.3, 0.9];

        for i in 0..8 {
            assert!(
                (result[i] - expected[i]).abs() < 1e-14,
                "Linear scale mismatch at {}: {} vs {}",
                i,
                result[i],
                expected[i]
            );
            assert!(out_mask[i], "Mask should remain true at {}", i);
        }
    }

    #[test]
    fn test_simd_linear_scale_clamping() {
        let values = [-5.0, 0.0, 5.0, 10.0, 15.0, 20.0, 2.5, 7.5];
        let mask = [true; 8];
        let (result, _) = simd_linear_scale_8(values, 0.0, 10.0, mask);

        // Values outside [0, 10] should clamp to 0.0 or 1.0
        assert_eq!(result[0], 0.0); // -5 < 0: clamp to 0
        assert_eq!(result[1], 0.0); // 0 ≤ 0: clamp to 0
        assert_eq!(result[2], 0.5); // 5: linear
        assert_eq!(result[3], 1.0); // 10 ≥ 10: clamp to 1
        assert_eq!(result[4], 1.0); // 15 > 10: clamp to 1
        assert_eq!(result[5], 1.0); // 20 > 10: clamp to 1
    }

    #[test]
    fn test_simd_log_scale_8() {
        let values = [1.0, 10.0, 100.0, 1000.0, 5.0, 50.0, 10.0, 100.0];
        let log_min = 1.0_f64.ln(); // ln(1) = 0
        let log_max = 100.0_f64.ln(); // ln(100) ≈ 4.605
        let log_range = log_max - log_min;
        let mask = [true; 8];

        let (result, out_mask) = simd_log_scale_8(values, log_min, log_range, mask);

        // All values are positive, so all should remain valid
        for item in &out_mask {
            assert!(*item, "All positive values should remain valid");
        }

        // ln(1) → 0, ln(100) → 1 (within range)
        assert!(
            (result[0] - 0.0).abs() < 1e-14,
            "log scale of min should be 0"
        ); // ln(1) = 0
        assert!(
            (result[2] - 1.0).abs() < 1e-14,
            "log scale of max should be 1"
        ); // ln(100) = log_max
        assert!(
            (result[3] - 1.0).abs() < 1e-14,
            "log scale of 1000 should clamp to 1"
        ); // 1000 > 100, clamps to 1

        // Verify log scale is increasing for in-range values
        assert!(result[0] < result[1]); // ln(1) < ln(10)
        assert!(result[1] < result[2]); // ln(10) < ln(100)
    }

    #[test]
    fn test_simd_gamma_correct_8() {
        let values = [0.0, 0.25, 0.5, 0.75, 1.0, 0.1, 0.9, 0.5];
        let mask = [true; 8];
        let gamma = 2.0; // Common gamma correction
        let gamma_inv = 1.0 / gamma; // 0.5

        let (result, out_mask) = simd_gamma_correct_8(values, gamma_inv, mask);

        // For gamma_inv = 0.5 (gamma = 2), result[i] = sqrt(values[i])
        assert!((result[0] - 0.0).abs() < 1e-14); // sqrt(0) = 0
        assert!((result[1] - 0.5).abs() < 1e-14); // sqrt(0.25) = 0.5
        assert!((result[2] - (0.5_f64).sqrt()).abs() < 1e-14); // sqrt(0.5)
        assert!((result[4] - 1.0).abs() < 1e-14); // sqrt(1) = 1

        // All should remain valid
        for (i, item) in out_mask.iter().enumerate() {
            assert!(*item, "Mask should remain true at {}", i);
        }
    }

    #[test]
    fn test_simd_colormap_sample_8_lookup() {
        // Create a simple test LUT: gradient from black to white
        let mut lut = [[0u8; 3]; 256];
        for (i, item) in lut.iter_mut().enumerate() {
            let val = i as u8;
            *item = [val, val, val]; // Grayscale gradient
        }

        let normalized = [0.0, 0.25, 0.5, 0.75, 1.0, 0.1, 0.9, 0.5];
        let mask = [true; 8];

        let (rgb_buffer, out_mask) = simd_colormap_sample_8(normalized, &lut, mask);

        // Verify LUT lookups
        // 0.0 * 255 = 0 → RGB(0, 0, 0)
        assert_eq!(rgb_buffer[0], 0);
        assert_eq!(rgb_buffer[1], 0);
        assert_eq!(rgb_buffer[2], 0);

        // 1.0 * 255 = 255 → RGB(255, 255, 255)
        let idx_white = 4 * 3;
        assert_eq!(rgb_buffer[idx_white], 255);
        assert_eq!(rgb_buffer[idx_white + 1], 255);
        assert_eq!(rgb_buffer[idx_white + 2], 255);

        // Mask unchanged
        for item in &out_mask {
            assert!(*item);
        }
    }

    #[test]
    fn test_simd_colormap_sample_8_invalid_pixels() {
        let lut = [[100u8; 3]; 256];
        let normalized = [0.5; 8];
        let mut mask = [true; 8];
        mask[2] = false; // Mark one pixel invalid
        mask[5] = false; // Mark another invalid

        let (rgb_buffer, out_mask) = simd_colormap_sample_8(normalized, &lut, mask);

        // Valid pixels should get LUT value
        assert_eq!(rgb_buffer[0], 100); // Pixel 0: 0.5 * 255 = 127 → lut[127]

        // Invalid pixels should get black (0, 0, 0)
        let idx_invalid = 2 * 3;
        assert_eq!(rgb_buffer[idx_invalid], 0);
        assert_eq!(rgb_buffer[idx_invalid + 1], 0);
        assert_eq!(rgb_buffer[idx_invalid + 2], 0);

        // Mask unchanged
        assert!(out_mask[0]);
        assert!(!out_mask[2]);
        assert!(!out_mask[5]);
    }
}

//─────────────────────────────────────────────────────────────────────────────
// Batch Scaling Wrapper for Integration with Main Render Loop
//─────────────────────────────────────────────────────────────────────────────

/// Batch process 8 raw values through scaling operation with caching
///
/// This wrapper function encapsulates the scaling step for 8 HEALPix values,
/// dispatching to the appropriate SIMD function based on scale type.
/// Designed to integrate with main render loop for efficient batch processing.
///
/// Input:
/// - values: 8 raw data values from HEALPix sampling
/// - min, max: scaling bounds
/// - log_cache: pre-computed (log_min, log_range) for log scale
/// - mask: validity mask from HEALPix sampling
///
/// Output:
/// - scaled: 8 normalized values in [0, 1]
/// - out_mask: updated validity mask
///
/// Note: Currently handles Linear and Log scales. Other scales (Asinh, Symlog, etc.)
/// require scalar path or additional transcendental implementations.
#[inline]
pub fn simd_batch_scale_8(
    values: [f64; 8],
    min: f64,
    max: f64,
    use_log: bool,
    log_cache: Option<(f64, f64)>,
    mask: [bool; 8],
) -> ([f64; 8], [bool; 8]) {
    if use_log {
        // Logarithmic scale: requires pre-computed cache
        if let Some((log_min, log_range)) = log_cache {
            simd_log_scale_8(values, log_min, log_range, mask)
        } else {
            // Fallback: use linear scale if cache not available
            simd_linear_scale_8(values, min, max, mask)
        }
    } else {
        // Linear scale: no cache needed
        simd_linear_scale_8(values, min, max, mask)
    }
}

//─────────────────────────────────────────────────────────────────────────────
// PixelValue Wrapper Functions (Phase 5.2: Main Loop Integration)
//─────────────────────────────────────────────────────────────────────────────
// These functions wrap SIMD scaling results into the PixelValue enum format
// used by the render loop, handling underflow/overflow classification.

use crate::PixelValue;

/// Convert SIMD linear scale results to PixelValue enum array
///
/// Maps normalized [0, 1] values to the PixelValue enum used in the render loop:
/// - 0.0 → PixelValue::Underflow
/// - 0.0 < t < 1.0 → PixelValue::Color(t)
/// - 1.0 → PixelValue::Overflow
/// - Invalid mask → PixelValue::Bad
///
/// This is the integration point between SIMD batch operations and the
/// per-pixel enum-based rendering pipeline.
#[inline]
pub fn simd_to_pixel_values(scaled: [f64; 8], mask: [bool; 8]) -> [PixelValue; 8] {
    [
        if !mask[0] {
            PixelValue::Bad
        } else if scaled[0] <= 0.0 {
            PixelValue::Underflow
        } else if scaled[0] >= 1.0 {
            PixelValue::Overflow
        } else {
            PixelValue::Color(scaled[0])
        },
        if !mask[1] {
            PixelValue::Bad
        } else if scaled[1] <= 0.0 {
            PixelValue::Underflow
        } else if scaled[1] >= 1.0 {
            PixelValue::Overflow
        } else {
            PixelValue::Color(scaled[1])
        },
        if !mask[2] {
            PixelValue::Bad
        } else if scaled[2] <= 0.0 {
            PixelValue::Underflow
        } else if scaled[2] >= 1.0 {
            PixelValue::Overflow
        } else {
            PixelValue::Color(scaled[2])
        },
        if !mask[3] {
            PixelValue::Bad
        } else if scaled[3] <= 0.0 {
            PixelValue::Underflow
        } else if scaled[3] >= 1.0 {
            PixelValue::Overflow
        } else {
            PixelValue::Color(scaled[3])
        },
        if !mask[4] {
            PixelValue::Bad
        } else if scaled[4] <= 0.0 {
            PixelValue::Underflow
        } else if scaled[4] >= 1.0 {
            PixelValue::Overflow
        } else {
            PixelValue::Color(scaled[4])
        },
        if !mask[5] {
            PixelValue::Bad
        } else if scaled[5] <= 0.0 {
            PixelValue::Underflow
        } else if scaled[5] >= 1.0 {
            PixelValue::Overflow
        } else {
            PixelValue::Color(scaled[5])
        },
        if !mask[6] {
            PixelValue::Bad
        } else if scaled[6] <= 0.0 {
            PixelValue::Underflow
        } else if scaled[6] >= 1.0 {
            PixelValue::Overflow
        } else {
            PixelValue::Color(scaled[6])
        },
        if !mask[7] {
            PixelValue::Bad
        } else if scaled[7] <= 0.0 {
            PixelValue::Underflow
        } else if scaled[7] >= 1.0 {
            PixelValue::Overflow
        } else {
            PixelValue::Color(scaled[7])
        },
    ]
}

#[cfg(test)]
mod batch_integration_tests {
    use super::*;

    #[test]
    fn test_batch_scale_linear() {
        let values = [0.0, 2.5, 5.0, 7.5, 10.0, 1.0, 3.0, 9.0];
        let mask = [true; 8];
        let (result, _) = simd_batch_scale_8(values, 0.0, 10.0, false, None, mask);

        // Should match linear scaling
        let expected = [0.0, 0.25, 0.5, 0.75, 1.0, 0.1, 0.3, 0.9];
        for i in 0..8 {
            assert!((result[i] - expected[i]).abs() < 1e-14);
        }
    }

    #[test]
    fn test_batch_scale_log() {
        let values = [1.0, 10.0, 100.0, 1000.0, 5.0, 50.0, 10.0, 100.0];
        let log_min = 1.0_f64.ln();
        let log_range = 100.0_f64.ln() - log_min;
        let mask = [true; 8];

        let (result, _) =
            simd_batch_scale_8(values, 1.0, 100.0, true, Some((log_min, log_range)), mask);

        // Should match log scaling
        assert!((result[0] - 0.0).abs() < 1e-14); // log(1) at min
        assert!((result[2] - 1.0).abs() < 1e-14); // log(100) at max
    }

    #[test]
    fn test_simd_to_pixel_values() {
        // Test conversion of SIMD results to PixelValue enum
        let scaled = [0.0, 0.5, 1.0, 0.25, 0.75, -0.1, 1.1, 0.5];
        let mask = [true, true, true, true, true, false, false, true];

        let pixel_values = simd_to_pixel_values(scaled, mask);

        // Check each value
        match pixel_values[0] {
            PixelValue::Underflow => {} // 0.0
            _ => panic!("Expected Underflow for value 0.0"),
        }

        match pixel_values[1] {
            PixelValue::Color(c) => assert_eq!(c, 0.5),
            _ => panic!("Expected Color(0.5)"),
        }

        match pixel_values[2] {
            PixelValue::Overflow => {} // 1.0
            _ => panic!("Expected Overflow for value 1.0"),
        }

        match pixel_values[3] {
            PixelValue::Color(c) => assert_eq!(c, 0.25),
            _ => panic!("Expected Color(0.25)"),
        }

        match pixel_values[4] {
            PixelValue::Color(c) => assert_eq!(c, 0.75),
            _ => panic!("Expected Color(0.75)"),
        }

        match pixel_values[5] {
            PixelValue::Bad => {} // mask[5] = false
            _ => panic!("Expected Bad for masked value"),
        }

        match pixel_values[6] {
            PixelValue::Bad => {} // mask[6] = false
            _ => panic!("Expected Bad for masked value"),
        }

        match pixel_values[7] {
            PixelValue::Color(c) => assert_eq!(c, 0.5),
            _ => panic!("Expected Color(0.5)"),
        }
    }
}

//─────────────────────────────────────────────────────────────────────────────
// Tier 5: Extended Batch Sizes (16-element Functions)
//─────────────────────────────────────────────────────────────────────────────
// Optimized batch processing for improved throughput on modern CPUs.
// These functions process 16 elements by chaining two 8-element operations.
// Future: Can be replaced with true AVX2 or AVX-512 implementations.

/// Vectorized sin_cos for 16 f64 values
///
/// Processes 16 angles by splitting into two 8-element batches
#[inline]
pub fn simd_sin_cos_16(angles: [f64; 16]) -> ([f64; 16], [f64; 16]) {
    let (sin_lo, cos_lo) = simd_sin_cos_8([
        angles[0], angles[1], angles[2], angles[3], angles[4], angles[5], angles[6], angles[7],
    ]);
    let (sin_hi, cos_hi) = simd_sin_cos_8([
        angles[8], angles[9], angles[10], angles[11], angles[12], angles[13], angles[14],
        angles[15],
    ]);

    let mut sin_result = [0.0; 16];
    let mut cos_result = [0.0; 16];

    sin_result[..8].copy_from_slice(&sin_lo);
    cos_result[..8].copy_from_slice(&cos_lo);
    sin_result[8..16].copy_from_slice(&sin_hi);
    cos_result[8..16].copy_from_slice(&cos_hi);

    (sin_result, cos_result)
}

/// Batch scale 16 values with validity masking (for 16-pixel rendering)
///
/// Processes 16 raw values through scaling operation, handling linear and log scales.
/// Internally processes as two 8-element batches for optimal CPU cache utilization.
#[inline]
pub fn simd_batch_scale_16(
    values: [f64; 16],
    min: f64,
    max: f64,
    use_log: bool,
    log_cache: Option<(f64, f64)>,
    mask: [bool; 16],
) -> ([f64; 16], [bool; 16]) {
    let (scaled_lo, mask_lo) = simd_batch_scale_8(
        [
            values[0], values[1], values[2], values[3], values[4], values[5], values[6], values[7],
        ],
        min,
        max,
        use_log,
        log_cache,
        [
            mask[0], mask[1], mask[2], mask[3], mask[4], mask[5], mask[6], mask[7],
        ],
    );

    let (scaled_hi, mask_hi) = simd_batch_scale_8(
        [
            values[8], values[9], values[10], values[11], values[12], values[13], values[14],
            values[15],
        ],
        min,
        max,
        use_log,
        log_cache,
        [
            mask[8], mask[9], mask[10], mask[11], mask[12], mask[13], mask[14], mask[15],
        ],
    );

    let mut result = [0.0; 16];
    let mut out_mask = [false; 16];

    result[..8].copy_from_slice(&scaled_lo);
    out_mask[..8].copy_from_slice(&mask_lo);
    result[8..16].copy_from_slice(&scaled_hi);
    out_mask[8..16].copy_from_slice(&mask_hi);

    (result, out_mask)
}

/// Convert 16 SIMD scaling results to PixelValue array
///
/// Processes 16 scaled values, converting to PixelValue enum format
/// by processing two 8-element batches.
#[inline]
pub fn simd_to_pixel_values_16(scaled: [f64; 16], mask: [bool; 16]) -> [PixelValue; 16] {
    let pixel_lo = simd_to_pixel_values(
        [
            scaled[0], scaled[1], scaled[2], scaled[3], scaled[4], scaled[5], scaled[6], scaled[7],
        ],
        [
            mask[0], mask[1], mask[2], mask[3], mask[4], mask[5], mask[6], mask[7],
        ],
    );

    let pixel_hi = simd_to_pixel_values(
        [
            scaled[8], scaled[9], scaled[10], scaled[11], scaled[12], scaled[13], scaled[14],
            scaled[15],
        ],
        [
            mask[8], mask[9], mask[10], mask[11], mask[12], mask[13], mask[14], mask[15],
        ],
    );

    let mut result = [PixelValue::Bad; 16];
    result[..8].copy_from_slice(&pixel_lo);
    result[8..16].copy_from_slice(&pixel_hi);

    result
}

#[cfg(test)]
mod batch_16_tests {
    use super::*;

    #[test]
    fn test_simd_sin_cos_16() {
        // Create test angles
        let mut angles = [0.0; 16];
        for (i, item) in angles.iter_mut().enumerate() {
            *item = (i as f64) * std::f64::consts::PI / 8.0;
        }

        let (sines, cosines) = simd_sin_cos_16(angles);

        // Verify results match individual sin_cos calls
        for i in 0..16 {
            let (expected_sin, expected_cos) = angles[i].sin_cos();
            assert!(
                (sines[i] - expected_sin).abs() < 1e-14,
                "sin mismatch at {}",
                i
            );
            assert!(
                (cosines[i] - expected_cos).abs() < 1e-14,
                "cos mismatch at {}",
                i
            );
        }
    }

    #[test]
    fn test_simd_batch_scale_16_linear() {
        let values = [
            0.0, 2.5, 5.0, 7.5, 10.0, 1.0, 3.0, 9.0, 2.0, 4.0, 6.0, 8.0, 1.5, 3.5, 5.5, 7.5,
        ];
        let mask = [true; 16];

        let (result, _) = simd_batch_scale_16(values, 0.0, 10.0, false, None, mask);

        // All values should match their individual linear scale results
        for i in 0..16 {
            let expected = if values[i] <= 0.0 {
                0.0
            } else if values[i] >= 10.0 {
                1.0
            } else {
                values[i] / 10.0
            };

            assert!(
                (result[i] - expected).abs() < 1e-14,
                "scale mismatch at {}: {} vs {}",
                i,
                result[i],
                expected
            );
        }
    }

    #[test]
    fn test_simd_to_pixel_values_16() {
        let scaled = [
            0.0, 0.25, 0.5, 0.75, 1.0, 0.1, 0.9, 0.5, 0.33, 0.67, -0.1, 1.1, 0.2, 0.8, 0.4, 0.6,
        ];
        let mask = [
            true, true, true, true, true, true, true, true, true, true, false, false, true, true,
            true, true,
        ];

        let pixel_values = simd_to_pixel_values_16(scaled, mask);

        // Verify first 8 elements match individual conversions
        match pixel_values[0] {
            PixelValue::Underflow => {}
            _ => panic!("Expected Underflow at 0"),
        }

        match pixel_values[4] {
            PixelValue::Overflow => {}
            _ => panic!("Expected Overflow at 4"),
        }

        match pixel_values[10] {
            PixelValue::Bad => {}
            _ => panic!("Expected Bad at 10 (unmasked)"),
        }

        match pixel_values[15] {
            PixelValue::Color(c) => assert_eq!(c, 0.6),
            _ => panic!("Expected Color(0.6) at 15"),
        }
    }
}

//─────────────────────────────────────────────────────────────────────────────
// Full Pipeline Integration Tests (Phase 5.2 Validation)
//─────────────────────────────────────────────────────────────────────────────
// NOTE: Full pipeline tests deferred to Phase 5.2 integration work.
// Phase 5.1 focuses on individual operation verification.
// Complete pipeline tests will be added after main render loop integration.
