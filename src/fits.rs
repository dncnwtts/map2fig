//! FITS file reading and data extraction.
//!
//! This module provides functions for reading HEALPix astronomical data from FITS files.
//! It supports both:
//! - **Dense maps**: Complete sky coverage (standard NSIDE² pixels)
//! - **Sparse maps**: Partial sky coverage with explicit PIXEL column indexing
//!
//! # FITS Format Support
//!
//! - Binary table format (.fits files)
//! - HEALPix RING or NEST pixel ordering
//! - Sparse maps with IMPLICIT or EXPLICIT indexing schemes via the INDXSCHM keyword
//!
//! # Sparse Map Handling
//!
//! For sparse maps with EXPLICIT indexing, this module automatically expands the data
//! to a full dense array with UNSEEN values for omitted pixels.
//!
//! # Examples
//!
//! ```ignore
//! use map2fig::read_healpix_column;
//!
//! let data = read_healpix_column("map.fits", 0);
//! println!("Loaded {} pixels", data.len());
//! ```

use std::fs::File;
use std::io::BufReader;

use fitsrs::hdu::data::bintable::{ColumnId, DataValue};
use fitsrs::{Fits, HDU, card::Value};

/// Read a HEALPix column from a FITS binary table.
///
/// Extracts data from a specific column of a HEALPix FITS binary table.
/// Automatically handles both dense and sparse (EXPLICIT) indexing schemes.
/// For sparse maps, expands the result to full NSIDE² size with UNSEEN values.
///
/// # Arguments
///
/// * `filename` - Path to the FITS file
/// * `col_idx`  - 0-based data column index (not the PIXEL column, just the data)
///
/// # Returns
///
/// Vector of f64 values with length = 12 * NSIDE²
/// - Dense maps: all pixels present in FITS
/// - Sparse maps: UNSEEN (-1.6375e30) for missing pixels
///
/// # Panics
///
/// Panics if:
/// - File cannot be opened
/// - FITS structure is invalid
/// - Column index is out of bounds
/// - Required HEALPix headers are missing
pub fn read_healpix_column(filename: &str, col_idx: usize) -> Vec<f64> {
    let f = File::open(filename).expect("Failed to open FITS file");
    let reader = BufReader::new(f);

    let mut fits = Fits::from_reader(reader);
    let mut result: Vec<f64> = Vec::new();
    let mut nside: i64 = 0;

    while let Some(Ok(hdu)) = fits.next() {
        if let HDU::XBinaryTable(hdu) = hdu {
            let header = hdu.get_header();

            // Check if this uses explicit indexing (sparse/partial sky map)
            let has_explicit_indexing = match header.get("INDXSCHM") {
                Some(Value::String { value, .. }) => value.trim() == "EXPLICIT",
                _ => false,
            };

            // Get NSIDE if not already set
            if nside == 0 {
                nside = match header.get("NSIDE") {
                    Some(Value::Integer { value, .. }) => *value,
                    _ => 0,
                };
            }

            let data = fits.get_data(&hdu);
            let mut table = data.table_data();

            // If explicit indexing, read both PIXEL and data columns together
            if has_explicit_indexing && nside > 0 {
                // For explicit indexing:
                // - Column 0 is always PIXEL indices
                // - Column 1+ are data columns
                // - User's --col N refers to the N-th data column
                // - Adjust file column: file_col = col_idx + 1
                let file_col_for_data = col_idx + 1;

                // Read both PIXEL (col 0) and data column
                let all_values: Vec<DataValue> = table
                    .select_fields(&[ColumnId::Index(0), ColumnId::Index(file_col_for_data)])
                    .collect();

                if all_values.is_empty() {
                    result = vec![f64::NEG_INFINITY; (12 * nside * nside) as usize];
                } else {
                    let n_rows = all_values.len() / 2;
                    let npix = (12 * nside * nside) as usize;
                    let mut full_map = vec![f64::NEG_INFINITY; npix];

                    // extract pixel indices and data
                    // NOTE: select_fields returns interleaved values [pix, data, pix, data, ...]
                    for row_idx in 0..n_rows {
                        let pix_idx = row_idx * 2;
                        let data_idx = row_idx * 2 + 1;

                        let pix = match &all_values[pix_idx] {
                            DataValue::Integer { value, .. } => *value as i64,
                            DataValue::Long { value, .. } => *value,
                            DataValue::Float { value, .. } => *value as i64,
                            DataValue::Double { value, .. } => *value as i64,
                            _ => -1,
                        };

                        let val = match &all_values[data_idx] {
                            DataValue::Double { value, .. } => *value,
                            DataValue::Float { value, .. } => *value as f64,
                            DataValue::Integer { value, .. } => *value as f64,
                            other => panic!("Unsupported column type in FITS table: {:?}", other),
                        };

                        if pix >= 0 && (pix as usize) < npix {
                            full_map[pix as usize] = val;
                        }
                    }
                    result = full_map;
                }
            } else {
                // Regular dense map: read column directly
                let values = table.select_fields(&[ColumnId::Index(col_idx)]);
                for cell in values {
                    match cell {
                        DataValue::Double { value, .. } => result.push(value),
                        DataValue::Float { value, .. } => result.push(value as f64),
                        DataValue::Integer { value, .. } => result.push(value as f64),
                        other => panic!("Unsupported column type in FITS table: {:?}", other),
                    }
                }
            }
        }
    }

    result
}
