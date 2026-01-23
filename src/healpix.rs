use std::f64::consts::PI;

pub const HPX_UNSEEN: f64 = -1.6375e30;
const HALF_PI: f64 = PI / 2.0;
const TWOPI: f64 = 2.0 * PI;
const INV_HALFPI: f64 = 2.0 / PI;
const TWOTHIRD: f64 = 2.0 / 3.0;

const JRLL: [i64; 12] = [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
const JPLL: [i64; 12] = [1, 3, 5, 7, 0, 2, 4, 6, 1, 3, 5, 7];

use std::fs::File;
use std::io::BufReader;

use fitsrs::{Fits, HDU, card::Value};
use fitsrs::hdu::header::Header;

#[derive(Debug, Clone, Copy)]
pub enum HealpixOrdering {
    Ring,
    Nested,
}

#[derive(Debug, Clone, Copy)]
pub struct HealpixMeta {
    pub ordering: HealpixOrdering,
    pub nside: i64,
}



pub fn read_healpix_meta(path: &str) -> Option<HealpixMeta> {
    let f = File::open(path).ok()?;
    let reader = BufReader::new(f);
    let mut fits = Fits::from_reader(reader);

    while let Some(Ok(hdu)) = fits.next() {
        match hdu {
            HDU::XImage(ref hdu_img) => {
                if let Some(meta) = extract_meta(hdu_img.get_header()) {
                    return Some(meta);
                }
            }
            HDU::XBinaryTable(ref hdu_bin) => {
                if let Some(meta) = extract_meta(hdu_bin.get_header()) {
                    return Some(meta);
                }
            }
            HDU::XASCIITable(ref hdu_ascii) => {
                if let Some(meta) = extract_meta(hdu_ascii.get_header()) {
                    return Some(meta);
                }
            }
            _ => {}
        }
    }

    None
}

fn extract_meta<X>(header: &Header<X>) -> Option<HealpixMeta> {
    let ordering = match header.get("ORDERING") {
        Some(Value::String { value, .. }) if value == "RING" => HealpixOrdering::Ring,
        Some(Value::String { value, .. }) if value == "NESTED" => HealpixOrdering::Nested,
        _ => return None,
    };

    let nside = match header.get("NSIDE") {
        Some(Value::Integer { value, .. }) => *value as i64,
        _ => return None,
    };

    Some(HealpixMeta { ordering, nside })
}



#[inline]
pub fn is_seen(v: f64) -> bool {
    v.is_finite() && v > HPX_UNSEEN
}

#[inline]
pub fn ang_dist(theta1: f64, phi1: f64, theta2: f64, phi2: f64) -> f64 {
    let cos_c = theta1.sin() * theta2.sin() * (phi1 - phi2).cos() + theta1.cos() * theta2.cos();
    cos_c.acos()
}

#[inline]
pub fn ang2pix(meta: HealpixMeta, theta: f64, phi: f64) -> i64 {
    match meta.ordering {
        HealpixOrdering::Ring => ang2pix_ring(meta.nside, theta, phi),
        HealpixOrdering::Nested => ang2pix_nest(meta.nside, theta, phi),
    }
}


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

pub fn ang2pix_ring(nside: i64, theta: f64, phi: f64) -> i64 {
    assert!(theta >= 0.0 && theta <= PI);

    let z = theta.cos();
    let za = z.abs();
    let tt = ((phi % TWOPI) + TWOPI) % TWOPI * INV_HALFPI;

    if za <= TWOTHIRD {
        let temp1 = nside as f64 * (0.5 + tt);
        let temp2 = nside as f64 * (0.75 * z);

        let jp = (temp1 - temp2).floor() as i64;
        let jm = (temp1 + temp2).floor() as i64;

        let ir = nside + 1 + jp - jm;
        let kshift = 1 - (ir & 1);

        let mut ip = (jp + jm - nside + kshift + 1) / 2;
        ip = imodulo(ip, 4 * nside);

        2 * nside * (nside - 1) + (ir - 1) * 4 * nside + ip
    } else {
        let tp = tt - tt.floor();
        let tmp = nside as f64 * (3.0 * (1.0 - za)).sqrt();

        let jp = (tp * tmp).floor() as i64;
        let jm = ((1.0 - tp) * tmp).floor() as i64;

        let ir = jp + jm + 1;
        let mut ip = (tt * ir as f64).floor() as i64;
        ip = imodulo(ip, 4 * ir);

        if z > 0.0 {
            2 * ir * (ir - 1) + ip
        } else {
            12 * nside * nside - 2 * ir * (ir + 1) + ip
        }
    }
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

pub fn ang2pix_nest(nside: i64, theta: f64, phi: f64) -> i64 {
    assert!(theta >= 0.0 && theta <= PI);

    let z = theta.cos();
    let za = z.abs();

    // φ mapped to [0,4)
    let tt = ((phi % TWOPI) + TWOPI) % TWOPI * INV_HALFPI;

    let (face, ix, iy): (usize, i64, i64);

    if za <= TWOTHIRD {
        // ===== Equatorial region =====
        let temp1 = nside as f64 * (0.5 + tt);
        let temp2 = nside as f64 * (0.75 * z);

        let jp = (temp1 - temp2).floor() as i64;
        let jm = (temp1 + temp2).floor() as i64;

        let ifp = jp / nside;
        let ifm = jm / nside;

        face = if ifp == ifm {
            (ifp | 4) as usize
        } else if ifp < ifm {
            ifp as usize
        } else {
            (ifm + 8) as usize
        };

        ix = jm & (nside - 1);
        iy = nside - (jp & (nside - 1)) - 1;
    } else {
        // ===== Polar caps =====
        let mut ntt = tt.floor() as i64;
        if ntt >= 4 {
            ntt = 3;
        }

        let tp = tt - ntt as f64;
        let tmp = nside as f64 * (3.0 * (1.0 - za)).sqrt();

        let mut jp = (tp * tmp).floor() as i64;
        let mut jm = ((1.0 - tp) * tmp).floor() as i64;

        if jp >= nside { jp = nside - 1; }
        if jm >= nside { jm = nside - 1; }

        if z >= 0.0 {
            face = ntt as usize;
            ix = nside - jm - 1;
            iy = nside - jp - 1;
        } else {
            face = (ntt + 8) as usize;
            ix = jp;
            iy = jm;
        }
    }

    xyf2nest(nside, ix, iy, face)
}




/// Convert (ix, iy, face) → NESTED pixel index
pub fn xyf2nest(nside: i64, ix: i64, iy: i64, face: usize) -> i64 {
    let mut morton: i64 = 0;

    // Interleave bits of ix and iy
    for bit in 0..32 {
        morton |= ((ix as i64 >> bit) & 1) << (2 * bit);
        morton |= ((iy as i64 >> bit) & 1) << (2 * bit + 1);
    }

    morton + (face as i64) * (nside as i64) * (nside as i64)
}

/// Convert NESTED pixel index → (ix, iy, face)
pub fn nest2xyf(nside: i64, pix: i64) -> (i64, i64, usize) {
    let npface = nside * nside;

    let face = (pix / npface) as usize;
    let mut p = (pix % npface) as u64;

    let mut ix: u64 = 0;
    let mut iy: u64 = 0;
    let mut bit: u32 = 0;

    // De-interleave bits (Morton decode)
    while p != 0 {
        ix |= (p & 1) << bit;
        p >>= 1;

        iy |= (p & 1) << bit;
        p >>= 1;

        bit += 1;
    }

    (ix as i64, iy as i64, face)
}

pub fn xyf2ring(nside: i64, ix: i64, iy: i64, face: usize) -> i64 {
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


pub fn ring2xyf(nside: i64, pix: i64) -> (i64, i64, usize) {
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

    (ix, iy, face as usize)
}

#[inline]
fn imodulo(a: i64, m: i64) -> i64 {
    let r = a % m;
    if r < 0 { r + m } else { r }
}

fn isqrt(x: i64) -> i64 {
    (x as f64).sqrt() as i64
}

fn special_div(a: i64, b: i64) -> i64 {
    if a >= 0 {
        a / b
    } else {
        -((-a - 1) / b) - 1
    }
}



/// Convert a nested pixel index to a ring pixel index
pub fn nest2ring(nside: i64, ipnest: i64) -> i64 {
    if !(nside as u64).is_power_of_two() {
        panic!("nside must be a power of two");
    }

    let (ix, iy, face) = nest2xyf(nside, ipnest);
    xyf2ring(nside, ix, iy, face) as i64
}

/// Convert a ring pixel index to a nested pixel index
pub fn ring2nest(nside: i64, ipring: i64) -> i64 {
    if !(nside as u64).is_power_of_two() {
        panic!("nside must be a power of two");
    }

    let (ix, iy, face) = ring2xyf(nside as i64, ipring as i64);
    xyf2nest(nside, ix, iy, face)
}

pub fn sample_healpix(
    map: &[f64],
    meta: HealpixMeta,
    theta: f64, // colatitude [0, pi]
    lon: f64,   // longitude [-pi, pi] (or [0, 2pi], depending on your convention)
) -> Option<f64> {
    if !(0.0..=std::f64::consts::PI).contains(&theta) {
        return None;
    }

    let ipix = ang2pix(meta, theta, lon) as usize;
    let val = map[ipix];

    Some(val)
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
        let mut seed: i64 = 0xdeadbeef;

        for _ in 0..10_000 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let pix = seed % npix as i64;

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
        let (ix, iy, face) = nest2xyf(nside as i64, pix as i64);
        let ring = xyf2ring(nside, ix as i64, iy as i64, face);
        let (ix2, iy2, face2) = ring2xyf(nside, ring);

        let pix2 = xyf2nest(nside, ix2, iy2, face2);
        assert_eq!(pix as i64, pix2);
    }
}
#[test]
fn test_nest_ring_roundtrip_simple() {
    let nside = 8;
    let npix = 12 * nside * nside;

    for pix in 0..npix {
        let ring = nest2ring(nside, pix as i64);
        let nest = ring2nest(nside, ring);
        assert_eq!(pix as i64, nest, "pix={} failed roundtrip", pix);
    }
}


#[test]
fn test_ang_roundtrip_nest() {
    let nside = 16;
    let npix = 12 * nside * nside;

    for ipix in 0..npix {
        let (theta, phi) = pix2ang_nest(nside, ipix);
        let ipix2 = ang2pix_nest(nside, theta, phi);
        assert_eq!(ipix, ipix2);
    }
}


#[test]
fn test_random_angles() {
    let nside = 64;

    for _ in 0..10000 {
        let theta = rand::random::<f64>() * PI;
        let phi = rand::random::<f64>() * 2.0 * PI;

        let ipix = ang2pix_nest(nside, theta, phi);
        let (theta2, phi2) = pix2ang_nest(nside, ipix);

        // Pixel center must lie in same pixel
        let ipix2 = ang2pix_nest(nside, theta2, phi2);
        assert_eq!(ipix, ipix2);
    }
}


#[test]
fn test_ang_pix_ang_consistency() {
    let nside = 8;
    let npix = 12 * nside * nside;
    const EPSILON: f64 = 1e-4;

    for pix in 0..npix {
        let (theta, phi) = pix2ang_ring(nside, pix);
        let pix2 = ang2pix_ring(nside, theta, phi);
        let d = ang_dist(theta, phi, pix2ang_ring(nside, pix2).0, pix2ang_ring(nside, pix2).1);
        assert!(d < EPSILON, "Too far: d={}", d);
    }
}




#[test]
fn test_pix_ang_pix_roundtrip_ring() {
    let nside = 32;
    let npix = 12 * nside * nside;

    for ipix in 0..npix {
        let (theta, phi) = pix2ang_ring(nside, ipix);
        let ipix2 = ang2pix_ring(nside, theta, phi);
        assert_eq!(ipix, ipix2);
    }
}

/// Calculate target nside for a given resolution to balance quality and performance
pub fn target_nside_for_resolution(width: usize, height: usize) -> i64 {
    // For very high resolution maps, we want to downgrade to improve cache performance
    // Target around 1024 nside for typical plot sizes
    let pixels = (width * height) as f64;
    let target_resolution = pixels.sqrt(); 
    let target_nside = target_resolution.round() as i64;
    // Ensure nside is a power of 2
    let mut nside = 1;
    while nside * 2 <= target_nside {
        nside *= 2;
    }
    nside
}

/// Downgrade a HEALPix map from high nside to lower nside by averaging pixels
pub fn downgrade_healpix_map(
    map: &[f64],
    source_nside: i64,
    target_nside: i64,
    ordering: HealpixOrdering,
) -> Vec<f64> {
    if source_nside <= target_nside {
        return map.to_vec();
    }

    let ratio = (source_nside / target_nside) as usize;
    let _factor = ratio * ratio; // Each target pixel covers factor source pixels
    let target_npix = (12 * target_nside * target_nside) as usize;
    let mut result = vec![0.0; target_npix];

    for target_pix in 0..target_npix {
        let mut sum = 0.0;
        let mut count = 0;

        // Convert target pixel to angles
        let (theta, phi) = match ordering {
            HealpixOrdering::Ring => pix2ang_ring(target_nside, target_pix as i64),
            HealpixOrdering::Nested => pix2ang_nest(target_nside, target_pix as i64),
        };

        // Sample the source pixels that cover this target pixel
        // For simplicity, sample a grid within the target pixel area
        let n_samples = ratio.min(4); // Limit samples for performance
        let step = 1.0 / n_samples as f64;

        for i in 0..n_samples {
            for j in 0..n_samples {
                let d_theta = (i as f64 + 0.5) * step - 0.5;
                let d_phi = (j as f64 + 0.5) * step - 0.5;

                let sample_theta = (theta + d_theta * std::f64::consts::PI / (2.0 * target_nside as f64)).clamp(0.0, std::f64::consts::PI);
                let sample_phi = (phi + d_phi * 2.0 * std::f64::consts::PI / target_nside as f64).rem_euclid(2.0 * std::f64::consts::PI);

                let source_pix = match ordering {
                    HealpixOrdering::Ring => ang2pix_ring(source_nside, sample_theta, sample_phi),
                    HealpixOrdering::Nested => ang2pix_nest(source_nside, sample_theta, sample_phi),
                } as usize;

                if source_pix < map.len() && is_seen(map[source_pix]) {
                    sum += map[source_pix];
                    count += 1;
                }
            }
        }

        result[target_pix] = if count > 0 { sum / count as f64 } else { HPX_UNSEEN };
    }

    result
}


