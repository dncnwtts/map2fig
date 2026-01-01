mod fits;
mod plot;
mod healpix;

use fits::read_healpix_column;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <fits_file> [column_index]", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let col_index: usize = if args.len() >= 3 {
        args[2].parse().expect("Column index must be a number")
    } else {
        0
    };

    let map = read_healpix_column(filename, col_index);
    println!("Total pixels: {}", map.len());
    println!("First 10 pixels: {:?}", &map[..10.min(map.len())]);

    plot::plot_mollweide(&map, 512, 1024, 512, "output.png");
    println!("Saved Mollweide projection to output.png");

    let nside = 16; // small for testing
    let map2 = generate_index_map(nside);
    plot::plot_mollweide(&map2, nside, 1024, 512, "output2.png");
    println!("Saved Mollweide projection to output.png");

    /*

    plot::plot_mollweide_oval(1024, 512, "test_map.png");
    println!("Saved mollweide_black.png");
    */



}



pub fn generate_index_map(nside: i64) -> Vec<f64> {
    let npix = 12 * nside * nside;
    let mut map = Vec::with_capacity(npix as usize);

    for pix in 0..npix {
        map.push(pix as f64); // simple 0..npix-1 map
    }

    map
}

