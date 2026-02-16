# Master Optimization Status & History

**Last Updated:** February 16, 2026  
**Current Baseline:** 14.176 seconds (3.1GB file, nside=8192)

---

## Executive Summary

This document consolidates all optimization attempts across the HEALPix Plotter project. Over the course of development, we've attempted **18+ optimization strategies** with varying results: some achieving significant speedups, others failing catastrophically. This document captures the lessons learned from each.

### Performance Timeline
```
Initial Baseline:         ~39.2s (v0.1)
After Tier 1 (Fast I/O):  ~20.08s (49% improvement)
After Tier 1.2 (Memory):  ~14.3s  (additional 28% improvement)
After Tier 4 (Rayon):     ~14.176s (36% over downsampling baseline)
Current Limit:            ~14.176s (memory bandwidth bound)
```

---

# ✅ SUCCESSFUL OPTIMIZATIONS

## 1. Tier 1: Direct Binary Float32 Reading [COMPLETED]

**Status:** ✅ **ACTIVE & RECOMMENDED**

### Achievement
- **Speedup:** 3.4× (2-3× for FITS reading)
- **Code:** `try_read_float32_column_fast()` in `src/fits.rs`
- **Method:** Bypass fitsrs DataValue enum, read binary f32 directly, convert to f64

### How It Works
```rust
// OLD: Parse through enum conversion (slow)
for row in table.select_fields(&[col_idx]) {
    match row {
        DataValue::Float { value } => result.push(value as f64),
        // ...
    }
}

// NEW: Direct binary reading (fast)
for row in 0..num_rows {
    let chunk = &mmap[row_offset .. row_offset + 4];
    let f32_val = f32::from_le_bytes([...]);
    result.push(f32_val as f64);
}
```

### Why It Works
- Eliminates enum allocation/matching overhead (~60% of FITS reading time)
- Leverages memory-mapped I/O (hardware prefetch)
- Compatible with existing fitsrs parser for metadata

### Implementation Details
- **File:** `src/fits.rs` lines ~60-200
- **Functions:** `try_read_float32_column_fast()`, `parse_tform()`, `find_binary_table_data_offset()`
- **Conditions:** Only activates for float32 columns (common for HEALPix)
- **Fallback:** Automatically uses slow path for other data types

### Lessons Learned
- ✅ Specialized fast paths are worth it for common cases
- ✅ Type conversion overhead is significant (60% of bottleneck)
- ✅ Binary format knowledge pays off

---

## 2. Tier 1.1: Memory-Mapped I/O [COMPLETED]

**Status:** ✅ **ACTIVE & RECOMMENDED**

### Achievement
- **Speedup:** 20-21% additional over base FITS reading
- **Code:** Uses `memmap2::Mmap` in `src/fits.rs`
- **Method:** Map entire FITS file to memory instead of buffered reading

### Why It Works
- Eliminates kernel `memcpy` overhead (rep_movs instruction)
- Improves cache locality by keeping file in page cache
- Allows fitsrs to read directly from mmap'd memory

### Implementation Details
- **File:** `src/fits.rs` lines ~220-225
- **Change:** Single line change: `let mmap = Mmap::map(&file)`
- **Trade-off:** Higher initial latency (file map operation) but better throughput

### Lessons Learned
- ✅ Kernel I/O can be significant bottleneck
- ✅ Memory mapping works well for large sequential reads
- ✅ Small code change, big impact

---

## 3. Tier 1.2: Streaming Percentile Computation [COMPLETED]

**Status:** ✅ **ACTIVE & CRITICAL FOR LARGE MAPS**

### Achievement
- **Memory:** 79% reduction (45GB → 9.4GB on nside=8192)
- **Speed:** 49% faster (39.2s → 20.08s)
- **Code:** `compute_percentile_from_map()` in `src/plot/mollweide.rs`

### Problem Solved
High-resolution maps (nside=8192 = 806M pixels) were allocating massive vectors:
1. `compute_mollweide_scale()` allocated full vector for percentile computation
2. `_plot_mollweide_pdf_impl()` allocated another full vector
3. Total: 2× (12.8GB) + overhead = 45GB peak memory

### Solution
Use streaming/sampling algorithm instead of materializing entire vector:
```rust
// OLD: Allocate 806M floats twice
let all_values = vec![0.0; 806_000_000];  // 6.4GB × 2
let percentiles = compute_percentile(&all_values);

// NEW: Stream sample 10M pixels max (80MB)
let percentiles = compute_percentile_from_map_streaming(map, sample_size=10M);
```

