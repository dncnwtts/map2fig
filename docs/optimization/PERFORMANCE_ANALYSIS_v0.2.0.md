# Performance Analysis - v0.2.0 Baseline

## Flamegraph Summary

Generated: February 15, 2026  
Test: `class_dr1_40GHz_skymap_n128.fits` (Nside=128, ~51k pixels)  
Total samples: 1.6B (632 profiling events)

## Top CPU Consumers

| Function/Component | Samples | % | Notes |
|---|---|---|---|
| [map2fig] (Rust) | 46.5M | 2.90% | Our code - needs symbol refinement |
| cairo_fill | 33.6M | 2.10% | PDF polygon rendering |
| [libcairo.so] generic | 30.9M | 1.94% | Cairo internals |
| [libcairo.so] generic | 25.8M | 1.61% | More Cairo internals |
| cairo_rectangle | 10.3M | 0.65% | PDF rectangle drawing |
| [libcairo.so] pixman | 2.6M | 0.16% | Pixel manipulation library |

## Key Observations

### 1. Cairo/PDF Dominates Output
- **Total Cairo overhead**: ~11% of samples (cairo_fill, rectangles, surface ops)
- This is expected for PDF generation but suggests optimization potential
- Cairo is being called repeatedly for each pixel/polygon

### 2. Rust Code Representation (2.90%)
- The `[map2fig]` entry at 2.90% includes all our Rust code
- Without debug symbols in release build, we can't see function-level breakdown
- Likely includes:
  - Projection calculations (HEALPix → Mollweide)
  - Data scaling operations
  - Pixel iteration/rasterization
  - Color mapping

### 3. No Major Allocation Overhead
- `free` / `malloc` / `memcpy`: all < 0.2% each
- Memory management is not a bottleneck
- Good: our buffer management is efficient

### 4. Math Operations
- `__acos_finite`: 0.16%
- `__sincos_fma`: 0.16%
- Trigonometric functions from projections are minimal overhead

## Optimization Opportunities (Ranked by Impact)

### Tier 1: High Impact (Potential 5-20% improvement)
1. **Reduce Cairo call frequency** (2.10% cairo_fill)
   - Current: Drawing each pixel individually as rectangles
   - Potential: Batch pixel operations, use image directly instead of polygons
   - Impact: Could save 20-30% of total render time

2. **Profile Rust code properly** (2.90% [map2fig])
   - Enable debug symbols in release build
   - Identify hottest functions in projection/scaling
   - Likely targets: `project_pixel()`, `scale_value()`, colormap lookup

### Tier 2: Medium Impact (Potential 2-5% improvement)
3. **Vectorize projection math**
   - Use SIMD for batch coordinate transforms
   - Expected impact: 10-20% on projection-heavy work

4. **Cache colormap interpolation**
   - Colormap lookups may be repeated frequently
   - Hash/cache approach could help

### Tier 3: Lower Priority (< 2% improvement each)
5. **Optimize Cairo rectangle calls** (0.65%)
6. **Reduce memcpy operations** (0.16%)

## Recommended Next Steps

✅ **DEBUG SYMBOLS ENABLED**
- Modified Cargo.toml to build release with debug info
- `strip = false`, `debug = true` in [profile.release]
- Binary size increased to 3.9M (acceptable for profiling)
- Re-profiled with symbols for function-level breakdown

### Analysis Findings
- With debug symbols enabled, next re-run of `./tools/scripts/profile.sh` will show:
  - Exact function names consuming CPU in the `[map2fig]` 2.90% block
  - Call stacks for profiling hotspots
  - Better targeting for optimization

### Next Optimization Steps
1. **Identify exact bottlenecks** (re-run profiling, analyze output):
   - Is it projection math? → SIMD vectorization
   - Is it scaling? → Lookup table optimization
   - Is it Cairo? → Rasterization strategy redesign

2. **Profile-guided optimization**:
   - Make changes based on flamegraph findings
   - Measure speedup with `./tools/scripts/profile.sh`
   - Iterate until reaching ~10-15% improvement target

3. **Focus on Tier 1** for maximum ROI:
   - Cairo rendering (2.10%) - hardest, biggest payoff
   - Rust function hotspots (2.90%) - depends on what they are

## Architecture Reminder

Current render pipeline:
1. Read FITS → Load HEALPix data
2. Apply scaling (log, hist-eq, etc.)
3. **Project pixels** to Mollweide coords → Tier 1 target
4. **Render to output** (Cairo for PDF, image crate for PNG)

The Cairo time (2.10%) is inherent to PDF generation, but the way we call it matters.
Current approach: Individual rectangles per pixel
Alternative: Raster image then embed

## Success Metrics

- **Target**: 10-15% speedup by v0.3
- **Measurement**: `./tools/scripts/profile.sh` before/after
- **Baseline**: 0.623s (linear scale)
- **Goal**: ~0.55s baseline

---

## Files for Deep Dive

When ready to profile the Rust code with symbols:
- `src/plot/mod.rs` - Main rendering logic
- `src/scale.rs` - Scaling operations
- `src/projection.rs` - Coordinate math
- `src/pipeline.rs` - Data flow orchestration
