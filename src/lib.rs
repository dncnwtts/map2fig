//! # HEALPix Plotter
//!
//! A Rust library for rendering HEALPix celestial maps in multiple projections (Mollweide, Hammer, Gnomonic).
//!
//! ## Overview
//!
//! This library reads astronomical data from FITS files containing HEALPix sky maps and generates
//! publication-quality visualizations in PDF and PNG formats. Key features include:
//!
//! - **Multiple projections**: Mollweide, Hammer, and Gnomonic
//! - **80+ colormaps**: matplotlib and custom colormaps
//! - **Data scaling**: linear, logarithmic, symlog, asinh, and histogram equalization
//! - **HEALPix support**: RING and NEST ordering, sparse and dense maps
//! - **Output formats**: High-quality PDF via Cairo and PNG via image crate
//! - **Customization**: full control over colors, scale, gamma, graticules, and more
//!
//! ## Quick Start
//!
//! ```ignore
//! use map2fig::plot::plot_mollweide_auto;
//! use map2fig::params::MollweideParams;
//!
//! let params = MollweideParams {
//!     fits_file: "map.fits".to_string(),
//!     output_file: "map.pdf".to_string(),
//!     ..Default::default()
//! };
//! plot_mollweide_auto(params);
//! ```
//!
//! ## Main Plotting Functions
//!
//! - [plot_mollweide_png] - Render Mollweide projection to PNG
//! - [plot_mollweide_pdf] - Render Mollweide projection to PDF
//! - [plot_mollweide_auto] - Render Mollweide with automatic format selection
//! - [plot_gnomonic_png] - Render Gnomonic projection to PNG
//! - [plot_gnomonic_pdf] - Render Gnomonic projection to PDF
//! - [plot_gnomonic_auto] - Render Gnomonic with automatic format selection
//! - [plot_hammer_png] - Render Hammer projection to PNG
//! - [plot_hammer_pdf] - Render Hammer projection to PDF
//! - [plot_hammer_auto] - Render Hammer with automatic format selection
//!
//! ## Core Types
//!
//! - [scale::Scale] - Data scaling method (linear, log, symlog, asinh, histogram)
//! - [NegMode] - Handling strategy for negative/masked values
//! - [Colormap] - Color palette for mapping data values to colors
//! - [colormap::get_colormap] - Get a colormap by name
//! - [healpix::HealpixMeta] - HEALPix metadata from FITS file
//! - [PixelMask] - Boolean mask for pixel filtering
//!
//! ## Data Processing
//!
//! - [healpix::read_healpix_meta] - Read HEALPix metadata from FITS
//! - [fits::read_healpix_column] - Read HEALPix data from FITS file
//! - [healpix::downgrade_healpix_map] - Resample HEALPix map to different resolution
//! - [scale::validate_scale_config] - Validate scaling parameters

pub mod plot;
pub mod healpix;
pub mod colormap;
pub mod fits;
pub mod colorbar;
pub mod render;
pub mod scale;
pub mod layout;
pub mod projection;
pub mod mollweide;
pub mod gnomonic;
pub mod gnomonic_graticule;
pub mod hammer;
pub mod cli;
pub mod pipeline;
pub mod constants;
pub mod latex;
pub mod latex_render;
pub mod rotation;
pub mod graticule;
pub mod params;
pub mod mask;
pub mod cli_builder;

// Re-export useful items
pub use plot::{plot_mollweide_png, plot_mollweide_pdf, plot_mollweide_auto, plot_gnomonic_auto, plot_gnomonic_png, plot_gnomonic_pdf, plot_hammer_auto, plot_hammer_png, plot_hammer_pdf};
pub use colormap::{get_colormap, Colormap};
pub use fits::read_healpix_column;
pub use cli::{Args, PlotConfig};
pub use scale::validate_scale_config;
pub use constants::*;
pub use mask::PixelMask;

use std::str::FromStr;
use image::{Rgba, RgbaImage};

/// Strategy for handling negative, masked, or invalid pixel values in HEALPix maps.
///
/// This enum determines how pixels with NaN, UNSEEN constants, or masked values are rendered.
///
/// # Variants
///
/// * `Zero` - Render invalid pixels with the colormap's minimum value color
/// * `Unseen` - Render invalid pixels with the "bad" color (usually white or transparent)
#[derive(Clone, Copy)]
pub enum NegMode {
    /// Render as low value (colormap minimum)
    Zero,
    /// Render with bad color (unseen pixels)
    Unseen,
}

