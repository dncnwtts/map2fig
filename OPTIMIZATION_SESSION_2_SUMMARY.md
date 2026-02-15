# HEALPix Plotter Optimization Progress Summary

## Current Performance Baseline
- **Execution Time:** 10.14 seconds (3-run average)
- **Overall Improvement from Start:** 55.1% (from 22.58s baseline)
- **Improvement from this session (Tier 2b + Tier 3a):** 7.3% (from 10.94s → 10.14s)

---

## Completed Optimizations ✅

### Tier 1: Optimized Data Loading (COMPLETED)
- **Change:** Eliminated Vec<DataValue> intermediate buffer in sparse FITS column extraction
- **Gain:** 30-35% improvement
- **Details:** Direct index → pixel value mapping, reduced memory traffic
- **Commit:** 427b21c

### Tier 1.5: Enabled MmapFitsReader (COMPLETED)
- **Change:** Memory-mapped I/O for column data
- **Gain:** 20-21% improvement  
- **Details:** Eliminated kernel memcpy, page cache reuse
- **Commit:** 427b21c (part of same commit)

### Tier 2b: Metadata Mmap I/O (COMPLETED this session)
- **Change:** Replaced BufReader with memory-mapped I/O for FITS metadata
- **Gain:** 4% improvement (6-point reduction in function CPU %)
- **Details:** Syscall/page fault overhead elimination
- **Commit:** 7180b8b

### Tier 3a: Lazy Pixel Buffer Initialization (COMPLETED this session)
- **Change:** Skip kernel zero-initialization of image buffers via unsafe Vec sizing
- **Gain:** 3.6% improvement (371ms saved)
- **Details:** Improved cache locality, not page fault reduction
- **Commit:** 53ad008

---

## Remaining Bottlenecks by Tier

### Tier 3: Vectorize Scaling Loop (RECOMMENDED NEXT)
**Status:** Not started  
**Predicted Gain:** 1-2% (modest, but easy)  
**Effort:** Low (1 hour)  
**Target:** Use SIMD for min/max/scaling computation in scale_value()

### Tier 3b: Cache-Aware Access Patterns (ALTERNATIVE)
**Status:** Not started  
**Predicted Gain:** 5-8% (if memory bandwidth is bottleneck)  
**Effort:** High (3-4 hours)  
**Target:** Reorder Mollweide projection loops to improve spatial locality

---

### Tier 3: Vectorize Scaling Loop
**Status:** Not started  
**Predicted Gain:** 1-2% (low ROI)  
**Effort:** Low (1 hour)  
**Target:** Use SIMD for min/max/scaling computation in scale_value()

---

### Tier 4: Parallel Block-Wise Loading
**Status:** Not started  
**Predicted Gain:** 6-10% (high ROI but risky)  
**Effort:** High (5+ hours, threading complexity)  
**Target:** Load FITS data in parallel chunks instead of sequential

---

## Performance Progression

```
Baseline (22.58s)
    ↓ (Tier 1: -30-35%)
10.94s (after data loading optimization)
    ↓ (Tier 2b: -4%)
10.51s (after metadata mmap)
    ↓ (Tier 3a: -3.6%)
10.14s (current after lazy buffer init)
    ↓ (Tier 3 + Tier 3b: -6-10% predicted)
~9.2-9.6s (best case after remaining tiers)
```

---

## Current Bottleneck Analysis

### Function Breakdown (perf report, Tier 3a binary)
```
26.35%  load_and_process_data (up from 18.27%, but fewer absolute cycles)
0.17%   sample_healpix_batch_simd
0.16%   plot_mollweide_pdf
0.08%   pixel_to_ang_batch
0.07%   render_projection_to_grid
0.04%   draw_colorbar_pdf
32%+    idle/intel_idle (I/O wait)
```

**Key Insight:** load_and_process_data percentage increased because total cycle count decreased more than the function's cycles. Absolute time is still highest consumer.

### Memory Metrics (Tier 3a)
```
Cache misses:     31.85% (614M / 1.9B refs) ← IMPROVED from 35%
Instructions/Cycle: 2.05  ← IMPROVED from 2.02
Page faults:      1.584M ← UNCHANGED (not the bottleneck)
dTLB misses:      0.13% of dTLB accesses ← unchanged
Memory bandwidth: Still high (cache miss dependent)
```

**Key Finding:** Lazy initialization improved cache efficiency, but page faults remain unchanged because we write to 100% of the buffer. True lazy allocation (mmap-based sparse files) not worth pursuing.

---

## Session Activities

