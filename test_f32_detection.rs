use std::fs::File;
use std::io::Cursor;
use memmap2::Mmap;
use fitsrs::{Fits, HDU, card::Value};

fn main() {
    let filepath = "tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits";
    
    let f = File::open(filepath).expect("Failed to open file");
    let mmap = unsafe { Mmap::map(&f).expect("Failed to mmap") };
    
    let cursor = Cursor::new(&mmap[..]);
    let mut fits = Fits::from_reader(cursor);
    let mut col_idx = 0;
    
    while let Some(Ok(hdu)) = fits.next() {
        if let HDU::XBinaryTable(hdu) = hdu {
            let header = hdu.get_header();
            
            println!("Found BINTABLE:");
            
            // Check TFORM
            let tform_key = format!("TFORM{}", col_idx + 1);
            match header.get(&tform_key) {
                Some(Value::String { value, .. }) => {
                    println!("  TFORM{}: {:?}", col_idx + 1, value);
                    let type_char = value.trim().chars().last();
                    println!("  Type char: {:?}", type_char);
                }
                _ => println!("  No {} found", tform_key),
            }
            
            // Check INDXSCHM
            match header.get("INDXSCHM") {
                Some(Value::String { value, .. }) => println!("  INDXSCHM: {:?}", value),
                _ => println!("  No INDXSCHM"),
            }
            
            // Check NSIDE
            match header.get("NSIDE") {
                Some(Value::Integer { value, .. }) => println!("  NSIDE: {}", value),
                _ => println!("  No NSIDE"),
            }
            
            // Check NAXIS1, NAXIS2
            match (header.get("NAXIS1"), header.get("NAXIS2")) {
                (Some(Value::Integer { value: n1, .. }), Some(Value::Integer { value: n2, .. })) => {
                    println!("  NAXIS1: {}, NAXIS2: {}", n1, n2);
                }
                _ => println!("  Missing NAXIS1 or NAXIS2"),
            }
            
            break;
        }
    }
}
