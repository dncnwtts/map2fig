use crate::fits::read_healpix_column_cached;
use crate::generate_index_map;
use crate::healpix::{
    HPX_UNSEEN, HealpixMeta, HealpixOrdering, downgrade_healpix_map,
    downgrade_healpix_map_checkerboard, downgrade_healpix_map_balanced, is_seen, read_healpix_meta,
    target_nside_for_resolution,
};
use crate::rotation::CoordSystem;
use crate::QualityLevel;
use std::str::FromStr;

/// Processed HEALPix data ready for plotting
pub struct ProcessedData {
    pub map: Vec<f64>,
    pub meta: HealpixMeta,
}

/// Load and process HEALPix data from FITS file
pub fn load_and_process_data(
    fits_path: &Option<String>,
    col: usize,
    scale_factor: f64,
    width: u32,
    verbose: bool,
    no_downgrade: bool,
    quality: &str,
) -> Result<ProcessedData, String> {
    use std::time::Instant;

    let Some(new_fits_path) = fits_path else {
        let map = generate_index_map(1);
        let meta = HealpixMeta {
            ordering: HealpixOrdering::Ring,
            nside: 1,
            coord: CoordSystem::G,
        };
        return Ok(ProcessedData { map, meta });
    };
    // Load metadata
    let meta = read_healpix_meta(new_fits_path).ok_or_else(|| {
        format!(
            "Could not determine HEALPix ordering / NSIDE for file: {}",
            new_fits_path
        )
    })?;

    // Load and scale data (Tier 5.2.1: Column caching)
    // Note: Zero-valued pixels are treated as masked/unseen pixels.
    // This is important for files with explicit masking where 0.0 represents bad/masked data.
    // We check for zero values BEFORE scaling, and also preserve existing HPX_UNSEEN values.
    let fits_read_start = Instant::now();
    let mut map = read_healpix_column_cached(new_fits_path, col);
    let fits_read_time = fits_read_start.elapsed();

    for v in &mut map {
        // Skip already-unseen pixels (from FITS file with explicit HPX_UNSEEN values)
        if !is_seen(*v) {
            continue;
        }

        // Convert zero-valued pixels to HPX_UNSEEN (mask indicator for this file)
        // Use a small threshold (1e-20) instead of exact comparison to handle floating-point precision
        if v.abs() < 1e-20 {
            *v = HPX_UNSEEN;
        } else {
            *v *= scale_factor;
        }
    }

    // Apply downgrade for high-resolution maps (unless disabled)
    let (final_map, final_meta) = if !no_downgrade && meta.nside > crate::HIGH_RES_NSIDE_THRESHOLD {
        let target_nside = target_nside_for_resolution(width as usize, (width / 2) as usize);

        if meta.nside > target_nside {
            if verbose {
                println!(
                    "Downgrading from nside={} to nside={} for {}x{} output",
                    meta.nside,
                    target_nside,
                    width,
                    width / 2
                );
            }

            let downgrade_start = Instant::now();
            
            // Parse quality level and select appropriate downsampling algorithm
            let quality_level = QualityLevel::from_str(quality)
                .unwrap_or(QualityLevel::Best);
            
            let downgraded_map = match quality_level {
                QualityLevel::Best => {
                    // Exact downsampling (current algorithm)
                    downgrade_healpix_map(&map, meta.nside, target_nside, meta.ordering)
                }
                QualityLevel::Balanced => {
                    // Hybrid sampling: 50% of pixels (every 2nd in one dimension)
                    // ~2× speedup with ~2-3% error (slight noisiness visible)
                    downgrade_healpix_map_balanced(&map, meta.nside, target_nside, meta.ordering)
                }
                QualityLevel::Fast => {
                    // Checkerboard sampling: 4× speedup with ~10% error
                    downgrade_healpix_map_checkerboard(&map, meta.nside, target_nside, meta.ordering)
                }
            };
            let downgrade_time = downgrade_start.elapsed();

            if verbose {
                eprintln!("  FITS read:      {:.3}s", fits_read_time.as_secs_f64());
                eprintln!("  Downgrade:      {:.3}s", downgrade_time.as_secs_f64());
            }

            (
                downgraded_map,
                HealpixMeta {
                    nside: target_nside,
                    ordering: meta.ordering,
                    coord: meta.coord,
                },
            )
        } else {
            (map, meta)
        }
    } else {
        (map, meta)
    };

    Ok(ProcessedData {
        map: final_map,
        meta: final_meta,
    })
}

