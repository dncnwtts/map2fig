use clap::Parser;
use crate::{Scale, NegMode, get_colormap, Colormap, validate_scale_config};
use std::str::FromStr;
use image::Rgba;
use crate::rotation::{ViewTransform,CoordSystem,view_rotation,DEG2RAD};

/// Simple HEALPix Mollweide plotter
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Input FITS file
    #[arg(short, long)]
    pub fits: Option<String>,

    /// Column index
    #[arg(short='i', long, default_value_t = 0)]
    pub col: usize,

    /// Colormap name
    #[arg(short='c', long, default_value = "viridis")]
    pub cmap: String,

    /// Output width in pixels
    #[arg(short, long, default_value_t = 1200)]
    pub width: u32,

    /// Output filename
    #[arg(short, long, default_value = "output.pdf")]
    pub out: String,

    /// Disable map border
    #[arg(long)]
    pub no_border: bool,

    /// Transparent background
    #[arg(long)]
    pub transparent: bool,

    /// Disable colorbar
    #[arg(long)]
    pub no_cbar: bool,

    /// Lower color scale limit
    #[arg(long, allow_negative_numbers = true)]
    pub min: Option<f64>,

    /// Upper color scale limit
    #[arg(long, allow_negative_numbers = true)]
    pub max: Option<f64>,

    /// Gamma correction
    #[arg(long, default_value_t = 1.0)]
    pub gamma: f64,

    /// Log scale
    #[arg(long)]
    pub log: bool,

    /// Symmetric log
    #[arg(long)]
    pub symlog: bool,

    /// Histogram equalization
    #[arg(long)]
    pub hist: bool,

    /// Linear region width for symlog
    #[arg(long)]
    pub linthresh: Option<f64>,

    /// Asinh scaling
    #[arg(long)]
    pub asinh: bool,

    /// Negative/invalid handling: zero or unseen
    #[arg(long, default_value = "unseen")]
    pub neg_mode: String,

    /// Bad pixel color: auto, gray, or r,g,b,a
    #[arg(long)]
    pub bad_color: Option<InputColor>,

    /// Background pixel color: transparent, gray, or r,g,b,a
    #[arg(long)]
    pub bg_color: Option<InputColor>,

    /// Planck logarithmic scaling
    #[arg(long)]
    pub planck_log: bool,

    /// Factor that multiplies the data itself for unit conversions.
    #[arg(long, default_value_t = 1.0)]
    pub scale: f64,

    /// Enable LaTeX-like mathematical rendering for colorbar labels
    #[arg(long)]
    pub latex: bool,

    /// Units string for colorbar (supports LaTeX syntax when --latex is enabled)
    #[arg(long)]
    pub units: Option<String>,

    /// Input coordinate system: gal, eq, ecl
    #[arg(long, default_value = "gal")]
    pub input_coord: String,
    
    /// Output coordinate system: gal, eq, ecl
    #[arg(long, default_value = "gal")]
    pub output_coord: String,

    /// Rotate view so that (lon,lat) becomes the new center [degrees]
    #[arg(long, value_name = "LON,LAT")]
    pub rotate_to: Option<String>,
    
    /// Roll angle around the new center [degrees]
    #[arg(long, default_value_t = 0.0)]
    pub roll: f64,

    /// Allows for more verbose output
    #[arg(long)]
    pub verbose: bool,
}

/// Bad color option
#[derive(Clone, Debug)]
pub enum InputColor {
    Gray,
    Underflow,
    Overflow,
    Transparent,
    Rgba(u8, u8, u8, u8),
}

impl FromStr for InputColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.to_lowercase();
        match s.as_str() {
            "under" => Ok(InputColor::Underflow),
            "over"  => Ok(InputColor::Overflow),
            "gray" | "grey" => Ok(InputColor::Gray),
            _ => {
                let parts: Vec<_> = s.split(',').collect();
                if parts.len() != 4 {
                    return Err("Expected r,g,b,a".into());
                }
                let vals: Result<Vec<u8>, _> = parts.iter().map(|x| x.parse()).collect();
                match vals {
                    Ok(v) => Ok(InputColor::Rgba(v[0], v[1], v[2], v[3])),
                    Err(_) => Err("RGBA values must be 0–255".into())
                }
            }
        }
    }
}