### Why It Works
- Percentiles don't need exact calculation (visual quality unchanged)
- Sampling 10M pixels ≈ 1.24% of full map (statistically sound)
- Memory footprint drops to 2-3× file size (vs 14.5×)

### Results Across File Sizes
| File Size | Old Memory | New Memory | Reduction |
|-----------|-----------|-----------|-----------|
| 25 MB     | 360 MB    | 72 MB     | 80% ✅    |
| 193 MB    | 2.8 GB    | 560 MB    | 80% ✅    |
| 577 MB    | 8.3 GB    | 1.7 GB    | 80% ✅    |
| 3.1 GB    | 45 GB     | 9.4 GB    | 79% ✅    |

### Lessons Learned
- ✅ Streaming algorithms avoid memory explosion
- ✅ Approximate results fine for visualization
- ✅ Critical for scaling to large datasets

---

## 4. Tier 4: Rayon Parallelization of Downsampling [COMPLETED]

**Status:** ✅ **ACTIVE & EFFECTIVE**

### Achievement
- **Speedup:** 1.36× wall clock (19.4s → 14.3s)
- **Code:** `downgrade_healpix_map_xyf_parallel()` in `src/healpix.rs`
- **Method:** Use Rayon `par_iter()` to distribute target pixel processing

### How It Works
```rust
// OLD: Single-threaded
let result: Vec<f64> = (0..target_npix)
    .map(|target_pix| {
        // Process 256 source accesses per pixel
    })
    .collect();

// NEW: Multi-threaded with Rayon
let result: Vec<f64> = (0..target_npix)
    .into_par_iter()
    .map(|target_pix| {
        // Each thread processes independent chunk
    })
    .collect();
```

### Why It Works (Not Cache Optimization, But Distribution)
- **Key insight:** Parallelization doesn't reduce L1 cache misses (1.079B → 1.098B, +1.7%)
- **What actually improves:** Distributes contention across cores
- **Mechanism:** Each core operates on ~1M pixels with independent L3 working set
  - Better effective bandwidth through parallel memory requests
  - Reduced single-core stall wait times
  - Memory bus utilization improves

### Performance Breakdown
| Metric | Single Core | Rayon (2-3 threads) | Change |
|--------|------------|-------------------|--------|
| **Wall Clock** | 23.15s | 14.57s | **1.59×** ✅ |
| **User CPU** | 18.44s | 28.52s | 1.55× (more cores) |
| **L1 Misses** | 1.079B | 1.098B | +1.7% (flat!) |
| **LLC Misses** | 53.9M | 172.2M | +219% distributed |
| **IPC per-core** | 2.16 | 1.61 | -25% (expected) |

### Threshold Optimization
- **Only parallelizes when beneficial:** >50K target pixels
- **Avoids overhead for small maps:** Uses scalar fallback
- **Auto-scales:** Works with 2-8 cores

### Lessons Learned
- ✅ Parallelization effective for memory-contention problems
- ✅ Distribution beats elimination for unbounded random access
- ✅ Rayon overhead is minimal (<1 second)
- ✅ Works well with 2-4 cores (limited by memory bandwidth)

---

## 5. Tier 5: GPU Acceleration (CUDA) [OPERATIONAL]

**Status:** ✅ **OPERATIONAL (Integer-Only, 292× Speedup)**

### Achievement
- **Speedup:** 173-292× (0.013-0.022s GPU vs 3.8s CPU)
- **Code:** `src/gpu/cuda/` module
- **Method:** Offload projection + colormap to GPU

### Important Caveat
**This is NOT production-ready for current mainline.** GPU acceleration is:
- ✅ Functional when CUDA is available
- ✅ Intelligent CPU fallback when GPU unavailable
- ⚠️ Constrained to **integer-only math** (CUDA 12.0 JIT limitation)
- ⚠️ Limited to **pre-scale then lookup** (no floating-point projection on GPU)

### How It Works
1. **CPU:** Load HEALPix data (f64) → Scale to 0-255 (u32) → Transfer to GPU
2. **GPU:** For each pixel: Load u32 value → Lookup colormap (ARGB) → Write output
3. **CPU:** Transfer result back → ARGB→RGBA conversion → Render to PDF/PNG

### Architecture
- **GPU Kernel:** Integer-only (zero float operations)
- **Constraint:** CUDA 12.0 JIT cannot compile full matrix math
- **Workaround:** Use integer indexing into pre-computed colormap