/// Subtract monopole (and optionally dipole) from a HEALPix map.
///
/// Uses least-squares fitting to compute the monopole and dipole components,
/// then subtracts them from the map. This follows the algorithm used in map_editor.
///
/// # Arguments
/// * `map`: Mutable reference to the map data
/// * `meta`: Metadata about the map
/// * `remove_monopole`: If true, remove the monopole
/// * `remove_dipole`: If true, remove the dipole (monopole is always included)
/// * `mask`: Optional mask (1.0 = good pixel, 0.0 = bad). If None, all UNSEEN pixels are masked.
/// * `verbose`: If true, print dipole parameters to stdout
///
/// The fit is performed only on pixels where mask == 1.0 (or not UNSEEN if no mask provided).
#[allow(clippy::collapsible_if, clippy::needless_range_loop)]
pub fn subtract_mono_dipole(
    map: &mut [f64],
    meta: HealpixMeta,
    remove_monopole: bool,
    remove_dipole: bool,
    mask: Option<&[f64]>,
    verbose: bool,
) {
    if !remove_monopole && !remove_dipole {
        return;
    }

    let npix = map.len();
    let nside = meta.nside;

    // Build harmonics: [1.0, x, y, z] for each pixel
    // where (x, y, z) is the unit vector in Cartesian coordinates
    let mut harmonics = vec![[0.0; 4]; npix];
    let mut valid_count = 0;

    for i in 0..npix {
        // Check if pixel is valid (not UNSEEN and not masked out)
        let is_valid = if let Some(m) = mask {
            is_seen(map[i]) && m[i] > 0.5
        } else {
            is_seen(map[i])
        };

        if is_valid {
            // Get pixel spherical coordinates
            let (theta, phi) = crate::healpix::pix2ang_ring(nside, i as i64);
            // Convert to Cartesian unit vector
            let vec = crate::rotation::sph_to_vec(theta, phi);

            harmonics[i][0] = 1.0;
            harmonics[i][1] = vec[0];
            harmonics[i][2] = vec[1];
            harmonics[i][3] = vec[2];
            valid_count += 1;
        }
    }

    if valid_count == 0 {
        if verbose {
            eprintln!("Warning: No valid pixels found for dipole/monopole fit");
        }
        return;
    }

    // Build the normal equations: A * x = b
    // A[j][k] = sum of harmonics[i][j] * harmonics[i][k] for all valid pixels
    // b[j] = sum of map[i] * harmonics[i][j] for all valid pixels
    #[allow(non_snake_case)]
    let mut A = [[0.0; 4]; 4];
    let mut b = [0.0; 4];

    for i in 0..npix {
        if !is_seen(map[i]) {
            continue;
        }
        if let Some(m) = mask {
            if m[i] <= 0.5 {
                continue;
            }
        }

        for j in 0..4 {
            b[j] += map[i] * harmonics[i][j];
            for k in j..4 {
                A[j][k] += harmonics[i][j] * harmonics[i][k];
            }
        }
    }

    // Symmetrize the matrix
    for j in 0..4 {
        for k in (j + 1)..4 {
            A[k][j] = A[j][k];
        }
    }

    // Solve the 4x4 linear system using Gaussian elimination with partial pivoting
    let multipoles = solve_4x4(A, b);

    if verbose {
        println!(
            "Monopole/Dipole fit: m={:.6e}, dx={:.6e}, dy={:.6e}, dz={:.6e}",
            multipoles[0], multipoles[1], multipoles[2], multipoles[3]
        );

        // Compute dipole amplitude and direction
        let dipole_amp = (multipoles[1] * multipoles[1]
            + multipoles[2] * multipoles[2]
            + multipoles[3] * multipoles[3])
            .sqrt();
        if dipole_amp > 0.0 {
            let lat = (multipoles[3] / dipole_amp).asin().to_degrees();
            let lon = multipoles[2].atan2(multipoles[1]).to_degrees();
            let lon = if lon < 0.0 { lon + 360.0 } else { lon };
            println!(
                "Dipole: Amplitude={:.6e}, Longitude={:.2}°, Latitude={:.2}°",
                dipole_amp, lon, lat
            );
        }
    }

    // Subtract the fit from the map
    let num_terms = if remove_dipole { 4 } else { 1 };

    for i in 0..npix {
        if is_seen(map[i]) {
            for j in 0..num_terms {
                map[i] -= multipoles[j] * harmonics[i][j];
            }
        }
    }
}

/// Solve a 4x4 linear system Ax = b using Gaussian elimination with partial pivoting.
#[allow(non_snake_case, clippy::needless_range_loop)]
fn solve_4x4(mut A: [[f64; 4]; 4], mut b: [f64; 4]) -> [f64; 4] {
    // Forward elimination with partial pivoting
    for col in 0..4 {
        // Find pivot
        let mut pivot_row = col;
        for row in (col + 1)..4 {
            if A[row][col].abs() > A[pivot_row][col].abs() {
                pivot_row = row;
            }
        }

        // Swap rows
        if pivot_row != col {
            for j in col..4 {
                (A[col][j], A[pivot_row][j]) = (A[pivot_row][j], A[col][j]);
            }
            b.swap(col, pivot_row);
        }

        // Check for singularity
        if A[col][col].abs() < 1e-15 {
            return [0.0; 4];
        }

        // Eliminate column
        for row in (col + 1)..4 {
            let factor = A[row][col] / A[col][col];
            for j in (col + 1)..4 {
                A[row][j] -= factor * A[col][j];
            }
            b[row] -= factor * b[col];
        }
    }

    // Back substitution
    let mut x = [0.0; 4];
    for i in (0..4).rev() {
        x[i] = b[i];
        for j in (i + 1)..4 {
            x[i] -= A[i][j] * x[j];
        }
        x[i] /= A[i][i];
    }

    x
}
