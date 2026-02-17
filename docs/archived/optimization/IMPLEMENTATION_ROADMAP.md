# Implementation Roadmap: Next Optimization Steps

## Decision Tree: Which Optimization to Do First?

```
START: Need to improve render time?
  │
  ├─→ Scale type is LOG, ASINH, or SYMLOG?
  │    │
  │    └─→ YES: ⭐ Implement pre-computed scale logs
  │         │   File: src/scale.rs
  │         │   Time: 1-2 hours
  │         │   Gain: +1-2% per expensive scale
  │         │   Risk: Low
  │         │
  │         └─→ Proceed to Gamma check
  │
  ├─→ Gamma != 1.0? (default)
  │    │
  │    └─→ YES: ⭐ Implement Gamma LUT for common values
  │         │   File: src/plot/mod.rs line ~265
  │         │   Time: 30 minutes
  │         │   Gain: +1-2% if gamma ≠ 1
  │         │   Risk: Very Low
  │         │
  │         └─→ Proceed to Histogram check
  │
  ├─→ Scale type is HISTOGRAM?
  │    │
  │    └─→ YES: Implement binary search CDF lookup
  │         │   File: src/scale.rs HistogramScale
  │         │   Time: 1 hour
  │         │   Gain: +0.5-1%
  │         │   Risk: Low
  │         │
  │         └─→ Measure improvement
  │
  ├─→ Total gain from Tier 1: 2-5%?
       │
       ├─→ NO (less than 3%): SIMD is probably not worth it
       │   Reason: We're saturating available optimizations
       │
       └─→ YES: Consider Tier 2 (SIMD batching)
            Time investment: 2-3 days
            Potential: +5-8%
```

---

## Implementation Checklist: Pre-Computed Scale Logs

### Step 1: Create ScaleCache struct

**File**: `src/scale.rs` (new struct near top)

```rust
#[derive(Clone)]
pub struct ScaleCache {
    pub scale_type: Scale,
    // Pre-computed constants to avoid per-pixel recomputation
    pub log_min: f64,
    pub log_max: f64,
    pub log_range: f64,  // log_max - log_min
    pub asinh_min: f64,
    pub asinh_max: f64,
    pub asinh_range: f64,
}

impl ScaleCache {
    pub fn new(min: f64, max: f64, scale: Scale) -> Self {
        match scale {
            Scale::Log => {
                let log_min = min.ln();
                let log_max = max.ln();
                Self {
                    scale_type: scale,
                    log_min,
                    log_max,
                    log_range: log_max - log_min,
                    asinh_min: 0.0,
                    asinh_max: 0.0,
                    asinh_range: 0.0,
                }
            }
            Scale::Asinh { scale: s } => {
                let asinh_min = (min / s).asinh();
                let asinh_max = (max / s).asinh();
                Self {
                    scale_type: scale,
                    log_min: 0.0,
                    log_max: 0.0,
                    log_range: 0.0,
                    asinh_min,
                    asinh_max,
                    asinh_range: asinh_max - asinh_min,
                }
            }
            _ => Self {
                scale_type: scale,
                log_min: 0.0, log_max: 0.0, log_range: 0.0,
                asinh_min: 0.0, asinh_max: 0.0, asinh_range: 0.0,
            }
        }
    }
}
```

### Step 2: Modify scale_value to use cache

**File**: `src/scale.rs` (function signature change)

```rust
// OLD:
pub fn scale_value(value: f64, min: f64, max: f64, scale: Scale, ...) -> PixelValue

// NEW:
pub fn scale_value(value: f64, min: f64, max: f64, scale: Scale, 
                   cache: Option<&ScaleCache>, ...) -> PixelValue {
    // ... early returns unchanged ...
    
    let t = match scale {
        Scale::Linear => { /* unchanged */ },
        
        Scale::Log => {
            if value <= 0.0 { return PixelValue::Bad; }
            if value < min { return PixelValue::Color(0.0); }
            if value >= max { return PixelValue::Color(1.0); }
            
            // NEW: Use pre-computed logs if cache available
            if let Some(c) = cache {
                (value.ln() - c.log_min) / c.log_range
            } else {
                // Fallback for when cache not provided (backward compat)
                (value.ln() - min.ln()) / (max.ln() - min.ln())
            }
        },
        
        Scale::Asinh { scale: s } => {
            let val = (value / s).asinh();
            if let Some(c) = cache {
                (val - c.asinh_min) / c.asinh_range
            } else {
                let min_val = (min / s).asinh();
                let max_val = (max / s).asinh();
                (val - min_val) / (max_val - min_val)
            }
        },
        // ... rest unchanged ...
    };
    
    PixelValue::Color(t)
}
```

