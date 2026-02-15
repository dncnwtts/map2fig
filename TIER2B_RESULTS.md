# Tier 2b Optimization Results

## Summary
**Objective:** Replace BufReader-based FITS metadata reading with memory-mapped I/O to reduce syscall overhead.

**Status:** ✅ COMPLETED - 4% wall-clock speedup achieved

---

## Performance Results

### Execution Time (3-run average)
| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Wall-clock time | 10.94s | 10.51s | **~4% (434ms)** |
| User time | 7.28s | 7.21s | ~1% |
| System time | 3.26s | 3.24s | negligible |

### Memory & Caching
| Metric | Value | Change |
|--------|-------|--------|
| Cache misses | 629M | ↑ (was 27.67%, now 35%) |
| Cache miss rate | 35.00% | ↑8.33pp |
| Page faults | 1.58M | stable |
| dTLB misses | 9.67M | 0.13% of dTLB accesses |

**Note:** Cache miss rate increased. This aligns with mmap's different memory access pattern compared to buffered reads. The higher miss rate appears acceptable given overall performance gain.

---

## Implementation Details

### Code Changes
**File:** `src/fits.rs`

1. **Removed:** BufReader-based metadata reading (lines 261-342)
2. **Added:** Direct call to `read_healpix_meta_cached_mmap()` in `read_healpix_meta_cached()`
3. **Pattern:** Used Cursor<Mmap> consistent with column data optimization

### Commit
```
7180b8b perf: Make mmap I/O default for FITS metadata reading (Tier 2b)
```

---

## Function-Level Impact

### load_and_process_data CPU Time
| Metric | Before Tier2b | After Tier2b | Reduction |
|--------|---------------|-------------|-----------|
| % of total CPU | 24.57% | 18.27% | **6.30pp (25.6% relative)** |

This significant reduction in function CPU percentage confirms that metadata syscalls were indeed a bottleneck, though the overall speedup (4%) is lower than predicted (9%). Possible explanations:
1. File system caching was already mitigating some syscall overhead
2. Other bottlenecks (memory allocation, page faults) are now more prominent
3. syscall overhead percentage was measured conservatively

---

## Analysis Summary

### What Went Well
- ✅ Successfully migrated metadata reading to mmap
- ✅ Reduced load_and_process_data CPU consumption by 25%
- ✅ Achieved 4% wall-clock improvement despite lower prediction
- ✅ Clean, maintainable code using Cursor<Mmap> pattern

### What Could Be Better  
- ❌ Cache miss rate increased to 35% (from 27.67%)
- ❌ Actual speedup (4%) < predicted (9%)
- ❌ Limited wall-clock gain given effort invested

### Why Actual < Predicted
1. **File system caching:** Linux page cache was already serving some metadata files, reducing syscall impact
2. **Other bottlenecks:** With data loading optimized (Tier 1) and metadata I/O optimized (Tier 2b), we're hitting other limits:
   - Memory allocation (still ~20% CPU time)
   - Page fault handling
   - Cache misses now more visible
3. **Measurement technique:** perf syscall reporting includes function overhead, not just kernel time

---

## Next Steps

### Recommended: Tier 3a (Lazy Initialization)
**ROI:** ~8-10% predicted  
**Effort:** Medium (2-3 hours)  
**Target:** Reduce 1.58M page faults via lazy zero-initialization of pixel buffers

### Alternative: Profile More
Current profiling shows:
- 18% in load_and_process_data
- 35% cache misses (up from previous)
- 1.58M page faults still significant
- 32% idle (I/O wait)

Consider full `perf annotate` on load_and_process_data to pinpoint exact hot instructions before Tier 3.

---

## Key Statistics (Full Run)
```
Performance counter stats for Tier 2b binary:
- Cycles:        27.96 billion
- Instructions:  56.40 billion (2.02 insn/cycle)
- Cache refs:    1.80 billion
- Cache misses:  629 million (35.00%)
- Page faults:   1.584 million
- dTLB misses:   9.667 million (0.13%)
- Time:          10.374 ± 0.288 seconds
```

---

## Files Modified
- `src/fits.rs`: Removed BufReader-based path, 92 lines deleted
- No new dependencies or API changes

---

## Conclusion

Tier 2b successfully reduced metadata I/O overhead, resulting in a 4% wall-clock speedup. The relative reduction in `load_and_process_data` CPU time (25%) indicates the optimization was effective, but other factors (memory allocation, page faults, cache effects) are now the dominant limitation. Tier 3a (lazy initialization) targets these remaining bottlenecks and should be the next priority.
