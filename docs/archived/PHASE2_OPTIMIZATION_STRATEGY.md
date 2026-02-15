# Phase 2: SIMD & Image Rendering Optimizations (Post-Cairo Batching)

## Current Status (v0.3.0 with Cairo Batching)

**Timing baseline after Cairo batching optimization**:
- PDF: 470 ms (was 617 ms, -23.8% improvement) ✓
- PNG: 170 ms (largely unchanged, expected)
- **Achieved**: Exceeded 10-15% target with 23.8%

## Theoretical Limits (Roofline Analysis)

From previous analysis:
- **Scalar minimum** (I/O-bound): ~130 ms
- **With full SIMD + parallelization**: ~100-110 ms
- **Current gap**: 470 - 130 = 340 ms potential savings
- **Remaining CPU-bound work**: ~300-350 ms

## Current Code Architecture (After Review)

The rendering pipeline already has:

✅ **Tier 5: 16-pixel batch processing** 
- Processes pixels in groups of 16 (2 × 8-pixel batches)
- Vectorization-friendly loop structure
- Found in `src/plot/mod.rs` lines 250-330+

✅ **Batch projection operations**
- `pixel_to_ang_batch()` processes 8 pixels at once
- Loop-unrolled for Instruction-Level Parallelism (ILP)
- Generic across Mollweide, Hammer, Gnomonic projections

✅ **Batch scaling operations**
- `simd_batch_scale_16()` processes 16 pixels
- Handles linear, log, histogram, asinh, symlog
- Validates pixel masks for unseen/invalid values

✅ **HEALPix sampling batch**
- `sample_healpix_batch_simd()` for 8 pixels
- Coordinate rotation & pixel indexing in batch form

✅ **SIMD wrapper library** (`src/simd.rs`)
- 60+ SIMD-friendly functions prefixed `simd_`
- Currently scalar implementations (fallback mode)
- Structured for easy conversion to packed_simd/portable_simd

## The Gap: Scalar Implementation

**Current bottleneck**: All `simd_*` functions are still scalar!

Example from `src/simd.rs`:
```rust
pub fn simd_sin_8(angles: [f64; 8]) -> [f64; 8] {
    [
        angles[0].sin(),  // ← Scalar sin, not vectorized
        angles[1].sin(),
        angles[2].sin(),
        // ... x 8 total
    ]
}
```

This processes 8 scalar sin() calls in sequence, missing 8× parallelism potential.

## Identified Optimization Opportunities

### **Phase 2A: Vectorize Core Math Operations**

**Target**: Replace scalar array operations with actual SIMD

**Functions to accelerate** (by time impact):
1. **`simd_sin_8 / simd_cos_8`** - Mollweide projection math
   - Mollweide inverse requires: sin(θ), cos(θ), asin
   - Called once per 8 pixels in batch
   - Current: 8 sequential scalar sin(), 8 sequential cos()
   - With SIMD: 1 vectorized sin_cos operation for 8 values simultaneously
   - **Potential**: 4-8× speedup on trigonometric math

2. **`simd_abs_8`** - Projection validity checks  
   - Used in abs() comparisons for projection boundaries
   - Highly vectorizable
   - **Potential**: 8× speedup

3. **`simd_madd_8`** - Matrix math for rotations + linear operations
   - Used in scale transformations, coordinate conversions
   - Multiply-add is vectorization sweet spot
   - **Potential**: 8× speedup

**Implementation approach**:
- Use `portable_simd` (RFC, now in nightly Rust)
  - OR `packed_simd` (stable alternative)
  - OR `std::simd` (when stabilized)
- Keep scalar fallback for portability
- Feature gate: `#[cfg(target_arch = "x86_64")]`

**Expected improvement**: 
- Mollweide projection: 50-75 ms → 25-40 ms (40-50% speedup)
- Overall PDF: 470 ms → 395-420 ms (additional 10-15%)

---

### **Phase 2B: Image Surface Pre-rendering**

**Target**: Eliminate remaining per-pixel rendering overhead

**Current pipeline** (with Cairo batching):
```
Project (scalar) → Scale (scalar) → Colormap (per-pixel) → Cairo rasterize → PDF output
                                                           ↑
                                                    ~256 calls now, down from 51k
```

**New pipeline** (image pre-rendering):
```
Project → Scale → Colormap → ImageBuffer.write (fast mem) → 1× Cairo paint() → PDF output
                              ↑
                         No per-pixel Cairo calls at all!
```

**How it works**:
```rust
// Instead of: for each pixel { output_sink.draw_pixel(x, y, rgba) }
// Do this:
let mut img_buffer = RgbaImage::new(width, height);
for py in 0..height {
    for px in 0..width {
        let (x, y, rgba) = /* projection + scaling + colormap */;
        img_buffer.put_pixel(x, y, rgba);  // ← Fast memory write
    }
}

// Then render to PDF as single operation:
let surface = ImageSurface::from_buffer(img_buffer);
cr.set_source_surface(&surface, 0, 0);
cr.paint();  // ← Single Cairo call!
```

**Trade-offs**:
- ✅ Eliminates resetting source color 256 times
- ✅ Single matrix transform for entire image
- ✅ Eliminates Cairo path building overhead
- ⚠️ Requires maintaining image buffer (~4MB for 1200×741 @ RGBA)
- ⚠️ Memory bandwidth usage (but likely prefetched)

