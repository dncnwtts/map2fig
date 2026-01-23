use crate::healpix::{read_healpix_meta, HealpixMeta, target_nside_for_resolution, downgrade_healpix_map};
use crate::fits::read_healpix_column;

/// Processed HEALPix data ready for plotting
pub struct ProcessedData {
    pub map: Vec<f64>,
    pub meta: HealpixMeta,
}

/// Load and process HEALPix data from FITS file
pub fn load_and_process_data(
    fits_path: &str,
    col: usize,
    scale_factor: f64,
    width: u32,
) -> Result<ProcessedData, String> {
    // Load metadata
    let meta = read_healpix_meta(fits_path)
        .ok_or_else(|| format!("Could not determine HEALPix ordering / NSIDE for file: {}", fits_path))?;

    // Load and scale data
    let mut map = read_healpix_column(fits_path, col);
    for v in &mut map {
        *v *= scale_factor;
    }

    // Apply downgrade for high-resolution maps
    let (final_map, final_meta) = if meta.nside > crate::HIGH_RES_NSIDE_THRESHOLD {
        println!("High-resolution map detected (nside={}), downgrading for performance", meta.nside);
        let target_nside = target_nside_for_resolution(width as usize, (width / 2) as usize);
        println!("Downgrading from nside={} to nside={} for {}x{} output",
                meta.nside, target_nside, width, width / 2);
        let downgraded_map = downgrade_healpix_map(&map, meta.nside, target_nside, meta.ordering);
        (downgraded_map, HealpixMeta { nside: target_nside, ordering: meta.ordering })
    } else {
        (map, meta)
    };

    Ok(ProcessedData { map: final_map, meta: final_meta })
}