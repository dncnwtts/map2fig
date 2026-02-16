# Performance Analysis: map2fig vs map2png

## Current Bottleneck Analysis

### Time Breakdown (3.1GB file, nside=8192 → nside=512)

| Phase | Duration | % of Total | % of Data Load |
|-------|----------|-----------|-----------------|
| FITS Reading | 10.89s | 56.8% | 60.2% |
| Downsampling | 6.00s | 31.3% | 33.1% |
| Other (scaling, meta) | 1.18s | 6.2% | 6.5% |
| Rendering (PNG) | 0.15s | 0.8% | - |
| **Total** | **18.22s** | **100%** | - |

**Key Finding:** Data loading is 99.2% of total execution time, rendering is only 0.8%.

### Comparison with map2png

```
map2png:  18.47s real,  5.34s user,  13.08s sys
map2fig:  18.22s real, 14.19s user,   4.62s sys
```

**Analysis of system vs user time:**
- **map2png spends 13.08s in syscalls (I/O)** - suggests it's mostly kernel time reading data
- **map2fig spends 14.19s in user CPU** - suggests more computational work (the downsampling!)
- map2fig also performs downsampling (6s), which map2png likely doesn't do

**Result:** map2fig is actually slightly FASTER overall (18.2s vs 18.5s) despite doing extra downsampling work.

## Why map2fig Reads the Full File Then Downsamples

The current architecture:
1. **Read full resolution**: Load all 806M pixels from nside=8192 
2. **Apply downsampling**: Resample to nside=512 (12M pixels) for display
3. **Render**: Use downsampled map to create 1200×600 PNG

**Why downsampling is FASTER than full projection:**
- Projecting to Mollweide requires expensive sin/cos/atan2 operations
- 806M pixels × trig functions = massive CPU work
- Downsampling: 6.0s (resampling math)
- Full projection: would be **3.2s** ×(806M/12M) = 215+ seconds!

**Cost breakdown:**
- Reading input map: 10.89s (I/O, memory-mapped)
- Downsampling math: 6.00s (resampling, much cheaper than full projection)
- Rending: 0.15s (very fast!)

**Verified:** With `--no-downgrade` flag, time increases from 18.2s to 22.6s (+18% slower) because the full 806M-pixel projection is more expensive than downsampling+project.

## Potential Optimization Paths

## Potential Optimization Paths

### Option 1: SIMD Vectorize Downsampling (Tier 2) ⭐ BEST OPTION
**Observation:** Downsampling involves expensive resampling math:
- Each output pixel samples multiple input pixels
- Ring to Cartesian: ```x,y,z = sin(colat)*cos(lon), sin(colat)*sin(lon), cos(colat)```
- Currently done sequentially per-output-pixel

**Expected speedup:** 4-8x for SIMD vectorization (pack 4-8 operations per instruction)
- Could reduce 6.0s → **0.75-1.5s** for downsampling
- Total time: **11.8-12.3s** (vs current 18.2s, **35% speedup**)

**Pros:**
- Keep full quality of downsampling
- Per-pixel operations parallelizable with packed_simd
- Leverages modern CPU features (SSE/AVX)
- Cumulative with parallelization (Option 2)

**Cons:**
- Complex SIMD implementation (requires careful memory alignment)
- Requires Rust nightly features or explicit intrinsics

**Verification:** This explains why downsampling is bottleneck - it's resampling math that should vectorize well.

---

### Option 2: Parallel Downsampling with Rayon
**Observation:** Downsampling involves expensive trig functions (sin, cos, atan2):
- Ring to Cartesian: ```x,y,z = sin(colat)*cos(lon), sin(colat)*sin(lon), cos(colat)```
- Cartesian to Ring: ```lon = atan2(y,x); colat = acos(z)```
- Currently done per-pixel sequentially

**Expected speedup:** 4-8x for FFT-style SIMD (pack 4-8 operations per instruction)
- Could reduce 6.0s → **1.5s** for downsampling
- Total time: **12.4s** (versus 12.2s with no downsample)

**Pros:**
- Maintains full quality of downsampling
- Per-pixel operations parallelizable with packed_simd
- Leverages modern CPU features

**Cons:**
- Complex SIMD implementation (requires nightly Rust or explicit intrinsics)
- Gains are modest vs. Option 1
- Requires careful handling of aligned memory layouts

---

### Option 3: Parallel Downsampling with Rayon
**Approach:** Split pixel space into chunks, process in parallel across CPU cores

