//! Data scaling and value normalization for visualization.
//!
//! This module provides methods for mapping raw data values to normalized [0.0, 1.0] range
//! for colormap sampling. It supports multiple scaling strategies:
//!
//! - **Linear**: Direct linear mapping `(val - min) / (max - min)`
//! - **Logarithmic**: `log(val / min) / log(max / min)` for positive data with wide dynamic range
//! - **SymLog**: Symmetric logarithm for data containing both positive and negative values
//! - **Asinh**: Inverse hyperbolic sine for data with wide dynamic range
//! - **Histogram Equalization**: Perceptual stretching using histogram distribution
//!
//! # Handling Invalid Data
//!
//! The [NegMode] enum controls treatment of masked/invalid pixels:
//! - [NegMode::Zero]: Render as minimum value
//! - [NegMode::Unseen]: Render as bad color (typically white)
//!
//! # Examples
//!
//! ```ignore
//! use map2fig::scale::{scale_value, Scale};
//! use map2fig::NegMode;
//!
//! let scaled = scale_value(5.0, 0.0, 10.0, Scale::Linear, NegMode::Zero, None, None);
//! // Result: PixelValue::Color(0.5)
//! ```

use crate::NegMode;
use crate::PixelValue;
use crate::colorbar::ColorbarTicks;
use crate::healpix::is_seen;
use std::cmp::Ordering;

/// Direct float comparison using std::cmp for faster sorting.
/// This avoids the NaN check overhead since data is pre-validated.
#[inline]
pub fn unsafe_float_cmp(a: &f64, b: &f64) -> Ordering {
    // Direct comparison - Rust will optimize this better than partial_cmp
    if a < b {
        Ordering::Less
    } else if a > b {
        Ordering::Greater
    } else {
        Ordering::Equal
    }
}

/// Validate scaling configuration parameters.
///
/// Ensures that scale parameters are compatible (e.g., log scale requires positive min value).
///
/// # Arguments
///
/// * `scale` - Data scaling method
/// * `min` - Optional minimum value for scaling range
/// * `max` - Optional maximum value for scaling range
///
/// # Panics
///
/// Panics with a helpful error message if configuration is invalid:
/// - Log scale without explicit --min specified
/// - Log scale with min ≤ 0 (logarithm undefined for non-positive values)
pub fn validate_scale_config(scale: &Scale, min: Option<f64>, max: Option<f64>) {
    if scale == &Scale::Log {
        let min = min.expect("log scale requires --min to be specified");
        if min <= 0.0 {
            panic!("Invalid --min value for log scale: {} (must be > 0)", min);
        }
    }

    if let (Some(min), Some(max)) = (min, max)
        && min >= max
    {
        panic!("Invalid scale range: min ({}) must be < max ({})", min, max);
    }
}

/// Pre-computed scale transformation constants to avoid per-pixel recomputation.
///
/// For log and asinh scales, we compute expensive constants once at the start
/// and reuse them for every pixel, avoiding redundant log() and asinh() calls.
#[derive(Clone, Debug)]
pub struct ScaleCache {
    /// Scale type this cache is for
    pub scale_type: Scale,
    /// Cached log(min) for Log scale
    pub log_min: f64,
    /// Cached log(max) for Log scale  
    pub log_range: f64,  // log_max - log_min
    /// Cached asinh(min) for Asinh scale
    pub asinh_min: f64,
    /// Cached range for Asinh
    pub asinh_range: f64,  // asinh_max - asinh_min
}

impl ScaleCache {
    /// Create a new scale cache, pre-computing expensive transformations.
    pub fn new(min: f64, max: f64, scale: Scale) -> Self {
        match scale {
            Scale::Log => {
                let log_min = min.ln();
                let log_max = max.ln();
                Self {
                    scale_type: scale,
                    log_min,
                    log_range: log_max - log_min,
                    asinh_min: 0.0,
                    asinh_range: 0.0,
                }
            }
            Scale::Asinh { scale: s } => {
                let asinh_min = (min / s).asinh();
                let asinh_max = (max / s).asinh();
                Self {
                    scale_type: scale,
                    log_min: 0.0,
                    log_range: 0.0,
                    asinh_min,
                    asinh_range: asinh_max - asinh_min,
                }
            }
            _ => Self {
                scale_type: scale,
                log_min: 0.0,
                log_range: 0.0,
                asinh_min: 0.0,
                asinh_range: 0.0,
            },
        }
    }
}

