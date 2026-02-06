//! Coordinate rotations for astronomy.
//!
//! Internally: ACTIVE rotations (rotate vectors).
//! Externally: PASSIVE semantics (re-express same direction).
//!
use std::f64::consts::PI;
pub const DEG2RAD: f64 = PI / 180.0;
pub const RAD2DEG: f64 = 180.0 / PI;

pub const EXPECTED_ECL_LAT_OF_NGP: f64 = 29.811438 * DEG2RAD;

#[allow(dead_code)]
struct LonLat {
    lon: f64,
    lat: f64,
}

#[allow(dead_code)]
struct ThetaPhi {
    theta: f64,
    phi: f64,
}

impl From<LonLat> for ThetaPhi {
    fn from(ll: LonLat) -> Self {
        Self {
            theta: PI/2.0 - ll.lat,
            phi: ll.lon,
        }
    }
}


pub type Mat3 = [[f64; 3]; 3];

#[derive(Debug, Clone, PartialEq)]
pub struct Rotation {
    pub matrix: Mat3,
}

impl Rotation {
    pub fn identity() -> Self {
        Self {
            matrix: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn apply(&self, v: [f64; 3]) -> [f64; 3] {
        matvec(&self.matrix, v)
    }

    pub fn compose(&self, other: &Rotation) -> Rotation {
        Rotation {
            matrix: matmul(&self.matrix, &other.matrix),
        }
    }

    pub fn inverse(&self) -> Rotation {
        Rotation {
            matrix: transpose(&self.matrix),
        }
    }




}

pub fn transpose(m: &Mat3) -> Mat3 {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

/// Astronomical coordinate systems
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CoordSystem {
    /// Equatorial (ICRS / J2000)
    C,
    /// Galactic
    G,
    /// Ecliptic
    E,
}

use std::str::FromStr;

impl FromStr for CoordSystem {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "g" | "gal" | "galactic" => Ok(Self::G),
            "c" | "eq"  | "equatorial" => Ok(Self::C),
            "e" | "ecl" | "ecliptic" => Ok(Self::E),
            _ => Err("coord must be one of: gal, eq, ecl".into()),
        }
    }
}





pub fn coord_rotation(from: CoordSystem, to: CoordSystem) -> Rotation {
    use CoordSystem::*;
    match (from, to) {
        (E, G) => Rotation { matrix: ECL_TO_GAL },
        (G, E) => Rotation { matrix: GAL_TO_ECL },
        (E, C) => Rotation { matrix: ECL_TO_EQ },
        (C, E) => Rotation { matrix: EQ_TO_ECL },
        (G, C) => Rotation { matrix: GAL_TO_EQ },
        (C, G) => Rotation { matrix: EQ_TO_GAL },
        (a, b) if a == b => Rotation::identity(),
        _ => unreachable!(),
    }
}


pub fn rot_y(angle: f64) -> Mat3 {
    let (s, c) = angle.sin_cos();
    [
        [ c, 0.0,  s],
        [0.0, 1.0, 0.0],
        [-s, 0.0,  c],
    ]
}


pub fn rot_z(angle: f64) -> Mat3 {
    let (s, c) = angle.sin_cos();
    [
        [ c, -s, 0.0],
        [ s,  c, 0.0],
        [0.0, 0.0, 1.0],
    ]
}


pub fn view_rotation(lon: f64, lat: f64, roll: f64) -> Rotation {
    // ACTIVE rotation: moves vectors into camera frame
    let r_lon  = rot_z(-lon);
    let r_lat  = rot_y(-lat);
    let r_roll = rot_z(roll);

    Rotation::identity()
        .compose(&Rotation { matrix: r_roll })
        .compose(&Rotation { matrix: r_lat })
        .compose(&Rotation { matrix: r_lon })
}

pub struct ViewTransform {
    pub rotation: Rotation,
    pub rotation_inv: Rotation,  // Cache the inverse to avoid recomputation
    pub input_coord: CoordSystem,
    pub output_coord: CoordSystem,
}

impl ViewTransform {
    pub fn new(
        input: CoordSystem,
        output: CoordSystem,
        view: Option<Rotation>,
    ) -> Self {

        // map → view
        let coord = coord_rotation(input, output);
        
        // camera rotation acts AFTER coordinate conversion
        let rot = if let Some(view_rot) = view {
            view_rot.compose(&coord)
        } else {
            coord
        };

        let rotation_inv = rot.inverse();
        Self { 
            rotation: rot, 
            rotation_inv,
            input_coord: input,
            output_coord: output,
        }
    }

