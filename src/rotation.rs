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
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_ascii_uppercase().as_str() {
            "C" => Some(CoordSystem::C),
            "G" => Some(CoordSystem::G),
            "E" => Some(CoordSystem::E),
            _ => None,
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Rotation {
    pub from: CoordSystem,
    pub to: CoordSystem,
}

impl Rotation {
    pub fn new(from: CoordSystem, to: CoordSystem) -> Option<Self> {
        if from == to {
            None // identity rotation → treated as no-op
        } else {
            Some(Self { from, to })
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        let s = s.to_ascii_uppercase();
        if s.len() != 2 {
            return None;
        }
        let from = CoordSystem::parse(&s[0..1])?;
        let to   = CoordSystem::parse(&s[1..2])?;
        Self::new(from, to)
    }
}



pub type Mat3 = [[f64; 3]; 3];

/// Obliquity of the ecliptic (J2000), radians
const OBLIQUITY: f64 = 23.439291111_f64.to_radians();

const GAL_TO_EQ: Mat3 = [
    [-0.0548755604, -0.8734370902, -0.4838350155],
    [ 0.4941094279, -0.4448296300,  0.7469822445],
    [-0.8676661490, -0.1980763734,  0.4559837762],
];

const EQ_TO_GAL: Mat3 = [
    [-0.0548755604,  0.4941094279, -0.8676661490],
    [-0.8734370902, -0.4448296300, -0.1980763734],
    [-0.4838350155,  0.7469822445,  0.4559837762],
];

pub const GAL_CENTER: [f64; 3] = [1.0, 0.0, 0.0];

/// North Galactic Pole, equatorial J2000
pub const NGP_EQ: [f64; 3] = [
    -0.8676661490,
    -0.1980763734,
     0.4559837762,
];

pub const NGP_ECL: [f64; 3] = [
    -0.8676661490,
    -0.4927284661,
     0.0669887394,
];


/// ℓ=90°, b=0°
pub const GAL_L90: [f64; 3] = [0.0, 1.0, 0.0];

/// b=+90°
pub const GAL_NGP: [f64; 3] = [0.0, 0.0, 1.0];

pub const ECL_NORTH: [f64; 3] = [0.0, 0.0, 1.0];

pub const ECL_EQUINOX: [f64; 3] = [1.0, 0.0, 0.0];

pub const GAL_CENTER_EQ: [f64; 3] = [
    -0.054876,   // x
     0.494110,   // y
    -0.867666,   // z
];

/// Galactic center, ecliptic J2000 (unit vector)
pub const GAL_CENTER_ECL: [f64; 3] = [
    -0.0548755604,
     0.1081987638,
    -0.9926135705,
];





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

fn matvec(m: &Mat3, v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0]*v[0] + m[0][1]*v[1] + m[0][2]*v[2],
        m[1][0]*v[0] + m[1][1]*v[1] + m[1][2]*v[2],
        m[2][0]*v[0] + m[2][1]*v[1] + m[2][2]*v[2],
    ]
}


fn sph_to_cart(theta: f64, phi: f64) -> [f64; 3] {
    let st = theta.sin();
    [
        st * phi.cos(),
        st * phi.sin(),
        theta.cos(),
    ]
}

fn cart_to_sph(v: [f64; 3]) -> (f64, f64) {
    let r = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
    let theta = (v[2] / r).acos();
    let phi = v[1].atan2(v[0]);
    (theta, phi)
}




fn rotation_matrix(from: CoordSystem, to: CoordSystem) -> Mat3 {
    use CoordSystem::*;
    match (from, to) {
        (C, G) => EQ_TO_GAL,
        (G, C) => GAL_TO_EQ,
        (C, E) => eq_to_ecl(),
        (E, C) => ecl_to_eq(),
        (G, E) => matmul(&eq_to_ecl(), &GAL_TO_EQ),
        (E, G) => matmul(&EQ_TO_GAL, &ecl_to_eq()),
        _ => [[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]],
    }
}


pub fn rotate_theta_phi(
    theta: f64,
    phi: f64,
    from: CoordSystem,
    to: CoordSystem,
) -> (f64, f64) {
    if from == to {
        return (theta, phi);
    }

    let v = sph_to_cart(theta, phi);
    let m = rotation_matrix(from, to);
    let v2 = matvec(&m, v);
    cart_to_sph(v2)
}


fn eq_to_ecl() -> Mat3 {
    let eps = OBLIQUITY;
    let (s, c) = eps.sin_cos();

    [
        [1.0,  0.0,  0.0],
        [0.0,   c,   s],
        [0.0,  -s,   c],
    ]
}

