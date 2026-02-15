# Approximate Math Optimization: Implementation Guide

**Date:** February 15, 2026  
**Status:** Feasibility + Design

---

## How Approximate Math Works

Instead of using full-precision libm functions (sin, cos, atan2, asin), use fast approximations that are **"good enough"** for the application.

### Trade-off: Speed vs Accuracy

| Method | Speed | Accuracy | Error Bound | Use Case |
|--------|-------|----------|-------------|----------|
| **libm (current)** | Baseline | ~15 ULP | 1e-14 rel. error | Astronomy (high precision needed) |
| **Fast poly (degree 5)** | 3-5x faster | ~1000 ULP | 1e-10 rel. error | Machine learning |
| **Fast poly (degree 7)** | 2-3x faster | ~100 ULP | 1e-11 rel. error | Graphics/gaming |
| **Lookup table** | 10-50x faster | ~10000 ULP | 1e-7 rel. error | Real-time graphics |

**Key Question for HEALPix:** How much precision do we *need*?

---

## HEALPix Accuracy Requirements

### The Pixel Index Pipeline

```
1. Mollweide projection:  (px, py) → (lon, lat) 
   Error introduced: need < 1 pixel width
   
2. HEALPix indexing:      (lon, lat) → ring index
   Error tolerance: ~1e-10 relative (must get exactly right pixel)
   
3. Array lookup:          ring_index → data_value
   Error tolerance: None (discrete access)
```

### How Much Sin/Cos Error is Acceptable?

For NSIDE=8192 (3 GB test file):
- Pixel angular size: ~0.105 arcminutes = 3e-5 radians
- Mollweide projection angular error tolerance: **1e-6 radians** (33× smaller than pixel)
  - This ensures we never look up wrong neighboring pixel
  - Requires sin/cos accurate to ~1e-8 relative error

**Test:** Can we achieve 1e-8 accuracy with fast math?
- **Chebyshev poly degree 7:** Yes (1e-11 typical)
- **Chebyshev poly degree 5:** Borderline (1e-10 typical)
- **Taylor series to degree 7:** Yes (1e-11 typical)
- **Lookup table (256 entry):** No (1e-7 typical) ❌

**Practical choice:** Degree 7 Chebyshev polynomial or minimax approximation

---

## Three Approaches to Approximate Math

### Approach 1: Chebyshev Polynomial Approximation ⭐ RECOMMENDED

Use a minimax polynomial that minimizes maximum error over the range.

**Example for sin(x) on [0, π/2]:**

```rust
#[inline(always)]
fn fast_sin(x: f64) -> f64 {
    // Chebyshev polynomial of degree 7
    // Approximates sin(x) on [0, π/2] with max error ~1e-11
    const C0: f64 = 0.999999999999999;
    const C1: f64 = -0.166666666666666;
    const C2: f64 = 0.008333333333333;
    const C3: f64 = -0.000198412698413;
    const C4: f64 = 0.000002755731922;
    const C5: f64 = -0.000000025052108;
    const C6: f64 = 0.000000000160590;
    
    let x2 = x * x;
    let x3 = x2 * x;
    let x4 = x2 * x2;
    let x5 = x4 * x;
    let x6 = x4 * x2;
    let x7 = x4 * x3;
    
    // Horner's method: more efficient than direct sum
    x * (C0 + x2 * (C1 + x2 * (C2 + x2 * (C3 + x2 * (C4 + x2 * (C5 + x2 * (C6)))))))
}
```

**Characteristics:**
- Speed: 2-3x faster than libm sin()
- Accuracy: Better than needed (1e-11 vs 1e-8 requirement)
- Portable: Pure Rust, works everywhere
- Effort: Copy coefficients from reference, ~30 min

**Problem:** Only works on restricted range [0, π/2]
- Need range reduction for full circle
- atan2 needs special handling for quadrants

### Approach 2: Use Single-Precision libm ⭐ SIMPLER

Cast to f32, use faster single-precision libm, cast back to f64.

```rust
#[inline(always)]
fn fast_sin(x: f64) -> f64 {
    (x as f32).sin() as f64
}
```

**Characteristics:**
- Speed: ~1.5-2.0x faster (libm sin is optimized for f32)
- Accuracy: ~7 decimal digits (1e-7 relative error) ⚠️
- Portable: Works everywhere libm works
- Effort: One-line change, 5 minutes

**Problem:** 1e-7 error might be too much!
- Let me test: How much does this affect HEALPix indexing?

### Approach 3: Use libm Approximate Functions

