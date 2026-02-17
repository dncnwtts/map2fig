# Current Performance Profile Analysis (Post Tier 1+2)

**Date:** February 16, 2025  
**Current Execution Time:** 10.94 seconds  
**Previous Baseline:** 22.58 seconds (51.5% improvement achieved)

---

## CPU Time Breakdown (from perf cycles:P)

### Application Code
```
24.57%  load_and_process_data() - main FITS reading/processing   
 0.07%  plot_mollweide_pdf()    - PDF rendering
 0.04%  sample_healpix_batch_simd() - Pixel sampling
 0.11%  ang2pix_ring()          - HEALPix coordinate conversion
```
**Total App Code: ~25%**

### Kernel/System
```
 9.02%  read syscalls           - FITS metadata reading (BufReader)
 8.90%  rep_movs_alternative    - Memory copy operation
 6.95%  page_fault (asm entry)  - Page fault handling
 5.30%  do_user_addr_fault      - Fault processing
 4.78%  handle_mm_fault         - Memory management
 4.60%  __handle_mm_fault       - MM subsystem
 4.44%  handle_pte_fault        - Page table entry
 4.11%  do_anonymous_page       - Page allocation
~2.58%+ allocation/LRU overhead
```
**Total Kernel/Memory: ~53%**

### Idle CPU
```
31.76%  CPU idle (swapper)      - Waiting for I/O or processing
```

---

## Root Cause Analysis

### 1. FITS Metadata Reading: 9.02% of Execution Time

**Issue:** `read_healpix_meta_cached()` still uses `BufReader`
```rust
let f = File::open(filename).ok()?;
let reader = BufReader::with_capacity(256 * 1024, f);  // ← BufReader
let mut fits = Fits::from_reader(reader);
```

**Why it's slow:**
- BufReader copies from kernel page cache to user buffer
- File header reading causes page faults (small reads, random access)
- Metadata is only ~10 KB but triggers multiple syscalls

**Context:** There IS a `read_healpix_meta_cached_mmap()` function defined, but it's only used when `MAP2FIX_USE_MMAP` environment variable is set.

**Fix Opportunity:** Make MmapFitsReader the default for metadata reading
- **Expected Gain:** 9% → 0.5% (8.5% reduction in execution time)  ← **VERY HIGH ROI**
- **Complexity:** 1-5 minute change
- **Risk:** Very low - already have mmap implementation

### 2. Memory Management Overhead: ~20% of Execution Time

**Issue:** Page faults (6.95% asm entry + 5.3% fault handling + ~8% other memory mgmt)

**Why it's happening:**
- `full_map = vec![f64::NEG_INFINITY; 12 * nside * nside]` allocates 192 MB
- Zero-initialization by kernel during first fault
- For sparse maps, allocating large array then only populating 10% is wasteful

**Breakdown of 20% overhead:**
```
6.95%  asm_exc_page_fault (entry)
5.30%  do_user_addr_fault (processing)
4.78%  handle_mm_fault
4.60%  __handle_mm_fault  
4.44%  handle_pte_fault
4.11%  do_anonymous_page (lazy allocation)
2.58%+ alloc_anon_folio, alloc_pages, LRU management
────────────────────────
~28% if we count full page fault handling chain
```

**Fix Opportunity #1: Lazy Initialization (Tier 3a - NEW!)**
- Don't zero-initialize the full array upfront
- Use `Vec::with_capacity()` + `push()` instead of `vec![; size]`
- Let OS fault in pages only as needed
- **Expected Gain:** Could reduce page faults by 30-40%
- **Estimated: 2-3% wall-clock improvement**

**Fix Opportunity #2: Reduce Vector Allocations (Tier 1.5 - FOLLOWUP)**
- The `pairs` vector still creates intermediate allocation
- Could use in-place updates or write directly
- **Expected Gain:** 1-2% wall-clock improvement

### 3. Memory Copy Overhead: 8.90% in `rep_movs_alternative`

**Issue:** Kernel bulk memory copy operation (part of page fault handling)

**Why it's happening:**
- Even with MmapFitsReader, initial page faults still need copy from page cache
- Less critical now that data I/O is optimized, but still present
- Self-time is 2.88%, parent overhead is 8.90%

**Current Status:** Acceptable for now (down from 18.76% in baseline)

---

## Ranking of Remaining Opportunities

### Tier 2b: Optimize Metadata Reading (***HIGHEST ROI***)
- **Opportunity:** 9.0% execution time
- **Effort:** 5 minutes
- **Expected Gain:** 8.5% wall-clock speedup
- **Complexity:** Trivial (change one function)
- **Risk:** None (mmap code already exists)
- **Result Target:** 10.94s → 10.0s

