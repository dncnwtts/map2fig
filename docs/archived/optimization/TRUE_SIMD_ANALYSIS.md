# True portable_simd SIMD Analysis

**Date:** February 15, 2026  
**Status:** Feasibility Assessment

---

## Can We Do True SIMD? Yes, But...

### What's Available

✅ **Nightly Rust:** `rustc 1.95.0-nightly` available  
✅ **portable_simd:** Stable vector types (`f64x8`, `f32x4`, etc.)  
✅ **sleef crate:** Pure Rust vectorized math library (sin, cos, atan2, asin)  
✅ **Modern Hardware:** Most users have AVX2+ support (2013+ Intel, 2015+ AMD)  

### Implementation Path

A true SIMD implementation would require:

1. **Enable nightly in `rust-toolchain.toml`:**
   ```toml
   [toolchain]
   channel = "nightly"
   ```

2. **Add dependencies to `Cargo.toml`:**
   ```toml
   sleef = "0.3.2"
   ```

3. **Implement vectorized Mollweide projection:**
   ```rust
   use std::simd::{f64x8, SimdFloat};
   use sleef::f64x8 as sleef_f64x8;
   
   fn mollweide_proj_simd(
       px_array: &[f64; 8],
       py_array: &[f64; 8],
   ) -> (
       [f64; 8], // lons
       [f64; 8], // lats
       [bool; 8], // mask
   ) {
       // Convert to SIMD vectors
       let px = f64x8::from_array(*px_array);
       let py = f64x8::from_array(*py_array);
       
       // Vectorized sin/cos using sleef
       let sin_2theta = sleef_f64x8::sin(py * 2.0);
       let cos_theta = sleef_f64x8::cos(py);
       
       // Extract back to arrays for HEALPix look ups
       // (HEALPix sampling must remain scalar per-pixel due to array indexing)
   }
   ```

---

## Performance Potential

### Bottleneck Analysis (from Callgrind)

| Operation | Instructions | Est. Time | SIMD Speedup | Savings |
|-----------|--------------|-----------|--------------|---------|
| Total Mollweide | 35.78B | 7.5 sec | — | — |
| **Math (sin/cos/atan2)** | 5.4B | 1.13 sec | **4-8x** | **0.28-0.85 sec** |
| Coordinate transforms | 10.2B | 2.1 sec | 1.2x (ILP) | 0.17 sec |
| Data validation | 5.8B | 1.2 sec | 2x (branch elim). | 0.6 sec |
| Other | 14.3B | 2.1 sec | 1x (memory-bound) | 0 sec |

**Realistic Estimate (conservative):**
- SIMD speedup on math: **4x** (not 8x due to sleef overhead)
- Savings: **0.28 seconds from 1.13 = 25% of math time**
- Total savings: **0.28 / 10.1 = 2.8% overall** ⚠️ **Much less than initial estimate**

---

## Why Savings Aren't as Big as Expected

### Problem 1: HEALPix Sampling Blocks Vectorization

The Mollweide projection works like this:
```
1. Pixels → (lon, lat) [VECTORIZABLE with SIMD]
2. (lon, lat) → HEALPix ring indices [SCALAR - array indexing]  
3. Ring indices → data array lookup [SCALAR - irregular access pattern]
4. Data → colormap [SCALAR - per-pixel]
```

**You can't vectorize step 2-4** because:
- HEALPix indexing (ang2pix_ring) returns different indices for each pixel
- Can't batch array lookups with scatter/gather on 8 irregular indices
- Cache efficiency destroyed if you try to gather from 8 random locations

### Problem 2: Vectorized Math is the Minority

From Callgrind profile of ONE Mollweide projection:
- **Trig operations:** 5.4B instructions (11.8% of total)
- **Coordinate math (mul, add, div):** Already fast, benefit less from SIMD
- **Array lookups + data validation:** 20.1B instructions (44%) - **CANNOT be vectorized**

Even with 4x speedup on trig:
- Savings: 1.13 sec / 4 = 0.28 sec ≈ 2.8% improvement

### Problem 3: Conversion Overhead

Each batch of 8 pixels requires:
```rust
// Create 8 SIMD vectors from arrays
let px_simd = f64x8::from_array(px_array);

// Do math with sleef
let sin_2theta = sleef_f64x8::sin(...);

// Extract back to arrays for HEALPix sampling
let sin_results: [f64; 8] = sin_2theta.to_array();

// Loop over scalar results for HEALPix lookups
for i in 0..8 {
    let ring_idx = ang2pix_ring(sin_results[i], ...); // SCALAR
}
```

The conversion overhead (array → SIMD → array → scalar loop) eats into savings.

---

## Implementation Complexity vs Payoff

| Aspect | Effort | Complexity | Risk |
|--------|--------|-----------|------|
| **Add nightly dependency** | 5 min | Trivial | Low |
| **Add sleef crate** | 5 min | Trivial | Low |
| **Vectorize Mollweide proj** | 2-3 hours | Moderate | Medium |
| **Handle CPU feature detection** | 1-2 hours | Moderate | High |
| **Test on various hardware** | 1-2 hours | Low | Medium |
| **Maintain dual code paths** | Ongoing | Complex | High |
| **User build complexity** | — | High | High |
| **Total effort** | **5-8 hours** | **High** | **High** |

