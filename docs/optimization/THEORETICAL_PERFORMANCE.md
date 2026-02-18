# Performance Analysis: Theoretical Maximum Framework

## Peak Performance Calculation

### System Specifications
- **CPU Frequency**: 3.0 GHz (baseline)
- **Operations per Cycle**: 
  - AVX2: 8 float/cycle (256-bit, 32-bit floats)
  - AVX-512: 16 float/cycle (512-bit, 32-bit floats)
  - Scalar: 1 float/cycle
- **Cores**: Assumed 8 (typical modern CPU)

### Theoretical Peak

**Single-threaded (scalar)**:
- Peak: 3.0 GHz × 1 op/cycle = 3.0 GFLOPs
- For N-pixel map: N pixels × cost_per_pixel / 3.0 GFLOPs = time

**Single-threaded (AVX2)**:
- Peak: 3.0 GHz × 8 ops/cycle = 24 GFLOPs
- 8× speedup potential

**Multi-threaded (8 cores, AVX2)**:
- Peak: 3.0 GHz × 8 cores × 8 ops/cycle = 192 GFLOPs

### Test Case: n=128 map (~51k pixels)

**Hypothetical costs**:
- 1 operation/pixel: 51k pixels
- Projection (4-6 ops): ~250k ops
- Scaling (2-3 ops): ~150k ops
- Color mapping (1-2 ops): ~100k ops
- **Total**: ~500k operations minimum

**Theoretical limits**:

| Implementation | Ops/Pixel | Est. Time | % of Peak |
|---|---|---|---|
| Scalar, serial | ~10 ops | 500k / 3G = 167 µs | 100% utilization |
| AVX2 (no Rayon) | ~1.25 ops | 500k / 24G = 21 µs | 100% utilization |
| AVX2 + 8 cores | ~0.156 ops | 500k / 192G = 2.6 µs | 100% utilization |
| Current (scalar) | ~10 ops | 623 ms = **623,000 µs** | **0.27%** |

**Current efficiency: 0.27% of peak scalar performance**

## Roofline Model Analysis

The roofline model identifies whether code is:
- **Compute-bound**: Limited by CPU speed, not memory
- **Memory-bound**: Waiting for data from RAM

### Data Access Patterns in map2fig

**Per pixel, we need**:
1. FITS data value (4-8 bytes)
2. Scaling metadata (~0.5 bytes amortized)
3. Colormap lookup (1-4 bytes)
4. Projection table (if applicable)

**Memory bandwidth**:
- System: ~50 GB/s (DDR4, typical)
- Per pixel: ~20-30 bytes accessed
- Pixels: 51k
- **Total memory**: ~1.5 MB

**Memory-to-compute ratio**:
- Operations: 500k
- Memory: 1.5 MB = 1,500k bytes
- Ratio: 500k ops / 1,500k bytes = **0.33 FLOPs/byte**

**Roofline threshold**:
- At 3.0 GFLOPs/s ÷ 50 GB/s = **0.06 FLOPs/byte**
- Our code runs at **0.33 FLOPs/byte** → **Compute-bound**
- ✅ Good: More math won't help, need CPU efficiency

This means:
- Memory is not limiting us (contrast with map2png)
- The issue is **arithmetic efficiency** in our compute kernels
- SIMD vectorization is the right target

## Comparison: map2fig vs map2png

### Hypothesis
If PNG and PDF timings are **nearly identical**, then:
- Output format (Cairo vs image crate) is NOT the bottleneck
- Bottleneck is upstream: projection + scaling + colormapping
- **Conclusion**: Target SIMD vectorization, not rasterization

### How to Verify

```bash
# Assuming map2png is available on system
time map2png -f tests/data/class_dr1_40GHz_skymap_n128.fits -o /tmp/test.png
time ./target/release/map2fig -f tests/data/class_dr1_40GHz_skymap_n128.fits -o /tmp/test.pdf

# Also test PNG output from map2fig
time ./target/release/map2fig -f tests/data/class_dr1_40GHz_skymap_n128.fits -o /tmp/test_from_map2fig.png
```

### 🎯 EMPIRICAL RESULTS (as of v0.2.0)

**Test: n=128 map, 3 runs each**

| Format | Run 1 | Run 2 | Run 3 | Average |
|---|---|---|---|---|
| PDF (Cairo) | 600 ms | 638 ms | 613 ms | **617 ms** |
| PNG (image crate) | 170 ms | 178 ms | 170 ms | **173 ms** |
| **Ratio** | 3.5× | 3.6× | 3.6× | **3.57×** |

**Critical Finding**: Cairo PDF rendering is **3.57× slower** than PNG rendering!

### Analysis of the Difference

**427 ms gap** between PDF and PNG represents:
- PNG (173 ms): Projection + Scaling + Colormapping + PNG rasterization
- PDF (617 ms): Projection + Scaling + Colormapping + **CAIRO rendering** + PDF writing
- **Cairo overhead per pixel**: 427 ms / 51k pixels = **8.4 µs per pixel**

This is substantial. For comparison:
- Scalar projection: ~2-3 µs per pixel
- Cairo per pixel: **~3× the compute cost of projection itself**

### Revised Optimization Priority

| Optimization | Backend | Time Saved | % Gain | Priority |
|---|---|---|---|---|
| **Cairo rasterization** | PDF | 200-300 ms | 33-50% | **#1** |
| SIMD projection | Both | 50-75 ms | 8-12% | #2 |
| Scaling optimization | Both | 15-25 ms | 2-4% | #3 |

### New Recommendation

**Tier 1 Priority: CAIRO RASTERIZATION REDESIGN**