### Performance vs CPU
| File | GPU | CPU | Speedup |
|------|-----|-----|---------|
| class_dr1 (128) | 0.013s | 3.8s | **292×** |
| cosmoglobe | 0.021s | 3.8s | **181×** |
| npipe (217) | 0.021s | 3.8s | **181×** |

### Status: Why Not Default?
The GPU acceleration works but has limitations:
1. Integer-only math reduces quality (quantization error)
2. Not suitable for publication-quality renders
3. Requires CUDA toolkit (desktop tool, not portable)
4. CPU version is already near-optimal for its baseline

**Current decision:** Keep GPU as opt-in feature, CPU baseline as default.

---

# ❌ FAILED OPTIMIZATIONS (Do Not Retry)

## 1. Tier 3: Cache-Sequential Sorting [FAILED]

**Status:** ❌ **CATASTROPHIC FAILURE - REVERTED**

### What We Tried
Build vector of (source_pix, target_pix) pairs, sort by source for sequential access:
```rust
let mut accesses: Vec<(usize, usize)> = Vec::new();
for target_pix in 0..target_npix {
    for source_pix in get_sources(target_pix) {
        accesses.push((source_pix, target_pix));
    }
}
accesses.sort_unstable_by_key(|a| a.0);  // Sort for sequential
// Process in source order...
```

### Why We Tried It
- Profiling showed 1.08B L1 cache misses (17.4% of time)
- Hypothesis: Sequential source access would reduce misses
- Theory: ~4 seconds saved = reasonable target

### What Actually Happened
| Metric | Before | After | Impact |
|--------|--------|-------|--------|
| **Wall Clock** | 19.4s | **68.3s** | **3.5× SLOWER** ❌ |
| **Sort Overhead** | — | ~35-40s | Dominating cost |
| **Allocation/Init** | — | ~5s | Extra vectors |
| **Memory Traffic** | — | Massive | 805M pairs = 13GB |
| **Instructions** | 134B | 418B | 3× more code |

### Why It Failed (Root Cause Analysis)

**Amdahl's Law in action:**
- Cache misses represent 17.4% of time (4 seconds of 23s baseline)
- Best-case speedup from fixing: 1.21× (save 4s)
- Actual cost of sorting: **49+ seconds**
- Result: -3.5× (catastrophic!)

**Key insight:** Sorting 805M items costs more than accepting random misses:
- Single L1 miss: ~10 cycles latency (overlapped with other work)
- Sorting 805M items: dedicated CPU work, fully blocking
- Algorithm dominance: Random access with misses < sequential access via sort

### Lessons Learned
- ❌ **DO NOT attempt cache-aware rearrangement for >100M data**
- ❌ **Cost of sorting >> benefit of sequential access**
- ✅ Modern CPUs overlap memory latency well
- ✅ Current nested-loop design is near-optimal for its scale

---

## 2. Tier 3 (Original): Downgrade-During-Parsing [FAILED]

**Status:** ❌ **PERFORMANCE REGRESSION - REVERTED**

### What We Tried
Fuse downsampling into FITS load phase to avoid intermediate vector allocation:
```rust
// OLD: Load all 50M pixels, then downsample to 12M
let data = read_healpix_column(fits_file);      // 50M floats
let downsampled = downgrade(&data);             // Downsample to 12M

// NEW: Downsample while parsing
for (i, value) in fits_parser.enumerate() {
    if should_include_pixel(i) {                 // Filter during load
        downsampled.push(value);
    }
}
```

### Why We Tried It
- Theory: Avoid 800M → 200M allocation overhead
- Memory savings: ~5-6% of total time
- Expected: Marginal 5-10% speedup

### What Actually Happened
- **Result:** 25% SLOWER (6.41s → 8.04s) ❌
- **Overhead:** Added expensive per-pixel coordinate conversions in hot loop
- **Cost:** ~50 CPU cycles × 50M pixels = 2.5B cycles added

### Root Cause
Each filtered pixel requires:
```rust
pix = source_pix;
pixel_coord = pix2ang_nest(pix);      // ~10 cycles
target_pix = ang2pix_nest(pixel_coord); // ~10 cycles each
```
Multiplied by 50M pixels = massive overhead.

### Why It Failed
- **Problem:** 6% of program (memory allocation) takes 39% of program time (downsampling)
- **Amdahl's Law:** Cannot optimize 6% by adding work to 39%
- **Cost > Benefit:** Transcendental math overhead >> allocation savings

