//! HEALPix coordinate system and data sampling utilities.
//!
//! This module provides core HEALPix functionality for hierarchical equal-area pixelization
//! of the sphere. It includes:
//!
//! - **Pixel ordering**: RING and NESTED schemes
//! - **Coordinate transformation**: Between angular (θ, φ) and pixel indices
//! - **FITS integration**: Reading HEALPix metadata from FITS headers
//! - **Data sampling**: Efficient sampling of HEALPix maps at arbitrary angular positions
//! - **Resolution management**: Computing appropriate NSIDE for given resolution
//! - **Resampling**: Downgrading maps to lower resolution
//!
//! # HEALPix Overview
//!
//! HEALPix is a hierarchical tessellation of the sphere with equal-area pixels. It provides:
//! - Isotropic resolution at all sky locations
//! - Hierarchical structure amenable to fast searches
//! - 12×NSIDE² pixels total, where NSIDE is the resolution parameter (power of 2)
//!
//! # RING vs NESTED Ordering
//!
//! - **RING**: Pixels ordered by latitude strips, simpler for visualization
//! - **NESTED**: Hierarchical quadtree ordering, better for data compression
//!
//! # Examples
//!
//! ```ignore
//! use map2fig::healpix::{read_healpix_meta, pix2ang_ring, sample_healpix};
//!
//! // Read metadata from FITS file
//! let meta = read_healpix_meta("map.fits").expect("Invalid FITS");
//! println!("NSIDE: {}, Ordering: {:?}", meta.nside, meta.ordering);
//!
//! // Convert pixel index to celestial coordinates
//! let (theta, phi) = pix2ang_ring(meta.nside, 0);
//! println!("Pixel 0 is at θ={}, φ={}", theta, phi);
//! ```

use std::f64::consts::PI;

pub const HPX_UNSEEN: f64 = -1.6375e30;
use crate::rotation::ViewTransform;
use crate::simd;

const HALF_PI: f64 = PI / 2.0;
const TWOPI: f64 = 2.0 * PI;
const INV_HALFPI: f64 = 2.0 / PI;
const TWOTHIRD: f64 = 2.0 / 3.0;

const JRLL: [i64; 12] = [2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4];
const JPLL: [i64; 12] = [1, 3, 5, 7, 0, 2, 4, 6, 1, 3, 5, 7];

use std::fs::File;
use std::io::BufReader;

use fitsrs::hdu::header::Header;
use fitsrs::{Fits, HDU, card::Value};

use crate::rotation::CoordSystem;
use crate::rotation::{sph_to_vec, vec_to_sph};

/// HEALPix pixel ordering scheme.
///
/// Two standard HEALPix pixel numbering schemes:
/// - **RING**: Pixels ordered by latitude rings (0° to 90°, 90° to -90°)
/// - **NESTED**: Hierarchical quadtree ordering for efficient data access
#[derive(Debug, Clone, Copy)]
pub enum HealpixOrdering {
    /// Ring ordering by latitude
    Ring,
    /// Nested quadtree ordering
    Nested,
}

/// HEALPix map metadata extracted from FITS header.
///
/// Contains essential information about a HEALPix celestial map,
/// read from the FITS binary table header.
///
/// # Fields
///
/// * `ordering` - Pixel ordering scheme (RING or NESTED)
/// * `nside` - Resolution parameter (12×nside² = total pixels, must be power of 2)
/// * `coord` - Celestial coordinate system (Galactic, Equatorial, etc.)
#[derive(Debug, Clone, Copy)]
pub struct HealpixMeta {
    /// Pixel ordering scheme (RING or NESTED)
    pub ordering: HealpixOrdering,
    /// Resolution parameter (1, 2, 4, 8, ..., 16384, ...)
    pub nside: i64,
    /// Coordinate system (Galactic, Equatorial, etc.)
    pub coord: CoordSystem,
}

