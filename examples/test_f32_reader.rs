use std::fs::File;
use memmap2::Mmap;

fn main() {
    let filename = "tests/data/npipe6v20_217_map_K.fits";
    println!("Testing f32 reader with: {}", filename);

    let f = File::open(filename).expect("Failed to open FITS file");
    let mmap = unsafe { Mmap::map(&f).expect("Failed to mmap FITS file") };

    println!("File size: {} bytes", mmap.len());
    
    // Try to load using the library
    let data = map2fig::read_healpix_column(filename, 0);
    
    println!("Successfully read {} pixels", data.len());
    println!("Data loaded successfully!");
}