### Lessons Learned
- ❌ **DO NOT move expensive operations into tight loops**
- ❌ **Allocation is cheap compared to math**
- ✅ **Decouple pipeline stages**
- ✅ Keep downsampling separate from I/O

---

## 3. F32 Precision Reduction [FAILED]

**Status:** ❌ **MINOR SLOWDOWN - NOT WORTH PURSUING**

### What We Tried
Reduce f64 math to f32 (faster on some CPUs):
- Option A: Cast f64 → f32 → f64 around math operations
- Option B: Use native f32 arrays throughout

### Results
- **Single conversion:** 2% slower
- **Native f32 arrays:** 3.7% slower
- **Cause:** Conversion overhead exceeds math speedup

### Why It Failed
- Math is only 11.8% of total CPU time
- Already well-optimized by LLVM (compiler does this better than we can)
- Real bottleneck: Mollweide projection (77.5%) and Cairo (not math)
- Conversion cost (~0.5 cycles per operation) > math speedup gains

### Lessons Learned
- ❌ **DO NOT attempt precision reduction for small % of total time**
- ✅ **Trust compiler for math optimization**
- ✅ **Profile first to find actual bottleneck**

---

## 4. Tier 3b: Pre-Allocation Optimization [FAILED]

**Status:** ❌ **SEVERE REGRESSION (71% SLOWER) - REVERTED**

### What We Tried
Pre-allocate 5 arrays outside rendering loop instead of stack allocation:
```rust
// OLD: Stack-allocated inside tight loop
for y in 0..height {
    let mut accum = vec![0.0; width];    // Stack allocation
    let mut hits = vec![0; width];
    // ...use arrays...
}

// NEW: Pre-allocate once
let mut accum = vec![0.0; width];   // Outside loop
let mut hits = vec![0; width];
for y in 0..height {
    // ...reuse arrays...
}
```

### Hypothesis
Reduce allocation churn in hot loop, improve performance.

### Reality
| Metric | Before | After | Loss |
|--------|--------|-------|------|
| **Wall Clock** | 10.83s | 18.57s | +71.4% ❌ |
| **Instructions** | 55.6B | 130.3B | +134% ❌ |
| **Cycles** | 28.6B | 50.6B | +77% ❌ |
| **Cache Misses** | 23.44% | 31.17% | +7.73pp ❌ |
| **LLC Misses** | 11.17% | 16.56% | +5.39pp ❌ |

### Why It Failed
**Modern compiler optimization fought us:**
1. **Compiler optimization inhibition:** LLVM was aggressively optimizing small lexical-scope arrays
2. **Register pressure:** Expanded array lifetime prevented register reuse
3. **Cache aliasing:** Conservative memory aliasing assumptions kicked in
4. **Lost optimizations:** Compiler couldn't apply loop-invariant code motion

**The smoking gun:** 134% increase in instructions (55.6B → 130.3B)
- Indicates massive overhead from register spills
- Compiler generating much more code
- Worse cache behavior overall

### Lessons Learned
- ❌ **DO NOT move allocations to outer scope in optimization attempts**
- ✅ **Trust compiler for allocation placement**
- ✅ **Lexical scope helps compiler optimize aggressively**
- ✅ **Measure instruction count and cache metrics**

---

## 5. SIMD Loop Unrolling [AVOIDED/NOT PURSUED]

**Status:** ⚠️ **NOT ATTEMPTED (Based on F32 Failure)**

### Why Avoided
F32 optimization failure (2-3.7% slower) taught us:
- Math is only 11.8% of total time
- Already optimized by LLVM
- SIMD would add complexity without benefit

### Estimated Impact
- **If pursued:** 3-5% best case (from 11.8% of time)
- **Against:** F32 failure proved compiler beats us on math
- **Conclusion:** Not worth effort

---

# ⏳ ATTEMPTED BUT INCOMPLETE / IN PROGRESS

## 1. FITS Column Reading Parallelization (Feb 16, 2026)

**Status:** ⚠️ **TESTED & REJECTED**

### What We Tried
Parallelize FITS dense map column reading with Rayon:
```rust
let values: Vec<DataValue> = table.select_fields(&[col_idx]).collect();
result = values.par_iter()
    .map(|cell| convert_to_f64(cell))
    .collect();
```

### Bottleneck We Targeted
FITS reading: 11.727s (82.7% of data load time)