fn ecl_to_eq() -> Mat3 {
    let eps = OBLIQUITY;
    let (s, c) = eps.sin_cos();

    [
        [1.0,  0.0,  0.0],
        [0.0,   c,  -s],
        [0.0,   s,   c],
    ]
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

#[inline(always)]
pub fn mat_vec(m: &[[f64; 3]; 3], v: [f64; 3]) -> [f64; 3] {
    [
        m[0][0]*v[0] + m[0][1]*v[1] + m[0][2]*v[2],
        m[1][0]*v[0] + m[1][1]*v[1] + m[1][2]*v[2],
        m[2][0]*v[0] + m[2][1]*v[1] + m[2][2]*v[2],
    ]
}

pub fn identity() -> [[f64; 3]; 3] {
    [
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0],
    ]
}

#[inline(always)]
pub fn gal_to_ecl_dummy(v: [f64; 3]) -> [f64; 3] {
    // Dummy tilt: ~60 degrees (NOT real gal→ecl)
    let eps = 60_f64.to_radians();
    let (s, c) = eps.sin_cos();

    [
        v[0],
        c * v[1] - s * v[2],
        s * v[1] + c * v[2],
    ]
}

#[inline(always)]
pub fn gal_to_ecl(v: [f64; 3]) -> [f64; 3] {
    let veq = mat_vec(&GAL_TO_EQ, v);
    mat_vec(&eq_to_ecl(), veq)
}

#[inline(always)]
pub fn ecl_to_gal(v: [f64; 3]) -> [f64; 3] {
    let veq = mat_vec(&ecl_to_eq(), v);
    mat_vec(&EQ_TO_GAL, veq)
}



#[inline(always)]
pub fn eq_to_ecl_vec(v: [f64; 3]) -> [f64; 3] {
    mat_vec(&eq_to_ecl(), v)
}

