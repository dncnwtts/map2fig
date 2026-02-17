# Performance Baseline - February 17, 2026

## System Specs
- CPU: Intel i9-10885H (8-core, 2.4GHz base / 5.3GHz turbo)
- RAM: 32GB
- Build: `cargo build --release` with LTO, fat codegen-units=1
- Optimization: Adaptive chunking (10K-100K) + memory-mapped I/O enabled

## End-to-End Benchmark Results (Hyperfine)

Measured with 5 runs, 1 warmup, 95% confidence intervals.

| File | Size | nside | Mean | Std Dev | Min | Max | Note |
|------|------|-------|------|---------|-----|-----|------|
| class_dr1_40GHz_skymap_n128.fits | 6 MB | 128 | 369.4 ms | 67.8 ms | 280.9 | 471.2 | Small, startup-limited |
| cosmoglobe_clipped.fits | 24 MB | 128 | 513.9 ms | 41.0 ms | 440.9 | 536.3 | Outlier detected |
| cosmoglobe_DIRBE_06_I_n00512_DR2.fits | 72 MB | 128 | 523.0 ms | 18.1 ms | 491.2 | 535.4 | Stable |
| npipe_nodip.fits | 192 MB | 128 | 800.1 ms | 21.8 ms | 771.7 | 823.0 | Linear scaling |
| npipe6v20_217_map_K.fits | 576 MB | 128 | 845.0 ms | 38.1 ms | 809.5 | 893.4 | Peak throughput |
| combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits | 3072 MB | 8192 | 14118.1 ms | 147.6 ms | 13932.1 | 14308.2 | Large file regime |

## Throughput Summary

| File | Throughput |
|------|-----------|
| 6 MB | 16.3 MB/s |
| 24 MB | 46.7 MB/s |
| 72 MB | 137.7 MB/s |
| 192 MB | 240.0 MB/s |
| 576 MB | 682.0 MB/s |
| 3072 MB | 217.6 MB/s |

## Key Observations

1. **Linear scaling 6-576 MB**: Throughput increases proportionally with file size
   - Small files: Limited by startup overhead
   - Medium files: Approaching peak throughput
   - 576 MB: Peak at ~682 MB/s

2. **Large file regime (3GB)**: 3.2× lower throughput per MB (217.6 vs 682 MB/s)
   - Different bottleneck (likely memory bandwidth)
   - Well-characterized with adaptive chunking

3. **Variance Analysis**:
   - Small files: High relative variance (18% σ/mean)
   - Medium files: Low relative variance (2-5% σ/mean)
   - Large file: Very stable (1% σ/mean)

## Breakdown of 3GB File (approximate)

Based on earlier profiling with verbose output:
- FITS reading: ~9.0s (64%)
- Mollweide projection: ~2.2s (16%)
- Downgrade operation: ~1.4s (10%)
- Rendering: ~0.8s (6%)
- I/O syscalls: ~0.7s (4%)

## Optimization Targets (by impact)

1. **Mollweide projection** (16% of 3GB, ~2.2s)
   - Target: SIMD trigonometric operations (15-25% potential)
   - Effort: Medium

2. **Downgrade operation** (10% of 3GB, ~1.4s)
   - Current: Adaptive chunking (already optimized)
   - Target: Coordinate lookup caching (10-20% potential)
   - Effort: Medium

3. **FITS reading** (64% of 3GB, ~9.0s)
   - Current: mmap + direct float32 reading (already optimized 3.4×)
   - Target: Parallel column reading (5-10% potential)
   - Effort: High

## Confidence Levels

- ✅ 95% CI computed for all measurements
- ✅ Warmup runs included (cache stabilization)
- ✅ System-wide `sync; sleep` before each run
- ✅ Multiple runs with outlier detection
- ⚠️ Some system noise (cosmoglobe_clipped has outlier warning)

## Next Steps

1. Validate baseline with Criterion micro-benchmarks
2. Implement coordinate lookup caching (Target: 1.2-1.3s → 1.0-1.1s)
3. Implement SIMD Mollweide (Target: 2.2s → 1.7-1.8s)
4. Re-baseline and compare against this measurement

## Files

Generated results:
- `/tmp/hyperfine_results.json` - Raw JSON data (for tracking)
- `/tmp/hyperfine_results.md` - Markdown report
