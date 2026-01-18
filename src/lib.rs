pub mod plot;
pub mod healpix;
pub mod colormap;
pub mod fits;
pub mod colorbar;
pub mod render;
pub mod scale;
pub mod layout;

// Re-export useful items
pub use plot::{plot_mollweide_png, plot_mollweide_pdf, plot_mollweide_auto};
pub use colormap::{get_colormap, Colormap};
pub use fits::read_healpix_column;

use clap::Parser;
use std::str::FromStr;
use image::{Rgba, RgbaImage};

#[derive(Clone, Copy)]
pub enum NegMode {
    Zero,
    Unseen,
}

#[derive(Clone, Copy, Debug)]
pub enum PixelValue {
    Color(f64),
    Bad,
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
                    return Err("Expected r,g,b,a".into());
                }
                let vals: Result<Vec<u8>, _> = parts.iter().map(|x| x.parse()).collect();
                match vals {
                    Ok(v) => Ok(BadColor::Rgba(v[0], v[1], v[2], v[3])),
                    Err(_) => Err("RGBA values must be 0–255".into())
                }
            }
        }
    }
}

use cairo::Context;
pub struct CairoRasterSink<'a> {
    cr: &'a Context,
}

pub trait PixelSink {
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

pub struct PngSink<'a> {
    pub img: &'a mut RgbaImage,
    pub x0: u32,
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



/// Simple HEALPix Mollweide plotter
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Input FITS file
    #[arg(short, long, default_value = "cosmoglobe_DIRBE_10_I_n00512_DR2_v3.1.fits")]
    pub fits: String,

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
    #[arg(long)]
    pub min: Option<f64>,

    /// Upper color scale limit
    #[arg(long)]
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
    pub bad_color: Option<BadColor>,

    #[arg(long)]
    pub planck_log: bool,
}



/// RGBA argument parser
#[derive(Clone, Debug)]
pub struct RgbaArg {
    pub r: u8,
    pub g: u8,
    pub b: u8,
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

/// Bad color option
#[derive(Clone, Debug)]
pub enum BadColor {
    Auto,
    Gray,
    Rgba(u8, u8, u8, u8),
}


// Optional test helper
pub fn generate_index_map(nside: i64) -> Vec<f64> {
    let npix = 12 * nside * nside;
    (0..npix).map(|i| i as f64).collect()
}


pub fn resolve_bad_color(bad: Option<BadColor>, cmap: &Colormap, transparent: bool) -> Rgba<u8> {
    match bad.unwrap_or(BadColor::Auto) {
        BadColor::Auto => {
            let c = cmap.under();
            Rgba([c[0], c[1], c[2], if transparent { 0 } else { 255 }])
        }
        BadColor::Gray => Rgba([128, 128, 128, if transparent { 0 } else { 255 }]),
        BadColor::Rgba(r,g,b,a) => Rgba([r,g,b,a]),
    }
}



pub fn validate_scale_config(scale: &Scale, min: Option<f64>, max: Option<f64>) {
    match scale {
        Scale::Log => {
            let min = min.expect("log scale requires --min to be specified");
            if min <= 0.0 {
                panic!(
                    "Invalid --min value for log scale: {} (must be > 0)",
                    min
                );
            }
        }
        _ => {}
    }

    if let (Some(min), Some(max)) = (min, max) {
        if min >= max {
            panic!(
                "Invalid scale range: min ({}) must be < max ({})",
                min, max
            );
        }
    }
}



use crate::scale::Scale; // <- Scale comes from plot module



// ===== unit tests =====
#[cfg(test)]
mod tests {
    use super::*;          // import everything from lib.rs
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
    /// Test BadColor parsing
    /// ----------------------------
    #[test]
    fn test_bad_color_parse() {
        assert!(matches!(BadColor::from_str("auto").unwrap(), BadColor::Auto));
        assert!(matches!(BadColor::from_str("gray").unwrap(), BadColor::Gray));
        assert!(matches!(BadColor::from_str("grey").unwrap(), BadColor::Gray));
        assert!(matches!(
            BadColor::from_str("255,128,0,255").unwrap(),
            BadColor::Rgba(255,128,0,255)
        ));
    }



    
    /// ----------------------------
    /// Test scale_value transformations
    /// ----------------------------
    #[test]
    fn test_scale_value_transformations() {
        let min = 1.0;
        let max = 100.0;
    
        // Linear scale
        let t = scale_value(50.0, min, max, Scale::Linear, NegMode::Zero, 1.0);
        match t {
            PixelValue::Color(c) => assert!((c - 0.4949).abs() < 1e-3),
            _ => panic!(),
        }
    
        // Log scale
        let t = scale_value(10.0, min, max, Scale::Log, NegMode::Zero, 1.0);
        match t {
            PixelValue::Color(c) => assert!((c - 0.5).abs() < 1e-3),
            _ => panic!(),
        }
    
        // Asinh scale
        let t = scale_value(50.0, min, max, Scale::Asinh { scale: 10.0 }, NegMode::Zero, 1.0);
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
        let min = 1.0;
        let max = 10.0;
    
        // Linear scale with Zero mode
        let t = scale_value(-5.0, min, max, Scale::Linear, NegMode::Zero, 1.0);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 0.0),
            _ => panic!("Linear + NegMode::Zero should return Color(0.0)"),
        }
    