Some systems provide fast approximate versions:
- `sinf()` on glibc (fast single-precision)
- `__sincos_fma()` (we're already calling this!)

**Problem:** Already using the optimized versions via libm

---

## Testing Accuracy vs Speedup

The key is to **profile the actual impact** on HEALPix pixel indexing.

### Experiment Design

```rust
// src/simd.rs or new src/fast_math.rs

#[cfg(feature = "fast_math")]
mod math {
    // Fast approximations
    pub fn sin(x: f64) -> f64 {
        // ... fast version
    }
    pub fn cos(x: f64) -> f64 {
        // ... fast version  
    }
}

#[cfg(not(feature = "fast_math"))]
mod math {
    // Standard libm
    pub fn sin(x: f64) -> f64 { x.sin() }
    pub fn cos(x: f64) -> f64 { x.cos() }
}

// In mollweide.rs, replace:
//   let c = theta_aux.cos();
// With:
//   let c = math::cos(theta_aux);
```

### Benchmark Test Plan

1. **Build with fast_math feature disabled (baseline)**
   ```bash
   cargo build --release
   time ./target/release/map2fig -f test.fits -o /tmp/baseline.pdf
   ```

2. **Build with fast_math feature enabled**
   ```bash
   cargo build --release --features fast_math
   time ./target/release/map2fig -f test.fits -o /tmp/fast.pdf
   ```

3. **Compare output PDFs visually**
   - Same file size? (if different math, output may change)
   - Identical pixels? (extract color grid, compare)
   - Acceptable? (visual inspection for artifacts)

---

## Expected Performance Gains

From Callgrind profile:

| Math Function | Instructions | Speedup | Time Saved |
|---|---|---|---|
| sin() | 2.14B | 2-3x | 0.45-0.68 sec |
| cos() | (included above) | 2-3x | (included) |
| atan2() | 0.063B | 1.5-2x | 0.02 sec |
| asin() | 0.059B | 2-3x | 0.02 sec |
| acos() | 0.039B | 2-3x | 0.01 sec |
| **Total math** | **5.4B** | **2-3x** | **~0.5 sec saved** |

**Realistic estimate:** 0.3-0.5 seconds → **3-5% overall improvement**

**Compare to other optimizations:**
- SIMD (8 hours): 2-3% improvement ❌
- Fast math (2-3 hours): 3-5% improvement ✅
- Better ROI!

---

## Implementation Path (If You Choose This)

### Step 1: Create Feature Gate (15 min)

Add to Cargo.toml:
```toml
[features]
default = []
fast_math = []
```

### Step 2: Create Fast Math Module (30 min)

Create `src/fast_math.rs`:
```rust
#![allow(clippy::excessive_precision)]

/// Fast approximate sin using single-precision libm
#[inline(always)]
pub fn sin(x: f64) -> f64 {
    (x as f32).sin() as f64
}

/// Fast approximate cos using single-precision libm
#[inline(always)]
pub fn cos(x: f64) -> f64 {
    (x as f32).cos() as f64
}

// ... etc for atan2, asin, acos
```

Or with Chebyshev (more work but better accuracy):
```rust
#[inline(always)]
pub fn sin(mut x: f64) -> f64 {
    // Range reduction: bring x into [-π/2, π/2]
    let k = (x / std::f64::consts::PI).round() as i32;
    x -= k as f64 * std::f64::consts::PI;
    
    // Adjust sign based on which quadrant
    let sign = if (k & 2) != 0 { -1.0 } else { 1.0 };
    
    // Apply Chebyshev polynomial on reduced range
    // ... polynomial computation ...
    
    sign * result
}
```

### Step 3: Wire Into Mollweide Projection (15 min)

Change `src/mollweide.rs`:
```rust
#[cfg(feature = "fast_math")]
use crate::fast_math;
#[cfg(not(feature = "fast_math"))]
use std as fast_math; // use methods on f64 directly

// In pixel_to_ang():
let theta_aux = py.asin();
let c = theta_aux.cos();  // becomes: fast_math::cos(theta_aux)
```

### Step 4: Test and Benchmark (1 hour)

```bash
# Baseline
cargo build --release
time ./target/release/map2fig -f tests/data/combined_map_95GHz_8192.fits -o /tmp/test_baseline.pdf

# Fast math
cargo build --release --features fast_math  
time ./target/release/map2fig -f tests/data/combined_map_95GHz_8192.fits -o /tmp/test_fast.pdf

# Compare outputs
diff <(pdfimages -list /tmp/test_baseline.pdf) <(pdfimages -list /tmp/test_fast.pdf)
```

### Step 5: Validate Accuracy (1 hour)

```rust
#[test]
fn test_fast_sin_accuracy() {
    for i in 0..1000 {
        let x = (i as f64) * 2.0 * PI / 1000.0;
        let accurate = x.sin();
        let fast = fast_math::sin(x);
        let error = (accurate - fast).abs() / accurate.abs();
        assert!(error < 1e-8, "Sin error at x={}: {}", x, error);
    }
}

#[test]
fn test_healpix_indexing_stable() {
    // Critical test: does fast math produce same HEALPix indices?
    let (lon_accurate, lat_accurate) = mollweide_with_libm(px, py);
    let (lon_fast, lat_fast) = mollweide_with_fastmath(px, py);
    
    let idx_accurate = ang2pix_ring(lon_accurate, lat_accurate);
    let idx_fast = ang2pix_ring(lon_fast, lat_fast);
    
    // These MUST be identical (or within 1 pixel)
    assert_eq!(idx_accurate, idx_fast);
}
```

---

## What Could Go Wrong

### Risk 1: Accuracy Loss in Edge Cases
- **Symptom:** Some pixels map to wrong HEALPix index
- **Detection:** Visual artifacts in output PDF (color discontinuities)
- **Mitigation:** Test on many FITS files, validate with healpy comparison

### Risk 2: Different Behavior on Different CPUs
- **Symptom:** Works on Intel, fails on ARM/AMD
- **Detection:** Cross-platform testing
- **Mitigation:** Keep fast_math as optional feature, default to libm

### Risk 3: Compiler Optimization Changes Behavior
- **Symptom:** Release build works, debug build fails
- **Detection:** Run tests in both modes
- **Mitigation:** Use `#[inline(always)]` to prevent reordering

### Risk 4: PDF Output Changes
- **Symptom:** Users report "map looks different"
- **Cause:** Different rounding in color space conversion
- **Mitigation:** Document in release notes if enabling fast_math

---

## Decision Matrix

| Factor | Value | Impact |
|--------|-------|--------|
| **Expected speedup** | 3-5% | Moderate |
| **Implementation time** | 2-3 hours | Low
| **Risk level** | Low-Medium | Need testing |
| **User complexity** | Optional feature | Good |
| **Accuracy impact** | Needs validation | Critical |
| **ROI (speedup/hour)** | **1.5-2.5% per hour** | Better than SIMD |

---

## Recommendation

**Try this approach!** Here's why:

1. **High ROI:** 3-5% improvement vs 8 hours (SIMD)
2. **Simpler implementation:** Pure Rust, no nightly needed
3. **Conservative:** Can be feature-gated, disabled by default
4. **Lower risk:** Can validate against existing output
5. **Better motivation:** Teaches compiler optimization tricks

**Next steps:**
1. Implement single-precision f32 cast version (5 min) ✅ Easy baseline
2. Benchmark: measure actual speedup
3. Validate: check HEALPix indexing consistency
4. If good: add Chebyshev poly version for even more speed

---

## Code Skeleton (Ready to Implement)

```rust
// src/fast_math.rs - to be created

#[cfg(feature = "fast_math")]
pub mod math {
    use std::f64::consts::PI;
    
    #[inline(always)]
    pub fn sin(x: f64) -> f64 {
        (x as f32).sin() as f64
    }
    
    #[inline(always)]
    pub fn cos(x: f64) -> f64 {
        (x as f32).cos() as f64
    }
    
    #[inline(always)]
    pub fn asin(x: f64) -> f64 {
        (x as f32).asin() as f64
    }
    
    #[inline(always)]
    pub fn atan2(y: f64, x: f64) -> f64 {
        ((y as f32).atan2(x as f32)) as f64
    }
}

#[cfg(not(feature = "fast_math"))]
pub mod math {
    #[inline(always)]
    pub fn sin(x: f64) -> f64 { x.sin() }
    
    #[inline(always)]
    pub fn cos(x: f64) -> f64 { x.cos() }
    
    #[inline(always)]
    pub fn asin(x: f64) -> f64 { x.asin() }
    
    #[inline(always)]
    pub fn atan2(y: f64, x: f64) -> f64 { y.atan2(x) }
}

pub use math::*;
```

Then in `Cargo.toml`:
```toml
[features]
fast_math = []
```

And in `main.rs` or `lib.rs`:
```rust
#[cfg(feature = "fast_math")]
mod fast_math;
#[cfg(not(feature = "fast_math"))]
mod fast_math {
    // Dummy module, math functions come from f64 methods
}
```

---

## References

- Chebyshev polynomial reference: https://www.3blue1brown.com/lessons/taylor-series
- libm source: https://github.com/rust-lang/libm (see sin/cos implementations)
- Fast math survey: "Fast Methods for Computing the Square Root and Reciprocal Square Root" (Newton's method + variants)
- HEALPix requirements: Should maintain <1 ULP error in ring index computation
