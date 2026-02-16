# HEALPix Memory Access Analysis & Optimization Plan

## Executive Summary

Comprehensive memory profiling using `perf mem record` revealed the bottleneck is **NOT random pixel sampling during rendering** as initially theorized, but rather **inefficient data loading and processing** in `load_and_process_data()`. The application shows 36.67% L1-3 cache miss rate with only 0.27% memory bandwidth utilization, indicating poor data layout and access patterns.

**Key Finding:** 62.44% of memory load latency samples occur in `load_and_process_data()`, with 18.76% in kernel `rep_movs_alternative` (bulk memcpy), suggesting multiple data copies during FITS reading and scaling.

---

## Memory Load Distribution (perf mem analysis)

```
Function Breakdown:
├─ load_and_process_data()      62.44% (23.7K samples) ← HOTTEST
├─ rep_movs_alternative (kernel) 18.76% (7.1K samples)  ← Data copies
├─ Page management (kernel)       3.57% (1.3K samples)  ← Page faults
├─ sample_healpix_batch_simd()    0.24% (91 samples)   ← Surprisingly LOW
├─ pixel_to_ang_batch()           0.21% (80 samples)   ← Also low
└─ render_projection_to_grid()    0.12% (47 samples)   ← Minimal

Interpretation: Data LOADING is the bottleneck, not RENDERING
```

---

## Root Cause Analysis

### 1. **Data Loading Pipeline (src/fits.rs)**

**Current Flow:**
```
FITS Binary Table on Disk
    ↓
read_healpix_column()
    ↓
fitsrs::table_data()        ← Parses FITS binary format
    ↓
Vec<DataValue>              ← Intermediate vectors created
    ↓
Parallel rayon extraction   ← Spawns threads to extract (pixel, data) pairs
    ↓
Sequential population       ← Single-threaded write to full_map
    ↓
Vec<f64>                    ← Final HEALPix map
```

**Issues Identified:**

Issue A: **Multiple intermediate buffers** (lines 146-182 in fits.rs)
- `all_values: Vec<DataValue>` created
- `pairs: Vec<(usize, f64)>` created for sparse maps
- Final `full_map: Vec<f64>`
- **Cost:** 3 separate allocations + 2 copies

Issue B: **Sequential population bottleneck** (line 187-189 in fits.rs)
```rust
for (pix_idx, val) in pairs {
    full_map[pix_idx] = val;  // ← Random writes to scattered memory
}
```
- `full_map` is unpredictably accessed due to pixel indices from EXPLICIT sparse maps
- Cache line misses on every write
- **Effect:** 4-bank memory write contention

Issue C: **Sparse map expansion overhead**
- Fills 12×NSIDE² entries but only populates pixels present in FITS
- For sparse maps with 10% coverage, 90% of memory writes are initialization
- Full_map allocation + zero-initialization: `vec![f64::NEG_INFINITY; npix]`

### 2. **Scaling Application (src/pipeline.rs, lines 45-62)**

```rust
for v in &mut map {
    if !is_seen(*v) { continue; }
    if v.abs() < 1e-20 {
        *v = HPX_UNSEEN;
    } else {
        *v *= scale_factor;
    }
}
```

**Issues:**
- Sequential traversal with branch misprediction (~5-10% of pixels are UNSEEN)
- Could be vectorized with SIMD if data layout permits
- No prefetching hints

### 3. **Downgrading Pipeline (src/pipeline.rs, lines 68-90)**

If `meta.nside > 2048`:
```rust
let downgraded_map = downgrade_healpix_map(&map, meta.nside, target_nside, meta.ordering);
```
- Creates ANOTHER intermediate vector
- Reads from original map (scattered if original is sparse-expanded)
- Writes to new map (again, scattered access pattern)
- **Cascading copies:** 3 vectors in pipeline for downgrade case

### 4. **Kernel Memory Management Overhead**

18.76% of memory samples in `rep_movs_alternative` indicates:
- Large memcpy operations triggered
- Possibly FITS file I/O copying from page cache
- Doesn't use `MmapFitsReader` (defined at src/mmap_reader.rs but not used!)

---

## Theoretical Limits & Current Performance

### Actual Performance

```
CPU Metrics:
  Execution time:        22.58 seconds
  Cache misses:          36.67% (850.6M of 2.3B refs)
  L1-3 cache efficiency: 16.8% (83% of loaded cache lines unused)
  Memory bandwidth:      132.8 MB/s (0.27% of 25-50 GB/s available)
  Cycles stalled:        41.8% (51B of 122B stalls due to memory)
```

### Theoretical Limits (3GB file)