### Tier 3a: Lazy Vector Initialization (***HIGH ROI***)
- **Opportunity:** 20% execution time (page fault overhead)
- **Effort:** 30 minutes
- **Expected Gain:** 2-3% wall-clock speedup
- **Complexity:** Medium (need careful implementation)
- **Risk:** Low (affects only allocation path)
- **Result Target:** 10.0s → 9.7-9.8s

### Tier 3: Vectorize Scaling Loop (***MEDIUM ROI***)
- **Opportunity:** Currently embedded in load_and_process_data
- **Effort:** 1-2 hours
- **Expected Gain:** 1-2% wall-clock speedup
- **Complexity:** Medium (SIMD intrinsics)
- **Risk:** Low (isolated function)
- **Result Target:** 9.7-9.8s → 9.5-9.6s

### Tier 4: Parallel Block-Wise Loading (***MEDIUM ROI***)
- **Opportunity:** Concurrent processing of FITS blocks
- **Effort:** 3-4 hours
- **Expected Gain:** 2-4% wall-clock speedup
- **Complexity:** High (rayon + careful coordination)
- **Risk:** Medium (threading + buffer management)
- **Result Target:** 9.5-9.6s → 9.1-9.2s

### Tier 1.5: Reduce Pair Vector Allocation (***LOW ROI***)
- **Opportunity:** ~1-2% execution time
- **Effort:** 45 minutes
- **Expected Gain:** 1% wall-clock speedup
- **Complexity:** Medium
- **Risk:** Low
- **Result Target:** saves ~110ms

---

## Performance Scaling Summary

```
Current:            10.94s  (100%)
+ Metadata mmap:    10.00s  (91%) ← Tier 2b (IMMEDIATE)
+ Lazy init:         9.75s  (89%) ← Tier 3a (QUICK FOLLOW-UP)
+ Scaling SIMD:      9.60s  (88%) ← Tier 3 (MEDIUM)
+ Parallel blocks:   9.20s  (84%) ← Tier 4 (LONG TERM)
─────────────────────────────────
Final target:        9.2s   (84% of current)  [15.9% total from baseline]
```

**Total potential improvement from remaining tiers: 15.9% more** (on top of existing 51.5%)

---

## Recommended Next Step

### **Immediate Action: Tier 2b - Metadata Mmap**

**Why:**
- Highest ROI (9% for 5 minutes)
- No risk (code already exists)
- Sets up lazy initialization for next step
- Directly completes the I/O optimization

**Implementation:**
Simply change the default path in `read_healpix_meta_cached()`:
```rust
// Current (line 253-254):
if use_mmap {
    return read_healpix_meta_cached_mmap(filename);
}

// Proposed: FLIP THE DEFAULT
if !use_mmap {
    // keep old BufReader implementation for compatibility
} else {
    // (new condition is redundant - just remove the check)
}
// OR simpler: just always call mmap version
return read_healpix_meta_cached_mmap(filename);
```

**Validation:**
```bash
# Measure improvement
time ./target/release/map2fig -f tests/data/combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits -o /tmp/test.pdf
# Expected: ~10.0 seconds (8.5% faster)

# Verify correctness
sudo perf stat -e cache-references,cache-misses ./target/release/map2fig [...]
# Should see read syscalls drop from 9.02% to <1%
```

---

## Secondary Action Plan

After Tier 2b, recommend **Tier 3a** (Lazy Initialization):
- Addresses 20% page fault overhead  
- Requires careful vector handling
- 2-3% additional improvement
- Opens door for parallel loading

Then **Tier 3 or 4** depending on your priorities:
- Tier 3 (SIMD): Easier, incremental 1-2% gain
- Tier 4 (Parallel): Harder but 2-4% gain

---

## Key Insights

1. **I/O is the Bottleneck, Not CPU:** Metadata reading hidden 9% overhead
2. **Kernel Activity Matters:** 28% of time in page fault handling (can be reduced with better allocation patterns)
3. **Buffer Strategy Crucial:** MmapFitsReader already implemented but not used as default
4. **Synergistic Opportunities:** Tier 2b enables Tier 3a, which enables Tier 4

---

## Files Affected

- **Tier 2b:** `src/fits.rs` line 253-254 (1-2 line change)
- **Tier 3a:** `src/fits.rs` lines 95-155 (refactor allocation strategy)
- **Tier 3:** `src/pipeline.rs` lines 45-62 (SIMD scaling loop)
- **Tier 4:** `src/fits.rs` lines 95-155 (parallel loading)

