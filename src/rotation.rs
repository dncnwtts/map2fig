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


const EQ_TO_GAL: Mat3 = [
    [-0.0548755604, -0.8734370902, -0.4838350155],
    [ 0.4941094279, -0.4448296300,  0.7469822445],
    [-0.8676661490, -0.1980763734,  0.4559837762],
];

const GAL_TO_EQ: Mat3 = [
    [-0.0548755604,  0.4941094279, -0.8676661490],
    [-0.8734370902, -0.4448296300, -0.1980763734],
    [-0.4838350155,  0.7469822445,  0.4559837762],
];


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


#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        let diff = (a - b).abs();
        diff < 1e-12 || diff < 1e-9 * b.abs().max(1.0)
    }


    fn vec_approx_eq(a: [f64; 3], b: [f64; 3]) -> bool {
        (0..3).all(|i| approx_eq(a[i], b[i]))
    }

    fn mat_approx_eq(a: [[f64; 3]; 3], b: [[f64; 3]; 3]) -> bool {
        (0..3).all(|i| (0..3).all(|j| approx_eq(a[i][j], b[i][j])))
    }

    fn identity() -> [[f64; 3]; 3] {
        [
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ]
    }

    fn transpose(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
        [
            [m[0][0], m[1][0], m[2][0]],
            [m[0][1], m[1][1], m[2][1]],
            [m[0][2], m[1][2], m[2][2]],
        ]
    }

    #[test]
    fn identity_rotation() {
        let r = rotation_matrix(CoordSystem::C, CoordSystem::C);
        assert!(mat_approx_eq(r, identity()));
    }

    #[test]
    fn inverse_rotations() {
        let ce = rotation_matrix(CoordSystem::C, CoordSystem::E);
        let ec = rotation_matrix(CoordSystem::E, CoordSystem::C);

        let prod = matmul(&ce, &ec);
        assert!(mat_approx_eq(prod, identity()));
    }

   #[test]
   fn galactic_inverse() {
       let cg = rotation_matrix(CoordSystem::C, CoordSystem::G);
       let gc = rotation_matrix(CoordSystem::G, CoordSystem::C);
   
       let prod = matmul(&cg, &gc);
   
       for i in 0..3 {
           for j in 0..3 {
               let expected = if i == j { 1.0 } else { 0.0 };
               assert!(
                   (prod[i][j] - expected).abs() < 1e-9,
                   "Mismatch at ({},{}): {}", i, j, prod[i][j]
               );
           }
       }
   }


    #[test]
    fn vector_round_trip_ecliptic() {
        let v = [0.3, -0.4, 0.866]; // arbitrary unit-ish vector

        let ce = rotation_matrix(CoordSystem::C, CoordSystem::E);
        let ec = rotation_matrix(CoordSystem::E, CoordSystem::C);

        let v1 = matvec(&ce, v);
        let v2 = matvec(&ec, v1);

        assert!(vec_approx_eq(v, v2));
    }

    #[test]
    fn rotation_is_orthonormal() {
        let r = rotation_matrix(CoordSystem::C, CoordSystem::E);
        let rt = transpose(&r);

        let prod = matmul(&rt, &r);
        assert!(mat_approx_eq(prod, identity()));
    }


    #[test]
    fn north_pole_unit_length() {
        let v = [0.0, 0.0, 1.0];
        let r = rotation_matrix(CoordSystem::C, CoordSystem::E);
        let v2 = matvec(&r, v);

        let norm = (v2[0]*v2[0] + v2[1]*v2[1] + v2[2]*v2[2]).sqrt();
        assert!(approx_eq(norm, 1.0));
    }
}
