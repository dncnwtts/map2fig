/// Pixel masking system for HEALPix maps
/// Handles: file-based masks, value-range masks, and coordinate system transformations
use crate::rotation::CoordSystem;
use image::Rgba;

/// Represents a pixel mask where true = valid, false = masked
#[derive(Clone, Debug)]
pub struct PixelMask {
    /// Per-pixel mask data (true = show, false = hide)
    pub data: Vec<bool>,
    /// Color to fill masked regions (None = transparent/background)
    pub fill_color: Option<Rgba<u8>>,
    /// Coordinate system of the mask (for coordinate mismatch detection)
    pub coord_system: CoordSystem,
    /// NSIDE of the mask (must match data NSIDE)
    pub nside: i64,
}

impl PixelMask {
    /// Create a mask from a boolean vector
    pub fn from_vec(
        data: Vec<bool>,
        nside: i64,
        fill_color: Option<Rgba<u8>>,
        coord_system: CoordSystem,
    ) -> Self {
        Self {
            data,
            fill_color,
            coord_system,
            nside,
        }
    }

    /// Check if a pixel is valid (not masked)
    #[inline]
    pub fn is_valid(&self, pixel_index: usize) -> bool {
        if pixel_index < self.data.len() {
            self.data[pixel_index]
        } else {
            true // Out of bounds pixels are considered valid by default
        }
    }

    /// Create a mask by value range
    /// masked_below: mask pixels with values < this threshold
    /// masked_above: mask pixels with values > this threshold
    pub fn from_value_range(
        data: &[f64],
        masked_below: Option<f64>,
        masked_above: Option<f64>,
        nside: i64,
        fill_color: Option<Rgba<u8>>,
        coord_system: CoordSystem,
    ) -> Self {
        let mask_data: Vec<bool> = data
            .iter()
            .map(|&val| {
                // Check if value is within valid range
                let below_ok = masked_below
                    .map(|threshold| val >= threshold)
                    .unwrap_or(true);
                let above_ok = masked_above
                    .map(|threshold| val <= threshold)
                    .unwrap_or(true);
                below_ok && above_ok
            })
            .collect();

        Self::from_vec(mask_data, nside, fill_color, coord_system)
    }

    /// Load mask from FITS file
    /// FITS file should contain a binary mask where 0=masked, 1=valid
    /// The mask should be in a binary table with NSIDE calculated from data size
    pub fn from_fits_file(
        path: &str,
        fill_color: Option<Rgba<u8>>,
        mask_coord: Option<CoordSystem>,
    ) -> Result<Self, String> {
        use fitsrs::hdu::data::bintable::{ColumnId, DataValue};
        use fitsrs::{Fits, HDU};
        use std::fs::File;
        use std::io::BufReader;

        // Open FITS file
        let f =
            File::open(path).map_err(|e| format!("Failed to open FITS file '{}': {}", path, e))?;
        let reader = BufReader::new(f);
        let mut fits = Fits::from_reader(reader);

        let mut mask_data: Vec<f64> = Vec::new();

        // Read mask from first binary table's first column (column 0)
        while let Some(Ok(hdu)) = fits.next() {
            if let HDU::XBinaryTable(hdu_ref) = hdu {
                let data = fits.get_data(&hdu_ref);
                let mut table = data.table_data();
                let values = table.select_fields(&[ColumnId::Index(0)]);

                for cell in values {
                    match cell {
                        DataValue::Double { value, .. } => mask_data.push(value),
                        DataValue::Float { value, .. } => mask_data.push(value as f64),
                        DataValue::Integer { value, .. } => mask_data.push(value as f64),
                        _ => {}
                    }
                }

                // Found data, stop looking
                if !mask_data.is_empty() {
                    break;
                }
            }
        }

        // Validate we got data
        if mask_data.is_empty() {
            return Err(format!(
                "No mask data found in FITS file '{}'. \
                Ensure file contains a binary table with mask in first column.",
                path
            ));
        }

        // Calculate nside from npix: nside = sqrt(npix / 12)
        let npix = mask_data.len() as f64;
        let nside_f = (npix / 12.0).sqrt();
        let nside = nside_f as i64;

        // Validate that npix is a valid HEALPix size
        if (nside * nside * 12) as f64 != npix {
            return Err(format!(
                "Invalid mask size: {} pixels. Expected 12*nside^2 for some integer nside.",
                mask_data.len()
            ));
        }

        if !(1..=1024).contains(&nside) {
            return Err(format!(
                "Calculated NSIDE {} is out of range [1, 1024]",
                nside
            ));
        }

        // Convert to boolean (0 or near-0 = masked, anything > 0.5 = valid)
        let mask_bool: Vec<bool> = mask_data.into_iter().map(|val| val > 0.5).collect();

        // Default to Galactic coordinate system (can be overridden)
        let coord_system = mask_coord.unwrap_or(CoordSystem::G);

        Ok(Self::from_vec(mask_bool, nside, fill_color, coord_system))
    }

    /// Check if mask needs transformation (coordinate mismatch)
    pub fn needs_transform(&self, data_coord: CoordSystem) -> bool {
        self.coord_system != data_coord
    }

    /// Get warning message if coordinate systems mismatch
    pub fn warn_coord_mismatch(&self, data_coord: CoordSystem, verbose: bool) -> Option<String> {
        if self.needs_transform(data_coord) && verbose {
            Some(format!(
                "Warning: Mask is in {:?} but data is in {:?}. \
                Mask will be applied directly without transformation. \
                Results may be incorrect if coordinates don't align!",
                self.coord_system, data_coord
            ))
        } else {
            None
        }
    }
}

/// Parse maskfill color string
pub fn parse_maskfill_color(color_str: &str) -> Option<Rgba<u8>> {
    match color_str.to_lowercase().as_str() {
        "transparent" => None,
        "gray" | "grey" => Some(Rgba([128, 128, 128, 255])),
        _ => {
            // Try to parse as r,g,b,a format
            let parts: Vec<&str> = color_str.split(',').collect();
            if parts.len() == 4
                && let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                    parts[0].trim().parse::<u8>(),
                    parts[1].trim().parse::<u8>(),
                    parts[2].trim().parse::<u8>(),
                    parts[3].trim().parse::<u8>(),
                )
            {
                return Some(Rgba([r, g, b, a]));
            }
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_range_masking() {
        let data = vec![0.5, 1.5, 2.5, 3.5];
        let mask =
            PixelMask::from_value_range(&data, Some(1.0), Some(3.0), 4, None, CoordSystem::G);

        assert!(!mask.is_valid(0)); // 0.5 < 1.0
        assert!(mask.is_valid(1)); // 1.5 in range
        assert!(mask.is_valid(2)); // 2.5 in range
        assert!(!mask.is_valid(3)); // 3.5 > 3.0
    }

    #[test]
    fn test_parse_maskfill_color() {
        assert_eq!(parse_maskfill_color("transparent"), None);
        assert_eq!(
            parse_maskfill_color("gray"),
            Some(Rgba([128, 128, 128, 255]))
        );
        assert_eq!(
            parse_maskfill_color("255,0,0,255"),
            Some(Rgba([255, 0, 0, 255]))
        );
    }
}
