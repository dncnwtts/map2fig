//! Coordinate rotations for astronomy.
//!
//! Internally: ACTIVE rotations (rotate vectors).
//! Externally: PASSIVE semantics (re-express same direction).
//!
use std::f64::consts::PI;
pub const DEG2RAD: f64 = PI / 180.0;
pub const RAD2DEG: f64 = 180.0 / PI;

pub const EXPECTED_ECL_LAT_OF_NGP: f64 = 29.811438 * DEG2RAD;

struct LonLat {
    lon: f64,
    lat: f64,
}

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

impl CoordSystem {
    pub fn from_str(s: &str) -> Result<Self, String> {
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
        Self { rotation: rot, rotation_inv }
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
    [-0.8734370902348850, -0.4448296299600112, -0.1980763734312015],
    [-0.4838350155487132,  0.7469822444972189,  0.4559837761750669],
];

/// Equatorial → Galactic (ACTIVE inverse)
pub const EQ_TO_GAL: Mat3 = [
    [-0.0548755604162154, -0.8734370902348850, -0.4838350155487132],
    [ 0.4941094278755837, -0.4448296299600112,  0.7469822444972189],
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


}
