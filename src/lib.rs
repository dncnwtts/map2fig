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
pub mod cli;
pub mod pipeline;
pub mod constants;
pub mod latex;
pub mod rotation;
pub mod graticule;

// Re-export useful items
pub use plot::{plot_mollweide_png, plot_mollweide_pdf, plot_mollweide_auto, plot_gnomonic_auto, plot_gnomonic_png, plot_gnomonic_pdf};
pub use colormap::{get_colormap, Colormap};
pub use fits::read_healpix_column;
pub use cli::{Args, PlotConfig};
pub use scale::validate_scale_config;
pub use constants::*;

use std::str::FromStr;
use image::{Rgba, RgbaImage};

pub const DEBUG_PROJECTION_OVERLAY: bool = true;

#[derive(Clone, Copy)]
pub enum NegMode {
    Zero,
    Unseen,
}

pub enum PixelValue {
    Color(f64),      // normalized [0,1]
    Underflow,       // < minv but finite
    Overflow,        // > maxv but finite
    Bad,             // NaN, UNSEEN, masked
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

// Optional test helper
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
        plot_mollweide_png(
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
            bad_color,
            meta,
            false,  // latex_rendering
            None,   // units
            &view,
            false,  // show_graticule
            None,   // grat_coord
            None,   // grat_overlay
            bad_color,  // overlay_color
            false,  // _show_labels
            15.0,   // dpar_deg
            15.0,   // dmer_deg
        );
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
    
        plot_mollweide_png(
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
            bad_color,
            meta,
            false,  // latex_rendering
            None,   // units
            &view,
            false,  // show_graticule
            None,   // grat_coord
            None,   // grat_overlay
            bad_color,  // overlay_color
            false,  // _show_labels
            15.0,   // dpar_deg
            15.0,   // dmer_deg
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
