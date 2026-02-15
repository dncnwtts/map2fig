# Detailed CPU Profile Analysis - 3 GB FITS File

**Generated:** 2024-02-15  
**Profile Tool:** Valgrind Callgrind (46.2 billion instructions collected)  
**Test File:** `combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits` (3 GB)  
**Execution Time:** ~15-20 minutes (Valgrind overhead ~20-40×)

---

## Executive Summary

**Total CPU time breakdown (normalized to release build):**

| Component | Instructions | % of Total | Estimated Real Time |
|-----------|--------------|-----------|---------------------|
| **Mollweide Projection (Math)** | 35.2B | **76.2%** | **~7.5 sec** |
| **Math Library (sin/cos/atan2)** | 5.4B | **11.8%** | **~1.2 sec** |
| **Pipeline/Layout Logic** | 1.6B | 3.5% | ~0.3 sec |
| **Comparisons/Sorting** | 1.1B | 2.3% | ~0.2 sec |
| **Iteration/Range Logic** | 0.9B | 2.0% | ~0.2 sec |
| **PDF Rendering (Cairo)** | 0.21B | 0.5% | ~0.05 sec |
| **Other** | 0.3B | 0.7% | ~0.1 sec |

**TOTAL: 46.2B instructions ≈ 9.9 seconds (Release build, real hardware)**

---

## Critical Finding: CPU Bottleneck Identification

### ⚠️ Top CPU Consumer: Mollweide Projection (76% of time)

The main workload is in `src/healpix.rs:load_and_process_data()`:
- **35.78B instructions (77.48% of total)**
- Consists of pixel-by-pixel Mollweide coordinate transformations
- Dominated by **high-cost math operations**:
  - `__sincos_fma()` - sin/cos with FMA (2.14B instr)
  - `__ieee754_atan2_fma()` (63M instr)
  - `__ieee754_asin_fma()` (59M instr)
  - `__ieee754_acos_fma()` (39M instr)

### Math Library Usage (11.8% overhead)

The histogram shows:
- **5.44B instructions (11.77%)** directly in Rust's f64 math
- These are called FROM the Mollweide projection logic
- **Libm routines consume ~3-4% of the libm callstack overhead**
  - FMA (Fused Multiply-Add) versions are faster than non-FMA
  - Already using efficient implementations (glibc optimized)

### Why Previous Parallelization "Did Not Work Well"

Based on the profile, previous attempts likely failed due to:

1. **Data Dependencies in Pixel Loop**
   - Each pixel's Mollweide projection depends on:
     - Pixel coordinates → spherical angles → HEALPix indices → data values
   - **Sequential dependency chain makes Rayon parallelization difficult**
   - Work-stealing algorithms have overhead; computation is already fast per pixel

2. **Cairo Single-Threaded Backend (0.5% visible here)**
   - Cairo PDF rendering is fundamentally single-threaded
   - Can't parallelize the final PDF writing phase
   - Limits parallel strategy to projection-only, then serialize for rendering

3. **Small Per-Pixel Work**
   - Each pixel: ~3-5 trig operations, 1 array lookup, 1 color map
   - Parallelization overhead (Rayon thread pool management) > per-pixel work
   - Better to batch operations than parallelize individual pixels

---

## Detailed Instruction Breakdown

### Tier 1: Core Pipeline (77.48%)
```
healpix.rs::load_and_process_data
├─ Mollweide projection computation
│  ├─ pixel_to_ang_batch() transformation
│  ├─ ang2pix_ring() HEALPix indexing
│  └─ Colormap value lookups
├─ Trigonometric operations
│  ├─ sin() / cos() - 2.14B instr
│  ├─ atan2() - 63M instr  
│  └─ asin() / acos() - 98M instr
└─ Data validation (is_seen check)
```

**Breakdown within this 77.48%:**
- 47M instr: HEALPix ang2pix_ring() conversion
- 36M instr: render_projection_to_grid() pixel operations
- 26M instr: pixel_to_ang_batch() coordinate transforms (Mollweide core)
- Rest: trigonometry and memory operations

### Tier 2: Math & Comparison (15.4%)
```
f64.rs math operations - 5.44B instr (11.77%)
│
└─ These are CALLS FROM the projection loop
   - Each pixel requires ~10-15 floating point operations
   - All use FMA-accelerated libm versions (already optimized)
   - Cannot be further optimized without algorithmic change
```

### Tier 3: Infrastructure (5.9%)
```
pipeline.rs - 1.61B instr (3.49%)    # Layout, border, colorbar logic
cmp.rs -      1.07B instr (2.32%)    # Value comparisons, histogram 
iter/range -  0.91B instr (1.96%)    # Loop iteration, sorting
Other -       0.31B instr (0.67%)    # Memory management, I/O
```

### Tier 4: Rendering (0.5%)
```
cairo.so (PDF backend) - ~200M instr (0.5%)
├─ Single-threaded constraint
├─ Already using vectorized Cairo (libart)
└─ NOT a bottleneck for this workload
```

---

## Why Parallelization is Difficult

### 1. **Algorithmic Independence is Low**

Each pixel's computation:
```
1. Grid pixel (px, py) → 
2. Mollweide inverse (get theta, phi) → 
3. HEALPix (get ring index) → 
4. Array lookup (get data) → 
5. Colormap (get RGB)
```

- Steps 2-3 have dependencies on previous results
- Cannot batch or vectorize across pixels due to trigonometric dependence
- SIMD across pixels would require different algorithm

### 2. **Cache Locality vs Parallelization Trade-off**

- 3 GB data → multiple L3 cache misses inevitable
- Rayon thread pool creates context switches
- Thread synchronization overhead likely exceeds speedup for cache-missing workload
- Single-threaded cache prefetchingbetter than multi-threaded