/// Pixel value classification during data-to-color mapping.
///
/// Each pixel in the rendered image is classified into one of four categories
/// based on its scaled value relative to the min/max bounds.
///
/// # Variants
///
/// * `Color(f64)` - Valid data value, normalized to [0.0, 1.0] range for colormap sampling
/// * `Underflow` - Valid finite value below the minimum, rendered with colormap minimum color
/// * `Overflow` - Valid finite value above the maximum, rendered with colormap maximum color
/// * `Bad` - NaN, UNSEEN constant, masked, or invalid pixel, rendered with bad color
#[derive(Debug, Clone, Copy)]
pub enum PixelValue {
    /// Valid normalized value [0.0, 1.0]
    Color(f64),
    /// Value below minimum
    Underflow,
    /// Value above maximum
    Overflow,
    /// Invalid/masked/NaN value
    Bad,
}




use cairo::Context;

/// Sink for rendering pixels to a Cairo drawing context.
///
/// This renders pixels directly to a Cairo context, which can be used for PDF output
/// or other Cairo-supported formats. Pixels are drawn as 1x1 rectangles.
pub struct CairoRasterSink<'a> {
    cr: &'a Context,
}

/// Trait for pixel rendering backends.
///
/// Implementations of this trait define how individual pixels are rendered to a target.
/// Different implementations support different output formats (PNG, PDF, etc.).
pub trait PixelSink {
    /// Draw a single pixel at the given coordinates with the specified RGBA color.
    ///
    /// # Arguments
    ///
    /// * `x` - X coordinate (0 = left)
    /// * `y` - Y coordinate (0 = top)
    /// * `rgba` - RGBA color value [0, 255]
    fn draw_pixel(&mut self, x: u32, y: u32, rgba: Rgba<u8>);
}


impl<'a> PixelSink for CairoRasterSink<'a> {
    fn draw_pixel(&mut self, x: u32, y: u32, rgba: Rgba<u8>) {
        self.cr.set_source_rgba(
            rgba[0] as f64 / 255.0,
            rgba[1] as f64 / 255.0,
            rgba[2] as f64 / 255.0,
            rgba[3] as f64 / 255.0,
        );
        self.cr.rectangle(x as f64, y as f64, 1.0, 1.0);
        let _ = self.cr.fill();
    }
}

/// Sink for rendering pixels directly to an in-memory PNG image buffer.
///
/// This sink writes pixels to an `RgbaImage` buffer with an optional offset.
/// Useful for creating PNG files or compositing multiple layers.
pub struct PngSink<'a> {
    /// Mutable reference to the image buffer
    pub img: &'a mut RgbaImage,
    /// X offset in the image where rendering starts
    pub x0: u32,
    /// Y offset in the image where rendering starts
    pub y0: u32,
}

impl<'a> PixelSink for PngSink<'a> {
    fn draw_pixel(&mut self, x: u32, y: u32, color: Rgba<u8>) {
        let ix = self.x0 + x;
        let iy = self.y0 + y;

        if ix < self.img.width() && iy < self.img.height() {
            self.img.put_pixel(ix, iy, color);
        }
    }
}


struct CairoImageSink<'a> {
    cr: &'a Context,
}

impl<'a> PixelSink for CairoImageSink<'a> {
    fn draw_pixel(&mut self, x: u32, y: u32, rgba: Rgba<u8>) {
        self.cr.set_source_rgba(
            rgba[0] as f64 / 255.0,
            rgba[1] as f64 / 255.0,
            rgba[2] as f64 / 255.0,
            rgba[3] as f64 / 255.0,
        );
        self.cr.rectangle(x as f64, y as f64, 1.0, 1.0);
        self.cr.fill().unwrap();
    }
}



/// RGBA color specified as comma-separated integers 0–255.
///
/// # Format
///
/// `r,g,b,a` where each value is in range [0, 255]
///
/// # Example
///
/// ```ignore
/// use std::str::FromStr;
/// use map2fig::RgbaArg;
///
/// let color = RgbaArg::from_str("255,128,0,255").unwrap();
/// assert_eq!(color.r, 255);
/// assert_eq!(color.g, 128);
/// assert_eq!(color.b, 0);
/// assert_eq!(color.a, 255);
/// ```
#[derive(Clone, Debug)]
pub struct RgbaArg {
    /// Red channel [0, 255]
    pub r: u8,
    /// Green channel [0, 255]
    pub g: u8,
    /// Blue channel [0, 255]
    pub b: u8,
    /// Alpha channel [0, 255]
    pub a: u8,
}