/// Read HEALPix metadata from a FITS file.
///
/// Extracts HEALPix-specific keywords from the FITS binary table header:
/// - NSIDE: Resolution parameter
/// - ORDERING: RING or NESTED
/// - COORDSYS: Galactic ('G') or Equatorial ('C')
///
/// # Arguments
///
/// * `path` - Path to the FITS file
///
/// # Returns
///
/// - `Some(HealpixMeta)` if valid HEALPix FITS file
/// - `None` if file doesn't exist, is invalid, or lacks HEALPix headers
///
/// # Caching (Tier 4.2a Optimization)
///
/// This function uses file-based metadata caching to avoid expensive FITS header
/// parsing on repeated calls. Cache is validated via file modification time.
/// Cache location: ~/.cache/map2fig/fits_meta_*.json
pub fn read_healpix_meta(path: &str) -> Option<HealpixMeta> {
    // Try cached lookup first (Tier 4.2a optimization)
    // This avoids expensive FITS header parsing if we've seen this file before
    if let Some((nside, ordering_str, _indxschm)) = crate::fits::read_healpix_meta_cached(path) {
        let ordering = if ordering_str == "NESTED" {
            HealpixOrdering::Nested
        } else {
            HealpixOrdering::Ring // Default to RING if unknown
        };

        return Some(HealpixMeta {
            ordering,
            nside,
            coord: CoordSystem::G, // Default coordinate system (will be overridden by caller if needed)
        });
    }

    // Cache miss or caching unavailable: parse FITS file directly
    let f = File::open(path).ok()?;
    let reader = BufReader::with_capacity(256 * 1024, f);
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
        Some(Value::String { value, .. }) if value.trim() == "RING" => HealpixOrdering::Ring,
        Some(Value::String { value, .. }) if value.trim() == "NESTED" => HealpixOrdering::Nested,
        _ => return None,
    };

    let nside = match header.get("NSIDE") {
        Some(Value::Integer { value, .. }) => *value,
        _ => return None,
    };

    let coord = match header.get("COORDSYS") {
        Some(Value::String { value, .. }) => match value.trim() {
            "C" | "CEL" | "CELESTIAL" => CoordSystem::C,
            "G" | "GAL" | "GALACTIC" => CoordSystem::G,
            "E" | "ECL" | "ECLIPTIC" => CoordSystem::E,
            _ => CoordSystem::G, // HEALPix default
        },
        None => CoordSystem::G, // HEALPix convention
        _ => todo!(),
    };

    Some(HealpixMeta {
        ordering,
        nside,
        coord,
    })
}

#[inline]
pub fn is_seen(v: f64) -> bool {
    // Use -1e30 threshold instead of exact HPX_UNSEEN constant to account for
    // floating-point precision variations in FITS files (e.g., CLASS files use
    // -1.637499996306027e+30 which is slightly different from -1.6375e30)
    v.is_finite() && v > -1e30
}

#[inline]
pub fn ang_dist(theta1: f64, phi1: f64, theta2: f64, phi2: f64) -> f64 {
    let cos_c = theta1.sin() * theta2.sin() * (phi1 - phi2).cos() + theta1.cos() * theta2.cos();
    cos_c.acos()
}