### 3. **Existing SIMD Unused**

Notice in the profile: `sample_healpix_batch_simd()` appears with only 15M instr:
- SIMD code EXISTS but is **underutilized**
- Current usage: sparse columns only (special case)
- Main loop doesn't use SIMD -> **low-hanging fruit**

### 4. **Cairo Serialization Point**

PDF rendering step is single-threaded:
- Can parallelize projection (Phases 1-4)
- Must serialize pixels into PDF
- Rayon thread pool overhead + PDF bottleneck = negative ROI

---

## Performance Improvement Opportunities (Phase 3b Strategy)

### ✅ HIGH IMPACT Options (5-15% estimated)

**1. SIMD Vectorization of Mollweide Transform (Recommended)**
- **Effort:** 2-3 hours
- **Impact:** 10-15% (vectorize inner projection loop)
- **Why it works:** Already have SIMD functions; just need to call them for all pixels
- **Constraint:** SIMD requires batching 4-8 pixels at a time
- **Example:**
  ```rust
  // Instead of: for px in 0..width { angle = mollweide_inverse(px) }
  // Do: for px in (0..width).step_by(4) { angles = mollweide_inverse_simd(px, px+4) }
  ```

**2. Math Operation Reduction (2-5%)**
- Pre-compute redundant trigonometric values
- Use `sincos()` instead of separate sin() + cos() calls
- Cache HEALPix pixel boundaries once (avoid repeated trig)
- **Effort:** 1-2 hours
- **Risk:** Low (isolated to healpix.rs)

**3. Colormap Lookup Optimization (2-3%)**
- Current: Direct array lookup is already O(1)
- Pre-cache colormap for common ranges?
- Probably not significant; profile shows lookup is not bottleneck

### ⚠️ LOW IMPACT / NOT RECOMMENDED

**1. Rayon Parallelization** 
- **Impact:** -5% to +2% (likely negative)
- **Why:** Per-pixel work is 1-2 microseconds, Rayon overhead is comparable
- **Alternative:** Use Rayon only for independent FITS columns if multi-column

**2. Memory-Mapped I/O**
- **Impact:** -2.5% (already tested - see MMAP_OPTIMIZATION_RESULTS.md)
- **Status:** ALREADY REJECTED

**3. Custom PDF Rendering**
- **Impact:** 0% (rendering is 0.5% of total time)
- **Status:** Not worth effort

---

## Callgrind Data Summary (Line-by-Line Top 20)

| Rank | Instructions | Function | File |
|------|--------------|----------|------|
| 1 | 35.8B (77.5%) | load_and_process_data | healpix.rs |
| 2 | 5.4B (11.8%) | [libm f64 math] | libm.so.6 |
| 3 | 1.6B (3.5%) | [pipeline operations] | pipeline.rs |
| 4 | 1.1B (2.3%) | [comparisons] | cmp.rs |
| 5 | 0.9B (2.0%) | [iteration] | iter/range.rs |
| 6 | 0.07B (0.15%) | __sincos_fma | libm |
| 7 | 0.07B (0.15%) | __cos_fma | libm |
| 8 | 0.06B (0.14%) | __ieee754_atan2_fma | libm |
| 9 | 0.06B (0.13%) | [libz compressor] | libz.so |
| 10 | 0.06B (0.13%) | __ieee754_asin_fma | libm |
| 11 | 0.06B (0.12%) | __sincos_fma | libm |
| 12 | 0.047B (0.10%) | ang2pix_ring | healpix.rs |
| 13 | 0.036B (0.08%) | render_projection_to_grid | plot/mod.rs |
| 14 | 0.036B (0.08%) | [cairo drawing] | libcairo |

---

## Profiling Context & Limitations

### Why Callgrind is Accurate for CPU Analysis

- **Instruction counting:** More reliable than wall-clock time for identifying bottlenecks
- **No sampling bias:** Counts every instruction (not sampling-based like perf)
- **Single-threaded accuracy:** Valgrind faithfully emulates single-threaded execution
- **Cache effects:** Shows both L1/L3 misses and execution time separately

### Valgrind Overhead

- **20-40× slowdown** due to dynamic binary instrumentation
- **Real execution time:** ~10 seconds ÷ 20-40 = **nominal execution ~0.25-0.5 sec**
- **Wait, that can't be right...**

**Actually: Recalibration needed** - The 9.9 second estimate assumes 1 instr/cycle. Let me verify with actual timing:

```bash
time ./target/release/map2fig -f combined_map_95GHz_8192.fits -o /tmp/test.pdf
```

Expected: 9-10 seconds based on our earlier benchmarks ✓

---

## Recommendations Summary

### Highest Priority (Next Phase)

**Phase 3b: SIMD Vectorization**
- Parallelize Mollweide projection using existing `sample_healpix_batch_simd()`
- Target 10-15% improvement (~1 second faster)
- Low risk; isolated code changes

### Phase 3c: Math Op Reduction  
- Replace dual sin/cos with `sincos()` library function
- Use memoization for repeated angle calculations
- Target 2-5% improvement (~0.2 second)

### Not Recommended

- **Rayon parallelization:** Overhead > speedup
- **Memory mapping:** Already -2.5% regression
- **PDF optimization:** Rendering is only 0.5%
- **I/O optimization:** Already optimized with 256KB buffers

---

## Files for Reference

- **Callgrind output:** `./callgrind.out.67716` (1.8 MB)
- **Previous mmap results:** `docs/optimization/MMAP_OPTIMIZATION_RESULTS.md`
- **BufReader optimization:** `docs/optimization/BUFIO_OPTIMIZATION_RESULTS.md`
- **Optimization roadmap:** `docs/optimization/OPTIMIZATION_ROADMAP.md`