impl FromStr for RgbaArg {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = s.split(',').collect();
        if parts.len() != 4 {
            return Err("Expected r,g,b,a".into());
        }
        let nums: Result<Vec<u8>, _> = parts.iter().map(|x| x.parse::<u8>()).collect();
        match nums {
            Ok(v) => Ok(Self { r: v[0], g: v[1], b: v[2], a: v[3] }),
            Err(_) => Err("RGBA values must be 0–255".into()),
        }
    }
}

/// Generate a simple index map for testing.
///
/// Creates a map with pixel values equal to their indices: 0, 1, 2, ..., (12*nside²-1).
/// Useful for diagnostic tests and understanding HEALPix pixel layout.
///
/// # Arguments
///
/// * `nside` - HEALPix resolution parameter (must be power of 2: 1, 2, 4, 8, 16, ...)
///
/// # Returns
///
/// Vector with 12*nside² elements where element i contains value i as f64.
///
/// # Example
///
/// ```
/// use map2fig::generate_index_map;
///
/// let map = generate_index_map(2);
/// assert_eq!(map.len(), 48);  // 12 * 2² = 48
/// assert_eq!(map[0], 0.0);
/// assert_eq!(map[47], 47.0);
/// ```
pub fn generate_index_map(nside: i64) -> Vec<f64> {
    let npix = 12 * nside * nside;
    (0..npix).map(|i| i as f64).collect()
}






use crate::scale::Scale; // <- Scale comes from plot module



// ===== unit tests =====
#[cfg(test)]
mod tests {
    use super::*;          // import everything from lib.rs
    use crate::cli::{InputColor, resolve_input_color}; // moved from lib.rs
    use image::Rgba;       // external deps still need explicit import
                           //
    #[test]
    fn test_generate_index_map() {
        let map = generate_index_map(2);
        assert_eq!(map.len(), 48);
        assert_eq!(map[0], 0.0);
        assert_eq!(map[47], 47.0);
    }


    /// ----------------------------
    /// Test RGBAArg parsing
    /// ----------------------------
    #[test]
    fn test_rgbaarg_from_str() {
        let rgba = RgbaArg::from_str("10,20,30,40").unwrap();
        assert_eq!((rgba.r, rgba.g, rgba.b, rgba.a), (10,20,30,40));

        assert!(RgbaArg::from_str("10,20,30").is_err());
        assert!(RgbaArg::from_str("a,b,c,d").is_err());
    }

    /// ----------------------------
    /// Test InputColor parsing
    /// ----------------------------
    #[test]
    fn test_bad_color_parse() {
        assert!(matches!(InputColor::from_str("gray").unwrap(), InputColor::Gray));
        assert!(matches!(InputColor::from_str("grey").unwrap(), InputColor::Gray));
        assert!(matches!(
            InputColor::from_str("255,128,0,255").unwrap(),
            InputColor::Rgba(255,128,0,255)
        ));
    }



    
    /// ----------------------------
    /// Test scale_value transformations
    /// ----------------------------
    #[test]
    fn test_scale_value_transformations() {
        use crate::scale::scale_value;

        let min = 1.0;
        let max = 100.0;
    
        // Linear scale
        let t = scale_value(50.0, min, max, Scale::Linear, NegMode::Zero, None);
        match t {
            PixelValue::Color(c) => assert!((c - 0.4949).abs() < 1e-3),
            _ => panic!(),
        }
    
        // Log scale
        let t = scale_value(10.0, min, max, Scale::Log, NegMode::Zero, None);
        match t {
            PixelValue::Color(c) => assert!((c - 0.5).abs() < 1e-3),
            _ => panic!(),
        }
    
        // Asinh scale
        let t = scale_value(50.0, min, max, Scale::Asinh { scale: 10.0 }, NegMode::Zero, None);
        match t {
            PixelValue::Color(c) => assert!(c > 0.0 && c < 1.0),
            _ => panic!(),
        }
    }
    
