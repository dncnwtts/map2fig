use crate::rotation::{CoordSystem, DEG2RAD, ViewTransform, view_rotation};
use crate::{Colormap, NegMode, Scale, get_colormap, validate_scale_config};
use clap::Parser;
use image::Rgba;
use std::str::FromStr;

/// Fast, publication-quality HEALPix sky map visualization
///
/// EXAMPLES:
///   map2fig input.fits                              # Auto-generates input.png
///   map2fig input.fits output.pdf                   # Explicit output filename
///   map2fig input.fits output.pdf -c viridis --log # With custom colormap and scaling
///   map2fig input.fits -c plasma --hist             # Auto-generates input.png with options
///
/// ADVANCED OPTIONS:
///   Use --projection (mollweide/gnomonic/hammer), --graticule, --rotate-to,
///   --lon, --lat for coordinate system and view changes. Many more options
///   available for color customization, masks, text labels, and fonts.
///   
///   Run 'grep -E "arg\(long" src/cli.rs' to see all available options,
///   or check the documentation.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, override_usage = "map2fig [FITS] [OUTPUT] [OPTIONS]")]
pub struct Args {
    /// Input FITS file
    #[arg(value_name = "FITS", help_heading = None)]
    pub fits: Option<String>,

    /// Output filename (PNG or PDF). If not provided, auto-generates from input filename (e.g., input.fits -> input.png)
    #[arg(value_name = "OUTPUT", help_heading = None)]
    pub out: Option<String>,

    /// Column index
    #[arg(short = 'i', long, default_value_t = 0, required = false)]
    pub col: usize,

    /// Colormap name
    #[arg(short = 'c', long, default_value = "planck", required = false)]
    pub cmap: String,

    /// Output width in pixels
    #[arg(short, long, default_value_t = 1200, required = false)]
    pub width: u32,

    /// Lower color scale limit
    #[arg(long, allow_negative_numbers = true)]
    pub min: Option<f64>,

    /// Upper color scale limit
    #[arg(long, allow_negative_numbers = true)]
    pub max: Option<f64>,

    /// Disable map border
    #[arg(long)]
    pub no_border: bool,

    /// Transparent background
    #[arg(long)]
    pub transparent: bool,

    /// Disable colorbar
    #[arg(long)]
    pub no_cbar: bool,

    /// Gamma correction
    #[arg(long, default_value_t = 1.0, required = false)]
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
    #[arg(long, hide = true, required = false)]
    pub linthresh: Option<f64>,

    /// Asinh scaling
    #[arg(long, hide = true, required = false)]
    pub asinh: bool,

    /// Negative/invalid handling: zero or unseen
    #[arg(long, default_value = "unseen", hide = true, required = false)]
    pub neg_mode: String,

    /// Bad pixel color: auto, gray, or r,g,b,a
    #[arg(long, hide = true, required = false)]
    pub bad_color: Option<InputColor>,

    /// Background pixel color: transparent, gray, or r,g,b,a
    #[arg(long, hide = true, required = false)]
    pub bg_color: Option<InputColor>,

    /// Planck logarithmic scaling
    #[arg(long, hide = true, required = false)]
    pub planck_log: bool,

    /// Factor that multiplies the data itself for unit conversions.
    #[arg(long, default_value_t = 1.0, required = false)]
    pub scale: f64,

    /// Enable LaTeX-like mathematical rendering for colorbar labels
    #[arg(long, required = false)]
    pub latex: bool,

    /// Units string for colorbar (supports LaTeX syntax when --latex is enabled)
    #[arg(long, required = false)]
    pub units: Option<String>,

    /// Input coordinate system: gal, eq, ecl
    #[arg(long, default_value = "gal", hide = true, required = false)]
    pub input_coord: String,

    /// Output coordinate system: gal, eq, ecl
    #[arg(long, default_value = "gal", hide = true, required = false)]
    pub output_coord: String,

    /// Rotate view so that (lon,lat) becomes the new center \[degrees\]
    #[arg(long, value_name = "LON,LAT", required = false)]
    pub rotate_to: Option<String>,

    /// Roll angle around the new center \[degrees\]
    #[arg(long, default_value_t = 0.0, required = false)]
    pub roll: f64,

    /// Projection type: mollweide, gnomonic, or hammer
    #[arg(long, default_value = "mollweide", required = false)]
    pub projection: String,

