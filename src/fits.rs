use std::fs::File;
use std::io::BufReader;

use fitsrs::{Fits, HDU};
use fitsrs::hdu::data::bintable::{ColumnId, DataValue};

/// Reads a HEALPix FITS binary table column and returns a Vec<f64>.
/// 
/// # Arguments
/// * `filename` - Path to the FITS file
/// * `col_idx`  - 0-based column index
pub fn read_healpix_column(filename: &str, col_idx: usize) -> Vec<f64> {
    let f = File::open(filename).expect("Failed to open FITS file");
    let reader = BufReader::new(f);

    let mut fits = Fits::from_reader(reader);
    let mut result: Vec<f64> = Vec::new();

    while let Some(Ok(hdu)) = fits.next() {
        if let HDU::XBinaryTable(hdu) = hdu {
            let data = fits.get_data(&hdu);
            let mut table = data.table_data();
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

    result
}

