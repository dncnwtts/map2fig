//! SIMD vectorization module for high-performance batch operations.
//!
//! Provides vectorized implementations of common mathematical operations
//! using portable SIMD or platform-specific intrinsics when available.
//!
//! All functions process 8 f64 values in parallel, matching Tier 2 batch size.

use std::f64::consts::PI;

/// Vectorized sine for 8 f64 values
///
/// Computes sin(x) for 8 angles simultaneously.
/// Uses scalar implementation but structured for future SIMD acceleration.
///
/// Input: 8 angles in radians
/// Output: 8 sine values in [-1, 1]
#[inline]
pub fn simd_sin_8(angles: [f64; 8]) -> [f64; 8] {
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
#[inline]
pub fn simd_cos_8(angles: [f64; 8]) -> [f64; 8] {
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
/// This is more efficient than calling simd_sin_8 and simd_cos_8 separately.
///
/// Returns: (sin_values, cos_values)
#[inline]
pub fn simd_sin_cos_8(angles: [f64; 8]) -> ([f64; 8], [f64; 8]) {
    let mut sines = [0.0; 8];
    let mut cosines = [0.0; 8];

    for i in 0..8 {
        (sines[i], cosines[i]) = angles[i].sin_cos();
    }

    (sines, cosines)
}

/// Vectorized inverse tangent (atan2) for 8 point pairs
///
/// Computes atan2(y, x) for 8 (y, x) coordinate pairs.
/// Handles all quadrants correctly.
///
/// Inputs: y and x arrays of 8 values each
/// Output: 8 angles in [-π, π]
#[inline]
pub fn simd_atan2_8(y: [f64; 8], x: [f64; 8]) -> [f64; 8] {
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
/// Input values must be in [-1, 1].
///
/// Output: 8 angles in [-π/2, π/2]
#[inline]
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
/// Input values must be in [-1, 1].
///
/// Output: 8 angles in [0, π]
#[inline]
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
#[inline]
pub fn simd_sqrt_8(x: [f64; 8]) -> [f64; 8] {
    [x[0].sqrt(), x[1].sqrt(), x[2].sqrt(), x[3].sqrt(),
     x[4].sqrt(), x[5].sqrt(), x[6].sqrt(), x[7].sqrt()]
}

/// Vectorized power function (y = x^exp)
///
/// Computes x^exp for 8 values simultaneously.
/// Used for gamma correction and scaling operations.
#[inline]
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
    [x[0].ln(), x[1].ln(), x[2].ln(), x[3].ln(),
     x[4].ln(), x[5].ln(), x[6].ln(), x[7].ln()]
}

/// Vectorized reciprocal (1/x)
///
/// Computes 1/x for 8 values simultaneously.
/// More efficient than division for reciprocals.
#[inline]
pub fn simd_recip_8(x: [f64; 8]) -> [f64; 8] {
    [x[0].recip(), x[1].recip(), x[2].recip(), x[3].recip(),
     x[4].recip(), x[5].recip(), x[6].recip(), x[7].recip()]
}

/// Vectorized absolute value
#[inline]
pub fn simd_abs_8(x: [f64; 8]) -> [f64; 8] {
    [x[0].abs(), x[1].abs(), x[2].abs(), x[3].abs(),
     x[4].abs(), x[5].abs(), x[6].abs(), x[7].abs()]
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
    let mag_sq = simd_madd_8(
        x, x,
        simd_madd_8(y, y, simd_mul_8(z, z))
    );
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
    a_x: [f64; 8], a_y: [f64; 8], a_z: [f64; 8],
    b_x: [f64; 8], b_y: [f64; 8], b_z: [f64; 8],
) -> [f64; 8] {
    simd_madd_8(
        a_x, b_x,
        simd_madd_8(a_y, b_y, simd_mul_8(a_z, b_z))
    )
}

/// Vectorized 3D cross product
///
/// Computes cross product for 8 3D vector pairs
/// c = a × b
pub fn simd_cross_8(
    a_x: [f64; 8], a_y: [f64; 8], a_z: [f64; 8],
    b_x: [f64; 8], b_y: [f64; 8], b_z: [f64; 8],
) -> ([f64; 8], [f64; 8], [f64; 8]) {
    let c_x = simd_add_8(
        simd_mul_8(a_y, b_z),
        simd_mul_8(simd_mul_8(a_z, b_y), [-1.0; 8])
    );
    let c_y = simd_add_8(
        simd_mul_8(a_z, b_x),
        simd_mul_8(simd_mul_8(a_x, b_z), [-1.0; 8])
    );
    let c_z = simd_add_8(
        simd_mul_8(a_x, b_y),
        simd_mul_8(simd_mul_8(a_y, b_x), [-1.0; 8])
    );
    
    (c_x, c_y, c_z)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-14;

    #[test]
    fn test_simd_sin_8() {
        let angles = [0.0, PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0, PI, -PI / 6.0, -PI / 2.0];
        let result = simd_sin_8(angles);

        for i in 0..8 {
            let expected = angles[i].sin();
            assert!((result[i] - expected).abs() < EPSILON, "sin mismatch at index {}", i);
        }
    }

    #[test]
    fn test_simd_cos_8() {
        let angles = [0.0, PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0, PI, -PI / 6.0, -PI / 2.0];
        let result = simd_cos_8(angles);

        for i in 0..8 {
            let expected = angles[i].cos();
            assert!((result[i] - expected).abs() < EPSILON, "cos mismatch at index {}", i);
        }
    }

    #[test]
    fn test_simd_sin_cos_8() {
        let angles = [0.0, PI / 6.0, PI / 4.0, PI / 3.0, PI / 2.0, PI, -PI / 6.0, -PI / 2.0];
        let (sines, cosines) = simd_sin_cos_8(angles);

        for i in 0..8 {
            let (expected_sin, expected_cos) = angles[i].sin_cos();
            assert!((sines[i] - expected_sin).abs() < EPSILON, "sin mismatch at index {}", i);
            assert!((cosines[i] - expected_cos).abs() < EPSILON, "cos mismatch at index {}", i);
        }
    }

    #[test]
    fn test_simd_atan2_8() {
        let y = [1.0, 1.0, 0.0, -1.0, -1.0, -1.0, 0.0, 1.0];
        let x = [1.0, 0.0, 1.0, 1.0, 0.0, -1.0, -1.0, -1.0];
        let result = simd_atan2_8(y, x);

        for i in 0..8 {
            let expected = y[i].atan2(x[i]);
            assert!((result[i] - expected).abs() < EPSILON, "atan2 mismatch at index {}", i);
        }
    }

    #[test]
    fn test_simd_asin_8() {
        let x = [-1.0, -0.5, 0.0, 0.5, 1.0, -0.707, 0.707, 0.866];
        let result = simd_asin_8(x);

        for i in 0..8 {
            let expected = x[i].asin();
            assert!((result[i] - expected).abs() < EPSILON, "asin mismatch at index {}", i);
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
            assert!((mag_sq - 1.0).abs() < EPSILON, "magnitude mismatch at index {}", i);
            // For y=0, z=0, normalized x should be 1.0
            assert!((nx[i] - 1.0).abs() < EPSILON, "normalized x mismatch at index {}", i);
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
            assert!((result[i] - b_x[i]).abs() < EPSILON, "dot product mismatchat index {}", i);
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
pub fn simd_vec_to_sph_8(
    x: [f64; 8],
    y: [f64; 8],
    z: [f64; 8],
) -> ([f64; 8], [f64; 8]) {
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
/// - x'[i] = m[0][0] * x[i] + m[0][1] * y[i] + m[0][2] * z[i]
/// - y'[i] = m[1][0] * x[i] + m[1][1] * y[i] + m[1][2] * z[i]
/// - z'[i] = m[2][0] * x[i] + m[2][1] * y[i] + m[2][2] * z[i]
///
/// Input:
/// - mat: 3x3 matrix (row-major, [row][col])
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
        simd_add_8(
            simd_mul_8(x, [mat[0][0]; 8]),
            simd_mul_8(y, [mat[0][1]; 8]),
        ),
        simd_mul_8(z, [mat[0][2]; 8]),
    );

    // Second row: [m10*x[i] + m11*y[i] + m12*z[i]]
    let y_new = simd_add_8(
        simd_add_8(
            simd_mul_8(x, [mat[1][0]; 8]),
            simd_mul_8(y, [mat[1][1]; 8]),
        ),
        simd_mul_8(z, [mat[1][2]; 8]),
    );

    // Third row: [m20*x[i] + m21*y[i] + m22*z[i]]
    let z_new = simd_add_8(
        simd_add_8(
            simd_mul_8(x, [mat[2][0]; 8]),
            simd_mul_8(y, [mat[2][1]; 8]),
        ),
        simd_mul_8(z, [mat[2][2]; 8]),
    );

    (x_new, y_new, z_new)
}

#[cfg(test)]
mod healpix_tests {
    use super::*;

    #[test]
    fn test_simd_sph_to_vec_8() {
        let theta = [0.0, PI / 2.0, PI, 0.0, PI / 4.0, PI / 4.0, PI / 3.0, PI / 6.0];
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
        let theta_in = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 0.1, 3.14];
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
