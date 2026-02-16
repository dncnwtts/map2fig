// Tier 1 Optimization Analysis: Direct Column Reading Without Type Conversion
//
// This analysis profiles the exact overhead of fitsrs DataValue enum conversion
// to determine if switching to a different FITS library or direct binary reading
// would be worthwhile.
//
// Build with: rustc -O tier1_analysis.rs -L target/release/deps --extern fitsrs=target/release/deps/libfitsrs-*.rlib
// Run with:   ./tier1_analysis tests/data/npipe6v20_217_map_K.fits

use std::fs::File;
use std::io::Cursor;
use std::time::Instant;

fn main() {
    let filename = std::env::args()
        .nth(1)
        .expect("Usage: tier1_analysis <fits_file>");

    println!("Tier 1 Optimization Analysis: FITS Column Reading Overhead");
    println!("File: {}", filename);
    println!("=".repeat(70));

    // Phase 1: Mmap and basic FITS structure
    let t0 = Instant::now();
    let f = File::open(&filename).expect("Failed to open FITS file");
    let mmap = unsafe { memmap2::Mmap::map(&f).expect("Failed to mmap") };
    let t_mmap = t0.elapsed();
    println!("\n[Phase 1] Memory map file: {:.3}s", t_mmap.as_secs_f64());

    // Phase 2: Parse FITS header and navigate to table
    let t0 = Instant::now();
    let cursor = Cursor::new(&mmap[..]);
    let mut fits = fitsrs::Fits::from_reader(cursor);
    
    let mut total_rows = 0;
    let mut col_format = String::new();
    
    while let Some(Ok(hdu)) = fits.next() {
        if let fitsrs::HDU::XBinaryTable(hdu) = hdu {
            let header = hdu.get_header();
            if let Some(fitsrs::card::Value::Integer { value, .. }) = header.get("NAXIS2") {
                total_rows = *value as usize;
            }
            // Try to get column format info
            if let Some(fitsrs::card::Value::String { value, .. }) = header.get("TFORM2") {
                col_format = value.trim().to_string();
            }
        }
    }
    let t_header = t0.elapsed();
    println!("[Phase 2] Parse FITS header: {:.3}s", t_header.as_secs_f64());
    println!("          Total rows in table: {}", total_rows);
    println!("          Column format code: {}", col_format);

    // Phase 3: Read column as DataValue enums (CURRENT APPROACH)
    let t0 = Instant::now();
    let cursor = Cursor::new(&mmap[..]);
    let mut fits = fitsrs::Fits::from_reader(cursor);
    let mut enum_values = Vec::new();
    
    while let Some(Ok(hdu)) = fits.next() {
        if let fitsrs::HDU::XBinaryTable(hdu) = hdu {
            let data = fits.get_data(&hdu);
            let mut table = data.table_data();
            let values: Vec<fitsrs::hdu::data::bintable::DataValue> = 
                table.select_fields(&[fitsrs::hdu::data::bintable::ColumnId::Index(1)])
                .collect();
            enum_values = values;
            break;
        }
    }
    let t_enum_read = t0.elapsed();
    println!("[Phase 3] Read column as enums: {:.3}s", t_enum_read.as_secs_f64());
    println!("          Loaded {} enum values", enum_values.len());

    // Phase 4: Convert enums to f64 (BOTTLENECK)
    let t0 = Instant::now();
    let mut f64_values = Vec::new();
    for cell in &enum_values {
        let val = match cell {
            fitsrs::hdu::data::bintable::DataValue::Double { value, .. } => *value,
            fitsrs::hdu::data::bintable::DataValue::Float { value, .. } => *value as f64,
            fitsrs::hdu::data::bintable::DataValue::Integer { value, .. } => *value as f64,
            _ => panic!("Unsupported type"),
        };
        f64_values.push(val);
    }
    let t_convert = t0.elapsed();
    println!("[Phase 4] Convert enums to f64: {:.3}s", t_convert.as_secs_f64());
    println!("          Rate: {:.1} million ops/sec", 
             enum_values.len() as f64 / 1e6 / t_convert.as_secs_f64());

    // Summary
    println!("\n" + "=".repeat(70));
    println!("Summary:");
    let total = t_mmap + t_header + t_enum_read + t_convert;
    println!("  Mmap:          {:6.2}% ({:.3}s)", 
             t_mmap.as_secs_f64() / total.as_secs_f64() * 100.0, t_mmap.as_secs_f64());
    println!("  Header parse:  {:6.2}% ({:.3}s)", 
             t_header.as_secs_f64() / total.as_secs_f64() * 100.0, t_header.as_secs_f64());
    println!("  Enum collect:  {:6.2}% ({:.3}s)", 
             t_enum_read.as_secs_f64() / total.as_secs_f64() * 100.0, t_enum_read.as_secs_f64());
    println!("  Enum convert:  {:6.2}% ({:.3}s) ← BOTTLENECK", 
             t_convert.as_secs_f64() / total.as_secs_f64() * 100.0, t_convert.as_secs_f64());
    println!("  Total:         {:.3}s", total.as_secs_f64());
    
    println!("\nTier 1 Target: Eliminate enum conversion phase (Phase 4)");
    println!("Expected improvement: {:.1}% speedup ({:.3}s saved)", 
             t_convert.as_secs_f64() / total.as_secs_f64() * 100.0,
             t_convert.as_secs_f64());
}
