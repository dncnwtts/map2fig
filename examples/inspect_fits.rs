/// Debug tool to inspect FITS file structure and data types
use std::io::Cursor;

fn main() {
    let files = vec![
        "tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits",
        "tests/data/npipe6v20_217_map_K.fits",
    ];

    for filepath in files {
        println!("\n╔══════════════════════════════════════════════════════════════╗");
        println!("║  File: {}", filepath);
        println!("╚══════════════════════════════════════════════════════════════╝");

        match inspect_fits(filepath) {
            Ok(info) => println!("{}", info),
            Err(e) => println!("Error: {}", e),
        }
    }
}

fn inspect_fits(filepath: &str) -> Result<String, String> {
    use fitsrs::{Fits, HDU, card::Value};
    use std::fs::File;

    let f = File::open(filepath).map_err(|e| format!("Cannot open: {}", e))?;
    let mut fits = Fits::from_reader(f);

    let mut output = String::new();
    let mut hdu_num = 0;

    while let Some(Ok(hdu)) = fits.next() {
        hdu_num += 1;

        match &hdu {
            HDU::XBinaryTable(table) => {
                output.push_str("✓ Binary Table HDU\n");

                let header = table.get_header();

                if let Some(Value::Integer { value, .. }) = header.get("NSIDE") {
                    output.push_str(&format!("  NSIDE: {}\n", value));
                }

                if let Some(Value::String { value, .. }) = header.get("INDXSCHM") {
                    output.push_str(&format!("  INDXSCHM: {}\n", value));
                } else {
                    output.push_str("  INDXSCHM: (not set - dense map)\n");
                }

                if let Some(Value::Integer { value, .. }) = header.get("NAXIS2") {
                    output.push_str(&format!("  Number of rows: {}\n", value));
                }

                output.push_str("  Columns:\n");

                for i in 1..=20 {
                    let tform_key = format!("TFORM{}", i);
                    let ttype_key = format!("TTYPE{}", i);

                    if let Some(Value::String { value: tform, .. }) = header.get(&tform_key) {
                        let ttype = header
                            .get(&ttype_key)
                            .and_then(|v| if let Value::String { value, .. } = v {
                                Some(value.clone())
                            } else {
                                None
                            })
                            .unwrap_or_else(|| "?".to_string());

                        let type_char = tform.chars().last().unwrap_or('?');
                        let dtype = match type_char {
                            'E' => "float32",
                            'D' => "float64",
                            'J' => "int32",
                            'K' => "int64",
                            _ => "unknown",
                        };

                        output.push_str(&format!("    Col {}: {} ({}, format: {})\n", i, ttype, dtype, tform));
                    } else {
                        break;
                    }
                }
            }
            _ => {
                output.push_str("✓ Primary HDU\n");
            }
        }
    }

    Ok(output)
}