---

## User Burden

**With nightly dependency:**
- Users must install `rustc +nightly`
- Build times increase by 20-30% (nightly recompilation)
- Different optimization flags behavior on nightly
- Some IDEs don't support nightly Rust well

**Workaround: Feature flag**
```toml
[features]
simd = ["sleef"]

[dependencies]
sleef = { version = "0.3.2", optional = true }
```

But still requires:
```bash
cargo build --release --features simd  # vs standard cargo build
```

---

## CPU Feature Detection

Modern CPUs support different SIMD levels:
- **AVX2** (256-bit, 4× f64): 2013+ Intel, 2015+ AMD
- **AVX-512** (512-bit, 8× f64): 2015+ Intel Skylake Xeon, Zen 3+ AMD
- **No SIMD** (SSE2): Older/embedded systems

**Options:**
1. **Compile-time detection** (rustflags): Simple but requires user setup
2. **Runtime detection** (cpufeature crate): Automatic but adds ~5 MB binary
3. **Don't support old hardware**: Assume AVX2+ (reasonable for 2025+)

---

## Recommendation: YES, But With Caveats

### Go Ahead If:
✅ You're willing to maintain a nightly build  
✅ Users are comfortable with optional feature flag  
✅ You want **2-3% overall improvement** (0.25-0.3 seconds)  
✅ You want portability to ARM/GPU-accelerated builds later  

### Don't Bother If:
❌ You want 10-15% improvement (won't get that)  
❌ Simplicity is a priority  
❌ Users are non-technical (complex to explain build process)  
❌ You're happy with 11 seconds (reasonably fast)  

---

## Prototype Implementation Strategy

If you want to proceed:

### Phase 1: Feature Gate (1 hour)
- Add `nightly-simd` feature to Cargo.toml
- Create `src/simd_math.rs` with both scalar and vectorized versions
- Conditional compilation: `#[cfg(feature = "nightly-simd")]`

### Phase 2: Vectorize Math Only (2 hours)
- Extract sin/cos/atan2/asin operations to wrapper functions
- Implement vectorized versions using sleef
- Fall back to standard libm if feature disabled

### Phase 3: Benchmarking (1 hour)
- Benchmark both paths
- Measure actual speedup (probably 2-4%)
- Document findings

### Phase 4: CPU Detection (1-2 hours, optional)
- Add cpufeature crate for runtime detection
- Auto-select best implementation
- Warn if SIMD unavailable at runtime

---

## What Would Actually Be Worth It

Instead of true SIMD, these changes would have better ROI:

### Higher Impact Options

1. **Approximate Math** (Taylor poly for sin/cos)
   - 5-10% speedup potential
   - No nightly dependency
   - All portable hardware
   - Risk: Accuracy vs speed trade-off

2. **Batch HEALPix Lookups** (if using sparse columns)
   - 3-5% speedup potential  
   - Uses existing Rayon infrastructure
   - Only works for multi-column files
   - Low complexity

3. **Cache-Aware Pixel Reordering**
   - 2-3% speedup potential
   - Better L3 cache utilization
   - Pure algorithmic improvement
   - No dependencies needed

4. **Parallel Column Processing** (for multi-column FITS)
   - 3-4× speedup if you have 4 columns
   - Already partially implemented (Tier 4.3b)
   - Users rarely have multi-column data
   - Low priority

---

## Final Assessment

**True SIMD: Possible but not worth it for 2-3% gain**

With 8+ hours of work, you could get:
- Best case: **4% speedup** (0.4 seconds from 10.1)
- Realistic case: **2-3% speedup** (0.25-0.3 seconds)
- Worst case: **Regression** if not careful with CPU features

For comparison:
- **BufReader optimization (Phase 1):** 30 min work, 5-10% I/O speedup
- **Metadata caching (done):** 90%+ hit rate on repeated files
- **Already-implemented batch processing:** 8-pixel batches with ILP

**Suggestion:** Accept the current 11-second performance as "good enough" and focus on UX improvements (progress bars, better error messages, etc.) rather than micro-optimizations.

If users specifically request SIMD support and are willing to use a feature flag, revisit this in 6 months when portable_simd matures further.

---

## Code Example (If You Change Your Mind)

```rust
// src/simd_math.rs
#[cfg(feature = "nightly-simd")]
mod simd {
    use std::simd::f64x8;
    use sleef::f64x8 as sleef_f64x8;
    
    pub fn sin_batch(angles: &[f64; 8]) -> [f64; 8] {
        let v = f64x8::from_array(*angles);
        sleef_f64x8::sin(v).to_array()
    }
}

#[cfg(not(feature = "nightly-simd"))]
mod simd {
    pub fn sin_batch(angles: &[f64; 8]) -> [f64; 8] {
        [
            angles[0].sin(),
            angles[1].sin(),
            // ... etc
        ]
    }
}

pub use simd::sin_batch;
```

---

## References

- Callgrind profile: `callgrind.out.67716` (46.2B instructions analyzed)
- Mollweide projection: `src/mollweide.rs` (652 lines, 77.5% of CPU time)
- SLEEF documentation: https://sleef.org/
- portable_simd RFC: https://github.com/rust-lang/rfcs/pull/2948