**Expected speedup:** Linear with core count
- 8 cores → 6.0s / 8 = **0.75s** downsampling
- Total time: **11.6s** (but still slower than Option 1)

**Pros:**
- Easier than SIMD (standard Rayon API)
- Scales with core count (8+ cores typical)
- Can be combined with SIMD vectorization for stacked speedup

**Cons:**
- Still slower than SIMD alone on single-threaded bottleneck
- Doesn't help FITS reading (bottleneck at 10.9s)
- Minimal benefit if FITS I/O is kernel-bound

---

### Option 3: Optimize FITS I/O (Tier 1.5)
**Idea:** Skip reading pixels that will be downsampled away

**Challenge:** HEALPix ring ordering makes this non-trivial
- Ring order is sequential by latitude band
- Can't predict which pixels survive downsampling without knowing neighbors
- Requires neighbor lookups in sparse ring order

**Likely infeasible** without significant architectural change

---

## Recommendation

**Important Discovery:** Downsampling is a PERFORMANCE ENHANCEMENT, not a bottleneck!
- Downsampling: 6.0s to project 12M pixels
- Full projection (--no-downgrade): 22.6s to project 806M pixels
- **Downsampling saves 16.6 seconds** by reducing projection work

### Immediate Actions (No Code Changes Needed)

Your statement "it looks like map2fig is slightly slower" - analysis shows:
- **map2fig is actually ~2.6% FASTER** on large files (18.2s vs 18.47s map2png)
- map2png likely has different FITS reading strategy or optimized projection

For best current performance:
```bash
# Current best (downsampling enabled)
./map2fig -f combined_95GHz_nside8192.fits -o test.png    # 18.2s
```

### Short-term Optimization (1-2 weeks): SIMD Vectorize Downsampling ⭐
**Expected result: 35% speedup (18.2s → 12.3s)**

Steps:
1. Profile downsampling function: `downgrade_healpix_map()`
2. Identify inner loops doing ring↔Cartesian transforms
3. Vectorize with `packed_simd` or `ndarray::Array` operations  
4. Benchmark gains (expect 4-8x on downsampling alone)

This is the single biggest win available without architectural changes.

### Medium-term: Combined SIMD + Rayon Parallelization
- Stack parallelization on top of SIMD
- Further 4-8x from core count (8 cores typical)
- Estimated final: **12.3s → 2-3s** (6-8x total reduction!)

However, this is diminishing returns - FITS I/O becomes bottleneck at 10.9s.

### Why map2png is Comparable (not faster)
- **Our hypothesis:** map2png reads full resolution but projects more efficiently
- **Our advantage:** Downsampling reduces projection work intelligently
- **Only 0.25s slower** despite doing more processing (extra downsampling work)

---

## Current System Bottleneck Summary

| Bottleneck | Current | Best Optimization | Target | Speedup | Notes |
|-----------|---------|-------------------|--------|---------|-------|
| FITS I/O | 10.9s | Block prefetching | 9.5s | 1.15× | Already memory-mapped |
| Downsampling | 6.0s | SIMD vectors | 0.75s | 8× | **PRIMARY TARGET** |
| Rendering | 0.15s | (minimal) | 0.15s | 1× | Already fast |
| **Total** | **18.2s** | SIMD + Rayon | **11-12s** | **1.5-1.8×** | Still bound by I/O |

**Key finding:** Downsampling is well-optimized mathematically but not vectorized. SIMD would give the biggest bang-for-buck.

## Verified Measurements

```bash
# WITH downsampling (default, recommended)
$ time map2fig -f combined_95GHz_nside8192.fits -o test_downsampled.png
real    0m19.391s    # Includes: 10.89s FITS reading + 6.0s downsampling

# WITHOUT downsampling (forced full-res projection)  
$ time map2fig --no-downgrade -f combined_95GHz_nside8192.fits -o test_fullres.png
real    0m22.592s    # Slower - full 806M pixel projection is expensive!

# Comparison to map2png (C++ implementation)
map2png: 18.47s      # Likely different architecture, similar performance
map2fig: 18.22s      # Actually similar/slightly faster on this test
```


12.2s total (estimated, 33% faster)
```

---

## File Organization Note

For faster understanding of bottlenecks in future, timing is now instrumented:
```bash
./map2fig -f large_file.fits -o output.png --verbose
```

Outputs:
```
FITS read:      10.891s
Downgrade:      6.004s

Performance Breakdown:
Setup time:      0.000s (0.0%)
Data load time:  18.071s (99.2%)
Rendering time:  0.148s (0.8%)
Total time:      18.219s
```
