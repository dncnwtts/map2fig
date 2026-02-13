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

use fitsrs::{Fits, HDU, card::Value};
use fitsrs::hdu::data::bintable::{ColumnId, DataValue};

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
    let mut explicit_indices: Vec<i64> = Vec::new();
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
            
            // If explicit indexing, read both PIXEL and data columns
            if has_explicit_indexing && nside > 0 {
                // Read both columns at once - select_fields returns an iterator through all selected cells
                let all_values: Vec<DataValue> = table.select_fields(&[ColumnId::Index(0), ColumnId::Index(col_idx)]).collect();
                let mut data_vec: Vec<f64> = Vec::new();
                
                // Process pairs of values: (pixel_index, data_value)
                for chunk in all_values.chunks(2) {
                    if chunk.len() == 2 {
                        // Extract pixel index from first value
                        let pix = match &chunk[0] {
                            DataValue::Integer { value, .. } => *value as i64,
                            DataValue::Long { value, .. } => *value,
                            other => {
                                eprintln!("Warning: unexpected pixel index type: {:?}", other);
                                -1
                            }
                        };
                        
                        // Extract data value from second value
                        let val = match &chunk[1] {
                            DataValue::Double { value, .. }  => *value,
                            DataValue::Float  { value, .. }  => *value as f64,
                            DataValue::Integer{ value, .. }  => *value as f64,
                            other => panic!("Unsupported column type in FITS table: {:?}", other),
                        };
                        
                        explicit_indices.push(pix);
                        data_vec.push(val);
                    }
                }
                
                // Expand sparse map to full dense array
                let npix = (12 * nside * nside) as usize;
                let mut full_map = vec![f64::NEG_INFINITY; npix];  // Use NEG_INF for missing pixels
                for (idx, &pix) in explicit_indices.iter().enumerate() {
                    if idx < data_vec.len() && pix >= 0 && (pix as usize) < npix {
                        full_map[pix as usize] = data_vec[idx];
                    }
                }
                result = full_map;
            } else {
                // Regular dense map: read column directly
                let values = table.select_fields(&[ColumnId::Index(col_idx)]);
                for cell in values {
                    match cell {
                        DataValue::Double { value, .. }  => result.push(value),
                        DataValue::Float  { value, .. }  => result.push(value as f64),
                        DataValue::Integer{ value, .. }  => result.push(value as f64),
                        other => panic!("Unsupported column type in FITS table: {:?}", other),
                    }
                }
            }
        }
    }

    result
}

