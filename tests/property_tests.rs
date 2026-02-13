//! Property-based tests for critical functions using proptest
//!
//! These tests verify mathematical properties that should ALWAYS hold,
//! across thousands of randomly-generated inputs. Unlike unit tests,
//! property tests discover edge cases humans wouldn't think of.

use map2fig::scale::{Scale, scale_value};
use map2fig::{NegMode, PixelValue};
use proptest::prelude::*;

// ============================================================================
// SCALING PROPERTIES
// ============================================================================

/// Property: Linear scaling always produces normalized output in [0.0, 1.0]
#[test]
fn prop_linear_scaling_bounded() {
    proptest!(|(
        value in -1e6f64..1e6f64,
        min in -1e6f64..0f64,
        max in 0f64..1e6f64,
    )| {
        let result = scale_value(value, min, max, Scale::Linear, NegMode::Zero, None);

        if let PixelValue::Color(t) = result {
            prop_assert!(
                t >= 0.0 && t <= 1.0,
                "Linear scale produced out-of-bounds value: {} (value={}, min={}, max={})",
                t, value, min, max
            );
        }
    });
}

/// Property: Values equal to min always scale to 0.0
#[test]
fn prop_linear_min_value_scales_to_zero() {
    proptest!(|(
        min in -1e6f64..1e6f64,
        max in -1e6f64..1e6f64,
    )| {
        let min = min.min(max);
        let max = max.max(min + 1e-6); // Ensure min < max

        let result = scale_value(min, min, max, Scale::Linear, NegMode::Zero, None);

        if let PixelValue::Color(t) = result {
            prop_assert!(
                (t - 0.0).abs() < 1e-10,
                "Value at min should scale to 0.0, got {} (min={}, max={})",
                t, min, max
            );
        }
    });
}

/// Property: Values equal to max always scale to 1.0
#[test]
fn prop_linear_max_value_scales_to_one() {
    proptest!(|(
        min in -1e6f64..1e6f64,
        max in -1e6f64..1e6f64,
    )| {
        let min = min.min(max);
        let max = max.max(min + 1e-6); // Ensure min < max

        let result = scale_value(max, min, max, Scale::Linear, NegMode::Zero, None);

        if let PixelValue::Color(t) = result {
            prop_assert!(
                (t - 1.0).abs() < 1e-10,
                "Value at max should scale to 1.0, got {} (min={}, max={})",
                t, min, max
            );
        }
    });
}

/// Property: Log scaling always produces normalized [0.0, 1.0] output
#[test]
fn prop_log_scaling_bounded() {
    proptest!(|(
        value in 1e-6f64..1e6f64,  // Log scale requires positive values
        min in 1e-6f64..1e3f64,
        max in 1e-6f64..1e6f64,
    )| {
        let min = min.min(max);
        let max = max.max(min * 1.0001); // Ensure min < max

        let result = scale_value(value, min, max, Scale::Log, NegMode::Zero, None);

        if let PixelValue::Color(t) = result {
            prop_assert!(
                t >= 0.0 && t <= 1.0,
                "Log scale produced out-of-bounds value: {} (value={}, min={}, max={})",
                t, value, min, max
            );
        }
    });
}

/// Property: SymLog scaling always produces normalized [0.0, 1.0] output
#[test]
fn prop_symlog_scaling_bounded() {
    proptest!(|(
        value in -1e6f64..1e6f64,
        min in -1e6f64..1e6f64,
        max in -1e6f64..1e6f64,
    )| {
        let min = min.min(max);
        let max = max.max(min + 1e-6); // Ensure min < max

        let result = scale_value(
            value, min, max,
            Scale::Symlog { linthresh: 1.0 },
            NegMode::Zero,
            None
        );

        if let PixelValue::Color(t) = result {
            prop_assert!(
                t >= 0.0 && t <= 1.0,
                "SymLog scale produced out-of-bounds value: {} (value={}, min={}, max={})",
                t, value, min, max
            );
        }
    });
}

/// Property: Asinh scaling always produces normalized [0.0, 1.0] output
#[test]
fn prop_asinh_scaling_bounded() {
    proptest!(|(
        value in -1e6f64..1e6f64,
        min in -1e6f64..1e6f64,
        max in -1e6f64..1e6f64,
    )| {
        let min = min.min(max);
        let max = max.max(min + 1e-6); // Ensure min < max

        let result = scale_value(
            value, min, max,
            Scale::Asinh { scale: 1.0 },
            NegMode::Zero,
            None
        );

        if let PixelValue::Color(t) = result {
            prop_assert!(
                t >= 0.0 && t <= 1.0,
                "Asinh scale produced out-of-bounds value: {} (value={}, min={}, max={})",
                t, value, min, max
            );
        }
    });
}

/// Property: NaN values always produce PixelValue::Bad
#[test]
fn prop_nan_always_bad() {
    proptest!(|(
        min in -1e6f64..1e6f64,
        max in -1e6f64..1e6f64,
    )| {
        let min = min.min(max);
        let max = max.max(min + 1e-6);

        let result = scale_value(f64::NAN, min, max, Scale::Linear, NegMode::Zero, None);

        match result {
            PixelValue::Bad => {/* OK */},
            other => prop_assert!(false, "NaN should produce PixelValue::Bad, got {:?}", other),
        }
    });
}

