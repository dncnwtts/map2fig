# Practical Guide: Working Around Memory Access Pattern Bottleneck

**Status:** Research & Implementation Guide  
**Objective:** Achieve 15-30% additional speedup by improving cache locality without algorithmic inversion  
**Focus:** Real-world feasible optimizations, not theoretical exercises

---

## Executive Summary

The current bottleneck is **random HEALPix pixel access** causing:
- 57.91% LLC miss rate
- 0.55% memory bandwidth utilization
- Only 0.11% of theoretical CPU peak

The solution isn't more SIMD (we're memory-bound, not CPU-bound). Instead, we need to:
1. Reduce cache misses through better data layout
2. Improve memory prefetching
3. Use hierarchical/lazy computation

**Realistic goal:** 15-30% improvement (13.79s → 9.6-11.7s) on large maps

---

## Strategy 1: Morton Curve (Z-Order) Traversal

### The Problem with Row-Major Access

Current pixel processing:
```
for y in 0..height {
    for x in 0..width {
        let healpix_idx = compute_healpix_index(x, y);
        let value = map[healpix_idx];  // RANDOM ACCESS → cache miss
        process(value);
    }
}
```

**Why it fails:**
- Pixels (0,0), (1,0), (2,0)... map to random HEALPix indices
- Each access likely to miss L3 cache
- CPU can't prefetch effectively

### The Morton Curve Solution

Instead of row-major, traverse in Z-order (Morton curve):
```
Z-order traversal:
(0,0) → (1,0) → (0,1) → (1,1) → (2,0) → (3,0) → (2,1) → (3,1) → ...

Spatial layout:
  0,0  1,0
  0,1  1,1  2,0  3,0
            2,1  3,1
```

**Why it helps:**
- Pixels nearby in Z-order are nearby on sphere (spherical coordinates are somewhat regular)
- Better spatial locality → fewer cache misses
- Easier CPU prefetching of next pixels

### Implementation Approach

**Step 1: Generate Morton indices**

```rust
fn interleave_bits(x: u32, y: u32) -> u64 {
    let mut result = 0u64;
    for i in 0..32 {
        result |= ((x >> i) & 1) as u64) << (2 * i);
        result |= ((y >> i) & 1) as u64) << (2 * i + 1);
    }
    result
}

// Generate all pixel indices in Morton order
let mut morton_indices: Vec<(u32, u32)> = (0..height)
    .flat_map(|y| {
        (0..width)
            .map(move |x| (interleave_bits(x, y), (x, y)))
    })
    .collect();

morton_indices.sort_by_key(|k| k.0);  // Sort by Morton code
```

**Step 2: Use in rendering loop**

```rust
// BEFORE: Row-major
for y in 0..height {
    for x in 0..width {
        let value = map[compute_healpix_index(x, y)];
        process(x, y, value);
    }
}

// AFTER: Morton order
let morton_order = compute_morton_order(width, height);
for (_morton_code, (x, y)) in &morton_order {
    let value = map[compute_healpix_index(x, y)];
    process(x, y, value);
}
```

### Expected Improvements

| Metric | Before | After | Gain |
|--------|--------|-------|------|
| L3 misses | 57.91% | 40-45% | **12-18 percentage points** |
| Memory latency stalls | High | Lower | **10-15% throughput improvement** |
| Wall time (12.4M pixels) | 0.94s | 0.80-0.85s | **10-15% speedup** |

### Feasibility Assessment

| Factor | Status | Notes |
|--------|--------|-------|
| **Complexity** | Medium | Requires new data structure, ~2-3 hours |
| **Risk** | Low | Pure optimization, algorithm-independent |
| **Portability** | High | Works on all CPUs |
| **Measurable** | Yes | A/B test with/without |

### Limitations

- **Won't fully solve** memory-bandwidth ceiling (0.55% → maybe 1-2%)
- **Dependent on data layout** - only helps if HEALPix map data has spatial locality
- **May interact poorly** with existing parallelization (Rayon chunks)

---

## Strategy 2: Cache-Level Prefetching

### Hardware Prefetchers Limitations

