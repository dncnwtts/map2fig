# Rayon Parallelization - Archived

**Status**: Archived - not merged to main  
**Branch**: `rayon-parallelization`  
**Date**: February 2026  

## Summary

Attempted to implement Rayon-based parallelization of HEALPix pixel rendering. While technically successful, the performance gains did not justify the added complexity.

## Results

### Performance Gains
- **Best case** (6000px, nside=512): 20.7% speedup
- **Typical case** (2400-4000px): 14-17% speedup
- **Large files** (3.1GB): 3-11% speedup (I/O limited)
- **I/O dominant**: <3% speedup

### vs. Original Estimates
- **Roadmap estimate**: 30-40% speedup
- **Actual delivery**: 10-20% in most cases
- **Gap reason**: High per-pixel computation overhead, cache-unfriendly access patterns, thread synchronization costs

## Why Archived

1. **Complexity vs. Gain Mismatch**
   - Added binary bloat (rayon dependency)
   - Added code paths (sequential and parallel variants)
   - Testing burden (verify both paths)
   - User documentation (when to use `--parallel`)
   - For 10-20% gain, not worth the ongoing maintenance

2. **Better Alternative Already Exists**
   - Automatic **downgrade** feature provides **2.5x speedup** with zero user intervention
   - Works for all typical use cases (1200-2400px)
   - No complexity, no user confusion

3. **Fundamental Limitations**
   - HEALPix sampling is not embarrassingly parallel
   - Per-pixel computation too small relative to thread overhead
   - Memory bandwidth saturation prevents higher gains
   - I/O dominates on large files (>1GB)

## What Was Implemented

If this is ever revisited, the following exists in the branch:

- Runtime dispatch system (`render_projection_to_grid_with_parallel`)
- Rayon row-level parallelization in `render_projection_to_grid_parallel`
- `--parallel` CLI flag
- `--no-downgrade` CLI flag (this one is useful and should be kept)
- Comprehensive benchmarking results

## Lessons Learned

1. **Parallelization isn't always the answer** - Especially when I/O and per-work-unit overhead dominate
2. **Algorithmic improvements beat parallelization** - The downgrade feature is worth more than any parallelization
3. **Test on real data early** - The 3.1GB combined_map revealed I/O constraints that pure Rust FITS parsing couldn't overcome
4. **Code simplicity matters** - The overhead of maintaining two code paths isn't worth 10-20% gains

## If You Need This Later

The branch contains all working code, benchmarks, and documentation. To revive:

```bash
git checkout rayon-parallelization
cargo build --release --features parallel
./target/release/map2fig -f data.fits -o output.pdf --parallel
```

But consider first:
1. Is the downgrade feature insufficient for your use case?
2. Are you specifically rendering ultra-high-resolution posters (6000px+)?
3. Would you use `--parallel` regularly enough to justify maintenance?

If all three answers are "yes", the branch is ready to merge.

## Recommendation

Keep this branch for historical reference, but focus on:
1. Maintaining the `--no-downgrade` flag for testing/benchmarking
2. Documenting when users should use automatic downgrade vs. custom resolutions
3. Other optimizations if needed (CFITSIO for I/O, SIMD for pixel computation, etc.)