        // Linear scale with Unseen mode
        let t = scale_value(-5.0, min, max, Scale::Linear, NegMode::Unseen, 1.0);
        assert!(matches!(t, PixelValue::Bad));
    
        // Check overflow still maps to 1.0
        let t = scale_value(20.0, min, max, Scale::Linear, NegMode::Unseen, 1.0);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 1.0),
            _ => panic!(),
        }
    }
    
    
    /// ----------------------------
    /// Test resolve_bad_color returns correct RGBA
    /// ----------------------------
    #[test]
    fn test_resolve_bad_color() {
        let cmap = get_colormap("viridis");
    
        let auto_color = resolve_bad_color(Some(BadColor::Auto), cmap, false);
        assert_eq!(auto_color.0[3], 255);
    
        let gray_color = resolve_bad_color(Some(BadColor::Gray), cmap, false);
        assert_eq!(gray_color, Rgba([128,128,128,255]));
    
        let custom_color = BadColor::Rgba(10,20,30,40);
        let c = resolve_bad_color(Some(custom_color), cmap, false);
        assert_eq!(c, Rgba([10,20,30,40]));
    }
    /// ----------------------------
    /// Test plotting with small map
    /// ----------------------------
    #[test]
    fn test_plot_small_map() {
        let map = generate_index_map(1); // 12 pixels
        let cmap = get_colormap("viridis");
        let bad_color = Rgba([128, 128, 128, 255]);
        let neg_mode = NegMode::Zero;
    
        let scale = Scale::Linear;
    
        // Should not panic
        plot_mollweide(
            &map,
            100,
            "test.png",
            None,
            None,
            cmap,
            true,   // colorbar
            false,  // not transparent
            true,   // border
            1.0,    // gamma
            scale,
            neg_mode,
            bad_color,
        );
    }
    
    
    
    #[test]
    fn test_linear_scale_clamping() {
        let min = 0.0;
        let max = 10.0;
    
        // Linear + NegMode::Zero → clamp to 0.0
        let t = scale_value(-5.0, min, max, Scale::Linear, NegMode::Zero, 1.0);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 0.0),
            _ => panic!("Linear + NegMode::Zero should return Color(0.0)"),
        }
    
        // Linear + NegMode::Unseen → Bad
        let t = scale_value(-5.0, min, max, Scale::Linear, NegMode::Unseen, 1.0);
        assert!(
            matches!(t, PixelValue::Bad),
            "Linear + NegMode::Unseen should return Bad"
        );
    
        // Above max clamps to 1.0 (never Bad)
        let t = scale_value(20.0, min, max, Scale::Linear, NegMode::Unseen, 1.0);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 1.0),
            _ => panic!("Values above max should clamp, not mark Bad"),
        }
    }
    
    
    #[test]
    fn test_log_scale_neg_mode() {
        let min = 1.0;
        let max = 100.0;
    
        // NegMode::Zero → maps to Color(0.0)
        let t = scale_value(-5.0, min, max, Scale::Log, NegMode::Zero, 1.0);
        match t {
            PixelValue::Color(c) => assert_eq!(c, 0.0),
            _ => panic!("Log + NegMode::Zero should return Color(0.0)"),
        }
    
        // NegMode::Unseen → Bad
        let t = scale_value(-5.0, min, max, Scale::Log, NegMode::Unseen, 1.0);
        assert!(matches!(t, PixelValue::Bad));
    }
    
    
    #[test]
    fn test_symlog_symmetry() {
        let min = -100.0;
        let max = 100.0;
        let linthresh = 10.0;
    
        let pos = scale_value(
            20.0,
            min,
            max,
            Scale::Symlog { linthresh },
            NegMode::Unseen,
            1.0,
        );
    
        let neg = scale_value(
            -20.0,
            min,
            max,
            Scale::Symlog { linthresh },
            NegMode::Unseen,
            1.0,
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
        let min = 0.0;
        let max = 100.0;
    
        let t1 = scale_value(
            10.0,
            min,
            max,
            Scale::Asinh { scale: 10.0 },
            NegMode::Unseen,
            1.0,
        );
    
        let t2 = scale_value(
            50.0,
            min,
            max,
            Scale::Asinh { scale: 10.0 },
            NegMode::Unseen,
            1.0,
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
        let map = generate_index_map(1);
        let cmap = get_colormap("plasma");
        let bad_color = Rgba([255, 0, 255, 255]);
    
        let scale = Scale::Symlog {
            linthresh: 10.0,
        };
    
        plot_mollweide(
            &map,
            64,
            "test_extreme.png",
            Some(-100.0),
            Some(100.0),
            cmap,
            false,  // no colorbar
            true,   // transparent
            false,  // no border
            2.2,    // gamma
            scale,
            NegMode::Unseen,
            bad_color,
        );
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
        use crate::plot::Scale;
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