    /// ----------------------------
    /// Test neg_mode handling across all scales
    /// ----------------------------
    #[test]
    fn test_neg_mode_behavior() {
        use crate::scale::scale_value;
        let min = 1.0;
        let max = 10.0;
    
        // Linear scale with Zero mode
        let t = scale_value(-5.0, min, max, Scale::Linear, NegMode::Zero, None);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 0.0),
            _ => panic!("Linear + NegMode::Zero should return Color(0.0)"),
        }
    
        // Check overflow still maps to 1.0
        let t = scale_value(20.0, min, max, Scale::Linear, NegMode::Unseen, None);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 1.0),
            _ => panic!(),
        }
    }
    
    
    /// ----------------------------
    /// Test resolve_input_color returns correct RGBA
    /// ----------------------------
    #[test]
    fn test_resolve_input_color() {
        let cmap = get_colormap("viridis");
    
    
        let gray_color = resolve_input_color(Some(InputColor::Gray), cmap, false);
        assert_eq!(gray_color, Rgba([128,128,128,255]));
    
        let custom_color = InputColor::Rgba(10,20,30,40);
        let c = resolve_input_color(Some(custom_color), cmap, false);
        assert_eq!(c, Rgba([10,20,30,40]));
    }
    /// ----------------------------
    /// Test plotting with small map
    /// ----------------------------
    #[test]
    fn test_plot_small_map() {
        use crate::healpix::{HealpixMeta, HealpixOrdering};
        use crate::rotation::{CoordSystem,ViewTransform};
        use crate::params::{PlotData, ScaleParams, ColorParams, DisplayParams, GraticuleParams, MollweideParams};
        
        let map = generate_index_map(1); // 12 pixels
        let cmap = get_colormap("viridis");
        let bad_color = Rgba([128, 128, 128, 255]);
        let neg_mode = NegMode::Zero;

        let meta = HealpixMeta { ordering: HealpixOrdering::Ring, nside: 1, coord: CoordSystem::G};
    
        let scale = Scale::Linear;
        let input = CoordSystem::G;
        let output = CoordSystem::G;
        let rot = None;
        let view = ViewTransform::new(input,output,rot);
    
        // Should not panic
        let params = MollweideParams {
            plot: PlotData { map: &map, width: 100, filename: "test.png" },
            scale: ScaleParams { minv: None, maxv: None, gamma: 1.0, scale, neg_mode },
            color: ColorParams { cmap, bad_color, bg_color: bad_color },
            display: DisplayParams { 
                show_colorbar: true, 
                transparent: false, 
                draw_border: true, 
                latex_rendering: false, 
                units: None,
                extend: crate::cli::Extend::None,
                tick_direction: crate::cli::TickDirection::Inward,
                tick_font_size: None,
                units_font_size: None,
                llabel: None,
                rlabel: None,
                label_font_size: None,
                mask: None,
                title: None,
                show_title: true,
                scale_text: true,
            },
            graticule: GraticuleParams { 
                show_graticule: false, 
                grat_coord: None, 
                grat_overlay: None, 
                overlay_color: bad_color, 
                show_labels: false, 
                dpar_deg: 15.0, 
                dmer_deg: 15.0 
            },
            meta,
            view: &view,
        };
        
        plot_mollweide_png(params);
    }
    
    
    
    #[test]
    fn test_linear_scale_clamping() {
        use crate::scale::scale_value;
        let min = 0.0;
        let max = 10.0;
    
        // Linear + NegMode::Zero → clamp to 0.0
        let t = scale_value(-5.0, min, max, Scale::Linear, NegMode::Zero, None);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 0.0),
            _ => panic!("Linear + NegMode::Zero should return Color(0.0)"),
        }
    
        // Above max clamps to 1.0 (never Bad)
        let t = scale_value(20.0, min, max, Scale::Linear, NegMode::Unseen, None);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 1.0),
            _ => panic!("Values above max should clamp, not mark Bad"),
        }
    }
    
    
    #[test]
    fn test_log_scale_neg_mode() {
        use crate::scale::scale_value;
        let min = 1.0;
        let max = 100.0;
    
        // NegMode::Zero → maps to Color(0.0)
        let t = scale_value(-5.0, min, max, Scale::Log, NegMode::Zero, None);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 0.0),
            _ => panic!("Log + NegMode::Zero should return Color(0.0)"),
        }
    
        // NegMode::Unseen → Bad
        let t = scale_value(-5.0, min, max, Scale::Log, NegMode::Unseen, None);
        assert!(matches!(t, PixelValue::Bad));
    }
    
    
    #[test]
    fn test_symlog_symmetry() {
        use crate::scale::scale_value;
        let min = -100.0;
        let max = 100.0;
        let linthresh = 10.0;
    
        let pos = scale_value(
            20.0,
            min,
            max,
            Scale::Symlog { linthresh },
            NegMode::Unseen,
            None,
        );
    
        let neg = scale_value(
            -20.0,
            min,
            max,
            Scale::Symlog { linthresh },
            NegMode::Unseen,
            None,
        );
    
        match (pos, neg) {
            (PixelValue::Color(p), PixelValue::Color(n)) => {
                assert!((p + n - 1.0).abs() < 1e-6);
            }
            _ => panic!("Symlog should produce Color for symmetric inputs"),
        }
    }
    
    #[test]
    fn test_asinh_monotonic() {
        use crate::scale::scale_value;
        let min = 0.0;
        let max = 100.0;
    
        let t1 = scale_value(
            10.0,
            min,
            max,
            Scale::Asinh { scale: 10.0 },
            NegMode::Unseen,
            None,
        );
    
        let t2 = scale_value(
            50.0,
            min,
            max,
            Scale::Asinh { scale: 10.0 },
            NegMode::Unseen,
            None,
        );
    
        match (t1, t2) {
            (PixelValue::Color(a), PixelValue::Color(b)) => {
                assert!(a < b);
                assert!(a >= 0.0 && b <= 1.0);
            }
            _ => panic!("Asinh scale should return Color"),
        }
    }
    
    
    #[test]
    fn test_colormap_sampling_bounds() {
        let cmap = get_colormap("viridis");
    
        let c0 = cmap.sample(0.0);
        let c1 = cmap.sample(1.0);
        let c_mid = cmap.sample(0.5);
    
        assert_eq!(c0, cmap.under());
        assert_eq!(c1, cmap.over());
        assert!(c_mid != c0 && c_mid != c1);
    }
    
    #[test]
    fn test_plot_extreme_options() {
        use crate::healpix::{HealpixMeta, HealpixOrdering};
        use crate::rotation::{CoordSystem, ViewTransform};
        use crate::params::{PlotData, ScaleParams, ColorParams, DisplayParams, GraticuleParams, MollweideParams};
        
        let map = generate_index_map(1);
        let cmap = get_colormap("plasma");
        let bad_color = Rgba([255, 0, 255, 255]);
    
        let scale = Scale::Symlog {
            linthresh: 10.0,
        };

        let meta = HealpixMeta { ordering: HealpixOrdering::Ring, nside: 1, coord: CoordSystem::G};
        let input = CoordSystem::G;
        let output = CoordSystem::G;
        let rot = None;
        let view = ViewTransform::new(input,output,rot);
    
        let params = MollweideParams {
            plot: PlotData { map: &map, width: 64, filename: "test_extreme.png" },
            scale: ScaleParams { minv: Some(-100.0), maxv: Some(100.0), gamma: 2.2, scale, neg_mode: NegMode::Unseen },
            color: ColorParams { cmap, bad_color, bg_color: bad_color },
            display: DisplayParams { 
                show_colorbar: false, 
                transparent: true, 
                draw_border: false, 
                latex_rendering: false, 
                units: None,
                extend: crate::cli::Extend::None,
                tick_direction: crate::cli::TickDirection::Inward,
                tick_font_size: None,
                units_font_size: None,
                llabel: None,
                rlabel: None,
                label_font_size: None,
                mask: None,
                title: None,
                show_title: true,
                scale_text: true,
            },
            graticule: GraticuleParams { 
                show_graticule: false, 
                grat_coord: None, 
                grat_overlay: None, 
                overlay_color: bad_color, 
                show_labels: false, 
                dpar_deg: 15.0, 
                dmer_deg: 15.0 
            },
            meta,
            view: &view,
        };
        
        plot_mollweide_png(params);
    }
    
    
    
    #[test]
    #[should_panic(expected = "log scale")]
    fn test_log_scale_panics_on_nonpositive_min() {
        validate_scale_config(&Scale::Log, Some(-1.0), Some(10.0));
    }
    
    
    #[test]
    #[should_panic]
    fn test_log_scale_panics_on_missing_min() {
        validate_scale_config(&Scale::Log, None, Some(10.0));
    }

    /// ----------------------------
    /// Test that log scale rejects non-positive minimums
    /// ----------------------------
    #[test]
    fn test_log_scale_rejects_nonpositive_min() {
        use scale::Scale;
        use crate::validate_scale_config;
    
        let max = Some(100.0);
    
        // min == 0 is invalid for log scale
        let min_zero = Some(0.0);
        let panicked = std::panic::catch_unwind(|| {
            validate_scale_config(&Scale::Log, min_zero, max);
        });
        assert!(panicked.is_err(), "Log scale with min=0 should panic");
    
        // min < 0 is invalid for log scale
        let min_neg = Some(-1.0);
        let panicked = std::panic::catch_unwind(|| {
            validate_scale_config(&Scale::Log, min_neg, max);
        });
        assert!(panicked.is_err(), "Log scale with negative min should panic");
    
        // min > 0 should NOT panic
        let min_pos = Some(1.0);
        let panicked = std::panic::catch_unwind(|| {
            validate_scale_config(&Scale::Log, min_pos, max);
        });
        assert!(panicked.is_ok(), "Log scale with positive min should not panic");
    }


}