### Results
- **Performance:** 35.944s (2.5× SLOWER) ❌
- **Cause:** Collection overhead (806M DataValue enums) + rayon coordination

### Why It Failed
1. **Collection cost:** `collect()` allocates massive intermediate vector (~5-8s overhead)
2. **Memory bandwidth:** Already saturated (66.76% LLC miss rate)
3. **Amdahl's Law:** Non-parallelizable portion (I/O, fitsrs parsing) dominates

### Key Learning
- ❌ **Cannot parallelize I/O-bound operations effectively**
- ❌ **Collection overhead kills parallelization benefits**
- ✅ **Streaming algorithms better than collect() + par_iter()**
- ✅ **Memory bandwidth is hard ceiling**

---

# 🚀 FUTURE OPTIMIZATION OPPORTUNITIES

## Not Yet Attempted / High Potential

### 1. Tier 2: SIMD Vectorization of Projection Math [HIGH IMPACT]

**Potential Gain:** 15-25% speedup  
**Effort:** MEDIUM  
**Difficulty:** Medium (requires SIMD intrinsics)

#### Current Bottleneck
- Mollweide projection: ~77.5% of CPU time
- Trigonometric operations: ~70% of projection time (~1.3s of 1.9s)
- Single-core scalar math

#### Strategy
```rust
// Current: Process 1 pixel at time
for target_pix in 0..num_pixels {
    x = compute_x(target_pix);      // ~10 cycles
    y = compute_y(target_pix);      // ~10 cycles
    theta = atan2(y, x);            // ~20 cycles (sin/cos)
}

// Proposed: Batch 4-8 pixels
for batch in pixels.chunks_exact(8) {
    xs = simd_compute_x_batch(batch);  // 4× speedup
    ys = simd_compute_y_batch(batch);  // 4× speedup
    thetas = simd_atan2_batch(...);    // 2× speedup (intrinsics available)
}
```

#### Implementation
- Use `packed_simd` crate or `std::simd` (nightly)
- Focus on atan2, sin, cos (expensive transcendentals)
- Batch size: 4-8 pixels per iteration

---

### 2. Cache-Aware Pixel Tiling [MEDIUM IMPACT]

**Potential Gain:** 10-15% speedup  
**Effort:** MEDIUM  
**Difficulty:** Medium (careful memory layout tuning)

#### Current Problem
Random memory access pattern (HEALPix Morton code) → L1 cache thrashing

#### Strategy
Process pixels in 64×64 spatial blocks instead of per-pixel:
```rust
for block_x in 0..(width/64) {
    for block_y in 0..(height/64) {
        // Pre-fetch block memory
        prefetch_block_data(block_x, block_y);
        
        // Process 64×64 pixels with better cache locality
        for y in block_y*64..(block_y+1)*64 {
            for x in block_x*64..(block_x+1)*64 {
                // Spatial neighbors in same block
            }
        }
    }
}
```

#### Expected Impact
- **Per-pixel improvement:** L3 cache hit rate +10-20%
- **Branch prediction:** Better for nearby pixels
- **Prefetch efficiency:** More predictable memory access

---

### 3. Cairo Rasterization Batching [MEDIUM IMPACT]

**Potential Gain:** 15-25% speedup  
**Effort:** MEDIUM  
**Difficulty:** Medium (Cairo API understanding)

#### Current Problem
Cairo does 51,000 individual rectangle fills (one per pixel):
```rust
for pixel in pixels {
    cairo_rectangle(x, y, 1, 1);
    cairo_set_source_rgb(color);
    cairo_fill();  // 51,000 individual calls!
}
```

#### Strategy
Group pixels by color, reduce fill calls:
```rust
let grouped = group_pixels_by_color(pixels);
for (color, pixels) in grouped {
    cairo_set_source_rgb(color);
    for (x, y) in pixels {
        cairo_rectangle(x, y, 1, 1);
    }
    cairo_fill();  // ~256 calls instead of 51,000
}
```

#### Expected Impact
- **Fill calls:** 51,000 → ~256 (200× reduction)
- **Overall speedup:** 15-25% (PDF rendering overhead)

---

### 4. GPU Projection (CUDA/HIP) [HIGHEST IMPACT]

**Potential Gain:** 3-5× speedup  
**Effort:** HIGH  
**Difficulty:** HIGH (GPU kernel development)

#### Current State
- GPU acceleration exists (integer-only, 292× speedup) but limited
- Projection math on GPU would eliminate 77.5% of CPU work
- Already partially prototyped (see GPU_DEPLOYMENT_SUMMARY.md)

