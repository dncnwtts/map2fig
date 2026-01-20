use crate::PixelValue;
use crate::NegMode;
use crate::healpix::is_seen;

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
    }
}


#[derive(Clone, Copy)]
pub enum Scale {
    Linear,
    Log,
    Asinh { scale: f64 },
    Symlog { linthresh: f64 },
    PlanckLog { linthresh: f64 },
}


pub fn scale_value(
    value: f64,
    min: f64,
    max: f64,
    scale: Scale,
    neg_mode: NegMode,
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

