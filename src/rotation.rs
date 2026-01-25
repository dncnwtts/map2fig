//! Coordinate rotations for astronomy.
//!
//! Internally: ACTIVE rotations (rotate vectors).
//! Externally: PASSIVE semantics (re-express same direction).

use std::f64::consts::PI;

pub type Mat3 = [[f64; 3]; 3];

/// Coordinate systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordSystem {
    C, // Equatorial (ICRS / J2000)
    G, // Galactic
    E, // Ecliptic
}

/// Obliquity of the ecliptic (J2000)
const OBLIQUITY: f64 = 23.439291111_f64.to_radians();

/// Galactic → Equatorial (ACTIVE, IAU 1958 / J2000)
pub const GAL_TO_EQ: Mat3 = [
    [-0.0548755604162154,  0.4941094278755837, -0.8676661490190047],
    [-0.8734370902348850, -0.4448296299600112, -0.1980763734312015],
    [-0.4838350155487132,  0.7469822444972189,  0.4559837761750669],
];

/// Equatorial → Galactic (ACTIVE inverse)
pub const EQ_TO_GAL: Mat3 = [
    [-0.0548755604162154, -0.8734370902348850, -0.4838350155487132],
    [ 0.4941094278755837, -0.4448296299600112,  0.7469822444972189],
    [-0.8676661490190047, -0.1980763734312015,  0.4559837761750669],
];

pub const NGP_ECL: [f64; 3] = [
    -0.8676661490,
    -0.4927284661,
     0.0669887394,
];

pub const GAL_CENTER_EQ: [f64; 3] = [
    -0.054876,
     0.494110,
    -0.867666,
];



/// Equatorial → Ecliptic (ACTIVE, rotate by −ε about +X)
pub fn eq_to_ecl(v: [f64; 3]) -> [f64; 3] {
    let (s, c) = OBLIQUITY.sin_cos();
    [
        v[0],
        c * v[1] + s * v[2],
       -s * v[1] + c * v[2],
    ]
}

/// Ecliptic → Equatorial (ACTIVE inverse)
pub fn ecl_to_eq(v: [f64; 3]) -> [f64; 3] {
    let (s, c) = OBLIQUITY.sin_cos();
    [
        v[0],
        c * v[1] - s * v[2],
        s * v[1] + c * v[2],
    ]
}

/// Galactic → Ecliptic (ACTIVE composition)
pub fn gal_to_ecl_active(v: [f64; 3]) -> [f64; 3] {
    eq_to_ecl(matvec(&GAL_TO_EQ, v))
}

/// Ecliptic → Galactic (ACTIVE inverse)
pub fn ecl_to_gal_active(v: [f64; 3]) -> [f64; 3] {
    matvec(&EQ_TO_GAL, ecl_to_eq(v))
}

#[inline(always)]
pub fn sph_to_vec(theta: f64, phi: f64) -> [f64; 3] {
    let st = theta.sin();
    [
        st * phi.cos(),
        st * phi.sin(),
        theta.cos(),
    ]
}

#[inline(always)]
pub fn vec_to_sph(v: [f64; 3]) -> (f64, f64) {
    let z = v[2].clamp(-1.0, 1.0);
    let theta = z.acos();
    let phi = v[1].atan2(v[0]).rem_euclid(2.0 * std::f64::consts::PI);
    (theta, phi)
}

//
// ──────────────────────────────────────────────────────────
// PASSIVE API (what Healpix should call)
// ──────────────────────────────────────────────────────────
//

/// PASSIVE: express a galactic vector in ecliptic coordinates
#[inline]
pub fn gal_to_ecl(v: [f64; 3]) -> [f64; 3] {
    // passive = active inverse
    ecl_to_gal_active(v)
}

/// PASSIVE: express an ecliptic vector in galactic coordinates
#[inline]
pub fn ecl_to_gal(v: [f64; 3]) -> [f64; 3] {
    gal_to_ecl_active(v)
}

//
// ──────────────────────────────────────────────────────────
// Linear algebra helpers
// ──────────────────────────────────────────────────────────
//

#[inline]
pub fn matvec(m: &Mat3, v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0]*v[0] + m[0][1]*v[1] + m[0][2]*v[2],
        m[1][0]*v[0] + m[1][1]*v[1] + m[1][2]*v[2],
        m[2][0]*v[0] + m[2][1]*v[1] + m[2][2]*v[2],
    ]
}

#[inline]
pub fn normalize(v: [f64; 3]) -> [f64; 3] {
    let n = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    [v[0]/n, v[1]/n, v[2]/n]
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const DEG2RAD: f64 = PI / 180.0;
    const TOL: f64 = 1e-9;

    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    /// Convert cartesian vector `v` into (theta, lon)
    fn vec_to_angles(v: [f64; 3]) -> (f64, f64) {
        let (theta, lon) = vec_to_sph(v);
        let lon = lon.rem_euclid(2.0 * PI);
        (theta, lon)
    }

    // #[test]
    // fn galactic_center_to_ecliptic_angles() {
    //     // Direction: l=0°, b=0°
    //     let v_gal = galactic_lonlat_to_vec(0.0 * DEG2RAD, 0.0 * DEG2RAD);

    //     // Passive transform → ecliptic
    //     let v_ecl = gal_to_ecl(v_gal);
    //     let (theta, lon) = vec_to_angles(v_ecl);

    //     // Known ecliptic coords from literature:
    //     // λ ≈ 266.14097°, β ≈ –5.5297°
    //     let expected_theta = (90.0 + 5.5297) * DEG2RAD;
    //     let expected_lon   = 266.14097 * DEG2RAD;

    //     assert!(approx_eq(theta, expected_theta, 1e-6));
    //     assert!(approx_eq(lon, expected_lon,   1e-6));
    // }

    // #[test]
    // fn north_galactic_pole_to_ecliptic_angles() {
    //     // Direction: b=+90°
    //     let v_ngp_gal = galactic_lonlat_to_vec(0.0, 90.0 * DEG2RAD);
    //     let v_ecl = gal_to_ecl(v_ngp_gal);
    //     let (theta, lon) = vec_to_angles(v_ecl);

    //     // Known ecliptic coords ≈ λ 179.32095°, β +29.811954°
    //     let expected_theta = (90.0 - 29.811954) * DEG2RAD;
    //     let expected_lon   = 179.32095 * DEG2RAD;

    //     assert!(approx_eq(theta, expected_theta, 1e-6));
    //     assert!(approx_eq(lon, expected_lon,     1e-6));
    // }

    #[test]
    fn north_ecliptic_pole_is_pole() {
        // At β=+90°, theta==0
        let v = sph_to_vec(0.0, 0.0); // north ecliptic pole in ecliptic coords
        let (theta, _) = vec_to_angles(v);
        assert!(approx_eq(theta, 0.0, 1e-12));
    }
}