    /// Center longitude in degrees (gnomonic: projection center; mollweide: rotation center)
    #[arg(
        long,
        alias = "gnom-lon",
        allow_negative_numbers = true,
        required = false
    )]
    pub lon: Option<f64>,

    /// Center latitude in degrees (gnomonic: projection center; mollweide: rotation center)
    #[arg(
        long,
        alias = "gnom-lat",
        allow_negative_numbers = true,
        required = false
    )]
    pub lat: Option<f64>,

    /// Field of view width in arcminutes (gnomonic projection only)
    #[arg(long, alias = "gnom-width", default_value_t = 300.0, required = false)]
    pub fov: f64,

    /// Resolution in arcmin/pixel (gnomonic projection only)
    #[arg(long, alias = "gnom-res", default_value_t = 1.0, required = false)]
    pub res: f64,

    /// Enable local grid graticule for gnomonic projection
    #[arg(long, alias = "gnom-graticule", hide = true, required = false)]
    pub local_graticule: bool,

    /// Graticule spacing for parallels \[degrees\] (gnomonic projection only)
    #[arg(
        long,
        alias = "gnom-grat-dlat",
        default_value_t = 1.0,
        hide = true,
        required = false
    )]
    pub local_grat_dlat: f64,

    /// Graticule spacing for meridians \[degrees\] (gnomonic projection only)
    #[arg(
        long,
        alias = "gnom-grat-dlon",
        default_value_t = 1.0,
        hide = true,
        required = false
    )]
    pub local_grat_dlon: f64,

    /// Graticule line width in pixels (applies to both local and mollweide graticules)
    #[arg(long, default_value_t = 1, hide = true, required = false)]
    pub grat_line_width: u32,

    /// Disable text labels (title and resolution/pixel size labels)
    #[arg(long, hide = true, required = false)]
    pub no_text: bool,

    /// Disable title display
    #[arg(long, hide = true, required = false)]
    pub no_title: bool,

    /// Disable text scaling with FOV (use constant text sizes, gnomonic projection only)
    #[arg(long, hide = true, required = false)]
    pub no_scale_text: bool,

    /// Custom title for map (gnomonic: default is (lon, lat) at center)
    #[arg(long, hide = true, required = false)]
    pub title: Option<String>,

    /// Allows for more verbose output
    #[arg(long, required = false)]
    pub verbose: bool,

    /// Disable automatic downgrading of high-resolution maps for performance
    #[arg(long, hide = true, required = false)]
    pub no_downgrade: bool,

    /// Enable graticule overlay (primary coordinate system for mollweide map)
    #[arg(long, required = false)]
    pub graticule: bool,

    /// Primary graticule coordinate system: gal, eq, ecl
    /// (defaults to the map's input coordinate system)
    #[arg(long, hide = true, required = false)]
    pub grat_coord: Option<String>,

    /// Secondary graticule coordinate system to overlay (e.g., show FK5 over Galactic)
    /// Specify one of: gal, eq, ecl. If set with --grat-coord, both systems will be displayed.
    #[arg(long, hide = true, required = false)]
    pub grat_coord_overlay: Option<String>,

    /// Color for secondary graticule overlay (hex #RRGGBB or r,g,b,a format)
    /// Default: yellow (#FFFF00) for good contrast on dark colormaps
    #[arg(long, default_value = "#FFFF00", hide = true, required = false)]
    pub grat_overlay_color: InputColor,

    /// Show coordinate labels on graticule lines (shows lat/lon values)
    #[arg(long, hide = true, required = false)]
    pub grat_labels: bool,

    /// Graticule spacing for parallels \[degrees\]
    #[arg(long, default_value_t = 15.0, hide = true, required = false)]
    pub grat_par: f64,

    /// Graticule spacing for meridians \[degrees\]
    #[arg(long, default_value_t = 15.0, hide = true, required = false)]
    pub grat_mer: f64,

    /// Extend colorbar with arrows at the ends: none, min, max, or both
    #[arg(long, default_value = "none", hide = true, required = false)]
    pub extend: String,

    /// Colorbar tick direction: inward/in (default) or outward/out
    #[arg(long, default_value = "inward", hide = true, required = false)]
    pub tick_direction: String,

    /// Tick label font size in points (default: auto-scaled, 12pt at width 800px)
    #[arg(long, hide = true, required = false)]
    pub tick_font_size: Option<f32>,

    /// Units text font size in points (default: auto-scaled, 16pt at width 800px)
    #[arg(long, hide = true, required = false)]
    pub units_font_size: Option<f32>,

    /// Text label for top-left corner
    #[arg(long, hide = true, required = false)]
    pub llabel: Option<String>,

    /// Text label for top-right corner
    #[arg(long, hide = true, required = false)]
    pub rlabel: Option<String>,

    /// Font size for labels in points (default: auto-scaled, 16pt at width 800px)
    #[arg(long, hide = true, required = false)]
    pub label_font_size: Option<f32>,

    /// Path to mask FITS file (binary mask: 0=masked, 1=valid)
    #[arg(long, hide = true, required = false)]
    pub mask_file: Option<String>,

    /// Mask pixels with values below this threshold
    #[arg(long, allow_negative_numbers = true, hide = true, required = false)]
    pub mask_below: Option<f64>,

    /// Mask pixels with values above this threshold
    #[arg(long, allow_negative_numbers = true, hide = true, required = false)]
    pub mask_above: Option<f64>,

    /// Color for masked regions: transparent, gray, or r,g,b,a format
    #[arg(long, default_value = "transparent", hide = true, required = false)]
    pub maskfill_color: String,

    /// Coordinate system of mask file (gal, eq, ecl) - auto-detects if not specified
    #[arg(long, hide = true, required = false)]
    pub mask_coord: Option<String>,

    /// Fast render mode: skip graticule, colorbar, and labels for faster iteration
    #[arg(long, hide = true, required = false)]
    pub fast_render: bool,

    /// PDF backend: cairo
    #[arg(long, default_value = "cairo", hide = true, required = false)]
    pub pdf_backend: String,
}