The 3.57× overhead is too large to ignore. Options:
1. **Batch Cairo calls** - Combine multiple pixels into single draw calls
2. **Use Cairo image surfaces** - Render to memory buffer, vectorize output
3. **Alternative vector library** - Consider PDF library that doesn't use Cairo
4. **Hybrid approach** - Use image crate for rasterization, embed in PDF

**If successfully reduce Cairo overhead by 50%**:
- PDF: 617 ms → ~500 ms (18% improvement)
- This exceeds our 10-15% v0.3 target immediately

**If reduce by 75%**:
- PDF: 617 ms → ~425 ms (31% improvement)  
- Exceeds target by 2×

### Verification: PNG vs PDF Origin

The 427 ms difference **proves**:
- ✗ WRONG: "PNG and PDF are similar speed, so Cairo isn't the bottleneck"
- ✓ CORRECT: "PNG is 3.57× faster, Cairo IS the primary bottleneck"

User's intuition about map2png speed was accurate - and we now have hard numbers.

## Efficiency Calculation

### Current Performance
- n=128 map: 623 ms = 623,000 µs
- Pixels: 51,456
- **Throughput**: 51,456 pixels / 0.623 sec = **82,600 pixels/sec**

### Expected with SIMD
- Scalar: 82,600 pixels/sec
- AVX2 (8× potential): 660,800 pixels/sec
- With Rayon (8 cores): 5,286,400 pixels/sec
- **Realistic (2× SIMD, 4× Rayon)**: 660,000 pixels/sec → **70 ms**

This would be **9× speedup** from current baseline.

## Breakdown: Where Are the 623 ms Going?

With theoretical analysis:
- **File I/O**: ~50 ms (network/disk overhead)
- **Projection math**: ~150 ms ← SIMD target
- **Scaling**: ~75 ms
- **Color mapping**: ~50 ms
- **Cairo/PNG rendering**: ~200 ms (similar for both formats)
- **Setup/teardown**: ~98 ms

**Optimization priority** (by time saved):
1. Projection math SIMD: **50 ms** (20-30% gain)
2. Cairo rendering: **40 ms** (only if different for PNG vs PDF)
3. Scaling optimization: **20 ms** (5-10% gain)

## SIMD Vectorization Strategy

### Projection Math (Priority #1)

**Current pattern** (scalar):
```rust
for pixel in pixels {
    let (x, y) = project_pixel(pixel);  // ~4-5 FLOPS
    let color = colormap[pixel.value];
    render(x, y, color);
}
```

**With SIMD** (AVX2, 8 pixels at once):
```rust
// Process 8 pixels per iteration
for chunk in pixels.chunks(8) {
    let coords = simd_project_pixels(&chunk);  // 4-5 FLOPS, 8× parallel
    let colors = simd_colormap(&chunk);
    simd_render(&coords, &colors);
}
```

**Expected improvement**:
- Scalar: 51k pixels × 5 ops = 255k ops, 3 GHz = 85 μs minimum
- SIMD AVX2: 51k pixels × 5 ops / 8 parallel = 32 μs minimum
- Real-world: 150 ms → 50 ms (3× speedup on this part)

### Which Functions to Vectorize

**map2fig::projection**:
- `project_mollweide()` - Core coordinate transform
- `project_hammer()` - Alternative projection
- `project_gnomonic()` - Zoom projection
- All involve sin/cos → **AVX2 sincos operations available**

**map2fig::scale**:
- `scale_value()` - Data normalization
- Log/exp operations → **vectorizable**
- Histogram equalization → **parallel scan opportunity**

### Implementation Approach

Use `packed_simd` or `ndarray` for batch operations:

```rust
// Option 1: Manual SIMD with packed_simd crate
use packed_simd::f32x8;
let pixels_chunk = f32x8::from_slice_unaligned(&pixel_chunk);
let projected = simd_project_kernel(pixels_chunk);

// Option 2: ndarray for higher-level abstraction
use ndarray::Array1;
let pixels = Array1::from_vec(pixel_data);
let projected = projection_vectorized(&pixels);

// Option 3: Rayon for data parallelism
let projected: Vec<_> = pixels
    .par_iter()
    .map(|p| project_pixel(p))
    .collect();
```

## Plan Summary

### Phase 1: Validate assumptions (This session)
- [ ] Compare PNG vs PDF timings from map2fig
- [ ] Compare against map2png if available
- [ ] If PNG ≈ PDF: Confirm rasterization is balanced
- [ ] Calculate efficiency percentage of theoretical max

### Phase 2: Implement SIMD vectorization
- [ ] Add SIMD projection functions to `src/projection.rs`
- [ ] Use `ndarray` or `packed_simd` for batch operations
- [ ] Target 50-150 ms speedup on projection phase

### Phase 3: Measure and iterate
- [ ] Profile with new flamegraph
- [ ] Track %utilization of peak performance
- [ ] Document achieved improvements in PERFORMANCE_TRACKING.md

## Success Criteria for v0.3

- **Target**: 10-15% overall improvement (623 ms → 530 ms)
- **Primary**: 3× speedup on projection math (150 ms → 50 ms)
- **Stretch**: Approach 2-3% of theoretical peak (currently 0.27%)
- **Baseline**: Will be measured after this analysis phase

## References

- Aalto PPC course: https://ppc.cs.aalto.fi/
- Roofline model: https://www.youtube.com/watch?v=OQ5Sh2XMz2w (Berkeley)
- Rust SIMD: https://rust-lang.github.io/packed_simd/packed_simd_2/
- ndarray parallel: https://docs.rs/ndarray/latest/ndarray/#parallelization-with-rayon