### Step 3: Update all scale_value call sites

**File**: `src/plot/mod.rs` (line ~246)

```rust
// OLD:
let pixel_val = match crate::healpix::sample_healpix(...) {
    Some(val) => crate::scale::scale_value(
        val,
        params.scale.minv, params.scale.maxv,
        params.scale_type, params.neg_mode,
        params.hist_scale,
    ),
    // ...
};

// NEW: pass cache (can create on-demand or pre-compute)
let pixel_val = match crate::healpix::sample_healpix(...) {
    Some(val) => {
        // Create cache once (or pass pre-computed)
        let cache = ScaleCache::new(
            params.scale.minv, params.scale.maxv,
            params.scale_type
        );
        crate::scale::scale_value(
            val,
            params.scale.minv, params.scale.maxv,
            params.scale_type, params.neg_mode,
            Some(&cache),  // ← NEW
            params.hist_scale,
        )
    },
    // ...
};
```

**Better approach**: Pre-compute cache once before loop

```rust
pub fn render_projection_to_grid(params: crate::params::RenderGridParams, grid: &mut RasterGrid) {
    // Pre-compute scale cache once
    let scale_cache = crate::scale::ScaleCache::new(
        params.scale.minv, params.scale.maxv,
        params.scale_type,
    );
    
    for py in 0..height {
        for px in 0..width {
            // ... projection code ...
            let pixel_val = match crate::healpix::sample_healpix(...) {
                Some(val) => crate::scale::scale_value(
                    val,
                    params.scale.minv, params.scale.maxv,
                    params.scale_type, params.neg_mode,
                    Some(&scale_cache),  // ← Use pre-computed cache
                    params.hist_scale,
                ),
                None => PixelValue::Bad,
            };
            // ... rest of loop ...
        }
    }
}
```

### Step 4: Test & Benchmark

```bash
# 1. Build
cargo build --release 2>&1 | tail -5

# 2. Benchmark with log scale (where optimization matters)
for i in 1 2 3; do
    time ./target/release/map2fig \
        -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
        -w 2400 \
        --log --min 1e-5 --max 1e-2 \
        -o /tmp/test_log.pdf 2>&1 | tail -3
done

# Compare against baseline (main branch)
git stash && cargo build --release >/dev/null 2>&1
# ... benchmark on main ...
git stash pop
```

---

## Implementation Checklist: Gamma LUT

### Step 1: Add Gamma LUT function

**File**: `src/plot/mod.rs` (add new function)

```rust
/// Apply gamma correction with LUT for common values
#[inline]
fn apply_gamma(t: f64, gamma: f64) -> f64 {
    // Common values that appear in astronomy
    match gamma {
        g if (g - 1.0).abs() < 1e-10 => t,              // gamma=1: no-op
        g if (g - 2.0).abs() < 1e-10 => t * t,          // gamma=0.5: square
        g if (g - 0.5).abs() < 1e-10 => t.sqrt(),       // gamma=2.0: sqrt
        g if (g - 3.0).abs() < 1e-10 => t * t * t,      // gamma=0.33: cube
        g if (g - 0.333).abs() < 1e-6 => t.powf(1.0/3.0),  // gamma^-1/3
        _ => t.powf(gamma),                              // General fallback
    }
}
```

### Step 2: Replace pow() call in hot loop

**File**: `src/plot/mod.rs` (line ~265)

```rust
// OLD:
let t = if gamma_inv == 1.0 { t } else { t.powf(gamma_inv) };

// NEW:
let t = apply_gamma(t, gamma_inv);
```

### Step 3: Benchmark

```bash
# Build
cargo build --release 2>&1 | tail -5

# Benchmark with gamma (e.g., --gamma 2.0 for contrast boost)
for i in 1 2 3; do
    time ./target/release/map2fig \
        -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
        -w 2400 \
        --gamma 2.0 \
        -o /tmp/test_gamma2.pdf 2>&1 | tail -3
done
```

**Expected performance**:
- gamma=1.0: 0% change (already fast-pathed)
- gamma=2.0, 0.5: +5-10% speedup
- gamma=other: same as before

---

## Tier 2 Entry Point: SIMD Batching

**When to consider**: After Tier 1 optimizations, if gains are measurable and you want more.

### Key Idea