### What Was Accomplished
1. ✅ Analyzed Tier 1+2 optimized binary with perf profiling
2. ✅ Identified small remaining bottlenecks
3. ✅ Implemented Tier 2b (metadata mmap) - 4% speedup
4. ✅ Verified Tier 2b improvement
5. ✅ Re-profiled to assess next target
6. ✅ Implemented Tier 3a (lazy buffer init) - 3.6% speedup
7. ✅ Verified Tier 3a improvement with detailed perf stat
8. ✅ Discovered page faults are NOT the bottleneck (cache is)
9. ✅ Updated documentation with findings
10. ✅ **Total this session: 7.3% improvement (10.94s → 10.14s)**

### What Wasn't Done
- Tier 3 SIMD optimization (recommended for next time)
- Tier 3b cache-aware loops (higher complexity)
- Full `perf c2c` cache contention analysis

---

## Recommendation for Next Session

### Option 1: Tier 3 - SIMD Math (RECOMMENDED - LOW RISK)
**Rationale:** 
- Quick win with minimal risk
- Low effort (1-2 hours)
- Modest gains (1-2%) but guarantees success
- Good stepping stone before higher-risk optimizations

**Steps:**
1. Profile scale_value() to confirm it's still hot
2. Add SIMD-based min/max/comparison operations
3. Benchmark and validate
4. Move to next tier

### Option 2: Tier 3b - Cache-Aware Loops (HIGHER RISK/REWARD)
**Rationale:**
- Cache miss rate is now limiting factor (31.85%)
- Could provide 5-8% gain if successful
- Requires understanding memory access patterns

**Steps:**
1. Profile cache miss sources with `perf c2c` (cache-to-cache)
2. Identify hottest innermost loops
3. Reorder Mollweide projection for better locality
4. Benchmark extensively

### Option 3: Measure Before Deciding
**Rationale:**
- We've achieved 55% total improvement (22.58s → 10.14s)
- 31.85% cache miss rate is concerning
- Might learn useful info with more detailed profiling

**Steps:**
1. Run `perf c2c` to find cache contention points
2. Profile with `-e LLC-loads,LLC-load-misses`
3. Assess whether memory bandwidth or algorithm is bottleneck
4. Determine if Tier 3 or 3b is better ROI

---

## Known Issues & Caveats

1. **Cache miss rate increased:** After mmap transition, cache misses went 27.67% → 35%. This is unexpected and warrants investigation, but didn't prevent overall speedup.

2. **Actual < predicted gains:** Tier 2b achieved 4% instead of predicted 9%. Likely due to:
   - File system cache already mitigating syscall overhead
   - Other bottlenecks now more prominent
   - Profiling prediction was conservative

3. **Page faults still high:** 1.58M page faults per run suggests memory access pattern is not optimal. Lazy initialization (Tier 3a) could help.

4. **Limited margin remaining:** We're at ~10.5s from 22.58s baseline. Further gains will be incremental (1-2% per optimization).

---

## File Status

### Modified This Session
- `src/render/mod.rs`: Added lazy allocation helper
- `src/plot/mollweide.rs`: Tier 3a optimization
- `src/render/pdf.rs`: Tier 3a optimization
- `OPTIMIZATION_SESSION_2_SUMMARY.md`: This file (updated)
- **New Docs:** TIER2B_RESULTS.md, TIER3A_RESULTS.md

### Important Existing Docs
- `CURRENT_BOTTLENECK_ANALYSIS.md`: Pre-optimization detailed analysis
- `.github/copilot-instructions.md`: Architecture overview
- `PERFORMANCE_OPTIMIZATION_RESULTS.md`: Previous Tier 1+2 results

---

## Metrics Summary

| Metric | Baseline | Current | Sessions Done | Target |
|--------|----------|---------|---------------|--------|
| Execution time | 22.58s | 10.14s | Tier 1+2+3a | <8s |
| Overall improvement | — | 55.1% | — | 70%+ |
| Cache misses | — | 31.85% | — | <25% |
| Page faults | — | 1.58M | — | <1M (stretch) |
| Load func CPU % | — | 26.35% | — | <15% |
| IPC (insn/cycle) | — | 2.05 | — | >2.2 |

---

## Next Profiling Commands (Ready to Use)

```bash
# Re-profile for comparison
sudo perf record -F 99 -g --all-cpus -o perf_tier3a.data \
    ./target/release/map2fig -f tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
    -o /tmp/perf_tier3a.pdf

# Measure page fault cost
sudo perf stat -e page-faults,dTLB-loads,dTLB-load-misses \
    ./target/release/map2fig -f tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
    -o /tmp/test.pdf

# Assembly-level analysis
sudo perf annotate -i perf_tier2b.data -s "map2fig::pipeline::load_and_process_data"
```

---

Generated: Session 2 (after Tier 3a completion)  
Status: Ready for next optimization phase (Tier 3 or 3b recommended)