#[inline]
fn ang2pix(meta: HealpixMeta, theta: f64, phi: f64) -> i64 {
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

fn ang2pix_ring(nside: i64, theta: f64, phi: f64) -> i64 {
    assert!((0.0..=PI).contains(&theta));

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

fn pix2ang_nest(nside: i64, ipix: i64) -> (f64, f64) {
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
    if jp > nl4 {
        jp -= nl4;
    }
    if jp < 1 {
        jp += nl4;
    }

    let phi = (jp as f64 - 0.5 * (kshift + 1) as f64) * HALF_PI / nr as f64;
    let theta = z.acos();

    (theta, phi)
}

fn ang2pix_nest(nside: i64, theta: f64, phi: f64) -> i64 {
    assert!((0.0..=PI).contains(&theta));

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

        if jp >= nside {
            jp = nside - 1;
        }
        if jm >= nside {
            jm = nside - 1;
        }

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
fn xyf2nest(nside: i64, ix: i64, iy: i64, face: usize) -> i64 {
    let mut morton: i64 = 0;

    // Interleave bits of ix and iy
    for bit in 0..32 {
        morton |= ((ix >> bit) & 1) << (2 * bit);
        morton |= ((iy >> bit) & 1) << (2 * bit + 1);
    }

    morton + (face as i64) * nside * nside
}

/// Convert NESTED pixel index → (ix, iy, face)
fn nest2xyf(nside: i64, pix: i64) -> (i64, i64, usize) {
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

fn xyf2ring(nside: i64, ix: i64, iy: i64, face: usize) -> i64 {
    let nl4 = 4 * nside;
    let jr = JRLL[face] * nside - ix - iy - 1;

    let (nr, kshift, n_before) = if jr < nside {
        let nr = jr;
        (nr, 0, 2 * nr * (nr - 1))
    } else if jr > 3 * nside {
        let nr = nl4 - jr;
        (nr, 0, 12 * nside * nside - 2 * (nr + 1) * nr)
    } else {
        let ncap = 2 * nside * (nside - 1);
        (nside, (jr - nside) & 1, ncap + (jr - nside) * nl4)
    };

    let mut jp = (JPLL[face] * nr + ix - iy + 1 + kshift) / 2;

    if jp > nl4 {
        jp -= nl4;
    } else if jp < 1 {
        jp += nl4;
    }

    n_before + jp - 1
}

fn ring2xyf(nside: i64, pix: i64) -> (i64, i64, usize) {
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
    if a >= 0 { a / b } else { -((-a - 1) / b) - 1 }
}

/// Convert a nested pixel index to a ring pixel index
#[allow(dead_code)]
fn nest2ring(nside: i64, ipnest: i64) -> i64 {
    if !(nside as u64).is_power_of_two() {
        panic!("nside must be a power of two");
    }

    let (ix, iy, face) = nest2xyf(nside, ipnest);
    xyf2ring(nside, ix, iy, face)
}

/// Convert a ring pixel index to a nested pixel index
#[allow(dead_code)]
fn ring2nest(nside: i64, ipring: i64) -> i64 {
    if !(nside as u64).is_power_of_two() {
        panic!("nside must be a power of two");
    }

    let (ix, iy, face) = ring2xyf(nside, ipring);
    xyf2nest(nside, ix, iy, face)
}

#[inline]
pub fn sample_healpix(
    map: &[f64],
    meta: HealpixMeta,
    view: &ViewTransform,
    theta: f64,
    lon: f64,
) -> Option<f64> {
    if !theta.is_finite() || !lon.is_finite() {
        return None;
    }

    // Direction on the screen / projection
    let v_view = sph_to_vec(theta, lon);

    // Rotate BACK into map coordinates using pre-computed inverse
    let v_map = view.apply_inverse(v_view);

    let (mut theta_m, mut lon_m) = vec_to_sph(v_map);

    theta_m = theta_m.clamp(0.0, PI);
    lon_m = lon_m.rem_euclid(2.0 * PI);

    let ipix = ang2pix(meta, theta_m, lon_m) as usize;
    map.get(ipix).copied()
}

/// Get the HEALPix pixel index for a given coordinate
pub fn sample_healpix_index(
    _map: &[f64],
    meta: HealpixMeta,
    view: &ViewTransform,
    theta: f64,
    lon: f64,
) -> Option<usize> {
    if !theta.is_finite() || !lon.is_finite() {
        return None;
    }

    // Direction on the screen / projection
    let v_view = sph_to_vec(theta, lon);

    // Rotate BACK into map coordinates using pre-computed inverse
    let v_map = view.apply_inverse(v_view);

    let (mut theta_m, mut lon_m) = vec_to_sph(v_map);

    theta_m = theta_m.clamp(0.0, PI);
    lon_m = lon_m.rem_euclid(2.0 * PI);

    let ipix = ang2pix(meta, theta_m, lon_m) as usize;
    Some(ipix)
}

/// Batch sample HEALPix: process 8 pixels in parallel
///
/// Input:
///   - map: HEALPix data array
///   - meta: HEALPix metadata
///   - view: view transformation
///   - thetas: array of 8 theta values
///   - lons: array of 8 longitude values
///
/// Output:
///   - samples: array of 8 HEALPix values (or 0.0 if invalid)
///   - mask: validity mask (true if sample succeeded)
///
/// This processes 8 independent sample operations with opportunity
/// for instruction-level parallelism and potential SIMD vectorization.
pub fn sample_healpix_batch(
    map: &[f64],
    meta: HealpixMeta,
    view: &ViewTransform,
    thetas: &[f64; 8],
    lons: &[f64; 8],
) -> ([f64; 8], [bool; 8]) {
    let mut samples = [0.0_f64; 8];
    let mut mask = [false; 8];

    // Unrolled loop: process 8 samples with independent computation
    for i in 0..8 {
        if !thetas[i].is_finite() || !lons[i].is_finite() {
            continue;
        }

        // Convert spherical to Cartesian
        let v_view = sph_to_vec(thetas[i], lons[i]);

        // Apply view transformation inverse
        let v_map = view.apply_inverse(v_view);

        // Convert back to spherical in map coordinates
        let (mut theta_m, mut lon_m) = vec_to_sph(v_map);

        // Clamp and normalize
        theta_m = theta_m.clamp(0.0, PI);
        lon_m = lon_m.rem_euclid(2.0 * PI);

        // Get HEALPix pixel index
        let ipix = ang2pix(meta, theta_m, lon_m) as usize;

        // Get value from map
        if let Some(&value) = map.get(ipix) {
            samples[i] = value;
            mask[i] = true;
        }
    }

    (samples, mask)
}

/// Batch sample HEALPix with SIMD acceleration (8 pixels)
///
/// Vectorized version using SIMD primitives for coordinate transformations.
/// Processes spherical-to-Cartesian and view transforms in parallel,
/// then applies scalar HEALPix indexing (independent per-pixel).
///
/// Input:
///   - map: HEALPix data array
///   - meta: HEALPix metadata
///   - view: view transformation
///   - thetas: array of 8 theta values
///   - lons: array of 8 longitude values
///
/// Output:
///   - samples: array of 8 HEALPix values (or 0.0 if invalid)
///   - mask: validity mask (true if sample succeeded)
#[inline]
pub fn sample_healpix_batch_simd(
    map: &[f64],
    meta: HealpixMeta,
    view: &ViewTransform,
    thetas: &[f64; 8],
    lons: &[f64; 8],
) -> ([f64; 8], [bool; 8]) {
    // Check validity of input angles
    let mut valid = [true; 8];
    for i in 0..8 {
        if !thetas[i].is_finite() || !lons[i].is_finite() {
            valid[i] = false;
        }
    }

    // Vectorized: convert spherical to Cartesian (8 theta-phi pairs)
    let (x_view, y_view, z_view) = simd::simd_sph_to_vec_8(*thetas, *lons);

    // Apply view transformation: v_map = view.apply_inverse(v_view)
    // This requires applying the inverse rotation matrix to 8 vectors
    let (x_map, y_map, z_map) =
        simd::simd_matvec3_8(view.rotation_inv.matrix, x_view, y_view, z_view);

    // Vectorized: convert Cartesian back to spherical (8 vectors)
    let (theta_m, lon_m) = simd::simd_vec_to_sph_8(x_map, y_map, z_map);

    // Scalar: clamp angles and get HEALPix pixel indices
    let mut samples = [0.0_f64; 8];
    let mut mask = [false; 8];

    for i in 0..8 {
        if !valid[i] {
            continue;
        }

        // Clamp and normalize angles
        let theta_clamped = theta_m[i].clamp(0.0, PI);
        let lon_normalized = lon_m[i].rem_euclid(2.0 * PI);

        // Get HEALPix pixel index
        let ipix = ang2pix(meta, theta_clamped, lon_normalized) as usize;

        // Get value from map
        if let Some(&value) = map.get(ipix) {
            samples[i] = value;
            mask[i] = true;
        }
    }

    (samples, mask)
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
            let pix = seed % npix;

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
        let (ix, iy, face) = nest2xyf(nside, pix);
        let ring = xyf2ring(nside, ix, iy, face);
        let (ix2, iy2, face2) = ring2xyf(nside, ring);

        let pix2 = xyf2nest(nside, ix2, iy2, face2);
        assert_eq!(pix, pix2);
    }
}
#[test]
fn test_nest_ring_roundtrip_simple() {
    let nside = 8;
    let npix = 12 * nside * nside;

    for pix in 0..npix {
        let ring = nest2ring(nside, pix);
        let nest = ring2nest(nside, ring);
        assert_eq!(pix, nest);
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
        let d = ang_dist(
            theta,
            phi,
            pix2ang_ring(nside, pix2).0,
            pix2ang_ring(nside, pix2).1,
        );
        assert!(d < EPSILON, "Too far: d={}", d);
    }
}

#[test]
fn test_is_seen_filters_exact_unseen_value() {
    // Test that exact UNSEEN sentinel is filtered
    assert!(!is_seen(HPX_UNSEEN), "Exact HPX_UNSEEN should be filtered");
}

#[test]
fn test_is_seen_filters_fits_class_unseen_value() {
    // CLASS FITS files use a slightly different floating-point representation
    // of the UNSEEN value: -1.637499996306027e+30 vs -1.6375e30
    // Our filter should catch both due to using -1e30 threshold
    let class_unseen = -1.637499996306027e30;
    assert!(
        !is_seen(class_unseen),
        "CLASS FITS UNSEEN value should be filtered"
    );
}

#[test]
fn test_is_seen_filters_very_negative_values() {
    // Any value much more negative than -1e30 should be filtered
    assert!(!is_seen(-2.0e30), "Very negative values should be filtered");
    assert!(
        !is_seen(-1.5e30),
        "Values near UNSEEN threshold should be filtered"
    );
    assert!(
        !is_seen(f64::NEG_INFINITY),
        "Negative infinity should be filtered (non-finite)"
    );
}

#[test]
fn test_is_seen_passes_valid_data() {
    // Valid scientific data values should NOT be filtered
    assert!(is_seen(1.234e-5), "Positive scientific value should pass");
    assert!(is_seen(-1.0e-6), "Small negative value should pass");
    assert!(is_seen(0.0), "Zero should pass");
    assert!(is_seen(1.0), "Positive value should pass");
    assert!(is_seen(-0.5e30), "Large negative value > -1e30 should pass");
}

#[test]
fn test_is_seen_filters_non_finite() {
    // Non-finite values should be filtered regardless of magnitude
    assert!(!is_seen(f64::NAN), "NaN should be filtered");
    assert!(!is_seen(f64::INFINITY), "Infinity should be filtered");
    assert!(
        !is_seen(f64::NEG_INFINITY),
        "Negative infinity should be filtered"
    );
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

#[test]
fn test_sample_healpix_batch_matches_scalar() {
    use crate::rotation::{CoordSystem, ViewTransform};

    // Create a simple test map
    let nside = 16;
    let npix = 12 * nside * nside;
    let map: Vec<f64> = (0..npix).map(|i| (i as f64) * 0.1).collect();

    let meta = HealpixMeta {
        nside,
        ordering: HealpixOrdering::Ring,
        coord: CoordSystem::G,
    };

    // Identity view transform (no rotation, no coordinate change)
    let view = ViewTransform::new(CoordSystem::G, CoordSystem::G, None);

    // Test coordinates
    let thetas = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.1, std::f64::consts::PI];
    let lons = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, std::f64::consts::TAU];

    // Get batch results
    let (batch_samples, batch_mask) = sample_healpix_batch(&map, meta, &view, &thetas, &lons);

    // Verify each against scalar version
    for i in 0..8 {
        let scalar_result = sample_healpix(&map, meta, &view, thetas[i], lons[i]);

        match (scalar_result, batch_mask[i]) {
            (Some(scalar), true) => {
                // Both valid - check values match
                assert!(
                    (batch_samples[i] - scalar).abs() < 1e-14,
                    "Sample mismatch at ({}, {}): batch={}, scalar={}",
                    thetas[i],
                    lons[i],
                    batch_samples[i],
                    scalar
                );
            }
            (None, false) => {
                // Both invalid - OK
            }
            _ => {
                panic!(
                    "Validity mismatch at ({}, {}): scalar_some={}, batch_valid={}",
                    thetas[i],
                    lons[i],
                    scalar_result.is_some(),
                    batch_mask[i]
                );
            }
        }
    }
}

#[test]
fn test_sample_healpix_batch_simd_matches_scalar() {
    use crate::rotation::{CoordSystem, ViewTransform};

    // Create a simple test map
    let nside = 16;
    let npix = 12 * nside * nside;
    let map: Vec<f64> = (0..npix).map(|i| (i as f64) * 0.1).collect();

    let meta = HealpixMeta {
        nside,
        ordering: HealpixOrdering::Ring,
        coord: CoordSystem::G,
    };

    // Identity view transform (no rotation, no coordinate change)
    let view = ViewTransform::new(CoordSystem::G, CoordSystem::G, None);

    // Test coordinates
    let thetas = [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.1, std::f64::consts::PI];
    let lons = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, std::f64::consts::TAU];

    // Get SIMD batch results
    let (simd_samples, simd_mask) = sample_healpix_batch_simd(&map, meta, &view, &thetas, &lons);

    // Verify each against scalar version
    for i in 0..8 {
        let scalar_result = sample_healpix(&map, meta, &view, thetas[i], lons[i]);

        match (scalar_result, simd_mask[i]) {
            (Some(scalar), true) => {
                // Both valid - check values match
                assert!(
                    (simd_samples[i] - scalar).abs() < 1e-12,
                    "SIMD Sample mismatch at ({}, {}): simd={}, scalar={}",
                    thetas[i],
                    lons[i],
                    simd_samples[i],
                    scalar
                );
            }
            (None, false) => {
                // Both invalid - OK
            }
            _ => {
                panic!(
                    "SIMD Validity mismatch at ({}, {}): scalar_some={}, simd_valid={}",
                    thetas[i],
                    lons[i],
                    scalar_result.is_some(),
                    simd_mask[i]
                );
            }
        }
    }
}

#[test]
fn test_sample_healpix_batch_simd_vs_batch() {
    use crate::rotation::{CoordSystem, ViewTransform};

    // Create a test map
    let nside = 16;
    let npix = 12 * nside * nside;
    let map: Vec<f64> = (0..npix).map(|i| ((i as f64) * 0.1) % 1.0).collect();

    let meta = HealpixMeta {
        nside,
        ordering: HealpixOrdering::Ring,
        coord: CoordSystem::G,
    };

    // Identity view transform
    let view = ViewTransform::new(CoordSystem::G, CoordSystem::G, None);

    // Test coordinates
    let thetas = [0.1, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.1];
    let lons = [0.5, 1.0, 1.5, 2.0, 3.0, 4.0, 5.0, 6.0];

    // Get batch and SIMD results
    let (batch_samples, batch_mask) = sample_healpix_batch(&map, meta, &view, &thetas, &lons);
    let (simd_samples, simd_mask) = sample_healpix_batch_simd(&map, meta, &view, &thetas, &lons);

    // Cross-validate
    for i in 0..8 {
        assert_eq!(
            batch_mask[i], simd_mask[i],
            "Mask mismatch at index {}: batch={}, simd={}",
            i, batch_mask[i], simd_mask[i]
        );

        if batch_mask[i] && simd_mask[i] {
            assert!(
                (batch_samples[i] - simd_samples[i]).abs() < 1e-13,
                "Batch vs SIMD sample mismatch at {}: batch={}, simd={}",
                i,
                batch_samples[i],
                simd_samples[i]
            );
        }
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
fn downgrade_healpix_map_ang(
    map: &[f64],
    source_nside: i64,
    target_nside: i64,
    ordering: HealpixOrdering,
) -> Vec<f64> {
    if source_nside <= target_nside {
        return map.to_vec();
    }

    let ratio = (source_nside / target_nside) as usize;
    // Each target pixel covers ratio*ratio source pixels
    let target_npix = (12 * target_nside * target_nside) as usize;
    let mut result = vec![0.0; target_npix];

    for (target_pix, result_elem) in result.iter_mut().enumerate() {
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

                let sample_theta = (theta
                    + d_theta * std::f64::consts::PI / (2.0 * target_nside as f64))
                    .clamp(0.0, std::f64::consts::PI);
                let sample_phi = (phi + d_phi * 2.0 * std::f64::consts::PI / target_nside as f64)
                    .rem_euclid(2.0 * std::f64::consts::PI);

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

        *result_elem = if count > 0 {
            sum / count as f64
        } else {
            HPX_UNSEEN
        };
    }

    result
}

fn downgrade_healpix_map_xyf(
    map: &[f64],
    source_nside: i64,
    target_nside: i64,
    ordering: HealpixOrdering,
) -> Vec<f64> {
    if source_nside <= target_nside {
        return map.to_vec();
    }
    assert_eq!(source_nside % target_nside, 0);

    let fact = source_nside / target_nside;
    let min_hits = 1;
    let target_npix = (12 * target_nside * target_nside) as usize;
    let mut result = vec![HPX_UNSEEN; target_npix];

    for (target_pix, result_elem) in result.iter_mut().enumerate() {
        // Convert target pixel to (x, y, face)
        let (x, y, face) = match ordering {
            HealpixOrdering::Ring => ring2xyf(target_nside, target_pix as i64),
            HealpixOrdering::Nested => nest2xyf(target_nside, target_pix as i64),
        };

        let mut sum = 0.0;
        let mut hits = 0usize;

        // Loop over corresponding source pixels
        let x0 = fact * x;
        let y0 = fact * y;

        for j in y0..(y0 + fact) {
            for i in x0..(x0 + fact) {
                let source_pix = match ordering {
                    HealpixOrdering::Ring => xyf2ring(source_nside, i, j, face),
                    HealpixOrdering::Nested => xyf2nest(source_nside, i, j, face),
                } as usize;

                let val = map[source_pix];
                if is_seen(val) {
                    sum += val;
                    hits += 1;
                }
            }
        }

        if hits >= min_hits {
            *result_elem = sum / hits as f64;
        }
    }

    result
}

pub fn downgrade_healpix_map(
    map: &[f64],
    source_nside: i64,
    target_nside: i64,
    ordering: HealpixOrdering,
) -> Vec<f64> {
    if target_nside < 256 {
        downgrade_healpix_map_ang(map, source_nside, target_nside, ordering)
    } else {
        downgrade_healpix_map_xyf(map, source_nside, target_nside, ordering)
    }
}
