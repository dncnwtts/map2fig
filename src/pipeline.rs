use crate::healpix::{read_healpix_meta, HealpixMeta, target_nside_for_resolution, downgrade_healpix_map,HealpixOrdering,HPX_UNSEEN};
use crate::fits::read_healpix_column;
use crate::generate_index_map;
use crate::rotation::CoordSystem;

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
) -> Result<ProcessedData, String> {
    let Some(new_fits_path) = fits_path 
        else {
            let map = generate_index_map(1);
            let meta = HealpixMeta {
                ordering: HealpixOrdering::Ring,
                nside:1,
                coord: CoordSystem::G
            };
            return Ok(ProcessedData { map, meta })
        };
    // Load metadata
    let meta = read_healpix_meta(new_fits_path)
        .ok_or_else(|| format!("Could not determine HEALPix ordering / NSIDE for file: {}", new_fits_path))?;

    // Load and scale data
    let mut map = read_healpix_column(new_fits_path, col);
    for v in &mut map {
        if *v == 0.0 {
            *v = HPX_UNSEEN;
        } else {
            *v *= scale_factor;
        }
    }


    // Apply downgrade for high-resolution maps
    let (final_map, final_meta) = if meta.nside > crate::HIGH_RES_NSIDE_THRESHOLD {

        let target_nside = target_nside_for_resolution(width as usize, (width / 2) as usize);

        if meta.nside  >target_nside {
            if verbose {println!("Downgrading from nside={} to nside={} for {}x{} output",
                    meta.nside, target_nside, width, width / 2);}

            let downgraded_map = downgrade_healpix_map(&map, meta.nside, 
                target_nside, meta.ordering);
            (
                downgraded_map, 
                HealpixMeta { 
                    nside: target_nside, 
                    ordering: meta.ordering,
                    coord: meta.coord,
             }
             )
        }
        else {
            (map, meta)
        }
    } else {
        (map, meta)
    };

    Ok(ProcessedData { map: final_map, meta: final_meta })
}
