# Downgrade Operation Profiling Report

**Date:** February 17, 2026  
**Test File:** combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits (3.1GB)  
**Operation:** Downgrade from nside=8192 to nside=512

## Performance Baseline

```
Total Execution: 13.856s
├─ FITS Read:     10.944s (78.9%)
├─ Downgrade:      1.541s (11.1%)
└─ Rendering:      0.138s (1.0%)
```

**Focus:** Downgrade is 11% of total time and takes **1.541 seconds**

## Computational Analysis

### Scale of Operation
- **Source pixels:** 8192² × 12 = 805M pixels
- **Target pixels:** 512² × 12 = 3.1M pixels
- **Downgrade factor:** 16× (16×16 = 256 source pixels per target)
- **Total coordinate conversions:** 808M

### Per-Operation Cost
- **Time per conversion:** 1.9 nanoseconds (800M conversions ÷ 1.541s)
- **This is quite efficient** - suggests CPU parallelization is highly effective

## Bottleneck Breakdown

Based on code analysis of `downgrade_healpix_map_xyf_parallel`:

```
Per target pixel:
  1. ring2xyf(target_nside, target_pix) - 1 call
  2. for j in y0..y0+16:
       for i in x0..x0+16:
         xyf2ring(source_nside, i, j, face) - 256 calls
  3. Aggregate results (sum, hits count)
```

**Estimated breakdown:**
- Coordinate conversions: **70-80%** of time (ring2xyf, xyf2ring with branch logic)
- HEALPix map memory access: **10-15%** (random access patterns, cache misses)
- Aggregation logic: **5-10%** (sum/hit counting)
- Parallelization overhead: **5-10%** (rayon thread management for 3.1M tasks)

## Optimization Opportunities

### 1. **Eliminate Parallelization Overhead** (Est. 5-15% gain)

**Current approach:** Rayon processes 3.1M target pixels in parallel
- Each task: minimal work (~260 coordinate conversions)
- Overhead: Task spawning, synchronization, load balancing

**Alternative:** Process in larger chunks
```rust
// Instead of:
(0..target_npix).into_par_iter().map(|target_pix| { ... })

// Try:
(0..target_npix).into_par_iter()
  .chunks(1000)  // Process in groups of 1000
  .map(|chunk| {
    chunk.iter().map(|target_pix| { ... }).collect()
  })
```

**Estimated gain:** 5-15% (reduce task spawning overhead)  
**Complexity:** LOW (simple code change)  
**Risk:** LOW (correctness unchanged)

---

### 2. **Precompute or Cache Coordinate Lookup** (Est. 10-20% gain)

**Current approach:** Dynamic ring2xyf/xyf2ring calculations per operation

**Key insight:** For a given downgrade (8192→512), the coordinate mapping is determinis­tic and repeated

**Optimization strategy:**
```rust
// Build lookup table for this downgrade once
struct DowngradeMap {
  target_coords: Vec<(i64, i64, i64)>,  // (x, y, face) for each target pixel
  source_lookup: Vec<Vec<usize>>,       // source pixel indices for each target
}

// Then inner loop becomes:
for source_pix in &downgrade_map.source_lookup[target_pix] {
  let val = map[source_pix];
  if is_seen(val) {
    sum += val;
    hits += 1;
  }
}
```

**Cost:**
- Memory: ~200MB for precomputed indices (25MB source + 25MB target coords)
- Time: 50-100ms to build lookup table

**Estimated gain:** 10-20% (eliminate coordinate conversion overhead)  
**Complexity:** MEDIUM (requires careful indexing)  
**Risk:** MEDIUM (correctness needs validation)

---

### 3. **Optimize Ring/Nest Conversion Functions** (Est. 5-15% gain)

**Current approach:** ring2xyf/xyf2ring use division and modulo operations

**Potential optimization:** Use bit manipulation instead
```rust
// Instead of: face = ring_pixel / 262144;
// Use:       face = ring_pixel >> 18;  // 2^18 = 262144

// Instead of: h = ring_pixel % 262144;
// Use:       h = ring_pixel & 0x3FFFF;  // 0x3FFFF = 262144-1
```

**Estimated gain:** 5-15% (faster modulo/division)  
**Complexity:** MEDIUM (careful with bit widths)  
**Risk:** MEDIUM (bitwise operations prone to off-by-one)

---

### 4. **Increase Chunk Size in Parallelization** (Est. 5-10% gain)

**Current code:**
```rust
if target_npix > 50_000 {
  downgrade_healpix_map_xyf_parallel(...)
}
```

**Observation:** 3.1M pixels with per-task-spawn overhead

**Try:** Work-stealing with larger work units
```rust
// Process in 10K pixel chunks instead of per-pixel
let chunk_size = 10_000;
(0..target_npix).into_par_iter()
  .step_by(chunk_size)
  .map(|base| {
    for target_pix in base..(base + chunk_size).min(target_npix) {
      // process
    }
  })
```

**Estimated gain:** 5-10% (less task overhead)  
**Complexity:** LOW  
**Risk:** LOW (correctness same)

---

### 5. **SIMD Aggregation** (Est. 2-5% gain)

**Limited impact** since aggregation is only 5-10% of time

**Approach:** Vectorize the inner sum/count loop
```rust
// Use f64x4 SIMD to process 4 pixels simultaneously
// Requires careful handling of irregular workloads
```

**Estimated gain:** 2-5% (small part of critical path)  
**Complexity:** HIGH (SIMD intrinsics, nightly features)  
**Risk:** MEDIUM (correctness of vectorized aggregation)

---

## Ranking by ROI (Effort vs Gain)

| Rank | Optimization | Gain | Effort | Complexity | Risk | Total Value |
|------|---------------|------|--------|------------|------|-------------|
| 1 | Coordinate lookup cache | 10-20% | 2-3h | MEDIUM | MEDIUM | ⭐⭐⭐⭐ |
| 2 | Eliminate parallelization overhead | 5-15% | 0.5h | LOW | LOW | ⭐⭐⭐⭐ |
| 3 | Optimize conversion functions (bit ops) | 5-15% | 1-2h | MEDIUM | MEDIUM | ⭐⭐⭐ |
| 4 | Increase chunk size | 5-10% | 0.5h | LOW | LOW | ⭐⭐⭐ |
| 5 | SIMD aggregation | 2-5% | 3-4h | HIGH | MEDIUM | ⭐⭐ |

## Recommendation

**Start with Strategy #2 (eliminate parallelization overhead)** - Quick win:
- 30 minutes of work
- ~5-10% immediate speedup
- Zero risk

Then **measure before/after** with nside=8192 test:
```bash
./target/release/map2fig -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
  -o /tmp/test.png --verbose 2>&1 | grep Downgrade
```

If gain is measurable, proceed to **Strategy #1 (coordinate caching)** for 10-20% additional gain.

## Current Status

✅ **Profiling complete**  
⏸️ **Optimization: Not started**  
📊 **Baseline downgrade time: 1.541s**  
🎯 **Target: < 1.1s (30% speedup) or < 1.2s (22% speedup)**

## Notes

- Downgrade operation is **memory-hungry** (sequential random access to 800M pixels)
- Parallelization is working well (**1.9ns per conversion** is excellent)
- Main bottleneck: **essential computation cost**, not algorithmic issue
- Further optimization requires either:
  1. Precaching results (trade memory for speed)
  2. Algorithmic simplification (accepting lower quality?)
  3. Fundamental architecture change (GPU downsampling?)
