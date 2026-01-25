//! Coordinate rotations for astronomy.
//!
//! Internally: ACTIVE rotations (rotate vectors).
//! Externally: PASSIVE semantics (re-express same direction).

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

/// Galactic longitude ℓ, latitude b → Cartesian unit vector
/// ℓ, b in radians
#[inline(always)]
pub fn galactic_lonlat_to_vec(l: f64, b: f64) -> [f64; 3] {
    let cb = b.cos();
    [
        cb * l.cos(),
        cb * l.sin(),
        b.sin(),
    ]
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

    fn vec_approx_eq(a: [f64; 3], b: [f64; 3], tol: f64) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < tol)
    }

    fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
        let mut sum = 0.0;
        for i in 0..3 {
            sum += a[i] * b[i];
        }
        sum
    }

    /// Convert cartesian vector `v` into (theta, lon)
    fn vec_to_angles(v: [f64; 3]) -> (f64, f64) {
        let (theta, lon) = vec_to_sph(v);
        let lon = lon.rem_euclid(2.0 * PI);
        (theta, lon)
    }

    /// Convert vector → (lon, lat)
    fn vec_to_lonlat(v: [f64; 3]) -> (f64, f64) {
        let r = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
        let lon = v[1].atan2(v[0]).rem_euclid(2.0 * std::f64::consts::PI);
        let lat = (v[2] / r).asin();
        (lon, lat)
    }
    
    #[test]
    fn north_galactic_pole_to_equatorial_angles() {
        let v_gal = galactic_lonlat_to_vec(0.0, 90.0 * DEG2RAD);
        let v_eq  = matvec(&GAL_TO_EQ, v_gal);
        let (ra, dec) = vec_to_lonlat(v_eq);
    
        let ra_exp  = 192.85948 * DEG2RAD;
        let dec_exp = 27.12825 * DEG2RAD;
    
        assert!(approx_eq(ra,  ra_exp,  1e-6), "RA mismatch");
        assert!(approx_eq(dec, dec_exp, 1e-6), "Dec mismatch");
    }
    
    



    #[test]
    fn north_ecliptic_pole_is_pole() {
        // At β=+90°, theta==0
        let v = sph_to_vec(0.0, 0.0); // north ecliptic pole in ecliptic coords
        let (theta, _) = vec_to_angles(v);
        assert!(approx_eq(theta, 0.0, 1e-12));
    }

    #[test]
    fn galactic_lonlat_round_trip_angles() {
        let cases = [
            (0.0, 0.0),
            (90.0, 0.0),
            (180.0, 0.0),
            (45.0, 30.0),
            (270.0, -45.0),
        ];
    
        for (l_deg, b_deg) in cases {
            let l = l_deg * DEG2RAD;
            let b = b_deg * DEG2RAD;
    
            let v_gal = galactic_lonlat_to_vec(l, b);
            let v_ecl = gal_to_ecl(v_gal);
            let v_back = ecl_to_gal(v_ecl);
    
            let (l2, b2) = vec_to_lonlat(v_back);
    
            assert!(
                approx_eq(b, b2, 1e-8),
                "Latitude round-trip failed: b={} → {}",
                b_deg, b2 / DEG2RAD
            );
    
            assert!(
                approx_eq(
                    (l - l2).sin(), 0.0, 1e-8
                ),
                "Longitude round-trip failed: l={} → {}",
                l_deg, l2 / DEG2RAD
            );
        }
    }

    #[test]
    fn equatorial_ecliptic_round_trip_angles() {
        let cases = [
            (0.0, 0.0),
            (90.0, 0.0),
            (180.0, 0.0),
            (45.0, 23.0),
            (300.0, -30.0),
        ];
    
        for (lon_deg, lat_deg) in cases {
            let lon = lon_deg * DEG2RAD;
            let lat = lat_deg * DEG2RAD;
    
            let v = [
                lat.cos() * lon.cos(),
                lat.cos() * lon.sin(),
                lat.sin(),
            ];
    
            let v_ecl = eq_to_ecl(v);
            let v_back = ecl_to_eq(v_ecl);
    
            let (lon2, lat2) = vec_to_lonlat(v_back);
    
            assert!(approx_eq(lat, lat2, 1e-10));
            assert!(approx_eq((lon - lon2).sin(), 0.0, 1e-10));
        }
    }
    #[test]
    fn poles_are_fixed_under_rotation() {
        let poles = [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -1.0],
        ];
    
        for p in poles {
            let p2 = ecl_to_gal(gal_to_ecl(p));
            assert!(
                vec_approx_eq(normalize(p), normalize(p2), 1e-12),
                "Pole not invariant: {:?} → {:?}", p, p2
            );
        }
    }
    
    #[test]
    fn angular_separation_is_preserved() {
        let v1 = galactic_lonlat_to_vec(10.0 * DEG2RAD, 20.0 * DEG2RAD);
        let v2 = galactic_lonlat_to_vec(80.0 * DEG2RAD, -10.0 * DEG2RAD);
    
        let sep = dot(v1, v2).acos();
    
        let v1r = gal_to_ecl(v1);
        let v2r = gal_to_ecl(v2);
    
        let sep_r = dot(v1r, v2r).acos();
    
        assert!((sep - sep_r).abs() < 1e-12);
    }

#[test]
fn gal_to_ecl_is_orthonormal() {
    let basis = [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ];

    let b: Vec<_> = basis.iter().map(|&v| gal_to_ecl(v)).collect();

    for i in 0..3 {
        assert!((dot(b[i], b[i]) - 1.0).abs() < 1e-12);
        for j in i+1..3 {
            assert!(dot(b[i], b[j]).abs() < 1e-12);
        }
    }
}

#[test]
fn galactic_equator_is_continuous_in_cartesian_space() {
    let mut last_v = None;

    for l in (0..360).step_by(5) {
        let v = galactic_lonlat_to_vec(l as f64 * DEG2RAD, 0.0);
        let v2 = normalize(gal_to_ecl(v));

        if let Some(prev) = last_v {
            let angle = dot(prev, v2).acos();
            assert!(angle < 0.2); // ~11 degrees
        }
        last_v = Some(v2);
    }
}


#[test]
fn random_round_trip_fuzz() {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..10_000 {
        let lon = rng.gen_range(0.0..2.0*PI);
        let lat = rng.gen_range(-PI/2.0..PI/2.0);

        let v = galactic_lonlat_to_vec(lon, lat);
        let v2 = ecl_to_gal(gal_to_ecl(v));

        assert!(vec_approx_eq(normalize(v), normalize(v2), 1e-10));
    }
}

}