/// Property: Infinity values always produce PixelValue::Bad
#[test]
fn prop_infinity_always_bad() {
    proptest!(|(
        min in -1e6f64..1e6f64,
        max in -1e6f64..1e6f64,
    )| {
        let min = min.min(max);
        let max = max.max(min + 1e-6);

        let pos_inf = scale_value(f64::INFINITY, min, max, Scale::Linear, NegMode::Zero, None);
        let neg_inf = scale_value(f64::NEG_INFINITY, min, max, Scale::Linear, NegMode::Zero, None);

        match pos_inf {
            PixelValue::Bad => {/* OK */},
            other => prop_assert!(false, "Positive infinity should produce PixelValue::Bad, got {:?}", other),
        }

        match neg_inf {
            PixelValue::Bad => {/* OK */},
            other => prop_assert!(false, "Negative infinity should produce PixelValue::Bad, got {:?}", other),
        }
    });
}

/// Property: Monotonicity - if value1 < value2, their scaled values should satisfy scale1 <= scale2
#[test]
fn prop_linear_monotonic() {
    proptest!(|(
        v1 in -1e6f64..1e6f64,
        v2 in -1e6f64..1e6f64,
        min in -1e6f64..0f64,
        max in 0f64..1e6f64,
    )| {
        let (v1, v2) = if v1 < v2 { (v1, v2) } else { (v2, v1) };

        let r1 = scale_value(v1, min, max, Scale::Linear, NegMode::Zero, None);
        let r2 = scale_value(v2, min, max, Scale::Linear, NegMode::Zero, None);

        if let (PixelValue::Color(t1), PixelValue::Color(t2)) = (r1, r2) {
            prop_assert!(
                t1 <= t2,
                "Monotonicity violated: {} scaled to {}, {} scaled to {}",
                v1, t1, v2, t2
            );
        }
    });
}

/// Property: Values outside [min, max] clamp to [0.0, 1.0]
#[test]
fn prop_clamping() {
    proptest!(|(
        offset in 1e3f64..1e6f64,
        min in -1e3f64..0f64,
        max in 0f64..1e3f64,
    )| {
        // Test value below min
        let below_min = min - offset;
        let result_below = scale_value(below_min, min, max, Scale::Linear, NegMode::Zero, None);
        if let PixelValue::Color(t) = result_below {
            prop_assert!((t - 0.0).abs() < 1e-10, "Values below min should clamp to 0.0, got {}", t);
        }

        // Test value above max
        let above_max = max + offset;
        let result_above = scale_value(above_max, min, max, Scale::Linear, NegMode::Zero, None);
        if let PixelValue::Color(t) = result_above {
            prop_assert!((t - 1.0).abs() < 1e-10, "Values above max should clamp to 1.0, got {}", t);
        }
    });
}

// ============================================================================
// EDGE CASE PROPERTIES
// ============================================================================

/// Property: When min == max, any valid value scales to 0.5
#[test]
fn prop_min_equals_max_scales_to_half() {
    proptest!(|(
        val in -1e6f64..1e6f64,
        same_val in -1e6f64..1e6f64,
    )| {
        // Ensure same_val is not NaN/Inf
        prop_assume!(same_val.is_finite());

        let result = scale_value(val, same_val, same_val, Scale::Linear, NegMode::Zero, None);

        if let PixelValue::Color(t) = result {
            prop_assert!(
                (t - 0.5).abs() < 1e-10,
                "When min == max, valid values should scale to 0.5, got {}",
                t
            );
        }
    });
}

/// Property: Negative scaling ranges work correctly
#[test]
fn prop_negative_range() {
    proptest!(|(
        value in -100f64..-1f64,
        min in -100f64..-50f64,
        max in -50f64..-1f64,
    )| {
        let result = scale_value(value, min, max, Scale::Linear, NegMode::Zero, None);

        if let PixelValue::Color(t) = result {
            prop_assert!(
                t >= 0.0 && t <= 1.0,
                "Negative range scaling should still produce normalized output"
            );
        }
    });
}

/// Property: Very small ranges (near-zero differences) don't cause division by zero
#[test]
fn prop_tiny_range_safe() {
    proptest!(|(
        min in -1e6f64..1e6f64,
    )| {
        let max = min + 1e-12; // Extremely small range
        let value = (min + max) / 2.0;

        let result = scale_value(value, min, max, Scale::Linear, NegMode::Zero, None);

        if let PixelValue::Color(t) = result {
            prop_assert!(
                t.is_finite(),
                "Very small ranges should not produce NaN/Inf"
            );
        }
    });
}
// ============================================================================
// COLORMAP PROPERTIES
// ============================================================================

/// Property: Colormap sampling always produces valid output without panicking
#[test]
fn prop_colormap_sampling_bounds() {
    use map2fig::get_colormap;

    proptest!(|(
        t in 0.0f64..=1.0f64,
    )| {
        let cmap = get_colormap("viridis");
        // Should not panic - the actual result is valid if no panic occurs
        let _ = cmap.sample(t);
    });
}

/// Property: All available colormaps work without panicking
#[test]
fn prop_all_colormaps_valid() {
    use map2fig::available_colormaps;
    use map2fig::get_colormap;

    let cmap_names = available_colormaps();
    for name in cmap_names {
        // Should not panic
        let cmap = get_colormap(name);

        // Sample across the range - just verify no panic
        for i in 0..=10 {
            let t = i as f64 / 10.0;
            let _ = cmap.sample(t);
        }
    }
}
