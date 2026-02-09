//! Parameter bundling structs for plot functions.
//!
//! This module organizes the many individual parameters of plotting functions
//! into logical, reusable structs. This improves code readability, maintainability,
//! and makes it easier to extend functionality without modifying function signatures.

use crate::colormap::Colormap;
use crate::healpix::HealpixMeta;
use crate::rotation::CoordSystem;
use crate::scale::Scale;
use crate::NegMode;
use image::Rgba;

/// Core plot data: map, dimensions, and output location.
pub struct PlotData<'a> {
    pub map: &'a [f64],
    pub width: u32,
    pub filename: &'a str,
}

/// Scale and color transformation parameters.
pub struct ScaleParams {
    pub minv: Option<f64>,
    pub maxv: Option<f64>,
    pub gamma: f64,
    pub scale: Scale,
    pub neg_mode: NegMode,
}

/// Color mapping parameters.
pub struct ColorParams<'a> {
    pub cmap: &'a Colormap,
    pub bad_color: Rgba<u8>,
    pub bg_color: Rgba<u8>,
}

/// Display and layout parameters.
pub struct DisplayParams {
    pub show_colorbar: bool,
    pub transparent: bool,
    pub draw_border: bool,
    pub latex_rendering: bool,
    pub units: Option<String>,
    pub extend: crate::cli::Extend,
    pub tick_direction: crate::cli::TickDirection,
    pub tick_font_size: Option<f32>,
    pub units_font_size: Option<f32>,
    pub rlabel: Option<String>,
    pub llabel: Option<String>,
    pub label_font_size: Option<f32>,
}

/// Graticule overlay parameters.
pub struct GraticuleParams {
    pub show_graticule: bool,
    pub grat_coord: Option<CoordSystem>,
    pub grat_overlay: Option<CoordSystem>,
    pub overlay_color: Rgba<u8>,
    pub show_labels: bool,
    pub dpar_deg: f64,
    pub dmer_deg: f64,
}

/// Mollweide projection parameters bundling all related data.
pub struct MollweideParams<'a> {
    pub plot: PlotData<'a>,
    pub scale: ScaleParams,
    pub color: ColorParams<'a>,
    pub display: DisplayParams,
    pub graticule: GraticuleParams,
    pub meta: HealpixMeta,
    pub view: &'a crate::rotation::ViewTransform,
}

/// Hammer projection parameters bundling all related data.
pub struct HammerParams<'a> {
    pub plot: PlotData<'a>,
    pub scale: ScaleParams,
    pub color: ColorParams<'a>,
    pub display: DisplayParams,
    pub graticule: GraticuleParams,
    pub meta: HealpixMeta,
    pub view: &'a crate::rotation::ViewTransform,
}

/// Gnomonic projection parameters bundling all related data.
pub struct GnomonicParams<'a> {
    pub plot: PlotData<'a>,
    pub scale: ScaleParams,
    pub color: ColorParams<'a>,
    pub display: DisplayParams,
    pub graticule: GraticuleParams,
    pub meta: HealpixMeta,
    pub view: &'a crate::rotation::ViewTransform,
    pub lon_deg: f64,
    pub lat_deg: f64,
    pub fov_arcmin: f64,
    pub resolution_arcmin: f64,
    pub roll_deg: f64,
    pub grat_line_width: u32,
}

/// Pixel rendering parameters for mollweide projection.
pub struct RenderMollweideParams<'a> {
    pub map: &'a [f64],
    pub scale: &'a crate::plot::MollweideScale,
    pub cmap: &'a Colormap,
    pub gamma: f64,
    pub scale_type: Scale,
    pub neg_mode: NegMode,
    pub bad_color: Rgba<u8>,
    pub meta: HealpixMeta,
    pub hist_scale: Option<&'a crate::scale::HistogramScale>,
    pub view: &'a crate::rotation::ViewTransform,
}

/// Grid rendering parameters for projection sampling.
pub struct RenderGridParams<'a> {
    pub map: &'a [f64],
    pub proj: &'a dyn crate::projection::Projection,
    pub scale: &'a crate::plot::MollweideScale,
    pub cmap: &'a Colormap,
    pub scale_type: Scale,
    pub neg_mode: NegMode,
    pub gamma: f64,
    pub bad_color: Rgba<u8>,
    pub meta: HealpixMeta,
    pub hist_scale: Option<&'a crate::scale::HistogramScale>,
    pub view: &'a crate::rotation::ViewTransform,
}

/// Colorbar rendering parameters for PDF output.
pub struct ColorbarParams<'a> {
    pub cmap: &'a Colormap,
    pub minv: f64,
    pub maxv: f64,
    pub scale_type: Scale,
    pub gamma: f64,
    pub hist_scale: Option<&'a crate::scale::HistogramScale>,
    pub latex_rendering: bool,
    pub units: Option<&'a str>,
    pub extend: &'a crate::cli::Extend,
    pub units_font_size: Option<f32>,
}

/// Graticule rendering parameters.
pub struct GraticuleRenderParams {
    pub color_r: f64,
    pub color_g: f64,
    pub color_b: f64,
    pub line_width: f64,
}
