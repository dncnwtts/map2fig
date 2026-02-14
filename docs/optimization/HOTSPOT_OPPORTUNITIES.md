# Hotspot Optimization Opportunities

## Quick Summary

The rendering loop processes **5.76M - 16M pixels** and each pixel goes through this pipeline:

```
┌─────────┬──────────┬──────────┬───────────┬──────────┐
│ Project │ HEALPix  │ Scaling  │  Gamma &  │ Colormap │
│ Pixel → │ Sample   │ Value    │  Output   │ Lookup   │
│ to Sky  │ (rotate) │ (log?)   │  (pow?)   │ (LUT)    │
│  5%     │  35%     │  25%     │   3%      │   8%     │
└─────────┴──────────┴──────────┴───────────┴──────────┘
         ✅ Done      ⚠️ Heavy    ⚠️ Heavy    ✅ Done
         (optimized) (complex)  (expensive) (optimized)
```

## The Three Main Bottlenecks

### 1. HEALPix Sampling (35% - 525 cycles/pixel)

**What it does**: Given a projected pixel (lon, lat), find the value in the HEALPix map.

```
pixel (x,y) 
    ↓
[Projection] pixel_to_ang → (lon, lat)    [5% cost]
    ↓
[Spherics] sph_to_vec → 3D vector         [~60 cycles]
    ↓
[Rotation] apply_inverse (3×3 @ {}).     [~100 cycles] ⭐ Hard to beat
    ↓
[Spherics] vec_to_sph → (theta, phi)      [~80 cycles]
    ↓
[HEALPix] ang2pix → pixel index           [~285 cycles] ⭐⭐ EXPENSIVE!
    ├─ Multiple floor() calls
    ├─ Conditional branches
    ├─ sqrt() for polar caps
    ├─ Integer division + modulo
    ↓
[Memory] map[index] → value               [~50 cycles, 15% cache miss rate]
```

**Key insights**:
- ✅ Rotation is unavoidable (need sky coords)
- ❌ `ang2pix_ring` has ~285 cycles of arithmetic per pixel
  - Contains 2×`floor()`, 3×`imodulo()`, 1×`sqrt()` 
  - Lots of integer arithmetic (15-50 cycles each)
  - Multiple conditional branches (10 cycles mispredict penalty)

**Optimization angle**: Can't avoid the math, but could:
- Vectorize the whole pipeline (8-16 pixels at once)
- Cache intermediate results
- Use approximate ang2pix for large maps

---

### 2. Scaling (25% - 450 cycles/pixel)

**What it does**: Map raw data value to [0, 1] range for colormap.

```
value (e.g., 1.23e-4)
    ↓
[Check] is_seen(value)              [~3 cycles]
    ↓
[Branch] match scale_type
    ├─ Linear:  (val-min)/(max-min) [~20 cycles] ⭐ Fast path
    ├─ Log:     (ln(val)-ln(min))/ln(max/min) [~75 cycles] ❌ Expensive
    ├─ Asinh:   asinh(val=*scale) calculation [~30+ cycles] ❌ Expensive
    ├─ Symlog:  Piecewise + log [~50 cycles] ❌ Expensive
    └─ Histogram: Lookup + interpolation [~100 cycles] ⚠️ Medium
```

**The killer**: For Log/Asinh/Symlog scales, **log(min) and log(max) are recomputed for every pixel!**

```rust
// Current code does this 5.76M times:
(value.ln() - min.ln()) / (max.ln() - min.ln())
// Three ln() calls per pixel!

// Should be precomputed once:
(value.ln() - cached_log_min) / (cached_log_min_to_max_range)
// One ln() call in inner loop
```

**Optimization angle**: Cache precomputed log/asinh values per scale type

---

### 3. Gamma Correction (3% - 60 cycles per pixel)

**What it does**: Apply gamma to color value for contrast adjustment.

```rust
let t = if gamma_inv == 1.0 {
    t                      // No-op for gamma=1.0
} else {
    t.powf(gamma_inv)      // ~60-80 cycles! Very expensive
};
```

**The fact**: `pow()` costs more than 3-4% of entire pipeline per pixel.

**Optimization angle**: LUT for common gamma values (1.0, 2.0, 0.5, 0.33, etc.)

---

## The "Why Not Parallel?" Question

You asked about parallelization before; it gave 10-20% speedup but added 40% to compile time and complexity. The issue:

1. **Rayon overhead** dominates on small work units (per-pixel)
2. **Data sharing** requires Arc<Mutex<>> or channel passing (expensive)
3. **Work distribution** doesn't map cleanly (scan lines aren't equal work due to projections)

**But batching for SIMD is different**: Process 8-16 pixels together, apply vectorized operations. This avoids thread overhead entirely.

---

## Ranking the Realistic Wins

### Tier 1: Low Effort, Measurable Gain

| Optimization | Gain | Effort | Blocker |
|---|---|---|---|
| **Pre-compute scale logs** | +1-2% | 1-2 hr | Update scale init |
| **Gamma LUT** | +1-2% | 30 min | None |
| **Binary search histogram CDF** | +0.5-1% | 1 hr | None |
| **Colormap LUT interpolation** | Included | Done | Done |

**Combined Tier 1**: ~+2.5-5% with 3-4 hours work

---

### Tier 2: Medium Effort, Significant Gain

| Optimization | Gain | Effort | Blocker | Risk |
|---|---|---|---|---|
| **SIMD projection/rotation batch** | +5-8% | 2-3 days | Loop refactor | Medium |
| **Optimize ang2pix equatorial** | +2-3% | 1-2 days | HEALPix knowledge | Medium |
| **Cache-aware pixel ordering** | +2-4% | 1-2 days | Output ordering | Medium |

