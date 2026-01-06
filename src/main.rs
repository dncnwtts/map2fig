mod fits;
mod plot;
mod healpix;
mod colormap;

use clap::Parser;
use fits::read_healpix_column;
use crate::colormap::Colormap;
use crate::plot::NegMode;
use image::Rgba;

#[derive(Clone, Debug)]
struct RgbaArg {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}


#[derive(Clone, Debug)]
enum BadColor {
    Auto,
    Gray,
    Rgba(u8, u8, u8, u8),
}

use std::str::FromStr;

impl FromStr for RgbaArg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(',').collect();
        if parts.len() != 4 {
            return Err("Expected RGB as r,g,b,a".into());
        }

        let nums: Result<Vec<u8>, _> =
            parts.iter().map(|p| p.parse::<u8>()).collect();

        match nums {
            Ok(v) => Ok(Self {
                r: v[0],
                g: v[1],
                b: v[2],
                a: v[3]
            }),
            Err(_) => Err("RGBA values must be 0–255".into()),
        }
    }
}

impl FromStr for BadColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();

        match s.as_str() {
            "auto" => Ok(BadColor::Auto),
            "gray" | "grey" => Ok(BadColor::Gray),
            _ => {
                let parts: Vec<_> = s.split(',').collect();
                if parts.len() != 4 {
                    return Err(
                        "Expected 'auto', 'gray', or RGBA as r,g,b,a".into()
                    );
                }

                let vals: Result<Vec<u8>, _> =
                    parts.iter().map(|p| p.parse::<u8>()).collect();

                match vals {
                    Ok(v) => Ok(BadColor::Rgba(v[0], v[1], v[2], v[3])),
                    Err(_) => Err("RGBA values must be 0–255".into()),
                }
            }
        }
    }
}



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

    #[arg(short, long, default_value_t = 1600)]
    width: u32,

    #[arg(short, long, default_value = "output.png")]
    out: String,

    /// Disable map border
    #[arg(long)]
    no_border: bool,

    /// Transparent background (PNG)
    #[arg(long)]
    transparent: bool,

    /// Disable colorbar
    #[arg(long = "no-cbar", default_value_t = false)]
    no_cbar: bool,

    /// Lower color scale limit
    #[arg(long)]
    min: Option<f64>,
    
    /// Upper color scale limit
    #[arg(long)]
    max: Option<f64>,

    /// Gamma correction for colormap (1.0 = linear)
    #[arg(long, default_value_t = 1.0)]
    gamma: f64,

    /// Use logarithmic color scaling (positive values only)
    #[arg(long)]
    log: bool,
    
    /// Use symmetric logarithmic scaling
    #[arg(long)]
    symlog: bool,
    
    /// Linear region half-width for symlog
    #[arg(long, default_value_t = 0.0)]
    linthresh: f64,

    /// Use asinh color scaling
    #[arg(long)]
    asinh: bool,
    
    /// Asinh scale parameter (larger = more linear)
    #[arg(long, default_value_t = 0.0)]
    asinh_scale: f64,
    
    /// How to handle invalid/negative values
    #[arg(long, default_value = "unseen")]
    neg_mode: String,


    /// RGBA color for bad / masked pixels
    #[arg(long, default_value = "255,0,255,255")]
    bad_color: RgbaArg,

    #[arg(long, default_value = "auto")]
    bad_color: BadColor,
}


fn resolve_bad_color(
    bad: BadColor,
    background: Rgba<u8>,
) -> Rgba<u8> {
    match bad {
        BadColor::Auto => background,
        BadColor::Gray => Rgba([128, 128, 128, 255]),
        BadColor::Rgba(r, g, b, a) => Rgba([r, g, b, a]),
    }
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
    if  args.log as u8 + args.symlog as u8 + args.asinh as u8 > 1 {
        panic!("Only one of --log, --symlog, or --asinh may be specified");
    }

    let neg_mode = match args.neg_mode.as_str() {
        "zero" => NegMode::Zero,
        "unseen" => NegMode::Unseen,
        _ => panic!("--neg-mode must be 'zero' or 'unseen'"),
    };
    let background = Rgba([255, 255, 255, 255]); // or transparent
    let bad_color = resolve_bad_color(args.bad_color, background);


    println!("Reading HEALPix column {} from {}", args.col, args.fits);
    let map = read_healpix_column(&args.fits, args.col);
    // let map = generate_index_map(1);

    println!("Plotting Mollweide projection, width={} px, colormap={:?}", args.width, cmap);
    plot::plot_mollweide(&map, args.width, &args.out, 
        args.min, args.max, cmap, !args.no_cbar, args.transparent, !args.no_border,
        args.gamma, args.log, args.symlog, args.asinh,
        args.linthresh, args.asinh_scale, neg_mode, bad_color);

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