#### Strategy
- Move full Mollweide projection to GPU
- Use floating-point operations (not integer-only)
- Handle colormap on CPU (small & fast)
- Keep rendering on PDF/PNG libraries

#### Challenge
CUDA 12.0 JIT limitations with floating-point math require:
- CUDA 12.2+ or custom PTX compilation
- Careful kernel design to avoid JIT incompatibilities

---

### 5. Streaming/Progressive Rendering [MEDIUM IMPACT]

**Potential Gain:** 5-10% perceived speedup  
**Effort:** HIGH  
**Difficulty:** HIGH (UI/architecture changes)

#### Idea
Display partial results while computation continues:
```
t=0s:   Display coarse preview (64:1 downsampling)
t=2s:   Refine to 16:1 detail
t=5s:   Show 4:1 detail
t=14s:  Final result
```

#### Why Valuable
- **Perceived performance:** Users see results faster
- **Interact ability:** Can cancel early if wrong map
- **Streaming:** Natural for iterative exploration

#### Implementation Complexity
- Requires output architecture change
- PDF generation more complex (incremental rendering)
- PNG friendly (can write progressively)

---

# 📊 Summary Table: All Optimization Attempts

| Tier | Name | Status | Impact | Notes |
|------|------|--------|--------|-------|
| **1** | Float32 binary read | ✅ SUCCESS | 3.4× FITS | Active |
| **1.1** | Memory-mapped I/O | ✅ SUCCESS | 20-21% | Active |
| **1.2** | Streaming percentile | ✅ SUCCESS | 79% memory | Active |
| **4** | Rayon parallelization | ✅ SUCCESS | 1.36× | Active |
| **5** | GPU acceleration | ✅ WORKING | 292× | Opt-in only |
| **3** | Cache sorting | ❌ FAILED | -3.5× | Do not retry |
| **3 (orig)** | Downgrade-in-parse | ❌ FAILED | -25% | Do not retry |
| **F32** | Precision reduction | ❌ FAILED | -3.7% | Do not retry |
| **3b** | Pre-allocation | ❌ FAILED | -71% | Do not retry |
| **FITS** | Rayon reading | ❌ FAILED | -2.5× | Do not retry |
| **Tier 2** | SIMD projection | ⏳ PROPOSED | 15-25% | Not attempted |
| **Tiling** | Cache-aware blocks | ⏳ PROPOSED | 10-15% | Not attempted |
| **Cairo** | Rasterization batch | ⏳ PROPOSED | 15-25% | Not attempted |
| **GPU 2.0** | GPU projection | ⏳ PROPOSED | 3-5× | Prototype exists |

---

# 🎯 Recommendations for Future Work

### Immediate (High Confidence)
1. **Keep all active optimizations:** Tier 1, 1.1, 1.2, Tier 4 (Rayon)
2. **Consider Cairo batching:** Medium effort, 15-25% gain
3. **Add SIMD vectorization:** Medium effort, 15-25% gain

### Medium-term (If further improvement needed)
4. **GPU projection:** Highest impact (3-5×) but high effort
5. **Cache-aware tiling:** 10-15% gain, medium effort

### Do Not Pursue
- ❌ Sorting-based optimizations
- ❌ Downgrade fusion
- ❌ Precision reduction
- ❌ Pre-allocation outside loops
- ❌ FITS reading parallelization (without redesign)

### Memory Bandwidth Ceiling
- **Current:** 14.176 seconds (2.29 cores, 268.9M LLC loads)
- **Realistic limit with current algorithm:** ~12-13 seconds (10-15% gain)
- **Hard wall:** Memory bandwidth (50-100 GB/s typical DDR4)
- **To break through:** Requires algorithmic change or GPU

---

# Conclusion

The HEALPix Plotter optimization journey demonstrates:
1. ✅ **Targeted optimizations work:** Tier 1 (3.4×), Tier 4 (1.36×)
2. ❌ **Blind optimizations fail:** Sorting (-3.5×), pre-allocation (-71%)
3. ✅ **Memory matters:** Streaming percentile saves 79% memory
4. 🎯 **Profile-driven approach is essential:** CPU profiling identified winning strategies
5. 🚀 **GPU promising but limited:** Current integer-only GPU shows potential (292×) but needs float support

**Current Status:** Optimized to near-CPU limits. Further gains require SIMD vectorization or GPU floating-point projection math.