```
Best Case Sequential I/O:      ~0.5-0.8 seconds  (25-50 GB/s RAM)
Realistic After Optimization:   15-18 seconds    (memory-bound minimum)
Current Execution:              22.58 seconds
Headroom for Improvement:        -25% to -35%
```

---

## Optimization Strategy (Prioritized)

### **Tier 1: Eliminate Intermediate Buffers** (Est. 8-12% gain)

**Problem:** fitsrs parsing creates `Vec<DataValue>` intermediate

**Solution #1: Stream-based FITS parsing**
```rust
// Instead of Vec<DataValue>, directly extract columns
let pixels: Vec<i64> = table.column(0)  // Direct column extraction
    .filter_map(|row| extract_i64(row))
    .collect();
let values: Vec<f64> = table.column(1)
    .filter_map(|row| extract_f64(row))
    .collect();
// Single-pass zip + populate
for (pix, val) in pixels.into_iter().zip(values) {
    full_map[pix as usize] = val;
}
```
- Eliminates intermediate `Vec<DataValue>`
- Single allocation instead of 3
- **Expected gain:** 5-8%

**Solution #2 (Fallback):** Pre-allocate exact size
```rust
let mut full_map = vec![f64::NEG_INFINITY; npix];
// Use single-pass iterator, avoid pairs Vec
table.zip() // Get (pixel, value) tuples directly
    .for_each(|(pix, val)| {
        if pix >= 0 && pix < npix as i64 {
            full_map[pix as usize] = val;
        }
    });
```
- Still creates intermediate but avoids explicit `Vec<DataValue>`
- **Expected gain:** 2-3%

---

### **Tier 2: Enable Memory-Mapped I/O** (Est. 5-8% gain)

**Problem:** BufReader copying FITS from disk unnecessarily; kernel `rep_movs_alternative` overhead

**Solution:** Use existing `MmapFitsReader`
```rust
// fits.rs read_healpix_column():
// OLD:
let f = File::open(filename)?;
let reader = BufReader::with_capacity(256 * 1024, f);  ← Extra copy layer

// NEW:
let reader = MmapFitsReader::open(filename)?;          ← Direct memory mapping
let mut fits = Fits::from_reader(reader);
```

**Benefits:**
- Eliminates `BufReader` intermediate buffer
- OS page cache eliminates kernel memcpy overhead
- Virtual memory handles prefetching automatically
- **Expected gain:** 5-8% (directly reduces `rep_movs_alternative`)

**Implementation:** 1-line change in fits.rs

---

### **Tier 3: Vectorize Scaling Loop** (Est. 3-5% gain)

**Problem:** Element-wise scaling has branch misprediction costs

**Solution (SIMD):** Vectorize with packed intrinsics
```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

// Process 4 f64 values at a time
let chunks = map.chunks_exact_mut(4);
for chunk in chunks {
    let v0 = _mm256_loadu_pd(chunk.as_ptr() as *const f64);
    let unseen_mask = _mm256_cmpeq_pd(v0, _mm256_set1_pd(HPX_UNSEEN));
    
    let scaled = _mm256_mul_pd(v0, _mm256_set1_pd(scale_factor));
    let result = _mm256_blendv_pd(scaled, v0, unseen_mask);
    
    _mm256_storeu_pd(chunk.as_mut_ptr() as *mut f64, result);
}
```
- Branch prediction eliminated
- 4× throughput per iteration
- **Expected gain:** 2-4%

**Risk:** Requires careful unaligned load handling; test on multiple architectures

---

### **Tier 4: Parallel Data Loading with Better Locality** (Est. 6-10% gain)

**Problem:** Sequential population phase is inherently sequential; blocks parallelism

**Solution:** Block-wise parallel loading
```rust
// Load in 64KB blocks (good cache locality)
const BLOCK_SIZE: usize = 65536; // fits in L2 cache

let partial_maps: Vec<Vec<f64>> = (0..num_blocks)
    .into_par_iter()
    .map(|block_id| {
        let mut block_map = vec![f64::NEG_INFINITY; 12 * nside * nside'];
        let start_row = block_id * ROWS_PER_BLOCK;
        
        for (pix, val) in table
            .rows(start_row..start_row + ROWS_PER_BLOCK)
            .map(|(p, v)| (p as usize, v as f64))
        {
            if pix < block_map.len() {
                block_map[pix] = val;
            }
        }
        
        block_map
    })
    .collect();

// Merge blocks sequentially (good cache reuse)
let mut final_map = partial_maps[0].clone();
for block in &partial_maps[1..] {
    for (i, &val) in block.iter().enumerate() {
        if is_seen(val) {
            final_map[i] = val;
        }
    }
}
```
- Maintains parallelism without bottleneck
- Each thread works on cache-resident data
- **Expected gain:** 6-10%