#[allow(dead_code)]
fn scale_t_to_value(t: f64, min: f64, max: f64, scale: Scale) -> f64 {
    match scale {
        Scale::Linear => min + t * (max - min),
        Scale::Log => {
            let lmin = min.ln();
            let lmax = max.ln();
            (lmin + t * (lmax - lmin)).exp()
        }
        Scale::Asinh { scale } => {
            let amin = (min / scale).asinh();
            let amax = (max / scale).asinh();
            scale * (amin + t * (amax - amin)).sinh()
        }
        _ => unimplemented!(),
    }
}

#[allow(dead_code)]
fn value_to_t(value: f64, min: f64, max: f64, scale: Scale) -> Option<f64> {
    match scale {
        Scale::Linear => Some((value - min) / (max - min)),

        Scale::Log => {
            if value <= 0.0 || min <= 0.0 {
                None
            } else {
                Some((value.ln() - min.ln()) / (max.ln() - min.ln()))
            }
        }

        Scale::Asinh { scale: s } => Some((value / s).asinh() / (max / s).asinh()),

        Scale::Symlog { linthresh } => {
            let f = |x: f64| {
                if x.abs() < linthresh {
                    x / linthresh
                } else {
                    x.signum() * (x.abs() / linthresh).ln()
                }
            };
            Some((f(value) - f(min)) / (f(max) - f(min)))
        }

        Scale::PlanckLog { linthresh } => {
            // use same mapping you already trust elsewhere
            let f = |x: f64| {
                if x.abs() < linthresh {
                    x / linthresh
                } else {
                    x.signum() * (1.0 + (x.abs() / linthresh).ln())
                }
            };
            Some((f(value) - f(min)) / (f(max) - f(min)))
        }

        Scale::Histogram => todo!(),
    }
}

pub struct HistogramScale {
    pub values: Vec<f64>, // sorted unique values or bin centers
    pub cdf: Vec<f64>,    // monotonically increasing [0,1]

    pub minv: f64,
    pub maxv: f64,
}

impl HistogramScale {
    pub fn lookup_cdf(&self, value: f64) -> Option<f64> {
        if self.values.is_empty() {
            return None;
        }

        match self
            .values
            .binary_search_by(|v| v.partial_cmp(&value).unwrap())
        {
            Ok(i) => Some(self.cdf[i]),
            Err(i) => {
                if i == 0 {
                    Some(0.0)
                } else if i >= self.cdf.len() {
                    Some(1.0)
                } else {
                    Some(self.cdf[i])
                }
            }
        }
    }
    pub fn inverse_cdf(&self, q: f64) -> Option<f64> {
        if self.cdf.is_empty() {
            return None;
        }

        match self.cdf.binary_search_by(|p| p.partial_cmp(&q).unwrap()) {
            Ok(i) => Some(self.values[i]),
            Err(i) => {
                if i == 0 {
                    Some(self.values[0])
                } else if i >= self.values.len() {
                    Some(*self.values.last().unwrap())
                } else {
                    Some(self.values[i])
                }
            }
        }
    }