**Expected improvement**:
- Saves ~50-75 ms in Cairo overhead (remaining after batching)
- PDF: 420 ms → 350-380 ms (additional 8-12%)

---

### **Phase 2C: Parallel Processing (Rayon)**

**Target**: Use multi-core processing for pixel batches

**Current**: Single-threaded, 16-pixel batch processing  
**Potential**: 8-core parallel, 128-pixel batch (8 × 16)

**Implementation**:
```rust
use rayon::prelude::*;

let row_chunks: Vec<_> = (0..height)
    .into_par_iter()
    .chunks(8)  // Process 8 rows in parallel (1 per core)
    .map(|rows| {
        let mut output = vec![];
        for py in rows {
            // 16-pixel batch processing (existing code)
            for px in (0..width).step_by(16) {
                /* existing batch logic */
            }
        }
        output
    })
    .collect();
```

**Challenges**:
- Need thread-safe colormap access
- Need to manage mutex/atomics for output sink
- Cairo contexts are single-threaded (likely bottleneck)

**Expected improvement**:
- Modest (4-6× at best due to Cairo being single-threaded)
- Worth pursuing only after Phase 2A vectorization

---

## Recommended Path: Phase 2A → Phase 2B

### **Why optimize in this order**:

1. **Phase 2A (SIMD vectorization) first**:
   - Directly targets mathematically-intensive operations
   - No architectural changes needed
   - Can be implemented incrementally (one simd_* function at a time)
   - Easier to test and verify
   - Conservative approach (fallback to scalar if needed)

2. **Then Phase 2B (image pre-rendering)**:
   - Addresses remaining Cairo overhead
   - Complements SIMD work
   - Single architectural change upfront
   - Low risk if Phase 2A already reduced compute

3. **Skip Phase 2C for now**:
   - Cairo is single-threaded (fundamental limit)
   - Multi-threading won't help much for PDF output
   - Could revisit for PNG or data-parallel analysis

---

## Implementation Plan: Phase 2A (SIMD Vectorization)

### Step 1: Set up infrastructure (30 min)
- Add `portable_simd` dependency with feature gate
- Create `src/simd_avx2.rs` for x86_64 implementations
- Keep `src/simd.rs` as dispatcher (scalar fallback)

### Step 2: Vectorize projection math (2-3 hours)
- Implement `simd_sin_cos_8_vectorized()` for Mollweide
- Test: Compare output with scalar version (should be bit-identical within FP tolerance)
- Benchmark: Measure speedup on Mollweide inverse

### Step 3: Vectorize scaling operations (1-2 hours)
- Implement vectorized log, exp, sqrt for scaling functions
- Handle NaN/invalid value masking

### Step 4: Benchmark full pipeline (1 hour)
- Profile with flamegraph after changes
- Expected: 470 ms → 390-420 ms for PDF
- Measure: Cache effects, instruction parallelism

### Step 5: If successful, plan Phase 2B

---

## Success Criteria for Phase 2

**Phase 2A goals**:
- ✅ Vectorize trigonometric operations (sin/cos/asin/atan2)
- ✅ Vectorize algebraic operations (sqrt, mul, add, madd)
- ✅ Achieve 10-15% additional speedup (470 ms → 400-423 ms)
- ✅ Maintain bit-identical output

**Phase 2B goals** (if Phase 2A alone insufficient):
- ✅ Implement image pre-rendering path
- ✅ Achieve additional 8-12% (total 18-25% from current)
- ✅ Output pixel-identical to original

**Overall v0.3→v0.4 target**: 
- 30%+ speedup from baseline (v0.2.0: 617 ms → <430 ms)
- Incremental improvements measured at each phase

---

## Risk Assessment

| Phase | Risk | Mitigation |
|-------|------|-----------|
| 2A (SIMD) | Output differences in FP rounding | Use exact comparisons, pixel-level diff tool |
| 2A | Platform-specific failures | Fallback to scalar, feature-gate |
| 2A | Cache misses from vectorized code | Profile with cachegrind |
| 2B (Image) | Memory bandwidth bottleneck | Measure before/after carefully |
| 2B | Cairo limitations with image surface | Prototype small example first |

---

## Alternatives Considered

### **Alternative 1: Pure Rust PDF library (printpdf)**
- ✅ Eliminate Cairo entirely
- ✅ Could achieve 40%+ speedup
- ❌ Complex implementation (several weeks)
- ❌ May lose LaTeX rendering capability
- ❌ Uncertain output quality
- **Decision**: Defer to Phase 3 if Phase 2 insufficient

### **Alternative 2: GPU rendering**
- ✅ Could achieve 10-50× speedup
- ❌ Requires CUDA/OpenGL setup
- ❌ Overkill for CLI tool
- ❌ Portability challenges
- **Decision**: Out of scope for this optimization

### **Alternative 3: Recompile with LTO / PGO**
- ✅ Easy to try (5 min)
- ✓ Could yield 5-10% free speedup
- ❌ Longer compile times
- **Decision**: Quick win to try before Phase 2A

---

## Next Steps

1. **Profile current v0.3.0 with flamegraph** to identify exact hot spots
2. **Decide: Portable SIMD or x86_64-specific intrinsics?**
3. **Start Phase 2A with step 1: Infrastructure setup**
4. **Iterate on sin/cos vectorization** with benchmarking

