# Tier 3 Optimization Failure Analysis: Downgrade-During-Parsing

**Date:** February 2025  
**Status:** ❌ FAILED - Optimization made performance 25% WORSE

## Summary

Attempted to implement Tier 3 optimization: "downgrade-during-parsing" to avoid creating a large 50M-pixel intermediate vector when performing downsampling from nside=8192 to nside=512.

**Result:** 25% performance degradation
- Baseline (original code): **6.41 seconds**
- With Tier 3: **8.04 seconds** (1.63s slower, 25% regression)

## Optimization Concept

### Theory
The original downsampling pipeline works as:
1. Load all 50M pixels into memory (type conversion overhead)
2. Downgrade in separate pass to 12M pixels
3. Scale data

**Proposed improvement:** Fuse steps 1 and 2
- For each pixel loaded: compute its angles → find target downsampled pixel → accumulate values
- Result: single vector of 12M pixels directly (avoid 50M allocation)
- Expected gain: **3-5%** from reduced memory allocations

### Implementation
Created `read_healpix_column_and_downgrade()` function in `src/fits.rs`:
- Pre-allocate target-nside vector (12M instead of 50M)
- Use accumulator map `Vec<(f64, count)>` to collect values per target pixel
- For each source pixel:
  - Convert pixel index → (theta, phi) using `pix2ang_nest()`
  - Convert angles → target pixel index using `ang2pix_ring()`
  - Accumulate value and count
- Finalize: compute average for each downsampled pixel

## Why It Failed

The optimization **added expensive work** that far outweighed any memory allocation savings:

### Cost Analysis
**Per-pixel operations added:**
- `pix2ang_nest()`: Compute theta, phi requires trigonometric calculations (sqrt, atan2)
- `ang2pix_ring()`: Reverse conversion using trigonometric and mathematical operations
- Total: ~50-100 CPU cycles per pixel × 50M pixels = **2.5+ billion cycles of transcendental math**

**Comparison to baseline:**
- Baseline downsampling: Simple nearest-neighbor or averaging lookup (~5 CPU cycles per pixel for top 12M)
- Tier 3 approach: Expensive coordinate conversions (~75 cycles per pixel for all 50M)
- **Net result: 15× more CPU work**

### Why Memory Allocation Isn't the Bottleneck
The theory assumed memory allocation overhead would be significant, but:
1. Allocation of 50M vs 12M vector is trivial (~400MB difference)
2. Kernel memory management amortizes cost across many operations
3. Cache pressure from coordinate conversion math dominates
4. Type conversions (FITS DataValue enum matching) still happen in load phase

## Root Cause
The fundamental limitation is that **downsampling inherently requires computing which source pixels map to each target pixel**. The original approach:
- Loads in source order (sequential, cache-friendly)
- Groups/averages into target pixels in second pass (also sequential)

Tier 3's approach:
- Loads in source order (same as original)
- **But immediately remaps each pixel via expensive coordinate conversion**
- Destroys cache locality and adds transcendental math per pixel

## Lesson: Amdahl's Law Applied to Optimization

The FITS loading pipeline breakdown (~6.4s total):
- File I/O: ~0.8s (12%)
- Type conversions: ~2.5s (39%) ← **This is the real bottleneck**
- Downsampling: ~0.4s (6%)
- Projection/rendering: ~2.7s (42%)

**Tier 3 attempted to "optimize" the 6% (downsampling) by:
1. Adding expensive work to the load phase
2. That added work exceeded the savings
3. Net result: overall speedup impossible via this approach

**Key insight:** Trying to eliminate the downsampling step by integrating it into loading doesn't work because:
- The "cost" being eliminated (0.4s) is less than 1% of total time
- Any additional per-pixel work in the load phase multiplies across 50M pixels
- 0.1 microsecond × 50M = 5 seconds of overhead

## Viable Alternative Approaches

### Tier 1: Direct Column Reading (30-40% gain) ⭐ HIGH PRIORITY
- Bypass `fitsrs`'s generic `DataValue` enum
- Read column directly into `Vec<f64>` 
- Eliminate the per-pixel type conversion match statement
- **Estimated gain:** 0.8-1.6 seconds from ~2.5s conversion overhead
- **Difficulty:** HIGH (requires unsafe code or FITS binary format knowledge)

### Tier 2: Vectorize Type Conversion (15-25% gain)
- Use SIMD to process multiple `DataValue` conversions in parallel
- Requires `packed_simd` or `portable_simd_` nightly features
- **Estimated gain:** 0.4-0.6 seconds
- **Difficulty:** MEDIUM

### Tier 4: Parallel HDU Parsing (10-20% gain)
- Read/parse multiple HDUs in parallel (if file has multiple extensions)
- Requires careful buffer management
- **Difficulty:** MEDIUM

### Tier 5: Fused Downgrading Post-Projection
- **Different approach:** Keep original load-downgrade pipeline
- Instead, downgrade HEALPix pixels **in the projection**, not in pre-processing
- Only compute target pixel for pixels that project to visible image region
- **Estimated gain:** 5-10% for typical images (skip ~90% of pixels)
- **Difficulty:** MEDIUM

## Conclusion

**This failure validates the Copilot Instructions' warning:**
> ⛔ **F32 Precision Reduction (Feb 15, 2026):** Attempted speedups by reducing precision were SLOWER due to conversion overhead. **DO NOT attempt precision reduction again.**

Same principle applies here: **Adding per-pixel operations in the hot path is slower than the memory allocations being saved**.

### Recommendations
1. **Do NOT retry downgrade-during-parsing** - it's fundamentally limited by Amdahl's Law
2. **Focus on Tier 1:** Direct column reading (biggest ROI: 30-40%)
3. **Secondary:** Tier 2 vectorization or Tier 4 parallel parsing
4. **Consider:** Tier 5 post-projection downsampling for specific use cases

### For Future Optimization Attempts
Always profile to find the **true bottleneck** (type conversions, not memory allocation). Optimizations that add per-pixel work to the hot path must save more than they cost - test them before implementing.