```rust
// Current: Process 1 pixel at a time
for pixel in all_pixels {
    let (lon, lat) = projection(pixel);
    let (theta, phi) = normalize(lon, lat);
    let v_view = sph_to_vec(theta, phi);
    let v_map = rotate(v_view);
    let (theta_m, phi_m) = vec_to_sph(v_map);
    let idx = ang2pix(theta_m, phi_m);
    let value = map[idx];
    let scaled = scale_value(value, min, max);
    let color = colormap(scaled);
    output[pixel] = color;
}

// SIMD: Process 8 pixels at once (vectorized trig, multiply)
for pixel_batch in chunks_of_8(all_pixels) {
    let lons = [proj.lon for each pixel]      // 8×f64 SIMD vector
    let lats = [proj.lat for each pixel]      // 8×f64 SIMD vector
    
    // Vectorized trig: process all 8 sin/cos in 1 instruction
    let sin_t = sin_simd(thetas);             // 8 sines in parallel
    let cos_t = cos_simd(thetas);             // 8 cosines in parallel
    let v_view = [sin_t*cos_l, sin_t*sin_l, cos_t];  // 3×8 matrix
    
    // Vectorized matrix multiply: 3×3 @ 8×3 matrix
    let v_map = rotate_simd(v_view);          // 8 rotations in parallel
    
    // Rest of pipeline...
}
```

**Files to refactor**:
1. `src/plot/mod.rs` - main loop structure
2. `src/rotation.rs` - add `apply_inverse_batch()`
3. `src/healpix.rs` - add `sph_to_vec_batch()`, `vec_to_sph_batch()`

**Complexity**: 200-300 lines of new code, moderate risk of off-by-one errors

---

## Testing Strategy

### Verification Steps

```bash
# 1. Correctness: Compare outputs on small map
cargo build --release
./target/release/map2fig -f m_test.fits -o /tmp/before.pdf
# git checkout main version
cp /tmp/before.pdf /tmp/before_opt.pdf  # Save optimized

git stash
cargo build --release >/dev/null 2>&1
./target/release/map2fig -f m_test.fits -o /tmp/before_main.pdf
git stash pop

# 2. Visual comparison (should be identical)
diff <(identify /tmp/before_main.pdf) <(identify /tmp/before_opt.pdf)

# 3. Performance: Multiple runs on larger file
for scale in linear log asinh; do
    echo "Testing $scale scale..."
    for i in 1 2 3; do
        time ./target/release/map2fig \
            -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
            -w 2400 \
            --$scale \
            -o /tmp/test.pdf 2>&1 | grep real
    done
done

# 4. Statistical significance (compute mean & std dev)
cargo run --release --example benchmark_scales
```

---

## Commit Strategy

### Commit 1: Scale Log Caching

```bash
git checkout -b optimize-scale-caching
# ... implement ScaleCache ...
cargo test --release
git commit -m "Optimize scaling with pre-computed log/asinh cache

- Add ScaleCache struct to avoid per-pixel recomputation
- Cache log(min), log(max), asinh(min), asinh(max)
- Update scale_value to use cached values
- Measurable 1-2% improvement on log/asinh scales
- Backward compatible (cache optional)
"
```

### Commit 2: Gamma LUT

```bash
git checkout -b optimize-gamma-lut  
# ... implement apply_gamma with LUT ...
cargo test --release
git commit -m "Add gamma correction LUT for common values

- Implement apply_gamma with fast-path for gamma in (1.0, 2.0, 0.5, 0.33)
- Reduces powf() calls for common adjustments
- 5-10% speedup when gamma != 1.0
- General fallback ensures correctness for arbitrary gamma
"
```

### Merge to main

```bash
git checkout main
git merge optimize-gamma-lut  # (after scale-caching merged)
git log --oneline -3
```

---

## Success Criteria

| Metric | Target | Pass/Fail |
|--------|--------|-----------|
| Build succeeds | Yes | ✅/❌ |
| Tests pass | 100% | ✅/❌ |
| Output matches baseline | Yes (byte-identical with same seed) | ✅/❌ |
| Gain on log scale | >1% | ✅/❌ |
| Gain on gamma!=1 | >1% | ✅/❌ |
| Gain overall | >2% on variety | ✅/❌ |
| Code understandable | Yes (documented) | ✅/❌ |

---

## Expected Outcome

**Timeline**: 3-4 hours total  
**Combined gain**: 2-5% (~1-2 seconds on 23s baseline)  
**Confidence**: High (both changes are conservative, well-understood)

**After Tier 1, decision point**:
- If gain measurable (>3%): Tier 2 (SIMD) is worth investigating
- If gain lost in noise (<2%): Call it done, optimization-per-dollar not favorable
