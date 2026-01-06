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

    let width = 1600;

    //plot::plot_mollweide(&map, width, "output.png", Some(0.0), Some(50.0));
    plot::plot_mollweide(&map, width, "output.png", None, None);
    println!("Saved Mollweide projection to output.png");


}



pub fn generate_index_map(nside: i64) -> Vec<f64> {
    let npix = 12 * nside * nside;
    let mut map = Vec::with_capacity(npix as usize);

    for pix in 0..npix {
        map.push(pix as f64); // simple 0..npix-1 map
    }

    map
}

