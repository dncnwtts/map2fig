use crate::PixelValue;
use crate::NegMode;
use crate::healpix::is_seen;
use crate::colorbar::ColorbarTicks;

pub fn scale_t_to_value(
    t: f64,
    min: f64,
    max: f64,
    scale: Scale,
) -> f64 {
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

pub fn value_to_t(
    value: f64,
    min: f64,
    max: f64,
    scale: Scale,
) -> Option<f64> {
    match scale {
        Scale::Linear => Some((value - min) / (max - min)),

        Scale::Log => {
            if value <= 0.0 || min <= 0.0 {
                None
            } else {
                Some(
                    (value.ln() - min.ln()) /
                    (max.ln() - min.ln())
                )
            }
        }

        Scale::Asinh { scale: s } => {
            Some(
                (value / s).asinh() /
                (max / s).asinh()
            )
        }

        Scale::Symlog { linthresh } => {
            let f = |x: f64| {
                if x.abs() < linthresh {
                    x / linthresh
                } else {
                    x.signum() * (x.abs() / linthresh).ln()
                }
            };
            Some(
                (f(value) - f(min)) /
                (f(max) - f(min))
            )
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
            Some(
                (f(value) - f(min)) /
                (f(max) - f(min))
            )
        }

        Scale::Histogram => todo!()
    }
}


pub struct HistogramScale {
    pub values: Vec<f64>,   // sorted unique values or bin centers
    pub cdf: Vec<f64>,      // monotonically increasing [0,1]

    pub minv: f64,
    pub maxv: f64,
}

#[derive(Clone, Copy, Debug)]
pub enum HistogramRange {
    Percentile { low: f64, high: f64 },
    Explicit { min: f64, max: f64 },
    Full,
}


pub fn build_histogram_scale(
    map: &[f64],
    range: HistogramRange,
    neg_mode: NegMode,
    bins: usize,
) -> HistogramScale {
    // ------------------------------------------------------------
    // 1. Collect valid values
    // ------------------------------------------------------------
    let mut vals: Vec<f64> = map
        .iter()
        .copied()
        .filter(|v| is_seen(*v))
        .filter(|v| match neg_mode {
            NegMode::Zero => true,
            NegMode::Unseen => *v >= 0.0,
        })
        .collect();

    if vals.is_empty() {
        return HistogramScale {
            values: vec![],
            cdf: vec![],
            minv: 0.0,
            maxv: 1.0,
        };
    }

    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = vals.len();

    // ------------------------------------------------------------
    // 2. Determine minv / maxv from HistogramRange
    // ------------------------------------------------------------
    let (minv, maxv) = match range {
        HistogramRange::Explicit { min, max } => (min, max),

        HistogramRange::Full => {
            println!("{}, {}, range?", vals[0], vals[n-1]);
            (vals[0], vals[n - 1])

        }

        HistogramRange::Percentile { low, high } => {
            assert!(
                (0.0..=1.0).contains(&low) && (0.0..=1.0).contains(&high) && low < high,
                "Invalid percentile range"
            );

            let ilo = ((n - 1) as f64 * low).round() as usize;
            let ihi = ((n - 1) as f64 * high).round() as usize;

            (vals[ilo], vals[ihi])
        }
    };

    // ------------------------------------------------------------
    // 3. Filter to chosen range
    // ------------------------------------------------------------
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

    // ------------------------------------------------------------
    // 4. Downsample into LUT
    // ------------------------------------------------------------
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


#[derive(Clone, Copy, PartialEq)]
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
    match scale {
        Scale::Histogram => {
            return histogram_ticks(
                hist.expect("Histogram ticks require histogram scale")
            );
        }

        _ => {}
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


pub fn histogram_ticks(hist: &HistogramScale) -> ColorbarTicks {
    let percentiles = [0.0, 0.1, 0.25, 0.5, 0.75, 0.9, 1.0];

    let mut major_values = Vec::new();
    let mut major_positions = Vec::new();

    for &p in &percentiles {
        let idx = (p * (hist.values.len() - 1) as f64).round() as usize;
        major_values.push(hist.values[idx]);
        major_positions.push(p);
    }

    ColorbarTicks {
        major_values,
        major_positions,
        minor_values: vec![],
        minor_positions: vec![],
    }
}



fn scale_position(
    value: f64,
    min: f64,
    max: f64,
    scale: &Scale,
) -> Option<f64> {
    match scale {
        Scale::Linear => {
            Some(((value - min) / (max - min)).clamp(0.0, 1.0))
        }

        Scale::Log => {
            if value <= 0.0 || min <= 0.0 {
                None
            } else {
                Some(
                    ((value.ln() - min.ln()) / (max.ln() - min.ln()))
                        .clamp(0.0, 1.0),
                )
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
            scale_position(value, min, max, &Scale::Symlog { linthresh: *linthresh })
        }

        Scale::Histogram => todo!() // intentionally unsupported here
    }
}

pub fn linear_ticks(min: f64, max: f64) -> ColorbarTicks {
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

pub fn log_ticks(min: f64, max: f64) -> ColorbarTicks {
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

pub fn asinh_ticks(min: f64, max: f64, scale: f64) -> ColorbarTicks {
    symlog_ticks(min, max, scale)
}


pub fn symlog_ticks(min: f64, max: f64, linthresh: f64) -> ColorbarTicks {
    let mut major_values = vec![0.0, linthresh, -linthresh];
    let mut minor_values = Vec::new();

    // linear core
    let n = 4;
    let step = linthresh / n as f64;

    for i in (-n+1)..=(n-1) {
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

    major_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
    minor_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

    ColorbarTicks {
        major_positions: vec![],
        minor_positions: vec![],
        major_values,
        minor_values,
    }
}





pub fn scale_value(
    value: f64,
    min: f64,
    max: f64,
    scale: Scale,
    neg_mode: NegMode,
    hist: Option<&HistogramScale>,
) -> PixelValue {
    if min >= max {
        panic!("min must be < max");
    }

    // Unseen / NaN handling
    if !is_seen(value) {
        return PixelValue::Bad;
    }
    
    let t = match scale {
        Scale::Linear => {
            if value <= min {
                0.0
            } else if value >= max {
                1.0
            } else {
                (value - min) / (max - min)
            }
        }

        Scale::Log => {
            if value <= 0.0 || value < min {
                return match neg_mode {
                    NegMode::Zero => PixelValue::Color(0.0),
                    NegMode::Unseen => PixelValue::Bad,
                };
            } else if value >= max {
                1.0
            } else {
                (value.ln() - min.ln()) / (max.ln() - min.ln())
            }
        }

    
        Scale::Asinh { scale } => {
            let val = (value / scale).asinh();
            let min_val = (min / scale).asinh();
            let max_val = (max / scale).asinh();
            (val - min_val) / (max_val - min_val)
        }
    
        // ✅ Symlog explicitly supports negative values
        Scale::Symlog { linthresh } => {
            let abs_val = value.abs();
            let max_abs = max.abs();
    
            if abs_val < linthresh {
                0.5 + 0.5 * (value / linthresh)
            } else {
                0.5
                    + 0.5
                        * value.signum()
                        * (linthresh + (abs_val - linthresh).ln())
                        / (linthresh + (max_abs - linthresh).ln())
            }
        }
    
        // ✅ PlanckLog also symmetric
        Scale::PlanckLog { linthresh } => {
            if value.abs() < linthresh {
                0.5 + 0.5 * (value / linthresh)
            } else {
                0.5
                    + 0.5
                        * value.signum()
                        * (linthresh + (value.abs() - linthresh).ln())
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
        
            match hist.values.binary_search_by(|v| v.partial_cmp(&value).unwrap()) {
                Ok(i) => hist.cdf[i],
                Err(i) => {
                    if i == 0 {
                        0.0
                    } else if i >= hist.cdf.len() {
                        1.0
                    } else {
                        hist.cdf[i]
                    }
                }
            }
        }


    };
    
    PixelValue::Color(t.clamp(0.0, 1.0))

}


#[test]
fn linear_underflow_always_saturates() {
    let t = scale_value(-5.0, 0.0, 10.0, Scale::Linear, NegMode::Unseen);
    match t {
        PixelValue::Color(c) => assert_eq!(c, 0.0),
        _ => panic!("Linear underflow should saturate, not go Bad"),
    }
}

