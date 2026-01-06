mod fits;
mod plot;
mod healpix;
mod colormap;

use std::path::PathBuf;

use clap::Parser;
use fits::read_healpix_column;
use crate::colormap::Colormap;

/// Simple HEALPix Mollweide plotter
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Input FITS file
    #[arg(short, long, default_value = "cosmoglobe_DIRBE_10_I_n00512_DR2_v3.1.fits")]
    fits: String,

    #[arg(short = 'i', long, default_value_t = 0)]
    col: usize,
    
    #[arg(short = 'c', long, default_value = "viridis")]
    cmap: String,


    /// Output width in pixels
    #[arg(short, long, default_value_t = 1600)]
    width: u32,

    /// Output filename
    #[arg(short, long, default_value = "output.png")]
    out: String,
}

fn main() {
    let args = Args::parse();

    // Map colormap string to Colormap enum
    let cmap = match args.cmap.to_lowercase().as_str() {
        "viridis" => Colormap::Viridis,
        "plasma"  => Colormap::Plasma,
        "inferno" => Colormap::Inferno,
        other     => {
            eprintln!("Unknown colormap: {}", other);
            std::process::exit(1);
        }
    };

    println!("Reading HEALPix column {} from {}", args.col, args.fits);
    let map = read_healpix_column(&args.fits, args.col);

    println!("Plotting Mollweide projection, width={} px, colormap={:?}", args.width, cmap);
    plot::plot_mollweide(&map, args.width, &args.out, None, None, cmap);

    println!("Saved Mollweide projection to {}", args.out);
}





pub fn generate_index_map(nside: i64) -> Vec<f64> {
    let npix = 12 * nside * nside;
    let mut map = Vec::with_capacity(npix as usize);

    for pix in 0..npix {
        map.push(pix as f64); // simple 0..npix-1 map
    }

    map
}

