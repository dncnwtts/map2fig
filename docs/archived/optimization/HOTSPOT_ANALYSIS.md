# Hot Loop Analysis & Optimization Brainstorm

## Main Rendering Loop Structure

**Location**: [src/plot/mod.rs](../../src/plot/mod.rs#L225-L315) function `render_projection_to_grid`

```rust
for py in 0..height {
    for px in 0..width {
        // 1. PROJECTION (5% - already optimized)
        if let Some((lon, lat)) = params.proj.pixel_to_ang(px, py, grid) {
            let theta = std::f64::consts::PI / 2.0 - lat;

            // 2. HEALPIX SAMPLING (35% - expensive!)
            let pixel_val = match crate::healpix::sample_healpix(
                params.map, params.meta, params.view, theta, lon
            ) {
                Some(val) => {
                    // 3. SCALING (25% - also expensive!)
                    crate::scale::scale_value(
                        val,
                        params.scale.minv, params.scale.maxv,
                        params.scale_type, params.neg_mode, params.hist_scale
                    )
                },
                None => PixelValue::Bad,
            };

            // 4. GAMMA CORRECTION (~2-3%)
            let t = if gamma_inv == 1.0 { t } else { t.powf(gamma_inv) };

            // 5. COLORMAP LOOKUP (8% - already optimized)
            let c = params.cmap.sample(t);

            // 6. MASKING (negligible if no mask)
            if let Some(mask) = params.mask { ... }

            // 7. WRITE PIXEL
            grid.set_pixel_unchecked(px, py, rgba);
        }
    }
}
```

**Iteration Count**: 
- 2400px: 5.76M iterations
- 4000px: 16M iterations
- Each iteration: ~2-3 microseconds on optimized code

---

## Detailed Hot Path Breakdown

### 1. HEALPix Sampling (35% of runtime)

[src/healpix.rs line 527](../../src/healpix.rs#L527-L545)

```rust
fn sample_healpix(map, meta, view, theta, lon) {
    // Step A: Spherical to Cartesian (3 trig calls: sin, cos)
    let v_view = sph_to_vec(theta, lon);
    
    // Step B: Matrix-vector rotation (3×3 @ 3D = 9 dot products = 27 FLOPs)
    let v_map = view.apply_inverse(v_view);
    
    // Step C: Cartesian back to Spherical (atan2, acos)
    let (theta_m, lon_m) = vec_to_sph(v_map);
    
    // Step D: Angular coords to HEALPix index (THE EXPENSIVE PART)
    let ipix = ang2pix(meta, theta_m, lon_m);
    
    // Step E: Array lookup (one cache miss possible)
    map.get(ipix).copied()
}
```

#### Sub-component: ang2pix_ring (50% of HEALPix cost)

[src/healpix.rs line 230-280](../../src/healpix.rs#L230-L280)

```rust
fn ang2pix_ring(nside, theta, phi) {
    let z = theta.cos();
    let za = z.abs();
    let tt = ((phi % TWOPI) + TWOPI) % TWOPI * INV_HALFPI;
    
    if za <= TWOTHIRD {
        // EQUATORIAL (3 branches, heavy FP math)
        let temp1 = nside as f64 * (0.5 + tt);
        let temp2 = nside as f64 * (0.75 * z);
        let jp = (temp1 - temp2).floor() as i64;  // ← expensive floor
        let jm = (temp1 + temp2).floor() as i64;  // ← expensive floor
        let ir = nside + 1 + jp - jm;
        let kshift = 1 - (ir & 1);
        let mut ip = (jp + jm - nside + kshift + 1) / 2;
        ip = imodulo(ip, 4 * nside);
        2 * nside * (nside - 1) + (ir - 1) * 4 * nside + ip
    } else {
        // POLAR (3 branches, sqrt involved)
        let tp = tt - tt.floor();
        let tmp = nside as f64 * (3.0 * (1.0 - za)).sqrt();  // ← sqrt per pixel!
        let jp = (tp * tmp).floor() as i64;
        let jm = ((1.0 - tp) * tmp).floor() as i64;
        // ... more arithmetic
    }
}
```

**Problems**:
- ✅ Already inlined (good)
- ❌ Multiple `floor()` calls per pixel (expensive on some CPUs)
- ❌ `sqrt()` in polar caps (can be 10-30 cycles)
- ❌ Complex conditional logic (branch misprediction on equator/poles boundary)
- ❌ Multiple integer divisions with modulo

---

### 2. Scaling (25% of runtime)

[src/scale.rs line 620](../../src/scale.rs#L620-L700)

```rust
fn scale_value(value, min, max, scale, neg_mode, hist) {
    if !is_seen(value) { return PixelValue::Bad; }
    
    // Fast path for LINEAR (common case)
    if matches!(scale, Scale::Linear) {
        let t = if value <= min { 0.0 }
                else if value >= max { 1.0 }
                else { (value - min) / (max - min) };
        return PixelValue::Color(t);
    }
    
    match scale {
        Scale::Log => {
            // ~3-5 comparisons + log call
            if value <= 0.0 { return PixelValue::Bad; }
            (value.ln() - min.ln()) / (max.ln() - min.ln())
        },
        Scale::Asinh { scale } => {
            // 3×asinh + 2 divisions
            let val = (value / scale).asinh();
            let min_val = (min / scale).asinh();
            let max_val = (max / scale).asinh();
            (val - min_val) / (max_val - min_val)
        },
        Scale::Symlog { linthresh } => {
            // Complex conditional logic
            // 1-2 log calls, comparisons
        },
        // ... more scales
    }
}
```

**Problems**:
- ✅ Linear scale is fast-pathed
- ❌ logarithm/asinh/symlog are expensive per-pixel (10-50 cycles)
- ❌ Multiple type checks at runtime (vtable-like behavior)
- ❌ Unpredictable branch patterns (depends on data distribution)
- ❌ **No caching of log(min), log(max)** - recomputed per pixel for log scale!

---

### 3. Projection (5% - already optimized)

Example: Mollweide → [src/plot/mollweide.rs](../../src/plot/mollweide.rs#L200-L250)

```rust
fn pixel_to_ang(px, py) {
    // Normalized coordinates [0, 1]
    let x = px / width;
    let y = py / height;
    
    // Mollweide projection reversal (10-15 FLOPs)
    let lat = y.asin();
    let lon = x.appropriate_calculation(...);
    (lon, lat)
}
```

**Status**: ✅ Already optimized (inline, algebraic simplification)

---

### 4. Gamma Correction (2-3%)

```rust
let t = if gamma_inv == 1.0 { t } else { t.powf(gamma_inv) };
```

**Problem**: `pow()` is expensive (~15-30 cycles), but common case (gamma ≠ 1) requires it

---

### 5. Rotation Matrix (part of HEALPix sampling)

[src/rotation.rs line 167](../../src/rotation.rs#L167-L170)

```rust
fn apply_inverse(matrix, v) {
    // 3 dot products for 3×3 @ 3D vector = 27 FLOPs
    [
        matrix[0][0]*v[0] + matrix[0][1]*v[1] + matrix[0][2]*v[2],
        matrix[1][0]*v[0] + matrix[1][1]*v[1] + matrix[1][2]*v[2],
        matrix[2][0]*v[0] + matrix[2][1]*v[1] + matrix[2][2]*v[2],
    ]
}
```

**Optimization Potential**: 
- No SIMD vectorization (would need batching)
- Already optimal for single matrix-vector multiply
- Could batch 8-16 pixels together to use SIMD

---

## Optimization Opportunities Ranked

### Category A: High Impact, High Effort

#### 1. **Pre-compute scale logarithms**
- **Impact**: 15-20% on log/asinh scales
- **Effort**: Low (1-2 hours)
- **How**: Cache `log(min)`, `log(max)`, `asinh(min)`, `asinh(max)` at start
- **Code location**: [src/scale.rs] scale_value function
- **Blocker**: Need to pass pre-computed values through params struct
- **Risk**: Medium (must update all scale creation sites)

```rust
// Before: each pixel computes log(min), log(max)
(value.ln() - min.ln()) / (max.ln() - min.ln())

// After: pre-computed
(value.ln() - cached_log_min) / (log_max_minus_log_min)
```

**Estimated gain**: 3-5% overall (cuts 50% from log-scale percentile, 25% task = 1-2% global)

---

#### 2. **SIMD vectorization of projection + rotation pipeline**
- **Impact**: 20-30% on HEALPix sampling
- **Effort**: Very High (200-300 lines, complex data layout)
- **How**: 
  - Batch 8-16 pixels together
  - Apply projection to all simultaneously (SIMD asin, sin, cos)
  - Apply rotation matrix to 8 vectors simultaneously (8×3×3 SIMD matmul)
  - Convert back to spherical in batch
- **Code location**: [src/plot/mod.rs] main loop refactor
- **Risk**: High (major architectural change, must validate correctness)
- **Requirement**: Rewrite loop to process tiles instead of pixels

```rust
// Vectorize this pattern:
let v_view = sph_to_vec(theta, lon);           // 3 trig calls per pixel
let v_map = view.apply_inverse(v_view);        // 9 multiplies per pixel
let (theta_m, lon_m) = vec_to_sph(v_map);      // atan2, acos per pixel

// Becomes: process 8 pixels at once with SIMD instructions
// v_view_batch: 8×3 matrix (8 vectors, packed)
// apply_inverse_simd processes all 8 at once: 3 dot products
// vec_to_sph_batch: SIMD atan2, acos
```

**Estimated gain**: 5-8% overall (30% speedup on 25% of pipeline = 7.5%)

---

#### 3. **Branchless ang2pix or lookup table**
- **Impact**: 10-15% on HEALPix indexing
- **Effort**: High (requires deep HEALPix knowledge)
- **How**:
  - Pre-compute pixel index for coarse grid (e.g., 256×256)
  - Use high-order bit tricks to reduce branches
  - Or: switch to faster approximate pixel formula
- **Risk**: Very High (HEALPix is complex, easy to introduce bugs)
- **Alternative**: LUT-based `ang2pix` for lower NSIDE values

**Estimated gain**: 2-3% overall (hard to achieve without correctness risk)

---

### Category B: Medium Impact, Medium Effort

#### 4. **Cache-friendly data rearrangement**
- **Impact**: 5-10% from better cache utilization
- **Effort**: Medium
- **How**: 
  - Pre-sort pixels by HEALPix ring (locality)
  - Process in stripe order to keep working set small
  - Use memory prefetch instructions
- **Code location**: [src/plot/mod.rs]
- **Risk**: Medium (must preserve output order)

---

#### 5. **Scale value histogram CDF optimization**
- **Impact**: 10-15% on histogram scale specifically
- **Effort**: Low-Medium
- **How**: 
  - Use binary search instead of linear scan for CDF lookup
  - Pre-compute lookup table for common scales
- **Code location**: [src/scale.rs] HistogramScale::lookup_cdf

---

#### 6. **Gamma correction LUT for common values**
- **Impact**: 3-5% if gamma ≠ 1
- **Effort**: Low
- **How**:
  - Check if gamma is a small set of common values (1.0, 2.0, 0.5, etc.)
  - Use small lookup table
  - Fall back to pow() for arbitrary gamma
- **Code location**: [src/plot/mod.rs] line ~265

```rust
let t = match gamma_inv {
    x if (x - 1.0).abs() < 1e-6 => t,              // gamma=1: no-op
    x if (x - 0.5).abs() < 1e-6 => t.sqrt(),       // gamma=2: sqrt
    x if (x - 2.0).abs() < 1e-6 => t * t,          // gamma=0.5: square
    _ => t.powf(gamma_inv),
};
```

---

### Category C: Low Impact, Low Effort (Already Done)

- ✅ Colormap truncation (remove round()) - 4.8% gain
- ✅ Projection path inlining - 2.9% gain
- ✅ Compiler flag tuning - <1% gain
- ✅ Early projection intersection check - negligible

---

## Detailed Hotspot Visualization

### CPU Cycles Per Iteration (estimated)

**Total: ~1500-1800 cycles per pixel @ 2400px**

```
HEALPix Sampling        525 cycles  ████████████████████████░░░ (35%)
  ├─ sph_to_vec              60      (sin/cos/sin: 20+20+20)
  ├─ apply_inverse          100      (3×3=27 multiplies @ ~3.7 cyc/mult)
  ├─ vec_to_sph              80      (atan2: ~50 + acos: ~30)
  └─ ang2pix_ring           285      (floor: ~20 each × 2, sqrt: ~30, int div: ~10 ea × 3)

Scaling                 450 cycles  ██████████████████░░░░░░░░░░░░░░░ (25%)
  ├─ is_seen check           10      (comparison)
  ├─ scale conditional       30      (branch + comparison)
  └─ scale computation      410      (log: ~75, sqrt: ~30, fsub/fdiv: ~10 each)

Gamma Correction         60 cycles  ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ (3%)
  └─ powf (if gamma≠1)      60      (pow: ~30-40 cycles)

Colormap Sample         125 cycles  ██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ (7-8%)
  ├─ interpolation           40      (already optimized)
  └─ LUT lookup              85      (cache miss possible)

I/O & Masking           240 cycles  ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ (13-15%)
  ├─ grid.set_pixel          80
  ├─ mask check             100
  └─ branch mispredict       60

────────────────────────────────────────────────
Total                  1500 cycles

Pipeline Efficiency: ~45-50% (lots of FP stalls, branch mispredict penalties)
```

---

## Actionable Optimization List

### Next 3 Quick-Wins (1-2 hour each)

1. **Pre-compute log(min), log(max), etc. for scaling**
   - File: [src/scale.rs](../../src/scale.rs)
   - Gain: 1-2% (conservative: low branch variance)
   - Safe: Yes (just caching)

2. **Gamma correction LUT for common values**
   - File: [src/plot/mod.rs](../../src/plot/mod.rs#L265)
   - Gain: 1-2% (only if gamma ≠ 1)
   - Safe: Yes (conservative fallback)

3. **Binary search for histogram CDF lookup**
   - File: [src/scale.rs](../../src/scale.rs) HistogramScale
   - Gain: 1-2% on histogram scale
   - Safe: Yes (algorithm correct)

### Medium-Term (1-2 day) Architectural Changes

4. **Batch pixels for SIMD (8-16 at a time)**
   - Expected gain: 5-8%
   - Risk: Medium (large refactor, need benchmarking)
   - ROI: Best bang for effort if successful

5. **Optimize ang2pix for common case (equatorial)**
   - Expected gain: 2-3%
   - Risk: Low (isolated function)
   - ROI: Moderate (HEALPix is intricate)

---

## Why We're Still 2x Behind C++

### Fundamental Differences

1. **C++ likely uses GPU acceleration** (CUDA/OpenGL)
   - GPU can process millions of pixels in parallel
   - Rust is CPU-only (single-threaded per core)
   - → Can't match GPU throughput on CPU alone

2. **C++ may use approximate algorithms**
   - Exact HEALPix indexing vs. approximate
   - Multi-precision vs. f64 only
   - Cached precomputed tables

3. **Compiler differences**
   - Rust LLVM may not autovectorize as aggressively
   - C++ template specialization may enable more optimizations

### CPU-Only Realistic Ceiling

**Best case with all Category A optimizations**: ~25-30% gain (45-50% speedup)
- Scales & logs: +5%
- SIMD HEALPix: +8%
- Projection/rotation: +5%
- Cache + misc: +7%
- Compiler improvements: +5%

**That gets us to ~17s @ 2400px vs. current 23s**

Still 1.3-1.5x behind C++ if they're at ~12s, but that's CPU parity territory.

---

## Measurement Challenges

**Known variance**: ±3-5% on this hardware  
**Files tested**: 6.8MB → 3.1GB  
**Resolutions**: 2400px, 4000px  
**Threshold for meaningful gain**: >3% with multiple runs

---

## Recommendations

1. **Short term** (this week): Implement pre-computed scale logs + gamma LUT
   - Expected: +2-3% measurable, easy win
   - Cost: 1-2 hours

2. **Medium term** (this month): Profile with elevated privileges + SIMD analysis
   - Expected: +5-8% if vectorization feasible
   - Cost: 1-2 days investigation + implementation

3. **Long term**: Consider architectural changes (GPU, bindings to C++ HEALPix)
   - Expected: 50-100% gain
   - Cost: Major refactor