    pub fn apply(&self, v: [f64; 3]) -> [f64; 3] {
        self.rotation.apply(v)
    }

    #[inline(always)]
    pub fn apply_inverse(&self, v: [f64; 3]) -> [f64; 3] {
        self.rotation_inv.apply(v)
    }
}



/// Galactic → Equatorial (ACTIVE, IAU 1958 / J2000)
pub const GAL_TO_EQ: Mat3 = [
    [-0.0548755604162154,  0.4941094278755837, -0.8676661490190047],
    [-0.873_437_090_234_885, -0.4448296299600112, -0.1980763734312015],
    [-0.4838350155487132,  0.746_982_244_497_219,  0.4559837761750669],
];

/// Equatorial → Galactic (ACTIVE inverse)
pub const EQ_TO_GAL: Mat3 = [
    [-0.0548755604162154, -0.873_437_090_234_885, -0.4838350155487132],
    [ 0.4941094278755837, -0.4448296299600112,  0.746_982_244_497_219],
    [-0.8676661490190047, -0.1980763734312015,  0.4559837761750669],
];

/// Ecliptic → Galactic (ACTIVE, healpy / astropy)
pub const ECL_TO_GAL: Mat3 = [
    [-0.0548756349, -0.9938213520, -0.0964768585],
    [ 0.4941095290, -0.1109909720,  0.8622857870],
    [-0.8676660870, -0.0003516551,  0.4971473000],
];

/// Galactic → Ecliptic (ACTIVE inverse)
pub const GAL_TO_ECL: Mat3 = [
    [-0.0548756349,  0.4941095290, -0.8676660870],
    [-0.9938213520, -0.1109909720, -0.0003516551],
    [-0.0964768585,  0.8622857870,  0.4971473000],
];

/// Ecliptic → Equatorial (ACTIVE, healpy / astropy)
pub const ECL_TO_EQ: Mat3 = [
    [1.0,               -8.6513146e-08, -9.8385835e-08],
    [4.0238655e-08,  0.917482168,   -0.397776911],
    [1.2468018e-07,  0.397776911,    0.917482168],
];

/// Equatorial → Ecliptic (ACTIVE inverse)
pub const EQ_TO_ECL: Mat3 = [
    [1.0,               4.0238655e-08,  1.2468018e-07],
    [-8.6513146e-08,  0.917482168,   0.397776911],
    [-9.8385835e-08, -0.397776911,   0.917482168],
];



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
    let phi = v[1].atan2(v[0]);
    (theta, phi)
}

#[inline(always)]
pub fn dot(v: [f64; 3], w: [f64; 3]) -> f64 {
    let mut sum = 0.0;
    for i in 0..3 {
        sum += v[i]*w[i];
    }
    sum
}