    fn value_at_quantile(&self, t: f64) -> Option<f64> {
        if self.values.is_empty() {
            return Some(0.0);
        }

        if t <= 0.0 {
            return Some(self.minv);
        }
        if t >= 1.0 {
            return Some(self.maxv);
        }

        match self.cdf.binary_search_by(|p| p.partial_cmp(&t).unwrap()) {
            Ok(i) => Some(self.values[i]),
            Err(i) => {
                if i == 0 {
                    Some(self.values[0])
                } else if i >= self.values.len() {
                    Some(*self.values.last().unwrap())
                } else {
                    // Linear interpolation between neighbors
                    let t0 = self.cdf[i - 1];
                    let t1 = self.cdf[i];
                    let v0 = self.values[i - 1];
                    let v1 = self.values[i];

                    let w = (t - t0) / (t1 - t0);
                    Some(v0 + w * (v1 - v0))
                }
            }
        }
    }
    pub fn distortion_profile(&self, n: usize) -> Vec<f64> {
        assert!(n >= 2, "distortion_profile requires n >= 2");

        // 1. Sample inverse CDF
        let mut v = Vec::with_capacity(n);
        for i in 0..n {
            let t = i as f64 / (n - 1) as f64;
            v.push(self.value_at_quantile(t).unwrap_or(0.0));
        }

        // 2. Finite-difference dv/dt
        let dt = 1.0 / (n - 1) as f64;
        let mut dvdt = vec![0.0; n];

        for i in 1..n - 1 {
            dvdt[i] = (v[i + 1] - v[i - 1]).abs() / (2.0 * dt);
        }

        dvdt[0] = (v[1] - v[0]).abs() / dt;
        dvdt[n - 1] = (v[n - 1] - v[n - 2]).abs() / dt;

        // 3. Compress dynamic range
        for d in dvdt.iter_mut() {
            *d = (1.0 + *d).ln();
        }

        // 4. Normalize to [0,1]
        let max_d = dvdt.iter().cloned().fold(0.0_f64, f64::max);

        if max_d > 0.0 {
            for d in dvdt.iter_mut() {
                *d /= max_d;
            }
        }

        dvdt
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HistogramRange {
    Percentile { low: f64, high: f64 },
    Explicit { min: f64, max: f64 },
    Full,
}

pub fn build_histogram_scale(map: &[f64], range: HistogramRange, bins: usize) -> HistogramScale {
    // 1. Filter valid values only
    let mut vals: Vec<f64> = map.iter().copied().filter(|v| is_seen(*v)).collect();
    if vals.is_empty() {
        return HistogramScale {
            values: vec![],
            cdf: vec![],
            minv: 0.0,
            maxv: 1.0,
        };
    }

    vals.sort_unstable_by(unsafe_float_cmp);
    let n = vals.len();

    // 2. Compute minv / maxv based on HistogramRange
    let (minv, maxv) = match range {
        HistogramRange::Explicit { min, max } => (min, max),
        HistogramRange::Full => (vals[0], vals[n - 1]),
        HistogramRange::Percentile { low, high } => {
            let ilo = ((n - 1) as f64 * low).round() as usize;
            let ihi = ((n - 1) as f64 * high).round() as usize;
            (vals[ilo], vals[ihi])
        }
    };

    // 3. Filter to minv/maxv range for histogram LUT
    let vals: Vec<f64> = vals
        .into_iter()
        .filter(|v| *v >= minv && *v <= maxv)
        .collect();
    if vals.is_empty() {
        return HistogramScale {
            values: vec![],
            cdf: vec![],
            minv,
            maxv,
        };
    }

    // 4. Downsample to bins
    let n = vals.len();
    let step = (n as f64 / bins as f64).ceil() as usize;
    let mut values = Vec::new();
    let mut cdf = Vec::new();

    for (i, chunk) in vals.chunks(step).enumerate() {
        let v = chunk[chunk.len() / 2];
        let t = (i * step) as f64 / (n - 1) as f64;
        values.push(v);
        cdf.push(t.min(1.0));
    }

    HistogramScale {
        values,
        cdf,
        minv,
        maxv,
    }
}

const TARGET_MAJOR_TICKS: usize = 7;

fn uniform_quantiles(n: usize) -> Vec<f64> {
    (0..n).map(|i| i as f64 / (n - 1) as f64).collect()
}

#[derive(Clone, Copy, PartialEq)]
#[derive(Debug)]
pub enum Scale {
    Linear,
    Log,
    Asinh { scale: f64 },
    Symlog { linthresh: f64 },
    PlanckLog { linthresh: f64 },
    Histogram,
}

pub fn generate_colorbar_ticks(
    min: f64,
    max: f64,
    scale: &Scale,
    hist: Option<&HistogramScale>,
) -> ColorbarTicks {
    if scale == &Scale::Histogram {
        let ticks = histogram_ticks(hist.expect("Histogram ticks require histogram scale"));
        return ticks;
    }

    let mut ticks = match scale {
        Scale::Linear => linear_ticks(min, max),
        Scale::Log => log_ticks(min, max),
        Scale::Symlog { linthresh } => symlog_ticks(min, max, *linthresh),
        Scale::Asinh { scale } => asinh_ticks(min, max, *scale),
        Scale::PlanckLog { linthresh } => symlog_ticks(min, max, *linthresh),
        Scale::Histogram => unreachable!(),
    };

    ticks.major_positions = ticks
        .major_values
        .iter()
        .filter_map(|&v| scale_position(v, min, max, scale))
        .collect();

    ticks.minor_positions = ticks
        .minor_values
        .iter()
        .filter_map(|&v| scale_position(v, min, max, scale))
        .collect();

    ticks
}

fn histogram_major_ticks(hist: &HistogramScale) -> (Vec<f64>, Vec<f64>) {
    let mut values = Vec::new();
    let mut positions = Vec::new();

    for q in uniform_quantiles(TARGET_MAJOR_TICKS) {
        if let Some(v) = hist.value_at_quantile(q) {
            values.push(v);
            positions.push(q);
        }
    }

    (values, positions)
}

fn histogram_minor_ticks(hist: &HistogramScale, _major_q: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let mut values = Vec::new();
    let mut positions = Vec::new();

    for q in uniform_quantiles(50) {
        // Always allow near edges
        if !(0.05..=0.95).contains(&q) {
            if let Some(v) = hist.value_at_quantile(q) {
                values.push(v);
                positions.push(q);
            }
            continue;
        }

        if let Some(v) = hist.value_at_quantile(q) {
            values.push(v);
            positions.push(q);
        }
    }

    (values, positions)
}

fn histogram_ticks(hist: &HistogramScale) -> ColorbarTicks {
    let (major_values, major_positions) = histogram_major_ticks(hist);
    let (minor_values, minor_positions) = histogram_minor_ticks(hist, &major_positions);

    ColorbarTicks {
        major_values,
        major_positions,
        minor_values,
        minor_positions,
    }
}

fn scale_position(value: f64, min: f64, max: f64, scale: &Scale) -> Option<f64> {
    match scale {
        Scale::Linear => Some(((value - min) / (max - min)).clamp(0.0, 1.0)),

        Scale::Log => {
            if value <= 0.0 || min <= 0.0 {
                None
            } else {
                Some(((value.ln() - min.ln()) / (max.ln() - min.ln())).clamp(0.0, 1.0))
            }
        }

        Scale::Asinh { scale } => {
            let v = (value / scale).asinh();
            let vmin = (min / scale).asinh();
            let vmax = (max / scale).asinh();
            Some(((v - vmin) / (vmax - vmin)).clamp(0.0, 1.0))
        }

        Scale::Symlog { linthresh } => {
            let v = value;
            let sign = v.signum();
            let abs = v.abs();

            let max_abs = max.abs().max(min.abs());
            if max_abs <= *linthresh {
                return Some(0.5);
            }

            let log_max = (max_abs / linthresh).ln();
            let linear_width = *linthresh;
            let total = linear_width + log_max;

            let mapped = if abs <= *linthresh {
                // Linear core
                0.5 + 0.5 * (v / total)
            } else {
                // Log wings
                let log_part = (abs / linthresh).ln();
                0.5 + 0.5 * sign * (linear_width + log_part) / total
            };

            Some(mapped.clamp(0.0, 1.0))
        }

        Scale::PlanckLog { linthresh } => {
            // identical behavior for ticks
            scale_position(
                value,
                min,
                max,
                &Scale::Symlog {
                    linthresh: *linthresh,
                },
            )
        }

        Scale::Histogram => todo!(), // intentionally unsupported here
    }
}

fn linear_ticks(min: f64, max: f64) -> ColorbarTicks {
    // Handle the case where all values are the same
    if min >= max {
        return ColorbarTicks {
            major_values: vec![min],
            major_positions: vec![0.5],
            minor_values: vec![],
            minor_positions: vec![],
        };
    }

    let span = max - min;
    let raw_step = span / 5.0;

    let pow10 = 10f64.powf(raw_step.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .iter()
        .map(|m| m * pow10)
        .find(|s| span / s <= 7.0)
        .unwrap();

    let start = (min / step).floor() * step;

    let mut major_values = Vec::new();
    let mut minor_values = Vec::new();

    let mut v = start;
    while v <= max + 1e-12 {
        if v >= min {
            major_values.push(v);
        }

        let minor_step = step / 5.0;
        for i in 1..5 {
            let mv = v + i as f64 * minor_step;
            if mv > min && mv < max {
                minor_values.push(mv);
            }
        }

        v += step;
    }

    ColorbarTicks {
        major_positions: vec![],
        minor_positions: vec![],
        major_values,
        minor_values,
    }
}

fn log_ticks(min: f64, max: f64) -> ColorbarTicks {
    let dmin = min.log10().floor() as i32;
    let dmax = max.log10().ceil() as i32;

    let mut major_values = Vec::new();
    let mut minor_values = Vec::new();

    for d in dmin..=dmax {
        let base = 10f64.powi(d);

        if base >= min && base <= max {
            major_values.push(base);
        }

        for m in 2..10 {
            let v = base * m as f64;
            if v >= min && v <= max {
                minor_values.push(v);
            }
        }
    }

    ColorbarTicks {
        major_positions: vec![],
        minor_positions: vec![],
        major_values,
        minor_values,
    }
}

fn asinh_ticks(min: f64, max: f64, scale: f64) -> ColorbarTicks {
    symlog_ticks(min, max, scale)
}

fn symlog_ticks(min: f64, max: f64, linthresh: f64) -> ColorbarTicks {
    let mut major_values = vec![0.0, linthresh, -linthresh];
    let mut minor_values = Vec::new();

    // linear core
    let n = 4;
    let step = linthresh / n as f64;

    for i in (-n + 1)..=(n - 1) {
        let v = i as f64 * step;
        if v != 0.0 {
            minor_values.push(v);
        }
    }

    // log wings
    let log_max = max.abs().log10().ceil() as i32;

    for d in 1..=log_max {
        let base = linthresh * 10f64.powi(d);

        for &sign in &[-1.0, 1.0] {
            let v = sign * base;
            if v >= min && v <= max {
                major_values.push(v);
            }

            for m in 2..10 {
                let mv = sign * base * m as f64;
                if mv.abs() > linthresh && mv >= min && mv <= max {
                    minor_values.push(mv);
                }
            }
        }
    }

    major_values.sort_unstable_by(unsafe_float_cmp);
    minor_values.sort_unstable_by(unsafe_float_cmp);

    ColorbarTicks {
        major_positions: vec![],
        minor_positions: vec![],
        major_values,
        minor_values,
    }
}

#[inline]
pub fn scale_value(
    value: f64,
    mut min: f64,
    mut max: f64,
    scale: Scale,
    neg_mode: NegMode,
    hist: Option<&HistogramScale>,
    cache: Option<&ScaleCache>,
) -> PixelValue {
    if min > max {
        if std::env::var("FUZZ_SILENT").is_err() {
            eprintln!(
                "WARNING: scale_value called with min > max ({} > {}), swapping automatically",
                min, max
            );
        }
        std::mem::swap(&mut min, &mut max);
    }

    // Handle the case where all valid values are the same
    if min == max {
        return if is_seen(value) {
            PixelValue::Color(0.5) // Map to middle of color scale
        } else {
            PixelValue::Bad
        };
    }

    // Unseen / NaN handling
    if !is_seen(value) {
        return PixelValue::Bad;
    }

    // Fast path for linear scale (most common case)
    if matches!(scale, Scale::Linear) {
        let t = if value <= min {
            0.0
        } else if value >= max {
            1.0
        } else {
            (value - min) / (max - min)
        };
        return PixelValue::Color(t);
    }

    let t = match scale {
        Scale::Linear => unreachable!(),

        Scale::Log => {
            if value <= 0.0 {
                // Negative values: apply neg_mode setting
                return match neg_mode {
                    NegMode::Zero => PixelValue::Color(0.0),
                    NegMode::Unseen => PixelValue::Bad,
                };
            } else if value < min {
                // Positive but under-range: always use minimum color
                return PixelValue::Color(0.0);
            } else if value >= max {
                1.0
            } else {
                // Use pre-computed log constants if available, avoiding 2× ln() calls per pixel
                if let Some(c) = cache {
                    (value.ln() - c.log_min) / c.log_range
                } else {
                    (value.ln() - min.ln()) / (max.ln() - min.ln())
                }
            }
        }

        Scale::Asinh { scale } => {
            let val = (value / scale).asinh();
            // Use pre-computed asinh constants if available, avoiding 2× asinh() calls per pixel
            if let Some(c) = cache {
                (val - c.asinh_min) / c.asinh_range
            } else {
                let min_val = (min / scale).asinh();
                let max_val = (max / scale).asinh();
                (val - min_val) / (max_val - min_val)
            }
        }

        // ✅ Symlog explicitly supports negative values
        Scale::Symlog { linthresh } => {
            let abs_val = value.abs();
            let max_abs = max.abs();

            if abs_val < linthresh {
                0.5 + 0.5 * (value / linthresh)
            } else {
                0.5 + 0.5 * value.signum() * (linthresh + (abs_val - linthresh).ln())
                    / (linthresh + (max_abs - linthresh).ln())
            }
        }

        // ✅ PlanckLog also symmetric
        Scale::PlanckLog { linthresh } => {
            if value.abs() < linthresh {
                0.5 + 0.5 * (value / linthresh)
            } else {
                0.5 + 0.5 * value.signum() * (linthresh + (value.abs() - linthresh).ln())
                    / (linthresh + (max - linthresh).ln())
            }
        }

        Scale::Histogram => {
            let hist = hist.expect("Histogram scale requires histogram");

            if value <= hist.minv {
                return PixelValue::Color(0.0);
            }
            if value >= hist.maxv {
                return PixelValue::Color(1.0);
            }

            // Binary search with linear interpolation for smooth CDF lookup
            match hist
                .values
                .binary_search_by(|v| v.partial_cmp(&value).unwrap())
            {
                Ok(i) => {
                    // Exact match: return CDF value directly
                    hist.cdf[i]
                }
                Err(i) => {
                    // Value falls between hist.values[i-1] and hist.values[i]
                    // Linear interpolation in CDF space
                    if i == 0 {
                        0.0
                    } else if i >= hist.values.len() {
                        1.0
                    } else {
                        // Interpolate between CDF[i-1] and CDF[i] based on value position
                        let v0 = hist.values[i - 1];
                        let v1 = hist.values[i];
                        let cdf0 = hist.cdf[i - 1];
                        let cdf1 = hist.cdf[i];

                        // Weight: where does value fall between v0 and v1?
                        let w = (value - v0) / (v1 - v0);
                        cdf0 + w * (cdf1 - cdf0)
                    }
                }
            }
        }
    };

    PixelValue::Color(t.clamp(0.0, 1.0))
}

#[test]
fn linear_underflow_always_saturates() {
    let t = scale_value(-5.0, 0.0, 10.0, Scale::Linear, NegMode::Unseen, None, None);
    match t {
        PixelValue::Color(c) => assert_eq!(c, 0.0),
        _ => panic!("Linear underflow should saturate, not go Bad"),
    }
}