#[inline(always)]
pub fn galactic_lonlat_to_vec(l_rad: f64, b_rad: f64) -> [f64; 3] {
    let theta = std::f64::consts::FRAC_PI_2 - b_rad;
    let phi = l_rad;
    sph_to_vec(theta, phi)
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    const TOL: f64 = 1e-9;
    const DEG2RAD: f64 = PI / 180.0;

    fn vec_approx_eq(v1: [f64; 3], v2: [f64; 3], tol: f64) -> bool {
        (0..3).all(|i| (v1[i] - v2[i]).abs() < tol)
    }

    fn normalize(v: [f64; 3]) -> [f64; 3] {
        let norm = (v[0]*v[0] + v[1]*v[1] + v[2]*v[2]).sqrt();
        [v[0]/norm, v[1]/norm, v[2]/norm]
    }

    fn sph_to_vec(theta: f64, phi: f64) -> [f64; 3] {
        let st = theta.sin();
        [st * phi.cos(), st * phi.sin(), theta.cos()]
    }

    #[test]
    fn galactic_center_to_ecliptic_round_trip() {
        // Galactic center: l = 0°, b = 0°
        let theta = PI / 2.0; // b = 0°
        let phi = 0.0;        // l = 0°
        let v_gal = sph_to_vec(theta, phi);

        let v_ecl = normalize(gal_to_ecl(v_gal));
        let v_gal_round = normalize(ecl_to_gal(v_ecl));

        // Golden vector in Galactic coords: [1,0,0] → round-trip should recover it
        assert!(vec_approx_eq(v_gal, v_gal_round, TOL),
            "Galactic center round-trip failed: v_gal_round = {:?}", v_gal_round);
    }

    #[test]
    fn galactic_plane_l90_to_ecliptic_round_trip() {
        // Galactic plane l=90°, b=0°
        let theta = PI / 2.0;          // b = 0°
        let phi = 90.0 * DEG2RAD;      // l = 90°
        let v_gal = sph_to_vec(theta, phi);

        let v_ecl = normalize(gal_to_ecl(v_gal));
        let v_gal_round = normalize(ecl_to_gal(v_ecl));

        // Golden vector: [0,1,0]
        assert!(vec_approx_eq(v_gal, v_gal_round, TOL),
            "Galactic plane l=90° round-trip failed: v_gal_round = {:?}", v_gal_round);
    }

    #[test]
    fn north_galactic_pole_to_ecliptic_round_trip() {
        // North Galactic Pole: b = +90°, l = anything (phi doesn't matter)
        let theta = 0.0; // b = +90° => theta = 0
        let phi = 0.0;
        let v_gal = sph_to_vec(theta, phi);

        let v_ecl = normalize(gal_to_ecl(v_gal));
        let v_gal_round = normalize(ecl_to_gal(v_ecl));

        // Golden vector: [0,0,1]
        assert!(vec_approx_eq(v_gal, v_gal_round, TOL),
            "NGP round-trip failed: v_gal_round = {:?}", v_gal_round);
    }

    #[test]
    fn ecliptic_to_galactic_and_back() {
        // Pick a few test points in ecliptic coords
        let test_points = [
            (0.0, 0.0),           // λ=0°, β=0°
            (90.0, 0.0),          // λ=90°, β=0°
            (180.0, 0.0),         // λ=180°, β=0°
            (0.0, 90.0),          // North ecliptic pole
            (0.0, -90.0),         // South ecliptic pole
        ];

        for (lon_deg, lat_deg) in test_points.iter() {
            let theta = PI/2.0 - (*lat_deg) * DEG2RAD; // θ = π/2 - β
            let phi = (*lon_deg) * DEG2RAD;
            let v_ecl = sph_to_vec(theta, phi);

            let v_gal = normalize(ecl_to_gal(v_ecl));
            let v_ecl_round = normalize(gal_to_ecl(v_gal));

            assert!(vec_approx_eq(v_ecl, v_ecl_round, TOL),
                "Ecliptic round-trip failed at λ={}°, β={}°: v_ecl_round = {:?}", lon_deg, lat_deg, v_ecl_round);
        }
    }


    #[test]
    fn galactic_center_round_trip() {
        let v_gal = [1.0, 0.0, 0.0]; // l=0°, b=0°
        let v_ecl = normalize(gal_to_ecl(v_gal));
        let v_gal_round = normalize(ecl_to_gal(v_ecl));
        assert!(vec_approx_eq(v_gal, v_gal_round, TOL),
            "Galactic center round-trip failed: {:?}", v_gal_round);
    }

    #[test]
    fn galactic_plane_l90_round_trip() {
        let v_gal = [0.0, 1.0, 0.0]; // l=90°, b=0°
        let v_ecl = normalize(gal_to_ecl(v_gal));
        let v_gal_round = normalize(ecl_to_gal(v_ecl));
        assert!(vec_approx_eq(v_gal, v_gal_round, TOL),
            "Galactic plane l=90° round-trip failed: {:?}", v_gal_round);
    }

    #[test]
    fn ecliptic_to_galactic_round_trip() {
        // Pick several points in ecliptic coords (λ,β) in degrees
        let test_points = [
            (0.0, 0.0),
            (90.0, 0.0),
            (180.0, 0.0),
            (0.0, 90.0),
            (0.0, -90.0),
        ];

        for &(lon_deg, lat_deg) in &test_points {
            let theta = PI/2.0 - lat_deg * DEG2RAD; // θ = π/2 - β
            let phi = lon_deg * DEG2RAD;
            let v_ecl = sph_to_vec(theta, phi);

            let v_gal = normalize(ecl_to_gal(v_ecl));
            let v_ecl_round = normalize(gal_to_ecl(v_gal));

            assert!(vec_approx_eq(v_ecl, v_ecl_round, TOL),
                "Ecliptic round-trip failed at λ={}°, β={}°: {:?}", lon_deg, lat_deg, v_ecl_round);
        }
    }


    #[test]
    fn galactic_center_matches_literature_ecliptic() {
        let v = normalize(gal_to_ecl(GAL_CENTER));
        let g = normalize(GAL_CENTER_ECL);
    
        assert!(
            vec_approx_eq(v, g, 1e-12),
            "GC → ecliptic mismatch: {:?} vs {:?}", v, g
        );
    }

    #[test]
    fn galactic_center_matches_literature_equatorial() {
        let v_eq = normalize(matvec(&GAL_TO_EQ, GAL_CENTER));
        let g_eq = normalize(GAL_CENTER_EQ);
    
        assert!(vec_approx_eq(v_eq, g_eq, 1e-12),
            "GC -> equatorial mismatch: {:?} vs {:?}", v_eq, g_eq
            );
    }

#[test]
fn north_galactic_pole_matches_equatorial() {
    let v_eq = normalize(matvec(&GAL_TO_EQ, [0.0, 0.0, 1.0]));
    let g_eq = normalize(NGP_EQ);

    assert!(
        vec_approx_eq(v_eq, g_eq, 1e-6),
        "NGP → equatorial mismatch: {:?} vs {:?}", v_eq, g_eq
    );
}

#[test]
fn north_galactic_pole_matches_ecliptic() {
    let v_ecl = normalize(gal_to_ecl([0.0, 0.0, 1.0]));
    let g_ecl = normalize(NGP_ECL);

    assert!(
        vec_approx_eq(v_ecl, g_ecl, 1e-6),
        "NGP → ecliptic mismatch: {:?} vs {:?}", v_ecl, g_ecl
    );
}

#[test]
fn north_galactic_pole_round_trip() {
    let v = [0.0, 0.0, 1.0];
    let v2 = normalize(ecl_to_gal(gal_to_ecl(v)));

    assert!(
        vec_approx_eq(v, v2, 1e-12),
        "NGP round-trip failed: {:?} vs {:?}", v, v2
    );
}


}

