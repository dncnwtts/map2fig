/// Debug why the F32 native path isn't being used
use std::fs::File;
use std::io::Cursor;

fn main() {
    println!("\nDebug: Why is f32 native path not triggering?\n");

    let filepath = "tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits";

    match test_f32_detection(filepath) {
        Ok(result) => println!("{}", result),
        Err(e) => println!("Error: {}", e),
    }
}

fn test_f32_detection(filepath: &str) -> Result<String, String> {
    use fitsrs::{Fits, HDU, card::Value};
    use memmap2::Mmap;

    let f = File::open(filepath).map_err(|e| format!("Cannot open: {}", e))?;
    let mmap = unsafe { Mmap::map(&f).map_err(|e| format!("Cannot mmap: {}", e))? };

    let mut output = String::new();
    output.push_str("Step 1: Parse FITS headers\n");

    let cursor = Cursor::new(&mmap[..]);
    let mut fits = Fits::from_reader(cursor);

    while let Some(Ok(hdu)) = fits.next() {
        if let HDU::XBinaryTable(hdu) = hdu {
            let header = hdu.get_header();

            // Check 1: Explicit indexing?
            let has_explicit_indexing = match header.get("INDXSCHM") {
                Some(Value::String { value, .. }) => value.trim() == "EXPLICIT",
                _ => false,
            };
            output.push_str(&format!(
                "  ✓ Explicit indexing check: {}\n",
                has_explicit_indexing
            ));

            if has_explicit_indexing {
                output.push_str("  ✗ BLOCKED: Has explicit indexing (would use fallback)\n");
                return Ok(output);
            }

            // Check 2: Get NSIDE
            let nside = match header.get("NSIDE") {
                Some(Value::Integer { value, .. }) => *value,
                _ => {
                    output.push_str("  ✗ BLOCKED: No NSIDE found\n");
                    return Ok(output);
                }
            };
            output.push_str(&format!("  ✓ NSIDE: {}\n", nside));

            // Check 3: Get column type
            let tform_key = "TFORM1";
            let tform_str = match header.get(tform_key) {
                Some(Value::String { value, .. }) => value.clone(),
                _ => {
                    output.push_str("  ✗ BLOCKED: No TFORM1 found\n");
                    return Ok(output);
                }
            };
            output.push_str(&format!("  ✓ TFORM: {}\n", tform_str));

            // Parse TFORM
            let type_char = tform_str.chars().last().unwrap_or('?');
            output.push_str(&format!("  Type char: {}\n", type_char));

            if type_char != 'E' {
                output.push_str(&format!(
                    "  ✗ BLOCKED: Not f32 (type_char={}, need 'E' for f32)\n",
                    type_char
                ));
                return Ok(output);
            }
            output.push_str("  ✓ Is f32\n");

            // Check 4: Get column offset
            let toffset_key = "TOFFSET1";
            let col_offset: usize = match header.get(toffset_key) {
                Some(Value::Integer { value, .. }) => *value as usize,
                _ => {
                    output.push_str("  ✗ BLOCKED: No TOFFSET1\n");
                    return Ok(output);
                }
            };
            output.push_str(&format!("  ✓ Column offset: {}\n", col_offset));

            // Check 5: Get row size
            let row_size: usize = match header.get("NAXIS1") {
                Some(Value::Integer { value, .. }) => *value as usize,
                _ => {
                    output.push_str("  ✗ BLOCKED: No NAXIS1\n");
                    return Ok(output);
                }
            };
            output.push_str(&format!("  ✓ Row size: {}\n", row_size));

            // Check 6: Get number of rows
            let num_rows: usize = match header.get("NAXIS2") {
                Some(Value::Integer { value, .. }) => *value as usize,
                _ => {
                    output.push_str("  ✗ BLOCKED: No NAXIS2\n");
                    return Ok(output);
                }
            };
            output.push_str(&format!("  ✓ Number of rows: {}\n", num_rows));

            // Check 7: Find data offset
            output.push_str("\nStep 2: Find data offset in binary\n");
            const FITS_BLOCK_SIZE: usize = 2880;

            let mut found_end = false;
            for block_num in 0..1000 {
                let block_start = block_num * FITS_BLOCK_SIZE;
                if block_start + 80 > mmap.len() {
                    output
                        .push_str("  ✗ BLOCKED: Reached end of mmap without finding END keyword\n");
                    return Ok(output);
                }

                let block = &mmap[block_start..];

                for card_off in (0..block.len()).step_by(80) {
                    if card_off + 3 <= block.len() {
                        let card_start = &block[card_off..card_off + 8];
                        if card_start.starts_with(b"END     ") {
                            let data_offset = (block_num + 1) * FITS_BLOCK_SIZE;
                            output.push_str(&format!(
                                "  ✓ Found END at block {}, data starts at offset {}\n",
                                block_num, data_offset
                            ));
                            found_end = true;

                            // Check 8: Verify we can read data
                            let test_read_start = data_offset;
                            let test_read_end = test_read_start + 16; // try to read 4 f32 values

                            if test_read_end > mmap.len() {
                                output.push_str(
                                    "  ✗ BLOCKED: Data offset would read past end of file\n",
                                );
                            } else {
                                output.push_str("  ✓ Data offset is valid\n");
                                output.push_str(
                                    "\n✅ ALL CHECKS PASSED - f32 native path SHOULD trigger!\n",
                                );
                                output.push_str("\nIf it didn't, there may be an issue with:\n");
                                output.push_str("- elem_count parsing from TFORM\n");
                                output
                                    .push_str("- Endianness assumptions (file uses big-endian?)\n");
                                output.push_str(
                                    "- Multiple columns (f32 path only handles single column)\n",
                                );
                            }

                            break;
                        }
                    }
                }

                if found_end {
                    break;
                }
            }

            if !found_end {
                output.push_str("  ✗ BLOCKED: Never found END keyword\n");
            }

            break;
        }
    }

    Ok(output)
}
