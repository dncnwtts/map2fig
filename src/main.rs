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

    plot::plot_mollweide(&map, 512, "output.png");
    println!("Saved Mollweide projection to output.png");

}

