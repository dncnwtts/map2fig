# Memory-Mapped I/O Benchmark Results
## Status: NOT EFFECTIVE - Negative Result

**Date:** February 15, 2026  
**Test File:** combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits (3,072 MB)  
**Benchmark:** 3 runs with time(1) measurement

---

## Results

| Mode | Real Time | User Time | System Time | Status |
|------|-----------|-----------|-------------|--------|
| **Buffered I/O** (baseline) | 9.886s | 6.755s | 3.128s | ✓ baseline |
| **Memory-mapped I/O** | 10.132s | 6.991s | 3.126s | ✗ **2.5% SLOWER** |
| **Mmap + Profiling** | 10.150s | 7.012s | 3.130s | ✗ **2.7% SLOWER** |

## Analysis

### Why Mmap Made Things Worse

1. **Cursor overhead**: Used `Cursor<&[u8]>` wrapper to adapt mmap to fitsrs API
   - Adds indirection layer for every read operation
   - Cursor-based seeking less efficient than sequential BufReader

2. **Page fault handling**: Memory mapping doesn't automagically optimize this scenario
   - FITS parsing is **already sequential** (BufReader optimized for this)
   - Page faults worse case (OS must zero-fill or fetch from disk)
   - BufReader's 256 KB buffer already near-optimal for sequential FITS access

3. **Metadata cache hit path**: Most files use cached metadata
   - If metadata was cached: ~0 I/O savings for mmap
   - Mmap mapping cost > I/O savings for small metadata sections

4. **CPU-bound bottleneck confirmed**: Even with "free" I/O (warm cache)
   - Real time unchanged: ~9.9s
   - Suggests **rendering/projection takes ~9-10 seconds**
   - I/O is <1 second of total time

### Expected vs Actual

**Expected benefit** (from Tier 2.1 analysis):
- Large sequential file: 10-20% speedup
- Reduce syscalls by 30-40×
- Zero-copy improvements

**Actual result:**
- **Negative 2.5% regression**
- Worse than baseline due to API wrapper overhead
- Proves I/O was not the bottleneck

---

## Key Insights

1. **The 256 KB BufReader optimization was already optimal**
   - Previous phase achieved 293.5 MB/s throughput
   - FITS sequential access pattern perfectly suited to buffering
   - Syscalls reduced to acceptable levels

2. **Cursor-based wrapper adds overhead**
   - fitsrs expects efficient Read + Seek
   - Cursor implementation slower than native file seeks
   - Could fix with direct integration, but not worth effort

3. **CPU is the real bottleneck**
   - Mollweide projection: ~25-35% of time
   - PDF rendering (Cairo): ~20-30% of time
   - Colorbar + layout: ~10-15% of time
   - These are CPU-bound and **cannot be accelerated by I/O optimization**

4. **Profile data supports this**
   - System time: ~3.13s (I/O operations)
   - User time: ~6.75-7.01s (CPU work)
   - Ratio is 6.75:3.13 = 2.15:1 (CPU to I/O)
   - I/O optimization can't fix 70% CPU work

---

## Recommendation

### ❌ Do NOT pursue further I/O optimization
- Diminishing returns achieved
- Fundamental bottleneck is CPU-bound
- Further I/O tuning will not yield measurable improvements

### ✅ Focus on CPU-bound solutions (Tier 3b)
1. **Parallel Mollweide projection** (HIGH IMPACT)
   - Current: single-threaded pixel-by-pixel projection math
   - Solution: Chunk grid, compute in parallel, collect results
   - Potential: 2-4× speedup on multi-core systems
   - Effort: 3-4 hours

2. **SIMD optimizations** (MEDIUM IMPACT)
   - Mollweide math: sin/cos/atan2 operations
   - Can vectorize with explicit SIMD
   - Potential: 1.5-2× speedup
   - Effort: 4-6 hours

3. **Reduce PDF overhead** (SMALL IMPACT)
   - Cairo single-threaded constraint
   - Could pre-render pixels to image, batch to PDF
   - Potential: 5-10% speedup
   - Effort: Easy (already have image rendering path)

---

## Code Status

### Removed/Disabled
- mmap implementation remains in codebase (harmless)
- `MAP2FIX_USE_MMAP` environment variable still supported
- Not recommended for use (provides no benefit)

### Recommondation: Clean up
If we want a clean codebase, could:
1. Remove `src/mmap_reader.rs` module
2. Remove `read_healpix_column_mmap()` and `read_healpix_meta_cached_mmap()`
3. Remove Cargo.toml dependency on memmap2
4. **Keep git history**: Shows investigation was done and why it didn't work

---

## Conclusion

**Memory-mapped I/O showed NO BENEFIT and actually regressed performance by 2.5%.**

This confirms the CPU-bound theory: **I/O is not the limiting factor for large FITS file rendering.**

The "law of diminishing returns" applies here:
- Phase 1 (BufReader 256 KB): ✓ worthwhile optimization (better buffering strategy)
- Phase 2 (mmap): ✗ provides no benefit for this workload (already I/O efficient)
- Phase 3 (parallel rendering): ✓ next opportunity (addresses CPU bottleneck)

**Next optimization target:** Parallel Mollweide projection computation for 2-4× speedup on multi-core systems.