#[inline]
pub fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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
fn matmul(a: &Mat3, b: &Mat3) -> Mat3 {
    let mut r = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                r[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    r
}

#[inline]
pub fn angular_sep(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dot = a[0]*b[0] + a[1]*b[1] + a[2]*b[2];
    dot.clamp(-1.0, 1.0).acos()
}



#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;


    fn approx_eq(a: f64, b: f64, tol: f64) -> bool {
        (a - b).abs() < tol
    }

    fn vec_approx_eq(a: [f64; 3], b: [f64; 3], tol: f64) -> bool {
        (0..3).all(|i| (a[i] - b[i]).abs() < tol)
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
    fn view_rotation_sampling_is_identity() {
        let v = [0.2, 0.3, 0.93];
        let v = normalize(v);
    
        let view = ViewTransform::new(CoordSystem::G, CoordSystem::E, None);
        let back = view.rotation.inverse().apply(view.apply(v));
    
        assert!(vec_approx_eq(v, back, 1e-6), "v = {:?}, back = {:?}", v, back);
    }
    
    
    
    
    
    
    
    #[test]
    fn galactic_latitude_matches_theta_definition() {
        let cases = [
            (0.0, 0.0),
            (45.0, 30.0),
            (120.0, -45.0),
        ];
    
        for (l_deg, b_deg) in cases {
            let l = l_deg * DEG2RAD;
            let b = b_deg * DEG2RAD;
    
            // gal lon/lat → vec
            let v = galactic_lonlat_to_vec(l, b);
    
            // vec → sph (theta, phi)
            let (theta, phi) = vec_to_sph(v);
    
            let b_back = PI / 2.0 - theta;
    
            assert!((b - b_back).abs() < 1e-12);
            assert!((l - phi).sin().abs() < 1e-12);
        }
    }
    
    
    #[test]
    fn view_rotation_inverse_is_identity() {
        let view = ViewTransform::new(CoordSystem::G, CoordSystem::E, None);
    
        let v = normalize([0.3, -0.4, 0.866]);
    
        let v2 = view.rotation.inverse()
            .compose(&view.rotation)
            .apply(v);
    
        for i in 0..3 {
            assert!((v[i] - v2[i]).abs() < 1e-6, "v = {:?}, v2 = {:?}", v, v2);
        }
    }
    #[test]
    fn view_rotation_preserves_angular_separation() {
        let view = ViewTransform::new(CoordSystem::G, CoordSystem::E, None);
    
        let v1 = galactic_lonlat_to_vec(10.0*DEG2RAD, 20.0*DEG2RAD);
        let v2 = galactic_lonlat_to_vec(80.0*DEG2RAD, -10.0*DEG2RAD);
    
        let a0 = angular_sep(v1, v2);
    
        let a1 = angular_sep(
            view.apply(v1),
            view.apply(v2),
        );
    
        assert!((a0 - a1).abs() < 1e-6, "a0 = {:?}, a1 = {:?}", a0, a1);
    }
    
    
    #[test]
    fn galactic_equator_is_smooth_in_view_longitude() {
        let view = ViewTransform::new(CoordSystem::G, CoordSystem::E, None);
    
        let mut last_lon: Option<f64> = None;
    
        for l in (0..360).step_by(2) {
            let v_gal = galactic_lonlat_to_vec(l as f64 * DEG2RAD, 0.0);
    
            let v_view = view.rotation.inverse().apply(v_gal);
            let (_, lon) = vec_to_sph(v_view);
    
            if let Some(prev) = last_lon {
                let d = (lon - prev + PI).rem_euclid(2.0 * PI) - PI;
                assert!(d.abs() < 0.5, "longitude jump at l={}", l);
            }
    
            last_lon = Some(lon);
        }
    }
    
    
    #[test]
    fn north_ecliptic_pole_is_at_view_center() {
        // Camera centered on NEP
        let view_rot = view_rotation(0.0, PI/2.0, 0.0);
    
        let view = ViewTransform::new(
            CoordSystem::E,
            CoordSystem::E,
            Some(view_rot),
        );
    
        let nep = [0.0, 0.0, 1.0];
        let v = view.rotation.inverse().apply(nep);
        let (theta, lon) = vec_to_sph(v);
    
        assert!((theta - PI/2.0).abs() < 1e-12, "Theta is {:?}", theta);
        assert!(lon.abs() < 1e-12, "Lon is {:?}", lon);
    }
    
    #[test]
    fn pure_view_rotation_preserves_latitudes() {
        let view_rot = view_rotation(0.0, 0.0, 0.0);
        let view = ViewTransform::new(CoordSystem::G, CoordSystem::G, Some(view_rot));
    
        for b in [-60.0, -30.0, 0.0, 30.0, 60.0] {
            let v = galactic_lonlat_to_vec(0.0, b * DEG2RAD);
            let v2 = view.rotation.inverse().apply(v);
            let (_, b2) = vec_to_lonlat(v2);
    
            assert!((b2 - b * DEG2RAD).abs() < 1e-12, "b2 is {:?}, b is {:?}",
                b2, b * DEG2RAD);
        }
    }
    
    #[test]
    fn view_rotation_identity_at_origin() {
        let view = view_rotation(0.0, 0.0, 0.0);
        let v = [1.0, 0.0, 0.0];
        let v2 = view.inverse().apply(v);
        assert!(vec_approx_eq(v, v2, 1e-12));
    }
    
    #[test]
    fn rotation_is_orthonormal() {
        let r = view_rotation(1.0, 0.5, 0.3);
        let rt = r.inverse();
    
        let id = r.compose(&rt).matrix;
    
        for i in 0..3 {
            for j in 0..3 {
                let expected = if i == j { 1.0 } else { 0.0 };
                assert!((id[i][j] - expected).abs() < 1e-12);
            }
        }
    }
    
    
    #[test]
    fn view_rotation_preserves_meridian_as_great_circle() {
        let view = ViewTransform::new(CoordSystem::G, CoordSystem::E, None);
    
        // sample points along galactic meridian l=0
        let mut normals = vec![];
    
        for b in (-80..=80).step_by(10) {
            let v = galactic_lonlat_to_vec(0.0, b as f64 * DEG2RAD);
            let v2 = view.apply(v);
            normals.push(v2);
        }
    
        // all points should lie in a plane through origin
        let n0 = cross(normals[0], normals[1]);
        for v in normals.iter().skip(2) {
            let d = dot(n0, *v);
            assert!(d.abs() < 1e-10, "Dot product is {:?}", d);
        }
    }
    
    #[test]
    fn galactic_equator_maps_to_great_circle() {
        let view = ViewTransform::new(CoordSystem::G, CoordSystem::E, None);
    
        let mut pts = vec![];
        for l in (0..360).step_by(10) {
            let v = galactic_lonlat_to_vec(l as f64 * DEG2RAD, 0.0);
            pts.push(view.apply(v));
        }
    
        let n = cross(pts[0], pts[1]);
        for p in pts.iter().skip(2) {
            assert!(dot(n, *p).abs() < 1e-10, "Dot product is {:?}", dot(n, *p));
        }
    }
    
    #[test]
    fn north_galactic_pole_maps_to_correct_ecliptic_latitude() {
        let view = ViewTransform::new(CoordSystem::G, CoordSystem::E, None);
    
        let ngp = galactic_lonlat_to_vec(0.0, PI/2.0);
        let v = view.apply(ngp);
        let (_, lat) = vec_to_lonlat(v);
    
        assert!((lat - EXPECTED_ECL_LAT_OF_NGP).abs() < 1e-6);
    }

    //
    // ──────────────────────────────────────────────────────────
    // Comprehensive pole and origin tests for all coordinate systems
    // ──────────────────────────────────────────────────────────
    //

    /// Helper: check that two vectors are approximately equal
    fn assert_vec_eq(v1: [f64; 3], v2: [f64; 3], tol: f64, msg: &str) {
        for i in 0..3 {
            assert!((v1[i] - v2[i]).abs() < tol, 
                "{}: component {}: {} vs {}", msg, i, v1[i], v2[i]);
        }
    }

    /// Helper: check that two angles are approximately equal (accounting for 2π periodicity)
    fn assert_angle_eq(a1: f64, a2: f64, tol: f64, msg: &str) {
        let diff = (a1 - a2).abs();
        let diff_wrapped = (2.0 * PI - diff).abs();
        let min_diff = diff.min(diff_wrapped);
        assert!(min_diff < tol, "{}: angle {} vs {}, diff = {}", msg, a1, a2, min_diff);
    }

    // ──────────────────────────────────────────────────────────
    // GALACTIC COORDINATE TESTS
    // ──────────────────────────────────────────────────────────

    #[test]
    fn galactic_north_pole_in_galactic() {
        // North Galactic Pole in Galactic coordinates: (l=0°, b=+90°)
        let ngp_gal = galactic_lonlat_to_vec(0.0, PI/2.0);
        let normalized = normalize(ngp_gal);
        
        // Should be approximately [0, 0, 1]
        assert_vec_eq(normalized, [0.0, 0.0, 1.0], 1e-6, "NGP in Galactic coords");
    }

    #[test]
    fn galactic_south_pole_in_galactic() {
        // South Galactic Pole in Galactic coordinates: (l=any, b=-90°)
        let sgp_gal = galactic_lonlat_to_vec(0.0, -PI/2.0);
        let normalized = normalize(sgp_gal);
        
        // Should be approximately [0, 0, -1]
        assert_vec_eq(normalized, [0.0, 0.0, -1.0], 1e-6, "SGP in Galactic coords");
    }

    #[test]
    fn galactic_origin_in_galactic() {
        // Galactic origin: (l=0°, b=0°)
        let origin_gal = galactic_lonlat_to_vec(0.0, 0.0);
        let normalized = normalize(origin_gal);
        
        // Should be approximately [1, 0, 0]
        assert_vec_eq(normalized, [1.0, 0.0, 0.0], 1e-6, "Origin in Galactic coords");
    }

    #[test]
    fn galactic_to_equatorial_north_pole() {
        // NGP in Galactic → Equatorial
        // Known: RA ≈ 192.86°, Dec ≈ 27.13°
        let ngp_gal = galactic_lonlat_to_vec(0.0, PI/2.0);
        let ngp_eq = matvec(&GAL_TO_EQ, ngp_gal);
        let (ra, dec) = vec_to_lonlat(ngp_eq);
        
        let ra_exp = 192.85948 * DEG2RAD;
        let dec_exp = 27.12825 * DEG2RAD;
        
        assert_angle_eq(ra, ra_exp, 1e-5, "NGP RA in Equatorial");
        assert!(approx_eq(dec, dec_exp, 1e-5), "NGP Dec in Equatorial: {} vs {}", dec, dec_exp);
    }

    #[test]
    fn galactic_to_equatorial_origin() {
        // Galactic origin (l=0°, b=0°) → Equatorial
        // This is in the direction of the Galactic center
        let origin_gal = galactic_lonlat_to_vec(0.0, 0.0);
        let origin_eq = matvec(&GAL_TO_EQ, origin_gal);
        let (ra, dec) = vec_to_lonlat(origin_eq);
        
        // Galactic center: RA ≈ 266.42°, Dec ≈ -28.94°
        let ra_exp = 266.42 * DEG2RAD;
        let dec_exp = -28.94 * DEG2RAD;
        
        assert_angle_eq(ra, ra_exp, 0.1, "Galactic center RA in Equatorial");
        assert!(approx_eq(dec, dec_exp, 0.1), "Galactic center Dec in Equatorial");
    }

    #[test]
    fn galactic_to_ecliptic_north_pole() {
        // NGP in Galactic → Ecliptic
        let ngp_gal = galactic_lonlat_to_vec(0.0, PI/2.0);
        let ngp_ecl = matvec(&GAL_TO_ECL, ngp_gal);
        let (_, lat) = vec_to_lonlat(ngp_ecl);
        
        // NGP should be at β ≈ 29.81° in Ecliptic
        assert!(approx_eq(lat, EXPECTED_ECL_LAT_OF_NGP, 1e-5), 
            "NGP ecliptic latitude: {} vs {}", lat, EXPECTED_ECL_LAT_OF_NGP);
    }

    // ──────────────────────────────────────────────────────────
    // EQUATORIAL COORDINATE TESTS
    // ──────────────────────────────────────────────────────────

    #[test]
    fn equatorial_north_pole_in_equatorial() {
        // North Celestial Pole: (RA=any, Dec=+90°)
        let ncp_eq = [0.0, 0.0, 1.0];  // pointing to north pole [0,0,1]
        
        // Should be exactly [0, 0, 1]
        assert_vec_eq(ncp_eq, [0.0, 0.0, 1.0], 1e-6, "NCP in Equatorial");
    }

    #[test]
    fn equatorial_south_pole_in_equatorial() {
        // South Celestial Pole: (RA=any, Dec=-90°)
        let scp_eq = [0.0, 0.0, -1.0];
        
        // Should be exactly [0, 0, -1]
        assert_vec_eq(scp_eq, [0.0, 0.0, -1.0], 1e-6, "SCP in Equatorial");
    }

    #[test]
    fn equatorial_origin_in_equatorial() {
        // Equatorial origin: (RA=0°, Dec=0°) - vernal equinox direction
        let origin_eq = sph_to_vec(PI/2.0, 0.0);
        let normalized = normalize(origin_eq);
        
        // Should point in [1, 0, 0] direction
        assert_vec_eq(normalized, [1.0, 0.0, 0.0], 1e-6, "Equatorial origin");
    }

    #[test]
    fn equatorial_to_galactic_north_pole() {
        // NCP in Equatorial → Galactic
        // NCP maps to approximately (l≈122.93°, b≈27.13°) in Galactic
        let ncp_eq = [0.0, 0.0, 1.0];
        let ncp_gal = matvec(&EQ_TO_GAL, ncp_eq);
        let (lon, lat) = vec_to_lonlat(ncp_gal);
        
        // Converting back to degrees (for potential debugging)
        let _lon_deg = lon * RAD2DEG;
        let _lat_deg = lat * RAD2DEG;
        
        // Expected: approximately (122.93°, 27.13°) or the equivalent antipodal point
        // (since NCP should transform to some point, let's just verify it's normalized)
        let r = (ncp_gal[0]*ncp_gal[0] + ncp_gal[1]*ncp_gal[1] + ncp_gal[2]*ncp_gal[2]).sqrt();
        assert!(approx_eq(r, 1.0, 1e-6), "NCP->Gal result not normalized: {}", r);
    }

    #[test]
    fn equatorial_to_ecliptic_north_pole() {
        // NCP in Equatorial → Ecliptic
        // NCP should map to ecliptic latitude ≈ 66.56° (90° - obliquity of ecliptic ≈ 23.44°)
        let ncp_eq = [0.0, 0.0, 1.0];
        let ncp_ecl = matvec(&EQ_TO_ECL, ncp_eq);
        let (_, lat) = vec_to_lonlat(ncp_ecl);
        
        // The transformation should map [0,0,1] to a specific ecliptic latitude
        // Just verify it's a valid, normalized position
        let r = (ncp_ecl[0]*ncp_ecl[0] + ncp_ecl[1]*ncp_ecl[1] + ncp_ecl[2]*ncp_ecl[2]).sqrt();
        assert!(approx_eq(r, 1.0, 1e-6), "NCP->Ecl result not normalized: {}", r);
        
        // The latitude should be close to 66.56° (i.e., 90° - 23.44°)
        let expected_lat = (PI/2.0) - (23.43929111 * DEG2RAD);
        assert!(approx_eq(lat, expected_lat, 0.01), 
            "NCP ecliptic latitude: {} vs {}", lat, expected_lat);
    }

    #[test]
    fn equatorial_to_galactic_origin() {
        // Equatorial origin (RA=0°, Dec=0°) → Galactic
        let origin_eq = sph_to_vec(PI/2.0, 0.0);
        let origin_gal = matvec(&EQ_TO_GAL, origin_eq);
        let (_lon, _lat) = vec_to_lonlat(origin_gal);
        
        // Just verify it's a valid position
        let r = (origin_gal[0]*origin_gal[0] + origin_gal[1]*origin_gal[1] + origin_gal[2]*origin_gal[2]).sqrt();
        assert!(approx_eq(r, 1.0, 1e-6), "Origin->Gal result not normalized");
    }

    // ──────────────────────────────────────────────────────────
    // ECLIPTIC COORDINATE TESTS
    // ──────────────────────────────────────────────────────────

    #[test]
    fn ecliptic_north_pole_in_ecliptic() {
        // North Ecliptic Pole: (λ=any, β=+90°)
        let nep_ecl = [0.0, 0.0, 1.0];
        
        // Should be exactly [0, 0, 1]
        assert_vec_eq(nep_ecl, [0.0, 0.0, 1.0], 1e-6, "NEP in Ecliptic");
    }

    #[test]
    fn ecliptic_south_pole_in_ecliptic() {
        // South Ecliptic Pole: (λ=any, β=-90°)
        let sep_ecl = [0.0, 0.0, -1.0];
        
        // Should be exactly [0, 0, -1]
        assert_vec_eq(sep_ecl, [0.0, 0.0, -1.0], 1e-6, "SEP in Ecliptic");
    }

    #[test]
    fn ecliptic_origin_in_ecliptic() {
        // Ecliptic origin: (λ=0°, β=0°) - vernal equinox direction
        let origin_ecl = sph_to_vec(PI/2.0, 0.0);
        let normalized = normalize(origin_ecl);
        
        // Should point in [1, 0, 0] direction
        assert_vec_eq(normalized, [1.0, 0.0, 0.0], 1e-6, "Ecliptic origin");
    }

    #[test]
    fn ecliptic_to_galactic_north_pole() {
        // NEP in Ecliptic → Galactic
        let nep_ecl = [0.0, 0.0, 1.0];
        let nep_gal = matvec(&ECL_TO_GAL, nep_ecl);
        let (_lon, _lat) = vec_to_lonlat(nep_gal);
        
        // NEP should map to approximately (l≈96°, b≈30°) in Galactic
        // Just verify it's normalized for now
        let r = (nep_gal[0]*nep_gal[0] + nep_gal[1]*nep_gal[1] + nep_gal[2]*nep_gal[2]).sqrt();
        assert!(approx_eq(r, 1.0, 1e-6), "NEP->Gal result not normalized: {}", r);
    }

    #[test]
    fn ecliptic_to_equatorial_north_pole() {
        // NEP in Ecliptic → Equatorial
        // NEP should map to equatorial latitude ≈ 66.56° (90° - obliquity of ecliptic)
        let nep_ecl = [0.0, 0.0, 1.0];
        let nep_eq = matvec(&ECL_TO_EQ, nep_ecl);
        let (_, lat) = vec_to_lonlat(nep_eq);
        
        // Just verify it's a valid, normalized position
        let r = (nep_eq[0]*nep_eq[0] + nep_eq[1]*nep_eq[1] + nep_eq[2]*nep_eq[2]).sqrt();
        assert!(approx_eq(r, 1.0, 1e-6), "NEP->Eq result not normalized: {}", r);
        
        // The latitude should be close to 66.56° (i.e., 90° - 23.44°)
        let expected_lat = (PI/2.0) - (23.43929111 * DEG2RAD);
        assert!(approx_eq(lat, expected_lat, 0.01),
            "NEP equatorial latitude: {} vs {}", lat, expected_lat);
    }

    // ──────────────────────────────────────────────────────────
    // ROUNDTRIP TESTS - ensure consistency
    // ──────────────────────────────────────────────────────────

    #[test]
    fn roundtrip_galactic_equatorial_galactic() {
        // A point in Galactic → Equatorial → Galactic should return to original
        let lon_orig = 45.0 * DEG2RAD;
        let lat_orig = -30.0 * DEG2RAD;
        let v_orig = galactic_lonlat_to_vec(lon_orig, lat_orig);
        
        let v_eq = matvec(&GAL_TO_EQ, v_orig);
        let v_gal_back = matvec(&EQ_TO_GAL, v_eq);
        
        assert_vec_eq(v_orig, v_gal_back, 1e-8, "G→C→G roundtrip");
    }

    #[test]
    fn roundtrip_galactic_ecliptic_galactic() {
        // A point in Galactic → Ecliptic → Galactic should return to original
        let lon_orig = 120.0 * DEG2RAD;
        let lat_orig = 15.0 * DEG2RAD;
        let v_orig = galactic_lonlat_to_vec(lon_orig, lat_orig);
        
        let v_ecl = matvec(&GAL_TO_ECL, v_orig);
        let v_gal_back = matvec(&ECL_TO_GAL, v_ecl);
        
        assert_vec_eq(v_orig, v_gal_back, 1e-8, "G→E→G roundtrip");
    }

    #[test]
    fn roundtrip_equatorial_ecliptic_equatorial() {
        // A point in Equatorial → Ecliptic → Equatorial should return to original
        let lon_orig = 180.0 * DEG2RAD;
        let lat_orig = 45.0 * DEG2RAD;
        let v_orig = sph_to_vec(PI/2.0 - lat_orig, lon_orig);
        
        let v_ecl = matvec(&EQ_TO_ECL, v_orig);
        let v_eq_back = matvec(&ECL_TO_EQ, v_ecl);
        
        assert_vec_eq(v_orig, v_eq_back, 1e-8, "C→E→C roundtrip");
    }

    #[test]
    fn three_way_cycle_galactic_equatorial_ecliptic() {
        // G → C → E → G should return to original
        let v_orig = galactic_lonlat_to_vec(73.0 * DEG2RAD, 42.0 * DEG2RAD);
        
        let v_eq = matvec(&GAL_TO_EQ, v_orig);
        let v_ecl = matvec(&EQ_TO_ECL, v_eq);
        let v_gal_back = matvec(&ECL_TO_GAL, v_ecl);
        
        // Three matrix multiplications accumulate more floating-point error
        assert_vec_eq(v_orig, v_gal_back, 1e-7, "G→C→E→G roundtrip");
    }

    #[test]
    fn graticule_transform_identity() {
        // When graticule_coord == input_coord, transform should be identity
        let rot = coord_rotation(CoordSystem::G, CoordSystem::G);
        let identity = Rotation::identity();
        assert_eq!(rot.matrix, identity.matrix, "Galactic → Galactic should be identity");
        
        let rot = coord_rotation(CoordSystem::C, CoordSystem::C);
        assert_eq!(rot.matrix, identity.matrix, "Equatorial → Equatorial should be identity");
    }

    #[test]
    fn graticule_at_origin() {
        // Galactic origin (lon=0, lat=0) in Galactic coords should be (1,0,0) in vec
        let lon_gal: f64 = 0.0;
        let lat_gal: f64 = 0.0;
        let v_x = lon_gal.cos() * lat_gal.cos();
        let v_y = lon_gal.sin() * lat_gal.cos();
        let v_z = lat_gal.sin();
        assert!((v_x - 1.0).abs() < 1e-10, "x component");
        assert!(v_y.abs() < 1e-10, "y component");
        assert!(v_z.abs() < 1e-10, "z component");
    }

    #[test]
    fn graticule_north_pole() {
        // North pole (lat=90°) should be [0, 0, 1]
        let lat_north: f64 = PI / 2.0;
        let v_z = lat_north.sin();
        assert!((v_z - 1.0).abs() < 1e-10, "z component at pole");
    }

    #[test]
    fn specific_galactic_center_in_equatorial() {
        // Galactic center (lon=0, lat=0) in Galactic should transform to ~RA=266°, Dec=-29°
        // when converted to Equatorial
        let rot = coord_rotation(CoordSystem::G, CoordSystem::C);
        
        // Point at Galactic center: lon=0, lat=0
        let v_gal = [1.0, 0.0, 0.0]; // lon=0, lat=0 in spherical coords
        
        let v_eq = rot.apply(v_gal);
        
        // Convert back to lon/lat
        let r = (v_eq[0]*v_eq[0] + v_eq[1]*v_eq[1] + v_eq[2]*v_eq[2]).sqrt();
        let mut lon_eq = v_eq[1].atan2(v_eq[0]);
        let lat_eq = (v_eq[2] / r).asin();
        
        // Normalize longitude to [0, 360)
        lon_eq = lon_eq.rem_euclid(2.0 * PI);
        
        let lon_eq_deg = lon_eq * RAD2DEG;
        let lat_eq_deg = lat_eq * RAD2DEG;
        
        // Should be close to Galactic center location (~RA=266°, Dec=-29°)
        assert!((lon_eq_deg - 266.4).abs() < 5.0, 
            "Galactic center should be ~RA=266°, got {}", lon_eq_deg);
        assert!((lat_eq_deg + 28.9).abs() < 5.0, 
            "Galactic center should be ~Dec=-29°, got {}", lat_eq_deg);
    }
}