/// Colorbar extend option: arrows at minimum, maximum, or both ends
#[derive(Clone, Debug, PartialEq)]
pub enum Extend {
    None,
    Min,
    Max,
    Both,
}

impl FromStr for Extend {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "none" => Ok(Extend::None),
            "min" => Ok(Extend::Min),
            "max" => Ok(Extend::Max),
            "both" => Ok(Extend::Both),
            _ => Err(format!(
                "Invalid extend option '{}'. Expected: none, min, max, or both",
                s
            )),
        }
    }
}

/// Colorbar tick direction: inward (in) or outward (out)
#[derive(Clone, Debug, PartialEq)]
pub enum TickDirection {
    Inward,  // Ticks point toward the colorbar (default)
    Outward, // Ticks point away from the colorbar
}

impl FromStr for TickDirection {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "inward" | "in" => Ok(TickDirection::Inward),
            "outward" | "out" => Ok(TickDirection::Outward),
            _ => Err(format!(
                "Invalid tick direction '{}'. Expected: inward (in) or outward (out)",
                s
            )),
        }
    }
}

/// Color option supporting hex (#RRGGBB), RGBA (r,g,b,a), or special keywords
#[derive(Clone, Debug)]
pub enum InputColor {
    Gray,
    Underflow,
    Overflow,
    Transparent,
    Rgba(u8, u8, u8, u8),
    Hex(String), // Store for later parsing with alpha
}

impl FromStr for InputColor {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "under" => Ok(InputColor::Underflow),
            "over" => Ok(InputColor::Overflow),
            "gray" | "grey" => Ok(InputColor::Gray),
            "transparent" | "trans" => Ok(InputColor::Transparent),
            _ => {
                // Try hex color first
                if s.starts_with('#') || (s.len() == 6 && s.chars().all(|c| c.is_ascii_hexdigit()))
                {
                    return Ok(InputColor::Hex(s.to_string()));
                }

                // Try RGBA format
                let parts: Vec<_> = s.split(',').collect();
                if parts.len() == 4 {
                    let vals: Result<Vec<u8>, _> = parts.iter().map(|x| x.trim().parse()).collect();
                    return match vals {
                        Ok(v) => Ok(InputColor::Rgba(v[0], v[1], v[2], v[3])),
                        Err(_) => Err("RGBA values must be 0–255".into()),
                    };
                }

                Err(format!(
                    "Invalid color format: '{}'. Expected hex (#RRGGBB), RGBA (r,g,b,a), or keyword (gray/transparent)",
                    s
                ))
            }
        }
    }
}

pub fn resolve_input_color(
    input: Option<InputColor>,
    cmap: &Colormap,
    transparent: bool,
) -> Rgba<u8> {
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
        InputColor::Transparent => Rgba([255, 255, 255, 0]),
        InputColor::Rgba(r, g, b, a) => Rgba([r, g, b, a]),
        InputColor::Hex(hex_str) => {
            match parse_hex_color(&hex_str, if transparent { 0 } else { 255 }) {
                Ok(color) => color,
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse hex color '{}': {}, using gray",
                        hex_str, e
                    );
                    Rgba([128, 128, 128, if transparent { 0 } else { 255 }])
                }
            }
        }
    }
}

/// Parse hex color string (e.g., "#FFFF00" or "FFFF00") to RGBA
pub fn parse_hex_color(hex: &str, alpha: u8) -> Result<Rgba<u8>, String> {
    let hex = hex.trim_start_matches('#');

    if hex.len() != 6 {
        return Err(format!("Hex color must be 6 digits, got: {}", hex));
    }

    let r =
        u8::from_str_radix(&hex[0..2], 16).map_err(|_| format!("Invalid hex color: {}", hex))?;
    let g =
        u8::from_str_radix(&hex[2..4], 16).map_err(|_| format!("Invalid hex color: {}", hex))?;
    let b =
        u8::from_str_radix(&hex[4..6], 16).map_err(|_| format!("Invalid hex color: {}", hex))?;

    Ok(Rgba([r, g, b, alpha]))
}