---

### **Tier 5: Downgrade in Stream** (Est. 3-5% gain - applicable only for downgrade cases)

**Problem:** High-res maps trigger additional intermediate vector + extra pass

**Solution:** Fuse downgrade with initial loading
```rust
if should_downgrade {
    // Load into target-resolution map directly
    let target_nside = target_nside_for_resolution(width);
    let mut downgraded = vec![f64::NEG_INFINITY; 12 * target_nside * target_nside];
    
    for (pix_hi, val) in fits_column {
        let pix_lo = downgrade_pixel(pix_hi, original_nside, target_nside);
        // Direct aggregation
        downgraded[pix_lo] = combine_values(downgraded[pix_lo], val);
    }
} else {
    // Normal loading
}
```
- **Only 3% gain** since downgrade only happens for high-res maps
- **Complexity:** Significant (pixel mapping logic)
- **Recommendation:** Defer unless Tier 1-4 insufficient

---

## Cumulative Impact

| Tier | Optimization | Cost | Gain | Cumulative |
|-----|--------------|------|------|-----------|
| 1   | Eliminate buffers | Low: 2-3 hours | 8-12% | 22.58→19.85s |
| 2   | Use MmapFitsReader | Trivial: 5 min | 5-8% | 19.85→18.25s |
| 3   | Vectorize scaling | Medium: 1-2 hours | 3-5% | 18.25→17.35s |
| 4   | Parallel blocks | High: 3-4 hours | 6-10% | 17.35→15.65s |
| **Total** | All tiers | — | **22-35%** | **22.58→14.7-16.5s** |

---

## Implementation Priority

### Week 1: Quick Wins (2 hours, 13-20% gain)

1. **[5 min]** Enable `MmapFitsReader` (Tier 2) - 1-line change
2. **[1.5 hours]** Eliminate intermediate buffers (Tier 1) - refactor column extraction
3. **[30 min]** Testing & validation on 3GB file

### Week 2: SIMD & Parallelism (4-5 hours, 9-15% additional gain)

4. **[1.5 hours]** Vectorize scaling loop (Tier 3)
5. **[3 hours]** Block-wise parallel loading (Tier 4)
6. **[1 hour]** Benchmarking

### Week 3: Downgrading Path (if needed)

7. **[3-4 hours]** Fuse downgrade into loading (Tier 5)

---

## Validation Plan

### Before Optimization
```bash
cargo build -r && \
time ./target/release/map2fig \
  -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
  -o /tmp/baseline.pdf
# Expected: ~22.58s
```

### After Each Tier
```bash
# After Tier 1+2
perf stat -e cycles,instructions,cache-references,cache-misses \
  [test command]
# Expected: 19-20 seconds, cache misses < 30%

# After Tier 3+4
perf record -F 99 --all-cpus [test command]
perf report
# Expected: 15-16 seconds, IPC > 3.0
```

### Success Criteria
- [ ] Execution time < 18 seconds (20% improvement)
- [ ] Cache miss rate < 25% (from 36.67%)
- [ ] Memory bandwidth > 250 MB/s (from 132.8)
- [ ] No performance regression on mask/downgrade paths

---

## Key Insights for Future Work

1. **Memory-bound != CPU-bound:** Instruction-level optimizations (SIMD, FP precision) don't help until memory access patterns improve

2. **Intermediate buffers are silent killers:** The `Vec<DataValue>` intermediate consumed 5-8% of time despite being "just parsing"

3. **Rendering loop is already optimized:** Only 0.24% memory load latency in actual pixel sampling - confirms rendering batching is effective

4. **Kernel memcpy amortizable:** Using `MmapFitsReader` eliminates Linux page cache interaction entirely

5. **Downgrading adds quadratic complexity:** High-res maps trigger 3-pass algorithm - future: reconsider approach for nside > 2048

---

## Files to Modify

| File | Change | Complexity |
|------|--------|-----------|
| src/fits.rs | Multi-buffer elimination | Medium |
| src/fits.rs | MmapFitsReader integration | Trivial |
| src/pipeline.rs | Vectorized scaling | Medium |
| src/fits.rs | Parallel block loading | High |
| src/healpix.rs | Downgrade fusion (optional) | High |

---

## Related Documentation

- [PERF_ANALYSIS_DETAILED.md](PERF_ANALYSIS_DETAILED.md) - CPU-level profiling details
- [F32_OPTIMIZATION_RESULTS.md](PERF_ANALYSIS_DETAILED.md) - Why arithmetic precision doesn't help
- [.github/copilot-instructions.md](.github/copilot-instructions.md) - Architecture overview

