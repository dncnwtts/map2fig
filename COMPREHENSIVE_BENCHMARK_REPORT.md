# Comprehensive Benchmark Report: All Test Files

## Executive Summary

Benchmarked 8 FITS files ranging from 0.7 MB to 3.1 GB (3 GB nside=8192 map). **Discovered unexpected non-linear performance scaling**, particularly a 2.6× throughput drop on the 3 GB file.

| File | Size | Time | MB/s | Downgrade | Notes |
|------|------|------|------|-----------|-------|
| class_dr1_40GHz_skymap_n128 | 6.8 MB | 0.142s | 47.8 | N/A | Small file, no downgrade |
| mhat_0_00_n00512_2025W17_4B | 0.7 MB | 0.162s | 4.1 | N/A | Tiny file, slow |
| cosmoglobe_clipped | 24.0 MB | 0.358s | 67.0 | N/A | Small, no downgrade |
| cosmoglobe_DIRBE_06_I_n00512_DR2 | 72.0 MB | 0.424s | 169.6 | N/A | No downgrade |
| cosmoglobe_DIRBE_10_I_n00512_DR2_v3.1 | 72.0 MB | 0.416s | 173.1 | N/A | No downgrade |
| npipe_nodip | 192.0 MB | 0.912s | 210.4 | 0.073s (7.9%) | Has downgrade! |
| npipe6v20_217_map_K | 576.0 MB | 0.965s | 596.9 | 0.093s (9.7%) | **Peak throughput** |
| **combined_map_95GHz_nside8192** | **3072.0 MB** | **13.337s** | **230.3** | **1.439s (10.8%)** | **⚠️ Anomaly detected** |

## Key Findings

### 1. Non-Linear Performance Scaling 🚨

**Critical Observation:** The 3 GB file is **2.6× slower per MB** than the 576 MB file:

```
576 MB file:   0.965s  → 596.9 MB/s
3072 MB file: 13.337s  → 230.3 MB/s

Ratio: 596.9 / 230.3 = 2.58× slowdown per MB
```

If the 3 GB file scaled linearly with the 576 MB file:
- Expected time: 13.337s / 2.6 = **5.1s** (vs actual 13.337s)
- Expected MB/s: 596.9 MB/s (constant)

**Actual time is 2.6× slower than expected.**

### 2. Downgrade Cost Analysis

Files with nside > 128 trigger downgrade operation:

| File | nside | Downgrade Time | % of Total | Pixels Downgraded |
|------|-------|-----------------|-----------|-------------------|
| npipe_nodip | 512 | 0.073s | 7.9% | 12M |
| npipe6v20_217_map_K | 512 | 0.093s | 9.7% | 12M |
| combined_map_95GHz_nside8192 | 8192 | 1.439s | 10.8% | 3.1B |

**Interesting pattern:** Downgrade cost scales with nside (input size), not output size. nside=8192→512 downsamples 3.1B→12M pixels but takes 1.4s because it processes all 3.1B input pixels.

### 3. Possible Explanations for 3 GB Anomaly

#### Hypothesis A: Memory Bandwidth Saturation 
- Smaller files fit in CPU L3 cache (20 MB)
- 3 GB file causes main memory traffic
- Memory bandwidth becomes bottleneck on sustained operations
- Cairo PDF rendering particularly sensitive to memory access patterns

#### Hypothesis B: Cache Misses / TLB Pressure
- 3 GB file involves ~750K pages
- TLB (Translation Lookaside Buffer) has ~256-512 entries
- High TLB miss rate on large allocations
- Cairo rendering loop more sensitive to cache misses than FITS reading

#### Hypothesis C: Cairo Rasterization Inefficiency
- PDF rendering via Cairo is known slow path
- Large pixel buffer (3.1B pixels = 3.1B float32 = 12 GB internally)
- Cairo's pixel sink implementation has sublinear performance
- PNG would likely be much faster (image crate is optimized)

#### Hypothesis D: Downgrade Operation Bottleneck
- 1.439s spent downsampling 3.1B pixels
- This is sequential work (not parallelized fully)
- Creates data dependency chains for subsequent operations

### 4. Throughput Analysis by File Size

```
File Size     Throughput    Trend Pattern
────────────────────────────────────────
0.7 MB        4.1 MB/s      🟡 Tiny file overhead
6.8 MB        47.8 MB/s     🟡 Still slow
24 MB         67.0 MB/s     🔵 Improving
72 MB         170-173 MB/s  🔵 Good
192 MB        210.4 MB/s    🟢 High
576 MB        596.9 MB/s    🟢 🟢 Peak!
3072 MB       230.3 MB/s    🔴 Collapsed!
```

**Pattern:** Linear improvement from 1-600 MB, then dramatic collapse at 3 GB.

## Recommendations for Investigation

### Priority 1: Determine Which Operation Causes Slowdown
Test the 3 GB file with different output formats:

```bash
# Test with PNG (image crate - should be faster)
./target/release/map2fig -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits -o /tmp/test.png

# Test with PDF (Cairo - known slow)
./target/release/map2fig -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits -o /tmp/test.pdf

# Profile with perf to see which function dominates
perf record -g ./target/release/map2fig -f ... -o /tmp/test.pdf
```

### Priority 2: Downgrade Operation Optimization
The 1.4s downgrade could be optimized further:
- Current: Chunked parallelization at 10K pixels per chunk
- Possible: Increase chunk size to 100K for nside=8192 (less task overhead)
- Possible: Cache coordinates if large file detected

### Priority 3: Memory Efficiency
Check if memory allocation strategy changes with file size:
- Verify vector pre-allocation sizes
- Check if Cairo is allocating separate buffers per operation

## Performance Expectations Going Forward

**By file size:**
- < 100 MB: 50-200 MB/s (startup overhead dominates)
- 100-600 MB: 200-600 MB/s (linear scaling, hardware optimal)
- > 1 GB: **Unknown** (appears to degrade, needs investigation)

**Optimization strategy:**
1. ✅ Tier 1: Direct float32 reading (DONE - 3.4× speedup)
2. ✅ Tier 1.1: Eliminate intermediate buffers (DONE - 30-35% speedup)
3. ✅ Tier 1.2: Streaming percentile (DONE - 79% memory reduction)
4. ✅ Chunked parallelization (DONE - 7.3% speedup)
5. ⏳ Tier 2: Identify and fix 3 GB performance wall
6. ⏳ Tier 3: SIMD Mollweide projection (15-25% estimate)

## Conclusion

System performs excellently on medium files (100-600 MB) with linear scaling up to ~600 MB/s. However, the 3 GB file shows a **2.6× performance regression per megabyte**, suggesting a fundamental bottleneck that only appears at large scales (architecture limit, memory bandwidth, or algorithmic change needed).

**Next action:** Profile the 3 GB rendering to identify exact bottleneck location and whether it's Cairo, downgrade, or memory-related.