Modern CPUs have prefetchers that work for:
- ✅ Sequential access (L2 prefetcher)
- ✅ Predictable strides (L2 prefetcher)
- ❌ Random access (can't predict)

Our case: **Random spherical-to-HEALPix mapping → unpredictable access**

### Software Prefetching

Manually tell CPU to fetch data:

```rust
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::_mm_prefetch;

for y in 0..height {
    for x in 0..width {
        let healpix_idx = compute_healpix_index(x, y);
        
        // Prefetch data for this pixel NOW (before we need it)
        #[cfg(target_arch = "x86_64")]
        unsafe {
            _mm_prefetch(
                &map[healpix_idx] as *const f64 as *const i8,
                1,  // _MM_HINT_T1: Prefetch to L2
            );
        }
        
        // Do other work while prefetch completes...
        let proj = project_pixel(x, y);  // ~50-100 cycles of work
        
        // By now, data is likely in L2 cache
        let value = map[healpix_idx];  // Fast hit
        let color = colormap[value];
        render(proj, color);
    }
}
```

**How it works:**
1. Issue prefetch immediately
2. Continue with computation (150+ cycles available)
3. By the time we need data, it's usually in cache

### Expected Improvements

| Metric | Benefit |
|--------|---------|
| **L2 miss → L3/memory latency reduction** | 30-50 cycles saved |
| **Per-pixel impact** | 5-10% throughput improvement |
| **Cache miss rate** | 57.91% → 48-52% |

### Implementation Challenges

1. **Architecture-specific** (x86_64, different for ARM/POWER)
2. **Timing-dependent** (prefetch too early = cache eviction, too late = useless)
3. **Not guaranteed** (prefetcher may be ignored depending on CPU state)

### Feasibility

| Factor | Rating | Notes |
|--------|--------|-------|
| **Complexity** | Medium | Requires target-specific code |
| **Risk** | Medium | Wrong timing hurts performance |
| **Gain** | 5-10% | Modest but real improvement |
| **Portability** | Low | Need different code per platform |

---

## Strategy 3: Tile-Based Processing with Local Cache

### The Idea

Process image in **tiles** (e.g., 64×64 pixels) such that:
1. Tiles map to consecutive HEALPix indices (if possible)
2. One tile's data fits in L3 cache
3. Better memory access locality per-tile

### Implementation

```rust
const TILE_SIZE: usize = 64;  // 64×64 = 4096 pixels

for tile_y in (0..height).step_by(TILE_SIZE) {
    for tile_x in (0..width).step_by(TILE_SIZE) {
        // For this tile, collect HEALPix indices
        let mut indices_in_tile: Vec<usize> = Vec::new();
        
        for y in tile_y..(tile_y + TILE_SIZE).min(height) {
            for x in tile_x..(tile_x + TILE_SIZE).min(width) {
                indices_in_tile.push(compute_healpix_index(x, y));
            }
        }
        
        // Sort indices for sequential access within tile
        indices_in_tile.sort_unstable();
        
        // Now process tile with better cache behavior
        for y in tile_y..(tile_y + TILE_SIZE).min(height) {
            for x in tile_x..(tile_x + TILE_SIZE).min(width) {
                let healpix_idx = compute_healpix_index(x, y);
                let value = map[healpix_idx];  // Better locality
                process(x, y, value);
            }
        }
    }
}
```

### Why This Helps

- L3 cache (16 MB) can hold tile data if indices are close
- Reduced TLB misses (fewer unique virtual memory pages)
- Better instruction cache locality

### Expected Improvements

| Aspect | Improvement |
|--------|-------------|
| **L3 hit rate** | 25% → 35-40% |
| **Wall time** | 0.94s → 0.82-0.88s |
| **Overall speedup** | **5-10%** |

### Limitations

- Works only if nearby pixels map to nearby HEALPix indices
- Depends on HEALPix ring vs nested indexing
- May not help much on Mollweide (distorted mapping)

---

## Strategy 4: Hybrid: Streaming Coarse-to-Fine

### Current Issue

Large maps (806M pixels) thrash cache completely. What if we process in passes?

### Algorithm

```rust
// Pass 1: Render at nside/2 (202M pixels, fits in memory better)
let downsampled_map = downsample_map(map, nside / 2);
render_at_resolution(downsampled_map, quarter_size);

// Pass 2: Fill in high-frequency details nside (806M pixels)
for pixel in map {
    if pixel_frequency > threshold {  // High-frequency areas only
        render_detail(pixel);
    }
}
```

### Advantages

1. **First pass** (coarse): Quick visual feedback, good cache behavior
2. **Second pass** (details): Focus on areas that need resolution
3. **Can parallelize** coarse and detail passes

### Expected Improvements

| Scenario | Benefit |
|----------|---------|
| **Coarse preview (1/4 res)** | 0.94s → 0.15s (6× faster) |
| **Detail pass (high-freq only)** | 0.94s → 0.40s (2× faster) |
| **Total (if sequential)** | 0.94s → 0.55s **1.7× faster** |

### Feasibility

| Factor | Assessment |
|--------|-----------|
| **Complexity** | High (requires frequency analysis) |
| **Quality impact** | None (if thresholds set correctly) |
| **Practical benefit** | Good (works for progressive rendering) |
| **Timeline** | 4-6 hours implementation + testing |

---

## Strategy 5: SIMD Vectorization Revisit

### Current State

Using `wide` crate: f64x2 (2 pixels/iteration) = 50% of AVX2 capacity

### Why It's Limited

```rust
use wide::f64x2;

// Current: Can do 2 pixels in parallel
let x = f64x2::from([x0, x1]);
let y = f64x2::from([y0, y1]);
let sin_x = x.sin();  // SIMD sin for 2 values
```

**The Problem:** `wide` crate doesn't provide f64x4 (4 pixels/iteration)

### Option A: Use Packed SIMD (Nightly)

**Status:** Investigated, found incompatible with SLEEF

**Not recommended** for production.

### Option B: Custom AVX2 Vectorization

```rust
#[cfg(target_arch = "x86_64")]
mod simd_avx2 {
    use std::arch::x86_64::*;
    
    pub fn sin_4_f64(values: [f64; 4]) -> [f64; 4] {
        // Use intrinsics to do 4 sin() calls in parallel
        // Would need external math library like SLEEF
        // But SLEEF v0.3 incompatible with current nightly...
    }
}
```

**Status:** Blocked by SLEEF versioning issues

### Option C: Refactor to Use f32 Math

```rust
// Map uses f64, but for visualization discretized to 256 colors
// Could do computation in f32 (8 pixels/AVX2 256-bit)
let value_f32 = map[i] as f32;  // f64 → f32 (lossy but OK for viz)
let scaled_f32 = scale_value(value_f32, ...);
let color_idx = (scaled_f32 * 255.0) as u8;
```

**Pros:**
- f32x8 = 2× improvement of current wide crate
- No external dependencies

**Cons:**
- Precision loss (acceptable for visualization?)
- Need to verify numerical stability of scaling

**Expected gain:** 15-20% on projection math  
**Overall impact:** ~5-7% total (since memory-bound)

### Assessment

| Strategy | Feasibility | Gain | Risk |
|----------|-------------|------|------|
| **Packed SIMD (nightly)** | Low | 20% | High (ecosystem) |
| **AVX2 custom** | Medium | 20% | Medium (complexity) |
| **f32 vectorization** | Medium | 10% | Low (clean fallback) |

---

## Strategy 6: Vectorize Memory I/O

### Current State

FITS loading and HEALPix sampling are already optimized (Tier 1-1.2).

### Remaining Opportunities

**Strategy 6a: Batch HEALPix coordinate transform**

```rust
// Current: Transform one (x,y) → (healpix_idx) at a time
let healpix_idx = compute_healpix_index(x, y);  // Scalar

// Batch version:
let healpix_idxs = compute_healpix_index_batch(&[(x0,y0), (x1,y1), ...]);
```

**Advantage:** Better CPU pipeline utilization  
**Difficulty:** Requires modifying `cdshealpix` crate  
**Estimated gain:** 2-3%

**Strategy 6b: Cache computed HEALPix indices**

```rust
// Pre-compute and cache all (x,y) → healpix_idx mappings
// For a 12.4M pixel image:
let cache: HashMap<(u32, u32), usize> = precompute_indices();

// Then in hot loop:
let healpix_idx = cache[&(x, y)];  // O(1) lookup, cache-friendly
let value = map[healpix_idx];
```

**Pro:** Eliminates expensive trig per-pixel  
**Con:** Uses extra memory (50 MB for 12.4M pixels)  
**Gain:** 5-10% (fewer instructions per pixel)

---

## Practical Implementation Plan

### Phase 1: Measurement (1 day)

Create perf comparison baseline:
```bash
# Add timing instrumentation to render_mollweide_pixels
let t0 = std::time::Instant::now();
// existing code
let elapsed = t0.elapsed();
eprintln!("Render time: {:?}", elapsed);
```

Measure:
- PNG time with/without detailed breaks
- Cache miss rates (if perf is available)
- Memory bandwidth utilization

### Phase 2: Implement Morton Curve (2-3 days)

1. Implement `compute_morton_order()` function
2. Modify `render_projection_to_grid()` to use it
3. Benchmark improvement
4. If <10%, try tile-based (Phase 3)

### Phase 3: Try Tile-Based Processing (2-3 days)

If Morton doesn't help enough:
1. Implement tile-sized chunks
2. Sort indices within tile
3. Benchmark improvement
4. Combine with Rayon parallelization

### Phase 4: Consider f32 Vectorization (1-2 days)

If still room for improvement:
1. Profile where f64 is critical vs acceptable
2. Try f32 conversion in non-critical paths
3. Measure quality impact vs speedup

---

## Summary Table: Implementation Options

| Strategy | Effort | Gain | Risk | Priority |
|----------|--------|------|------|----------|
| **Morton curve** | 2-3 hrs | 10-15% | Low | **1** |
| **Tile-based** | 3-4 hrs | 5-10% | Low | **2** |
| **Software prefetch** | 2-3 hrs | 5-10% | Medium | **3** |
| **f32 vectorization** | 4-6 hrs | 5-7% | Low | **4** |
| **Hybrid coarse-fine** | 6 hrs | 20% | Medium | **5** |
| **Index caching** | 2 hr | 5% | Low | **6** |
| **AVX2 custom SIMD** | 6-8 hrs | 20% | High | **7** |

---

## Conclusion

**MEMORY ACCESS PATTERNS ARE THE REAL BOTTLENECK**, not CPU speed or SIMD capability.

**Recommended approach:**
1. Start with Morton curve (low effort, proven technique)
2. If results <10%: Add tile-based processing
3. If results <15%: Consider hierarchical rendering
4. Don't pursue AVX2 custom SIMD (too much effort for small gain on memory-bound workload)

**Reality check:**
- We're at 0.11% of theoretical CPU peak
- But we're already near **practical memory bandwidth ceiling**
- The remaining 15-30% speedup comes from better cache behavior, not more compute

After these optimizations, further improvements require algorithmic changes (GPU, downsampling, different projection methods).