pub fn resolve_input_color(input: Option<InputColor>, cmap: &Colormap, transparent: bool) -> Rgba<u8> {
    match input.unwrap_or(InputColor::Gray) {
        InputColor::Underflow => {
            let c = cmap.under();
            Rgba([c[0], c[1], c[2], if transparent { 0 } else { 255 }])
        }
        InputColor::Overflow => {
            let c = cmap.over();
            Rgba([c[0], c[1], c[2], if transparent { 0 } else { 255 }])
        }
        InputColor::Gray => Rgba([128, 128, 128, if transparent { 0 } else { 255 }]),
        InputColor::Transparent => Rgba([255,255,255,0]),
        InputColor::Rgba(r,g,b,a) => Rgba([r,g,b,a]),
    }
}

/// Resolved configuration for plotting
pub struct PlotConfig {
    pub scale: Scale,
    pub colormap: &'static Colormap,
    pub neg_mode: NegMode,
    pub bad_color_rgba: image::Rgba<u8>,
    pub bg_color_rgba: image::Rgba<u8>,
    pub latex_rendering: bool,
    pub units: Option<String>,
}

impl Args {
    pub fn resolve_config(&self) -> Result<PlotConfig, String> {
        // Resolve scale
        let (scale, cmap_name) = if self.planck_log {
            (
                Scale::PlanckLog {
                    linthresh: self.linthresh.unwrap_or(300.0),
                },
                "planck-log",
            )
        } else {
            let scale = if self.symlog {
                Scale::Symlog {
                    linthresh: self.linthresh.unwrap_or(1.0),
                }
            } else if self.asinh {
                Scale::Asinh {
                    scale: self.linthresh.unwrap_or(1.0),
                }
            } else if self.log {
                Scale::Log
            } else if self.hist {
                Scale::Histogram
            } else {
                Scale::Linear
            };

            (scale, self.cmap.as_str())
        };

        // Validate scale configuration
        validate_scale_config(&scale, self.min, self.max);

        // Get colormap
        let colormap = get_colormap(cmap_name);

        // Resolve negative mode
        let neg_mode = match self.neg_mode.as_str() {
            "zero" => NegMode::Zero,
            "unseen" => NegMode::Unseen,
            _ => return Err("--neg-mode must be 'zero' or 'unseen'".to_string()),
        };

        // Resolve colors
        let bad_color_rgba = resolve_input_color(self.bad_color.clone().or(Some(InputColor::Gray)), colormap, self.transparent);
        let bg_color_rgba = resolve_input_color(self.bg_color.clone().or(Some(InputColor::Transparent)), colormap, self.transparent);

        Ok(PlotConfig {
            scale,
            colormap,
            neg_mode,
            bad_color_rgba,
            bg_color_rgba,
            latex_rendering: self.latex,
            units: self.units.clone(),
        })
    }

    pub fn resolve_view_transform(&self) -> Result<ViewTransform, String> {
        let input = CoordSystem::from_str(&self.input_coord)?;
        let output = CoordSystem::from_str(&self.output_coord)?;

        let view = if let Some(ref s) = self.rotate_to {
            let parts: Vec<_> = s.split(',').collect();
            if parts.len() != 2 {
                return Err("--rotate-to expects lon,lat".into());
            }
            let lon = parts[0].parse::<f64>().map_err(|_| "bad lon")? * DEG2RAD;
            let lat = parts[1].parse::<f64>().map_err(|_| "bad lat")? * DEG2RAD;
            let roll = self.roll * DEG2RAD;
            Some(view_rotation(lon, lat, roll))
        } else {
            None
        };

        Ok(ViewTransform::new(input, output, view))
    }
}

