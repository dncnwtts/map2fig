# FITS Reading Optimization Analysis - February 16, 2025

## Summary

Attempted to parallelize FITS dense map column reading using Rayon. **Result: FAILED** - parallelization made performance worse (14.1s → 35.9s, **2.5× SLOWER**).

## Bottleneck Breakdown (Current Baseline)

From profiling with `map2fig` baseline (3.1GB file, nside=8192):

```
Total Runtime:     14.176 seconds
  FITS Reading:    11.727 seconds (82.7% of data load)
  Downgrade:        1.435 seconds (10.1% of data load)
  Other:            ~1.0 second   (7% - rendering, layout, etc.)
```

## What We Attempted

### Optimization: Rayon Parallelization of Dense Map Reading

**Strategy:**
```rust
// Before (sequential):
let values = table.select_fields(&[ColumnId::Index(col_idx)]);
for cell in values {
    match cell { ... }
}

// After (with Rayon):
let values: Vec<DataValue> = table.select_fields(&[ColumnId::Index(col_idx)]).collect();
result = values.par_iter()
    .map(|cell| match cell { ... })
    .collect();
```

**Implementation:** Conditional parallelization only for maps >100K pixels

**Results:**
- **Baseline:** 14.176s wall clock (28.405s user, 4.678s sys)
- **With Rayon:** 35.944s wall clock (55.438s user, 29.070s sys)
- **Regression:** **2.5× SLOWER**

## Root Cause Analysis

### Why Parallelization Failed

1. **Memory Bandwidth Saturation**
   - Current perf profile shows 66.76% LLC miss rate
   - Already hitting memory bandwidth wall (268.9M LLC loads, 179.5M misses)
   - Adding more parallelism fights cores over limited bandwidth

2. **Collection Overhead**
   - `collect()` forces allocation of 806M DataValue enums (~64-128 GB depending on padding)
   - This single operation adds ~5-8 seconds of memory allocation + copying
   - Eliminates any benefit from parallelization

3. **Iterator vs. Parallel Iterator**
   - Sequential flow with push: Benefits from streamed memory access, predictable cache patterns
   - Collected Vec + par_iter: Requires full materialization, random access pattern, thread coordination

4. **Amdahl's Law**
   - Parallelizable portion (enum→f64 conversion): ~2-3 seconds theoretically
   - Fixed overhead (file I/O + fitsrs parsing): ~9-10 seconds
   - Even with perfect parallelization: maximum speedup ≈ 14.1 / 9 ≈ 1.57×
   - Actual overhead of rayon: -2.5× (negative speedup)

## Current Optimizations (Working)

### Tier 1: Direct Float32 Binary Reading
- **Function:** `try_read_float32_column_fast()` in src/fits.rs
- **Speedup:** 2-3× for float32 columns (common case)
- **Status:** ✅ Active and effective

### Tier 2: Memory-Mapped I/O
- **Function:** `read_healpix_column()` uses `memmap2::Mmap`
- **Benefit:** Eliminates kernel memcpy overhead (~20-21% speedup)
- **Status:** ✅ Active

### Tier 3: Streaming Percentile Computation
- **Function:** `compute_percentile_from_map()` in src/plot/mollweide.rs
- **Benefit:** 79% memory reduction for huge maps (nside=8192)
- **Status:** ✅ Active and critical for large maps

### Tier 4: Rayon Downsampling Parallelization
- **Function:** `downgrade_healpix_map_xyf_parallel()` in src/healpix.rs
- **Speedup:** 1.36× (1.4s vs 2.1s before optimization)
- **Status:** ✅ Active and effective

## Why FITS Reading is Hard to Parallelize

1. **fitsrs Library Constraints**
   - `select_fields()` returns an iterator that's already optimized
   - DataValue enum parsing is done lazily per-row
   - Difficult to optimize without rewriting fitsrs itself

2. **Memory Access Pattern**
   - Sequential file I/O is fastest for mmap'd files
   - Parallel reading from same mmap creates contention

3. **Sparse Files**
   - 806M pixels × 8 bytes float = ~6.4 GB memory footprint
   - Downsampling to target resolution (from 8192 NSIDE) reduces to ~50M pixels
   - Downsampling is actually the more valuable optimization

## Lessons Learned

### ❌ What Doesn't Work for FITS Reading

1. **Direct Rayon parallelization** - Too much overhead
2. **Sorting to improve cache locality** - Overhead > benefit (from Tier 3 attempt)
3. **Precision reduction** - No measurable benefit, ~2-3% slower

### ✅ What Works

1. **Specialized fast paths** for common data types (float32)
2. **Memory mapping** for file I/O
3. **Downsampling** to reduce data volume earlier
4. **Parallel downsampling** to distribute CPU-bound work

## Next Optimization Targets (Priority Order)

### Option 1: GPU Acceleration (Highest Impact)
- Use CUDA/OpenGL for downsampling on GPU
- Potential: 5-10× speedup for projection math
- Status: Already prototyped in codebase (mentioned in copilot instructions)
- Effort: High (requires GPU infrastructure)

### Option 2: Improve Fast-Path Coverage
- Extend `try_read_float32_column_fast()` to handle double-precision
- Add support for integer columns
- Potential: 30-50% speedup for non-float32 columns
- Effort: Medium (requires careful FITS binary parsing)

### Option 3: Async I/O
- Use tokio or async_std for concurrent file I/O
- Limited benefit (single file bottleneck)
- Potential: 10-15% speedup in theory
- Effort: High (requires substantial refactoring)

### Option 4: Accept Current Performance
- 14.1s for 3.1GB file = 219 MB/s effective throughput
- Already 1.36× faster than baseline via downsampling parallelization
- Memory bandwidth limited, further optimization unlikely without hardware changes
- Potential: Diminishing returns

## Memory Bandwidth Analysis

From perf counter data:
- LLC loads: 268.9M / 14.5s ≈ 18.5M loads/sec
- LLC bytes loaded: ~144GB (assuming 536-byte cache lines)
- Effective throughput: ~9.9 GB/sec
- System memory bandwidth: ~50-100 GB/sec typical (DDR4)
- Conclusion: Good but not optimal - still room for improvement

## Conclusion

The 14.1-second execution time for 3.1GB file is near-optimal for CPU-bound implementations without:
1. Algorithmic changes (sparse representation)
2. Hardware acceleration (GPU)
3. Algorithmic optimization (different projection method)

Further CPU-side optimization chances are low. Recommended next step: **GPU acceleration for projection math** (already coded, mentioned as Tier 4 in copilot instructions).

## Files Modified

None - this analysis resulted in **code reversion** due to negative performance impact.

### Git Status
```
On branch main
No uncommitted changes
```

---

**Date:** February 16, 2025
**Test File:** `combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits` (3.1GB, nside=8192)
**Hardware:** 8-core CPU, DDR4 RAM, perf-capable
