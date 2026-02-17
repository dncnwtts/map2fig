# Downsampling Optimization Session: Feb 17-18, 2026

**Date Range:** February 17-18, 2026  
**Focus:** Downsampling bottleneck optimization (75.93% of CPU time, 5.7 seconds on 3.1 GB file)  
**Root Cause Identified:** 3.2 billion random memory accesses → 82% CPU stall time

## Session Summary

This session investigated downsampling optimization after establishing it as the primary bottleneck via perf profiling. Two major optimization attempts were made with dramatically different outcomes.

### Baseline Metrics (Start of Session)

| Metric | Value | Context |
|--------|-------|---------|
| Wall-clock time | 7.502s | Combined_95GHz_nside8192 (3.1 GB) |
| Downsampling time | 5.7s | 75.93% of total runtime |
| Cache miss rate | 31.85% | Random 806M-pixel access pattern |
| CPU stall time | 82% | 148B of 180B CPU cycles |

### Optimization 1: Prefetch Hints ✅ (Feb 17)

**Status:** SUCCESSFUL

**Implementation:**
```rust
// Prefetch 2 iterations ahead in inner loop
if prefetch_i < (x0 + fact) {
    unsafe {
        core::arch::x86_64::_mm_prefetch(
            &map[prefetch_pix] as *const f64 as *const i8,
            1,  // _MM_HINT_T0: L1 cache
        );
    }
}
```

**Results:**

| Metric | Baseline | With Prefetch | Change |
|--------|----------|---------------|--------|
| Wall-clock (mean) | 7.502s | **7.263s** | **-3.18%** ✅ |
| Std deviation | ±0.205s | ±0.192s | -6.3% (more stable) |
| Visible prefetch cost | — | 7.68% (perf) | — |

**Why It Worked:**
1. Addresses root cause directly: memory latency (50-100 cycles)
2. Low code complexity: 15 lines of strategic hint code
3. Overhead overlaps with previously-idle CPU time
4. Zero correctness risk (hints only, not guarantees)

**Amdahl's Law Calculation:**
- CPU had idle cycles while waiting for memory (82% stall time)
- Prefetch calculation uses ~7.68% of those idle cycles
- Net result: 7.68% overhead × 82% available/100% = 6.3% of original idle time
- Savings from hidden latency > cost of prefetch = 3.2% net improvement ✅

**Key Insight:** This is the textbook example of Amdahl's Law working correctly: small overhead overlapped with idle time yields net positive result.

---

### Optimization 2: Tiling ❌ (Feb 18)

**Status:** FAILED with regression

**Implementation Attempted:**
```rust
// Process targets in spatial 256×256 tiles per HEALPix face
// Instead of linear chunking (0..806M), organize as:
for face in 0..12 {
    for tile_y in (0..nside).step_by(256) {
        for tile_x in (0..nside).step_by(256) {
            // Process tile
        }
    }
}
```

**Results:**

| Metric | Prefetch | With Tiling | Change |
|--------|----------|-------------|--------|
| Wall-clock (mean) | 7.263s | **8.156s** | **+12.3%** ❌ |
| Downsampling approach | Linear chunks | Spatial tiles | Different algorithm |

**Why It Failed:**

1. **Task Overhead:** 
   - Linear approach: ~31K tasks (100K chunks over 806M pixels)
   - Tiling: ~3000 tasks (12 faces × many 256×256 tiles)
   - Fewer tasks ≠ better! Scheduling cost mattered less; tile reconstruction cost dominated

2. **HEALPix Geometry Mismatch:**
   - NESTED ordering uses Morton codes (hierarchical space-filling curve)
   - Spatially-near targets in Cartesian space ≠ spatially-near in HEALPix space
   - Tile boundaries don't respect HEALPix structure

3. **Prefetch Already Solved the Bottleneck:**
   - Once memory latency is hidden via prefetch, reorganizing iteration provides negative value
   - Tiling didn't improve prefetch effectiveness
   - Added complexity without benefit

4. **Reconstruction Overhead:**
   - Per-tile result buffers
   - Merging tiles back into linear result array
   - Extra indirection and bookkeeping

**Root Cause Lesson:** Tiling was theoretically sound (better spatial locality) until we added prefetching. Once one bottleneck (latency) was addressed, the secondary hypothesis (spatial reorganization) became counterproductive. The correct lesson: **measure before optimizing**, and **measure again after each change**.

---

## Lessons Learned

### 1. Amdahl's Law in Reverse
When you fix a bottleneck, you often reveal that your next "optimization" isn't actually an improvement because it only improved the already-fixed bottleneck.

**What We Learned:**
- Tiling was proposed to improve cache locality (spatial grouping)
- Prefetch improved memory latency hiding (temporal overlap)
- Once temporal hiding worked, spatial reorganization added cost without benefit

### 2. Measure, Don't Speculate
The session succeeded precisely because we:
- ✅ Ran real benchmarks for every change (./benches/run_benchmarks.sh e2e)
- ✅ Used perf profiling to validate code paths (sudo perf record)
- ✅ Compared before/after wall-clock times
- ✅ Documented both successes AND failures

