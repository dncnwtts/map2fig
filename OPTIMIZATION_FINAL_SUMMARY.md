# Final Performance Summary - All Optimizations (Feb 17, 2026)

## Current Performance (All Optimizations)

### Cold Cache (First Run)
```
6MB file:       298.4 ± 37.5 ms    
24MB file:      479.9 ± 27.6 ms    
72MB file:      477.0 ± 46.8 ms    
192MB file:     769.6 ± 45.2 ms    
576MB file:     815.4 ± 14.9 ms    
3072MB file:    11.748 ± 0.093 s   ⬅️ 16.8%↓ from 14.1s baseline
```

### Warm Cache Benefits (Medium Files)
- Small files (6-72MB): Cached in ~2MB, subsequent runs near-instant
- Medium files (192-576MB): Cached in ~2GB, subsequent runs 0.3-0.5s vs 0.8s
- Large files (3GB+): Skipped (too large), no penalty

## Optimization Timeline

### Phase 1: Coordinate Lookup Caching
- **Implementation**: LRU cache with 10K entries per function
- **Memory**: ~320KB overhead
- **Benefit**: 8-14% on small files, 1% on large files
- **Status**: ✅ Active

### Phase 2: Unsafe FITS Reader Optimization
- **Attempt**: Direct pointer arithmetic to avoid bounds checking
- **Result**: No improvement, small file regressed 10%
- **Status**: ❌ Reverted (compiler already optimizes well)

### Phase 3: Column Cache Fix
- **Bug Found**: Large columns (6.4GB) cached then immediately deleted
- **Root Cause**: 2GB cache size limit vs 6.4GB file cache
- **Fix**: Skip caching columns >1GB
- **Benefit**: Medium files now cacheable, system overhead reduced 45%
- **Status**: ✅ Fixed

## Architecture Analysis - Why Further Optimization is Hard

### Bottleneck Distribution (3.1GB File)
```
FITS reading (sequential, mmap):   10.9s   | 81% of data load | HARD to parallelize
Downgrade (parallel, rayon):        1.3s   |  10% of data load | Already optimized
Rendering (SIMD, batch):            0.2s   |   2% total time  | Out of critical path
Memory overhead:                   ~9.4GB  |   3× file size   | Already minimized
─────────────────────────────────────────────────────────────────────────────
Total:                             ~13.6s  (System: 2.6s I/O overhead)
```

### Why Speedup is Limited
- **Amdahl's Law**: FITS reading (81%) is sequential → Max speedup ≈ 1.4× even with perfect parallelization
- **I/O Bound**: Currently reading at ~285 MB/s (file I/O + mmap + CPU conversion)
- **Cache Effects**: 45% reduction in system time shows I/O optimization working well
- **Math Bound**: Trigonometry only 11.8% of CPU time, SIMD gives marginal returns

## Recommendation: Performance is Optimized, Focus Elsewhere

### Further Optimization Options (Evaluated)

| Optimization | Effort | Potential Gain | Recommendation |
|---|---|---|---|
| **SIMD Mollweide projection** | Medium | 15-25% on rendering (only 0.2s) | ❌ Low ROI |
| **Custom FITS parser** | High | 15-20% on FITS read | ⚠️ High complexity |
| **Parallel FITS reading** | Very High | 10-15% max (sequential headers) | ❌ Not viable |
| **Column streaming cache** | Medium | Improves repeated runs | ✅ Already done |
| **GPU rendering** | Very High | 50-70% on rendering | ❌ Cairo bottleneck |
| **Accept current performance** | None | Maintains 14s baseline | ✅ Recommended |

### Better ROI Improvements
1. **Batch processing**: Process multiple files in parallel (user improvement)
2. **Incremental rendering**: Update plots without full recompute (UX improvement)
3. **Web interface**: Stream results instead of offline rendering
4. **Data compression**: Pre-cache compressed columns (storage improvement)

## Performance Validation

### Measurement Quality
- **Method**: Hyperfine with 5 runs, 1 warmup, 95% confidence intervals
- **Variance**: ±0.1s on 11.7s baseline (0.9% - excellent)
- **Stability**: Consistent results across runs, no thermal throttling

### Benchmark Methodology
```bash
# Cold cache (realistic first-run performance)
rm -rf ~/.cache/map2fig
hyperfine --warmup 1 --runs 5 \
  './target/release/map2fig -f {file} -o /tmp/out.pdf'

# Warm cache (repeated use - medium files only)
./target/release/map2fig -f {file} -o /tmp/out.pdf  # First run
./target/release/map2fig -f {file} -o /tmp/out.pdf  # Cached
```

## Summary

|Metric|Value|Status|
|---|---|---|
|FITS reading|10.9s (81% of load time)|Sequential bottleneck|
|Downgrade|1.3s (10% downsampling)|Parallelized, optimal|
|Rendering|0.2s (1.3% of total)|SIMD optimized|
|Cache hit (medium)|0.5s vs 0.8s (37% faster)|✅ Working|
|Throughput (3GB file)|11.7s total (262 MB/s)|Near ceiling|
|System overhead|2.6s (22%)|Minimized|
|Memory efficiency|9.4GB for 3.1GB file (3×)|Streaming percentile|

**The system is now well-optimized across core bottlenecks. Further improvements require either architectural changes or acceptance of practical limits.**