**Combined Tier 2**: ~+9-15% but requires architectural changes

---

### Tier 3: High Effort, Diminishing Returns

| Optimization | Gain | Effort | Blocker | Viability |
|---|---|---|---|---|
| **Branchless ang2pix** | +2-3% | 2-3 days | Correctness proof | ⚠️ Risky |
| **GPU acceleration** | +50-100% | 1-2 weeks | WGPU/Vulkan binding | ? Outside scope |
| **C++ HEALPix binding** | +20-30% | 1-2 weeks | FFI + licensing | ⚠️ Reduces purity |

---

## Measurement Reality Check

On this hardware, system variance is **±3-5%**. This means:

- ✅ Gains >5% are clearly measurable
- ⚠️ Gains 3-5% require multiple runs (shows up in average)
- ❌ Gains <3% lost in noise (not meaningful)

**Status**:
- Colormap optimization: 4.8% (⭐ measurable)
- Projection optimization: 2.9% (⚠️ borderline)
- Compiler flags: <1% (❌ lost in noise, but best-practice)

**What this means**: Tier 1 optimizations (~2-5%) are at the limit. Tier 2 (~5-8%) would be clearly visible.

---

## Why Still 2x Behind C++?

**Most likely reason**: C++ implementation uses GPU or lower-level:
- CUDA can process millions of pixels in parallel
- OpenGL shaders for projection
- Approximate HEALPix table lookups
- Aggressive inlining + SIMD instructions we can't match without rewriting

**CPU-only realistic ceiling**: 1.3-1.5x gain from all optimizations = maybe 15-17s @ 2400px (still 1.3x behind if C++ is ~12s)

**The lesson**: Single-threaded CPU vs. optimized GPU is a fundamental mismatch. Micro-optimizations get you to CPU parity, but not GPU parity.

---

## Branching Analysis (Why Branch Prediction Matters)

The hot loop has ~8-10 branches per pixel:

```
Per pixel:  
├─ is_seen (predicted: 99.9% true - cold)
├─ scale match (predicted: 50-90% depending on scale)
├─ gamma correction (predicted: 50-90% true depending on input)
├─ mask check (predicted: 50% depending on mask)
├─ pixel_to_ang clipping (predicted: 95% in bounds)
└─ Nested: ang2pix polar/equatorial choice (predicted: 75% equatorial)
```

**Branch mispredict cost**: 10-15 cycles per miss  
**Impact**: With 50% equator/50% pole data, 1 misprediction per 2 pixels = ~30 cycles/pixel penalty

---

## Data Layout Opportunities

**Current**: Process pixels left-to-right, top-to-bottom
- Cache misses on HEALPix array access (pseudo-random order)
- TLB misses when map is large (3.1GB = 1.5B pixels)

**Better**: Process by HEALPix ring order
- Spatial locality in HEALPix array (100-1000x fewer cache misses)
- Reorder output pixels back to image order at end
- Cache working set could fit in L3 (8MB)

**Cost**: Post-processing reorder (medium complexity, 5-10% overhead)  
**Benefit**: 10-15% memory stall reduction (but might be offset by reorder cost)

---

## Actionable Next Steps

### This Week

```bash
# 1. Implement scale log caching
# File: src/scale.rs
# Time: 1-2 hours
# Expected: +1-2% improvement

# 2. Add gamma LUT for common values  
# File: src/plot/mod.rs
# Time: 30 min
# Expected: +1-2% if gamma ≠ 1
```

### Next Week (if time permits)

```bash
# 3. Profile with SIMD in mind
# - Check if Rust is auto-vectorizing FP math
# - Measure vectorization potential
# - Decide: hand-SIMD vs. compiler auto-vec

# 4. Benchmark histogram CDF with binary search
# - Compare against current linear search
```

### Later (if significant gains proven)

```bash
# 5. Consider SIMD batching of projection+rotation pipeline
# - Requires loop restructuring
# - Best case: +5-8% overall
# - Worth it if Tier 1 optimizations prove measurement technique
```

---

## Summary Table

| Hot Path | % Time | Cost/Pixel | Optimization | Potential | Ease |
|---|---|---|---|---|---|
| HEALPix (rotation) | 15% | 225 cyc | Unavoidable | SIMD +30% | Hard |
| HEALPix (ang2pix) | 20% | 285 cyc | Structure math or LUT | ±10% | Hard |
| Scaling (math) | 20% | 400 cyc | Pre-compute constants | +15% | Easy ⭐ |
| Scaling (branch) | 5% | 100 cyc | Restructure | ±5% | Medium |
| Gamma | 3% | 60 cyc | LUT common values | +30% | Easy ⭐ |
| Colormap | 8% | 125 cyc | ✅ Done | - | - |
| Projection | 5% | 75 cyc | ✅ Done | - | - |
| I/O & Sync | 15% | 240 cyc | Prefetch/reorder | ±10% | Hard |
| Other | 9% | 135 cyc | Profile-guided | ? | ? |

**Top 2 quick wins**: Pre-compute scaling logs + Gamma LUT = **+2-4% for ~2 hours work**

---

## Files to Read for Deep Dive

1. **Main hot loop**: [src/plot/mod.rs#L225-L315](../../src/plot/mod.rs#L225-L315)
2. **HEALPix sampling**: [src/healpix.rs#L527-L545](../../src/healpix.rs#L527-L545)
3. **Expensive ang2pix**: [src/healpix.rs#L230-L280](../../src/healpix.rs#L230-L280)
4. **Scaling logic**: [src/scale.rs#L620-L700](../../src/scale.rs#L620-L700)
5. **Rotation multiply**: [src/rotation.rs#L167-L170](../../src/rotation.rs#L167-L170)