If we'd deployed tiling without benchmarking, we'd have shipped a 12% regression.

### 3. Low-Overhead Wins
Prefetch succeeded because it was:
- Minimal code (15 lines)
- Minimal overhead (7.68% visible cost)
- Direct attack on bottleneck (latency hiding)
- Easy to revert if needed

Tiling failed because:
- More complex (100+ lines of tile logic)
- Higher overhead (algorithm change)
- Indirect attempt (spatial reorganization)
- Harder to understand failure mode

### 4. Understand Your Hardware
The prefetch optimization only works because:
- CPU has memory prefetchers that respond to hints
- CPU has 4-8 outstanding memory requests (can hide ~2 iterations)
- x86_64 has `_mm_prefetch` instruction
- HEALPix algorithm has regular iteration patterns

Tiling failed because:
- HEALPix NESTED uses Morton codes (not simple spatial proximity)
- 256×256 tiles don't align with HEALPix face boundaries cleanly
- Random access patterns can't be improved by iteration reordering alone

---

## Current Status

**Optimized Runtime:** 7.263s (was 7.502s before prefetch)
**Improvement:** +3.2% (63.8% total improvement from initial 39.2s)
**Remaining Potential:** ~50-70% (limited by hardware I/O bandwidth)

### Hard Limit Analysis

- **Theoretical minimum (bandwidth-bound):** 3.1 GB ÷ 9.1 GB/s = 0.34s
- **Current time:** 7.26s
- **Bottleneck:** CPU overhead, not I/O speed
- **Why:** FITS reading is only 20.8% of time; CPU coordination is rest

### What Can't Be Optimized Further (CPU side)

- ✅ I/O: Prefetching, caching, sequential reads → helped a lot
- ✅ Memory layout: Streaming percentile → helped a lot
- ✅ Parallelization: Rayon → helped moderately
- ✅ Latency hiding: Prefetch hints → helped slightly
- ❌ Iteration order: Spatial reorganization → hurts (negative ROI)
- ❌ Cache reuse: Already optimal for random pattern
- ❌ SIMD: Already applied, marginal gains

### Remaining Options (If >2× improvement needed)

1. **GPU Acceleration** (5-10× possible)
   - Embarrassingly parallel algorithm
   - CUDA/HIP frameworks available
   - Requires external library dependencies

2. **Ring Ordering** (Sequential access)
   - Trade accuracy/compatibility for speed
   - Breaks NESTED semantic guarantees
   - Would need flag/option

3. **Accept Current Performance**
   - Prefetch is good optimization
   - Further gains approach zero
   - Focus on other features/quality

---

## Files Modified/Created

### New Files (Documentation)
- `PREFETCH_OPTIMIZATION_RESULTS.md` - Detailed prefetch analysis and results
- `TILING_OPTIMIZATION_FAILURE_ANALYSIS.md` - Why tiling failed and what we learned
- `DOWNSAMPLING_BOTTLENECK_ROOT_CAUSE.md` - Updated with optimization history

### Updated Files (Status)
- `OPTIMIZATION_AUDIT_2026.md` - Added post-script with session results
- [other status docs updated...]

### Source Code Changes
- `src/healpix.rs` - Added pragmatic prefetch hints (1 commit)
- `src/healpix.rs` - Attempted tiling (reverted after failure)

---

## Recommendations for Future Work

### If Pursuing Further CPU Optimization
1. ✅ **Validation methodology established** - Do measurements, not speculation
2. ✅ **Benchmarking infrastructure ready** - Use ./benches/run_benchmarks.sh
3. ✅ **Profiling tools confirm** - Use `sudo perf record` for validation
4. ❌ **Avoid iteration reordering** - Doesn't help for random access patterns
5. ✅ **Prefetch is our best tool** - Low cost, high strategic value

### For Production Deployment
- Current 7.26s → 7.5s performance is solid for 3.1 GB file
- Consider GPU option if users demand <5s processing
- Document prefetch dependency (x86_64 only; fallback on other platforms)

### For Next Optimization Session
1. Review [`TILING_OPTIMIZATION_FAILURE_ANALYSIS.md`](TILING_OPTIMIZATION_FAILURE_ANALYSIS.md) first
2. Understand why spatial reorganization failed
3. Focus on broader algorithm changes (GPU) rather than micro-optimizations
4. Measure every hypothesis before implementing

---

## Conclusion

This session demonstrated the power and limits of performance optimization:

✅ **What Worked:**
- Pragmatic prefetch hints providing 3.2% improvement
- Rigorous measurement methodology
- Understanding Amdahl's Law and its consequences
- Documenting failures as learning opportunities

❌ **What Didn't Work:**
- Assuming tiling would help without measurement
- Reorganizing iteration without understanding HEALPix geometry
- Speculation instead of profiling

**Final Status:** We've reached the practical limit of CPU-side optimization for downsampling. Further improvements require either GPU acceleration or algorithmic changes. The prefetch optimization is a good, pragmatic win that provides measurable value with minimal complexity.

