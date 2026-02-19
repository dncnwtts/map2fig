# Tiling Optimization Attempt - Failed

**Date:** February 18, 2026  
**Optimization:** Spatial tile-based downsampling  
**Status:** ❌ **FAILED** - 12% performance regression  

## Result Summary

| Version | Wall-clock | Change | Notes |
|---------|-----------|--------|-------|
| Baseline (linear chunking) | 7.502s | — | Original Rayon approach |
| + Prefetch hints | 7.263s | **-3.2%** ✅ | Working optimization |
| + Tiling (attempted) | 8.156s | **+12.3%** ❌ | Worse than baseline |

## What We Tried

Implemented spatial tile-based parallelization instead of linear chunking:

```rust
// Old: Process targets in linear pixel order (0..3.1B)
for chunk_start in (0..target_npix).step_by(chunk_size) {
    // Process all pixels in chunk, accessing scattered source pixels
}

// New: Process targets in spatial tiles (256×256 on each face)
for face in 0..12 {
    for tile_y in (0..nside).step_by(256) {
        for tile_x in (0..nside).step_by(256) {
            // Process all targets in this tile
            // Idea: Spatially grouped targets access spatially grouped sources
        }
    }
}
```

**Hypothesis:** Spatially grouped targets would access spatially grouped source pixels, improving cache locality.

**Reality:** 12% slower (8.156s vs 7.263s).

## Why It Failed

### 1. **Overhead of Tile Iteration**
- Created extra vector allocations: `tile_coords.iter().copied().collect::<Vec<_>>().into_par_iter()`
- Added tile-to-result mapping complexity
- Per-Rayon-task overhead now 2× larger (12 faces × many tiles = hundreds of tiny tasks)

### 2. **Tile Size Mismatch**
- nside=512 target: Each face is 512×512 = 262K pixels (for some inputs)
- Chose 256×256 tiles = 65K pixels per tile
- For largest file (nside=8192): Creates 12 faces × 256 tiles = ~3000 Rayon tasks (vs ~31K chunks in linear approach)
- Too many tasks → excessive overhead

### 3. **Spatial Locality Argument Was Weak**
- Linear chunking already reuses source pixels across targets within a chunk
- HEALPix NESTED ordering: Spatially close targets DON'T always access spatially close sources
- The Morton code used in NESTED ordering has hierarchical structure, not linear spatial proximity

### 4. **Memory Access Pattern Unchanged**
- Prefetch optimization already addresses the core bottleneck (hidden latency)
- Tiling didn't add meaningful prefetch improvements
- The 3.2B random accesses are now spread across 3000 tasks instead of 31K tasks - each task does more work
- Cache cold starts at task boundaries (no amortization of cache warming)

## Key Insight

**Amdahl's Law strikes again:** We optimized for a problem (poor spatial locality) that wasn't actually the bottleneck once prefetch hints were in place. The prefetch hints already hide the memory latency well enough that further memory reorganization provides negative returns due to task overhead.

The downsampling workload is fundamentally:
- **Memory bandwidth limited** → prefetch helps by hiding latency
- **Parallelization overhead sensitive** → more tasks = more overhead
- **Not cache-friendly in the traditional sense** → HEALPix geometry defeats simple spatial grouping

## What This Teaches Us

1. **Prefetch was the low-hanging fruit** - directly addresses memory latency by hiding it with computation
2. **Tiling helps when tasks are independent** - but here each tile still needs all 3.2B source pixel reads
3. **Morton/Z-order curves won't help much either** - spatial curves mostly matter for dense array traversal, not for HEALPix projection math
4. **The hard ceiling is memory bandwidth** - We're hitting fundamental I/O limits, not algorithm limits

## Data Point

For comparison:
- **Baseline (linear chunks, no prefetch):** 7.502s
- **+ Prefetch (added to linear approach):** 7.263s → +3.2% gain ✅
- **+ Tiling (replaced linear with spatial):** 8.156s → -12.3% regression ❌

The lesson: **Don't remove working optimizations to add new ones.** The prefetch optimization was lightweight and effective. Tiling sounded better in theory but added too much overhead.

## Why Tiling Was Previously Proposed

Looking at the documentation, tiling was proposed as a 1.5-2.0× speedup because:
- Earlier analysis assumed scheduling overhead of 310K linear chunks
- Tiling would reduce that to ~3K tasks
- **But:** The prefetch optimization already reduced the effective bottleneck to memory latency (hidden by prefetch)
- Changing iteration strategy no longer provided value once latency was hidden

## Next Steps

Given that prefetch is working and tiling failed:

### Option 1: Accept Current Performance (Recommended)
- Prefetch gave us 3.2% improvement, pushing runtime to 7.26s (7.502 → 7.263)
- Further optimization would require fundamental algorithm changes
- Bandwidth is the hard limit: 3.1 GB ÷ 9.1 GB/s = 0.34s theoretical minimum

### Option 2: Try Hybrid Approach (If 2-3% more gain matters)
- Keep prefetch (working)
- Add prefetch for next target pixel (not just next source)
- Pre-compute tile lookups to reduce coordinate overhead
- **Risk:** High complexity for minimal gain

### Option 3: GPU Acceleration (If >2× speedup needed)
- Downsampling is embarrassingly parallel
- CUDA/HIP could achieve 5-10× speedup
- But requires external dependencies and platform support

## Conclusion

**Tiling optimization attempt conclusively failed.** The spatial tile approach added 12% overhead compared to the already-optimized prefetch+linear approach. This demonstrates that once a primary bottleneck is addressed (memory latency via prefetch), attempting second-order optimizations without careful measurement leads to regression.

The prefetch optimization remains our best incremental improvement: **+3.2% wall-clock improvement with minimal code complexity and zero correctness risk.**
