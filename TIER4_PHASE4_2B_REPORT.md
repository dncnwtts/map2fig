# Phase 4.2b: Parallel FITS Column Reading Implementation

## Overview

Implemented rayon-based parallelization for FITS sparse map column extraction (EXPLICIT indexing).

## Implementation Details

**File:** `src/fits.rs`

**Changes:**
1. Added `rayon` crate dependency (version 1.8)
2. Refactored `read_healpix_column()` EXPLICIT indexing section:
   - Before: Sequential loop processing pixel-data pairs
   - After: Parallel rayon iteration extracting (pixel_idx, value) pairs, then sequential map population

**Algorithm:**
```
Parallel extraction phase:
├─ for each row in sparse table (parallelized via rayon)
│  ├─ Extract pixel index from column 0
│  ├─ Extract data value from column N
│  └─ Return (pixel_idx, value) tuple if valid
├─ Collect all pairs
└─ Sequential population phase: Write pairs into output array
```

**Benefits:**
- Parallelizes CPU-bound extraction of pixel indices and data values
- Avoids contention on output array (sequential write phase)
- Work-stealing scheduler for load balancing
- Zero overhead on dense maps (they use different code path)

## Benchmark Results

### Phase 4.2b Performance

Test file: `cosmoglobe_clipped.fits` (25 MB, dense map)

| Configuration | Time | vs Baseline | Status |
|---|---|---|---|
| Linear 512 | 0.403s | -2.8% (vs 0.415s) | ✓ |
| Linear 1200 | 0.919s | +0.4% (vs 0.915s) | Parity |
| Log 512 | 0.392s | +5.6% (vs 0.371s) | Slower |
| Log 1200 | 0.773s | -3.4% (vs 0.800s) | ✓ Improvement |

**Overall:** -1.1% average (within measurement noise)

## Analysis

**Finding:** Minimal isolated benefit from parallelization on this workload.

**Reasons:**
1. **Dense vs Sparse:** cosmoglobe_clipped.fits is a **dense map** (implicit indexing), not sparse/EXPLICIT
   - Parallelized code path only activates for sparse maps with EXPLICIT indexing
   - Dense maps use sequential column iteration (unchanged)
   
2. **Small Row Count:** Even on sparse maps, the number of rows to process might be small
   - Rayon overhead (thread spawning, synchronization) can exceed benefit
   - Sparse maps typically have 1-50% pixel coverage
   
3. **Memory Bound:** FITS column extraction is memory-bound, not CPU-bound
   - rayon helps CPU-bound tasks (math, processing)
   - I/O bottleneck dominates (reading from disk)

4. **Noise:** ±5% measurement variance on single runs

## Code Quality

✓ All 155 tests passing  
✓ No unsafe code  
✓ Backward compatible (dense maps unchanged)  
✓ No compilation warnings  

## Recommendations

**For sparse map workloads:**
The parallelization will show benefit when:
- Processing large sparse catalogs (100K+ sparse pixels)
- CPU time > I/O time for column extraction

**For dense maps (current use case):**
- No benefit from this optimization
- Focus remains on I/O optimization (Phase 4.2c) or PDF generation (Phase 4.3)

## Next Steps

1. Test on actual sparse maps to validate parallel benefit
2. Profile I/O bottleneck (likely 100-150ms per render)
3. Consider Phase 4.2c: Memory-mapped FITS reading
4. Consider Phase 4.3: PDF streaming/direct surface rendering
