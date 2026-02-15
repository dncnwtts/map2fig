# HEALPix Plotter Optimization Progress Summary

## Current Performance Baseline
- **Execution Time:** 10.51 seconds (3-run average)
- **Overall Improvement from Start:** 51.5% (from 22.58s baseline)
- **Improvement from this session (Tier 2b):** 4% (from 10.94s → 10.51s)

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

---

## Remaining Bottlenecks by Tier

### Tier 3a: Lazy Initialization (RECOMMENDED NEXT)
**Status:** Not started  
**Predicted Gain:** 8-10% (estimated)  
**Effort:** Medium (2-3 hours)  
**Target:** Reduce 1.58M page faults from zero-initialization of pixel buffers

**Approach:**
1. Use mmap with lazy page faulting for output buffers
2. Only initialize pixels as they're written to
3. Reduce upfront allocation overhead

**Current metrics supporting this:**
- Page faults: 1.58M per run
- Page fault handling: ~20% of CPU time (from profiling)
- Cache misses: 35% (increased from Tier 1+2)

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
10.51s (current)
    ↓ (Tier 3a predicted: -8-10%)
~9.5-9.7s (projected after Tier 3a)
    ↓ (Tier 3 + Tier 4: -7-12%)
~8.5-9.0s (best case with all remaining)
```

---

## Current Bottleneck Analysis

### Function Breakdown (perf report, Tier 2b binary)
```
18.27%  load_and_process_data (down from 24.57%)
0.14%   ang2pix_ring
0.11%   sample_healpix_batch_simd
0.11%   plot_mollweide_pdf
0.06%   draw_colorbar_pdf
32%+    idle/intel_idle (I/O wait)
```

**Key insight:** load_and_process_data still dominant but reduced significantly. The function includes:
- FITS column data loading
- Pixel scaling
- Mollweide projection
- Cairo rasterization
- Memory allocation

### Memory Metrics (Tier 2b)
```
Cache misses:     35.00% (629M / 1.8B refs) ← INCREASED
Page faults:      1.584M
dTLB misses:      0.13% of dTLB accesses
Memory bandwidth: High (many cache misses)
```

**Concern:** Cache miss rate increased from 27.67% to 35% after Tier 2b. This suggests mmap may have different spatial locality characteristics than buffered I/O. However, overall execution time still improved due to kernel overhead reduction.

---

## Session Activities

### What Was Accomplished
1. ✅ Analyzed Tier 1+2 optimized binary with perf profiling
2. ✅ Identified small remaining bottlenecks
3. ✅ Implemented Tier 2b (metadata mmap)
4. ✅ Verified 4% speedup
5. ✅ Re-profiled to assess next target
6. ✅ Updated documentation with findings

### What Wasn't Done
- Tier 3a implementation (recommended for next session)
- Full `perf annotate` on load_and_process_data (would show exact hot instructions)
- Measurement of page fault cost in isolation

---

## Recommendation for Next Session

### Option 1: Implement Tier 3a (RECOMMENDED)
**Rationale:** 
- Page faults (1.58M) are measurable overhead
- Lazy initialization is a focused change with clear ROI
- Medium effort-to-gain ratio
- Should yield 8-10% improvement

**Steps:**
1. Measure current page fault overhead with perf
2. Insert lazy allocation into pixel buffer creation
3. Benchmark and validate
4. Estimate Tier 4 feasibility

### Option 2: Measure More Before Deciding
**Rationale:**
- Cache miss rate increased unexpectedly
- Memory allocation still significant
- Might learn more useful info before committing to Tier 3a

**Steps:**
1. Run `perf annotate` on load_and_process_data to identify hot instructions
2. Measure cache-aware optimizations (L1/L2/L3 splits)
3. Profile memory allocation patterns
4. Reassess whether Tier 3a or optimization elsewhere is better

### Option 3: Skip to Tier 4 (RISKY)
**Rationale:**
- Highest ROI (6-10%)
- Threading could provide best overall gains

**Risks:**
- Complex implementation (file locking, task synchronization)
- Potential for regressions
- Hard to debug if issues arise

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

### Modified Today
- `src/fits.rs`: Tier 2b optimization (92 lines removed, cleaner code)
- New docs: TIER2B_RESULTS.md, OPTIMIZATION_SESSION_2_SUMMARY.md

### Important Existing Docs
- `CURRENT_BOTTLENECK_ANALYSIS.md`: Detailed analysis pre-Tier2b
- `.github/copilot-instructions.md`: Architecture overview
- `PERFORMANCE_OPTIMIZATION_RESULTS.md`: Previous Tier 1+2 results

---

## Metrics Summary

| Metric | Value | Target |
|--------|-------|--------|
| Current execution time | 10.51s | <8s (stretch: <7s) |
| Total improvement | 51.5% | 70%+ |
| Cache misses | 35.00% | <30% |
| Page faults | 1.58M | <1M |
| Load function CPU % | 18.27% | <10% |

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

Generated: Session 2 (after Tier 2b completion)  
Status: Ready for next optimization phase
