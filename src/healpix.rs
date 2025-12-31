use std::f64::consts::PI;

const HALF_PI: f64 = PI / 2.0;
const JRLL: [i32; 12] = [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
const JPLL: [i32; 12] = [1, 3, 5, 7, 0, 2, 4, 6, 1, 3, 5, 7];


/*
pub fn pix2ang_ring(nside: i64, ipix: i64) -> (f64, f64) {
    let npix = 12 * nside * nside;
    let ncap = 2 * nside * (nside - 1);
    let fact2 = 4.0 / npix as f64;

    let (z, phi) = if ipix < ncap {
        // North polar cap
        let iring = (1 + isqrt(1 + 2 * ipix)) >> 1;
        let iphi = (ipix + 1) - 2 * iring * (iring - 1);

        let z = 1.0 - (iring * iring) as f64 * fact2;
        let phi = (iphi as f64 - 0.5) * HALF_PI / iring as f64;
        (z, phi)

    } else if ipix < (npix - ncap) {
        // Equatorial region
        let fact1 = (2 * nside) as f64 * fact2;
        let ip = ipix - ncap;
        let iring = ip / (4 * nside) + nside;
        let iphi = ip % (4 * nside) + 1;

        let fodd = if ((iring + nside) & 1) != 0 { 1.0 } else { 0.5 };
        let nl2 = 2 * nside;

        let z = (nl2 - iring) as f64 * fact1;
        let phi = (iphi as f64 - fodd) * PI / nl2 as f64;
        (z, phi)

    } else {
        // South polar cap
        let ip = npix - ipix;
        let iring = (1 + isqrt(2 * ip - 1)) >> 1;
        let iphi = 4 * iring + 1 - (ip - 2 * iring * (iring - 1));

        let z = -1.0 + (iring * iring) as f64 * fact2;
        let phi = (iphi as f64 - 0.5) * HALF_PI / iring as f64;
        (z, phi)
    };

    let theta = z.acos();
    (theta, phi)
}


pub fn pix2ang_nest(nside: i64, ipix: i64) -> (f64, f64) {
    let npix = 12 * nside * nside;
    let nl4 = 4 * nside;
    let fact2 = 4.0 / npix as f64;

    let (ix, iy, face) = nest2xyf(nside, ipix);
    let jr = JRLL[face] * nside - ix - iy - 1;

    let (z, nr, kshift) = if jr < nside {
        let nr = jr;
        let z = 1.0 - (nr * nr) as f64 * fact2;
        (z, nr, 0)
    } else if jr > 3 * nside {
        let nr = nl4 - jr;
        let z = (nr * nr) as f64 * fact2 - 1.0;
        (z, nr, 0)
    } else {
        let fact1 = (2 * nside) as f64 * fact2;
        let z = (2 * nside - jr) as f64 * fact1;
        (z, nside, (jr - nside) & 1)
    };

    let mut jp = (JPLL[face] * nr + ix - iy + 1 + kshift) / 2;
    if jp > nl4 { jp -= nl4; }
    if jp < 1   { jp += nl4; }

    let phi = (jp as f64 - 0.5 * (kshift + 1) as f64) * HALF_PI / nr as f64;
    let theta = z.acos();

    (theta, phi)
}
*/


/// Convert (ix, iy, face) → NESTED pixel index
pub fn xyf2nest(nside: u32, ix: u32, iy: u32, face: u32) -> u64 {
    let mut morton: u64 = 0;

    // Interleave bits of ix and iy
    for bit in 0..32 {
        morton |= ((ix as u64 >> bit) & 1) << (2 * bit);
        morton |= ((iy as u64 >> bit) & 1) << (2 * bit + 1);
    }

    morton + (face as u64) * (nside as u64) * (nside as u64)
}

/// Convert NESTED pixel index → (ix, iy, face)
pub fn nest2xyf(nside: u32, pix: u64) -> (u32, u32, u32) {
    let npface = (nside as u64) * (nside as u64);

    let face = (pix / npface) as u32;
    let mut p = pix % npface;

    let mut ix: u32 = 0;
    let mut iy: u32 = 0;
    let mut bit: u32 = 0;

    // De-interleave bits
    while p != 0 {
        ix |= ((p & 1) as u32) << bit;
        p >>= 1;
        iy |= ((p & 1) as u32) << bit;
        p >>= 1;
        bit += 1;
    }

    (ix, iy, face)
}

pub fn xyf2ring(nside: i32, ix: i32, iy: i32, face: i32) -> i32 {
    let nl4 = 4 * nside;
    let jr = JRLL[face as usize] * nside - ix - iy - 1;

    let (nr, kshift, n_before) = if jr < nside {
        let nr = jr;
        (nr, 0, 2 * nr * (nr - 1))
    } else if jr > 3 * nside {
        let nr = nl4 - jr;
        (
            nr,
            0,
            12 * nside * nside - 2 * (nr + 1) * nr,
        )
    } else {
        let ncap = 2 * nside * (nside - 1);
        (
            nside,
            (jr - nside) & 1,
            ncap + (jr - nside) * nl4,
        )
    };

    let mut jp =
        (JPLL[face as usize] * nr + ix - iy + 1 + kshift) / 2;

    if jp > nl4 {
        jp -= nl4;
    } else if jp < 1 {
        jp += nl4;
    }

    n_before + jp - 1
}


pub fn ring2xyf(nside: i32, pix: i32) -> (i32, i32, i32) {
    let ncap = 2 * nside * (nside - 1);
    let npix = 12 * nside * nside;
    let nl2 = 2 * nside;

    let (iring, iphi, kshift, nr, face) = if pix < ncap {
        let iring = (1 + isqrt(1 + 2 * pix)) >> 1;
        let iphi = (pix + 1) - 2 * iring * (iring - 1);
        let nr = iring;
        let face = special_div(iphi - 1, nr);
        (iring, iphi, 0, nr, face)
    } else if pix < npix - ncap {
        let ip = pix - ncap;
        let iring = ip / (4 * nside) + nside;
        let iphi = (ip % (4 * nside)) + 1;
        let kshift = (iring + nside) & 1;
        let nr = nside;

        let ire = iring - nside + 1;
        let irm = nl2 + 2 - ire;
        let ifm = (iphi - ire / 2 + nside - 1) / nside;
        let ifp = (iphi - irm / 2 + nside - 1) / nside;

        let face = if ifp == ifm {
            ifp | 4
        } else if ifp < ifm {
            ifp
        } else {
            ifm + 8
        };

        (iring, iphi, kshift, nr, face)
    } else {
        let ip = npix - pix;
        let mut iring = (1 + isqrt(2 * ip - 1)) >> 1;
        let iphi = 4 * iring + 1 - (ip - 2 * iring * (iring - 1));
        let nr = iring;
        iring = 4 * nside - iring;
        let face = 8 + special_div(iphi - 1, nr);
        (iring, iphi, 0, nr, face)
    };

    let irt = iring - JRLL[face as usize] * nside + 1;
    let mut ipt = 2 * iphi - JPLL[face as usize] * nr - kshift - 1;
    if ipt >= nl2 {
        ipt -= 8 * nside;
    }

    let ix = (ipt - irt) >> 1;
    let iy = (-(ipt + irt)) >> 1;

    (ix, iy, face)
}

fn isqrt(x: i32) -> i32 {
    (x as f64).sqrt() as i32
}

fn special_div(a: i32, b: i32) -> i32 {
    if a >= 0 {
        a / b
    } else {
        -((-a - 1) / b) - 1
    }
}

/// Convert a nested pixel index to a ring pixel index
pub fn nest2ring(nside: u32, ipnest: u64) -> u64 {
    if !nside.is_power_of_two() {
        panic!("nside must be a power of two");
    }

    let (ix, iy, face) = nest2xyf(nside, ipnest);
    xyf2ring(nside as i32, ix as i32, iy as i32, face as i32) as u64
}

/// Convert a ring pixel index to a nested pixel index
pub fn ring2nest(nside: u32, ipring: u64) -> u64 {
    if !nside.is_power_of_two() {
        panic!("nside must be a power of two");
    }

    let (ix, iy, face) = ring2xyf(nside as i32, ipring as i32);
    xyf2nest(nside, ix as u32, iy as u32, face as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xyf_nest_invertibility_small() {
        let nside = 8;

        for face in 0..12 {
            for ix in 0..nside {
                for iy in 0..nside {
                    let pix = xyf2nest(nside, ix, iy, face);
                    let (ix2, iy2, face2) = nest2xyf(nside, pix);

                    assert_eq!(face, face2);
                    assert_eq!(ix, ix2);
                    assert_eq!(iy, iy2);
                }
            }
        }
    }

    #[test]
    fn test_random_pixels() {
        let nside = 64;
        let npix = 12 * nside * nside;

        // Deterministic pseudo-random sampling
        let mut seed: u64 = 0xdeadbeef;

        for _ in 0..10_000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let pix = seed % npix as u64;

            let (ix, iy, face) = nest2xyf(nside, pix);
            let pix2 = xyf2nest(nside, ix, iy, face);

            assert_eq!(pix, pix2);
        }
    }
}

#[test]
fn test_xyf_ring_invertibility() {
    let nside = 8;

    for face in 0..12 {
        for ix in 0..nside {
            for iy in 0..nside {
                let ring = xyf2ring(nside, ix, iy, face);
                let (ix2, iy2, face2) = ring2xyf(nside, ring);

                assert_eq!(face, face2);
                assert_eq!(ix, ix2);
                assert_eq!(iy, iy2);
            }
        }
    }
}

#[test]
fn test_nest_ring_roundtrip() {
    let nside = 32;
    let npix = 12 * nside * nside;

    for pix in (0..npix).step_by(97) {
        let (ix, iy, face) = nest2xyf(nside as u32, pix as u64);
        let ring = xyf2ring(nside, ix as i32, iy as i32, face as i32);
        let (ix2, iy2, face2) = ring2xyf(nside, ring);

        let pix2 = xyf2nest(nside as u32, ix2 as u32, iy2 as u32, face2 as u32);
        assert_eq!(pix as u64, pix2);
    }
}
#[test]
fn test_nest_ring_roundtrip_simple() {
    let nside = 8;
    let npix = 12 * nside * nside;

    for pix in 0..npix {
        let ring = nest2ring(nside, pix as u64);
        let nest = ring2nest(nside, ring);
        assert_eq!(pix as u64, nest, "pix={} failed roundtrip", pix);
    }
}