/// Resolve an InputColor to RGBA with specified alpha value
/// This is useful for overlay colors where you want specific transparency
pub fn resolve_color_with_alpha(color: &InputColor, alpha: u8) -> Result<Rgba<u8>, String> {
    match color {
        InputColor::Hex(hex_str) => parse_hex_color(hex_str, alpha),
        InputColor::Rgba(r, g, b, _) => Ok(Rgba([*r, *g, *b, alpha])), // Override alpha
        InputColor::Gray => Ok(Rgba([128, 128, 128, alpha])),
        InputColor::Transparent => Ok(Rgba([255, 255, 255, 0])), // Transparent is always fully transparent
        InputColor::Underflow => Ok(Rgba([100, 100, 100, alpha])), // Fallback colors
        InputColor::Overflow => Ok(Rgba([200, 200, 200, alpha])),
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
    /// Validate that projection-specific arguments are only used with their respective projection
    pub fn validate_projection_args(&self) -> Result<(), String> {
        let projection = self.projection.to_lowercase();

        // Check gnomonic-only args (--lon and --lat are shared, used for rotation in mollweide)
        let gnom_only_provided = self.fov != 300.0
            || self.res != 1.0
            || self.local_graticule
            || self.local_grat_dlat != 1.0
            || self.local_grat_dlon != 1.0;

        if gnom_only_provided && projection != "gnomonic" {
            return Err(
                "Gnomonic-specific arguments (--fov, --res, --local-graticule, \
                 --local-grat-dlat, --local-grat-dlon) can only be used with \
                 --projection gnomonic"
                    .to_string(),
            );
        }

        // Check mollweide-specific args (also applies to hammer)
        let mollweide_args_provided = self.graticule;

        if mollweide_args_provided && projection != "mollweide" && projection != "hammer" {
            return Err(
                "Mollweide/Hammer projection arguments (--graticule, --grat-coord, --grat-par, \
                 --grat-mer) can only be used with --projection mollweide or hammer"
                    .to_string(),
            );
        }

        Ok(())
    }

    pub fn resolve_config(&self) -> Result<PlotConfig, String> {
        // Validate projection-specific arguments first
        self.validate_projection_args()?;

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
        let bad_color_rgba = resolve_input_color(
            self.bad_color.clone().or(Some(InputColor::Gray)),
            colormap,
            self.transparent,
        );
        let bg_color_rgba = resolve_input_color(
            self.bg_color.clone().or(Some(InputColor::Transparent)),
            colormap,
            self.transparent,
        );

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

    /// Determine output filename from --out flag or input FITS filename.
    ///
    /// # Logic
    /// 1. If --out is explicitly provided, use it
    /// 2. If input FITS file is provided, use it with .fits → .pdf extension
    /// 3. If neither, use "map2fig_test.pdf" as fallback
    pub fn get_output_filename(&self) -> String {
        if let Some(ref out) = self.out {
            out.clone()
        } else if let Some(ref fits) = self.fits {
            let path = std::path::Path::new(fits);
            if let Some(stem) = path.file_stem() {
                format!("{}.pdf", stem.to_string_lossy())
            } else {
                "map2fig_test.pdf".to_string()
            }
        } else {
            "map2fig_test.pdf".to_string()
        }
    }

    /// Check if coordinate transformation is needed.
    ///
    /// Returns Some(from_to_string) if input_coord != output_coord
    pub fn describe_coord_transform(&self) -> Option<String> {
        if self.input_coord != self.output_coord {
            Some(format!(
                "Rotating from {} to {} coordinates",
                self.describe_coord(&self.input_coord),
                self.describe_coord(&self.output_coord)
            ))
        } else {
            None
        }
    }

    /// Get human-readable coordinate system name
    fn describe_coord(&self, coord: &str) -> String {
        match coord.to_lowercase().as_str() {
            "gal" | "galactic" => "Galactic".to_string(),
            "eq" | "equatorial" => "Equatorial".to_string(),
            "ecl" | "ecliptic" => "Ecliptic".to_string(),
            _ => coord.to_string(),
        }
    }

    pub fn resolve_view_transform(&self) -> Result<ViewTransform, String> {
        let input = CoordSystem::from_str(&self.input_coord)?;
        let output = CoordSystem::from_str(&self.output_coord)?;

        // Note: view_rotation is ONLY applied explicitly via --rotate-to
        // For gnomonic: --lon/--lat are projection center (no view rotation)
        // For mollweide: use --rotate-to to apply rotation if needed
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
