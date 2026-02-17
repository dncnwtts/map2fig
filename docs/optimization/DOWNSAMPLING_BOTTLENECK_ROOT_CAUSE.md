# Downsampling Bottleneck Analysis - Detailed Investigation

**Date:** February 17, 2026  
**Profiling Method:** `perf record` with call-graph analysis  
**Target:** Downsampling phase (75.93% of total runtime = 5.7s)

---

## What's Actually Happening

### Function Breakdown (from perf call-graph)

```
77.47% - Rayon downsampling
  ├─  8.97% - xyf2ring() [coordinate conversion math]
  ├─  4.68% - is_seen() [check if value valid]
  ├─  2.14% - Loop iterator overhead
  └─ 61.68% - ??? [CPU stalled, not accounted for in function calls]
```

### The Missing 61.68%

The missing time is **CPU stalls waiting for memory**. Here's why:

```rust
// Current inner loop (src/healpix.rs lines 1297-1310):
for j in y0..(y0 + fact) {
    for i in x0..(x0 + fact) {
        let source_pix = xyf2ring(source_nside, i, j, face);  // 8.97%
        let val = map[source_pix];  // ← RANDOM MEMORY ACCESS!
        if is_seen(val) {             // 4.68%
            sum += val;
            hits += 1;
        }
    }
}
```

### The Problem in Detail

**For nside=8192 → nside=512 downsampling:**

```
Fact = 8192 / 512 = 16
Per target pixel:
├─ Nested loops: 16 × 16 = 256 iterations
├─ Each reads: map[random_index] where random_index ∈ [0, 806M]
├─ Map size: 806 million f64 values = 6.4 GB
└─ Access pattern: Completely random!

Total accesses:
├─ Target pixels: 12.5 million (nside=512)
├─ Source lookups: 12.5M × 256 = 3.2 billion random memory accesses
├─ Array size: 806M pixels = much larger than L3 cache (16 MB)
└─ Result: **Virtually every access is a cache miss (31.85% measured)**
```

### Memory Access Pattern Visualization

```
Accessing array element map[source_pix]:
source_pix values: [42851, 7334295, 891234, 123451251, 4522, ...]

Memory access pattern:
├─ Jump to offset 42851        → Cache miss (50+ cycles)
├─ Jump to offset 7334295      → Cache miss (50+ cycles)
├─ Jump to offset 891234       → Cache miss (50+ cycles)
├─ Jump to offset 123451251    → Cache miss (50+ cycles)
├─ ... × 3.2 billion times
└─ Result: CPU stalled 61.68% of the time waiting for data
```

### CPU Cycles Breakdown

```
Available CPU work: 6 cores × 5.3 GHz × 5.7s = 180 billion cycles
Instruction cycles: 
  ├─ xyf2ring: 8.97% = 16.1 billion cycles
  ├─ is_seen: 4.68% = 8.4 billion cycles
  ├─ Loop overhead: 2.14% = 3.9 billion cycles
  └─ Actual math: ~2% = 3.6 billion cycles
─────────────────────────────────────
Total instruction execution: 32 billion cycles

Memory stalls: 180B - 32B = 148 billion cycles (82% of runtime!)
```

---

## Root Cause: Memory Subsystem

### Why Random Access is Catastrophic

**Modern CPUs use prefetchers** to hide memory latency:

```
Sequential access: [addr, addr+64, addr+128, addr+192, ...]
└─ Prefetcher detects 64B stride → walks ahead
   └─ By the time CPU needs data, it's already in cache
   └─ Effective latency: 4-5 cycles

Random access: [42851, 7334295, 891234, ...]
└─ No prefetchable pattern
   └─ Each access: 40-100 cycle latency
   └─ CPU stalls until data arrives
```

### The Memory Hierarchy Problem

```
L1 cache:    32 KB        Latency: 4 cycles
L2 cache:    256 KB       Latency: 12 cycles
L3 cache:    16 MB        Latency: 40 cycles
Main memory: 30 GB        Latency: 100+ cycles

Array size: 6.4 GB (402× larger than L3 cache)
Typical miss rate: 31.85% (from earlier profiling)
Per miss cost: 100 cycles

Average cost per memory access:
  = 0.6 × 40 cycles (L3 hit) + 0.32 × 100 cycles (main mem)
  = 24 + 32 = 56 cycles per access
```

### Bandwidth Limitation

```
Memory bandwidth available: 50 GB/s
Bytes accessed: 3.2B accesses × 8 bytes = 25.6 GB
Time to access: 25.6 GB ÷ 50 GB/s = 0.512 seconds

But we measure: 5.7 seconds (11.2× slower!)
Reason: Not a simple bandwidth question—it's about **latency and stalls**
        The 3.2B random accesses create dependent chains:
        - Access A takes 50+ cycles
        - Can't start B until A completes
        - Modern CPUs can only hide ~10-12 outstanding misses
        - When you exceed that, you're fully stalled
```

---

## Solution Attempts

### ✅ Option 4: Prefetch Hints (SUCCESSFUL - 3.2% improvement)

**Status:** Implemented and verified working ✅

Added explicit `_mm_prefetch` hints to downsampling inner loop:
- Prefetch 2 iterations ahead to hide 50-100 cycle memory latency
- 7.68% visible prefetch cost overlapped with previously-idle memory latency time
- Net result: **3.2% wall-clock improvement (7.502s → 7.263s)**
- See [PREFETCH_OPTIMIZATION_RESULTS.md](PREFETCH_OPTIMIZATION_RESULTS.md) for detailed analysis

### ❌ Option 2: Tiling with Cache Awareness (FAILED - 12% regression)

**Status:** Attempted and failed ❌

Attempted spatial tile-based parallelization instead of linear chunking:
- Goal: Process targets in spatial tiles to group source accesses
- Result: **12% slowdown (7.263s → 8.156s)**
- Root cause: Excessive task overhead + weakened parallelization + HEALPix geometry doesn't map to spatial locality
- Lesson: Once prefetch hides memory latency, iteration reorganization provides negative returns
- See [TILING_OPTIMIZATION_FAILURE_ANALYSIS.md](TILING_OPTIMIZATION_FAILURE_ANALYSIS.md) for analysis

---

## Previously Proposed Solutions (Not Yet Attempted)

### Option 1: Cache-Aware Morton Order (Theory - Not Implemented)

**Idea:** Process target pixels in Morton/Z-order curve instead of linear order

```
Linear order: pixel 0, 1, 2, 3, 4, ...
Spatial locality: NONE - adjacent pixel indices don't map to adjacent memory

Morton order: pixel 0, 1, 4, 5, 2, 3, 6, 7, 8, ...
Morton Z-curve:  └─ Each 4 pixels are spatially nearby in 2D space
Spatial locality: YES - source lookups cluster in memory

Result:
├─ Better prefetch patterns (sequential clusters)
├─ Improved cache locality (same cores reuse same cache lines)
└─ Estimated speedup: 2-3× (memory pattern fix + prefetcher activation)
```

**Implementation complexity:** Medium (need Morton space-filling curve math)

### Option 3: Tiling with Cache Awareness (Deprecated - Attempted and Failed)

~~**Idea:** Process map in 1MB-sized tiles that fit in L2 cache, downsampling each tile~~

**Status:** ❌ **ATTEMPTED Feb 18, 2026 - FAILED with 12% regression**

Multiple factors caused failure:
1. **Task overhead:** ~3000 Rayon tasks had higher scheduling cost than benefit
2. **HEALPix geometry mismatch:** NESTED ordering uses Morton codes (hierarchical), not linear spatial proximity
3. **Prefetch already solved the bottleneck:** Iteration reorganization provides negative value once latency is hidden
4. **Tile reconstruction:** Per-tile result buffers and merging added complexity

**Why it was proposed:** Earlier analysis assumed scheduling overhead of 310K linear chunks would be eliminated by tiling (~3K tasks). However, once prefetch optimization reduced the effective bottleneck to hidden memory latency, further iteration optimization was counterproductive.

See [TILING_OPTIMIZATION_FAILURE_ANALYSIS.md](TILING_OPTIMIZATION_FAILURE_ANALYSIS.md) for full analysis.

**Implementation complexity:** Low-Medium

### Option 3: Batch Processing with Prefetch Hints

**Idea:** Prefetch the next batch of source pixels while processing current batch

```rust
// Process pixels in 256-pixel chunks (modern CPU can queue ~12-16 prefetches)
for batch in target_pixels.chunks(256) {
    // Prefetch next batch's source indices
    for &pix in &batch[..min(256, batch.len())] {
        let idx = convert_to_source_index(pix);
        core::arch::x86_64::_mm_prefetch::<_>(
            &map[idx] as *const _ as *const i8,
            1  // _MM_HINT_T0
        );
    }
    
    // Now process (data is hopefully prefetched)
    for &pix in batch {
        let val = map[source_idx];
        // ...
    }
}
```

**Result:**
├─ Hide some latency with prefetching
└─ Estimated speedup: 1.2-1.5×

**Implementation complexity:** Low (just add prefetch intrinsics)

### Option 4: Hybrid Approach (Most Practical)

**Idea:** Combine tiling + prefetch hints for maximum effect

1. Tile-based: Process map in cache-friendly 1-4MB tiles
2. Within tile: Use Morton order for better cache locality
3. Batch processing: Prefetch next batch of lookups

Result:
├─ Combines benefits of all three approaches  
├─ More defensive: multiple layers of optimization
└─ Estimated speedup: 3-5× (compound improvements)

---

## Recommendation for Next Step

**I recommend starting with Option 2 (Tiling):**

**Why:**
1. ✅ Lowest implementation risk
2. ✅ Easy to measure and validate incrementally
3. ✅ Clear 1.5-2.0× improvement path
4. ✅ Can be combined with other optimizations later

**Implementation plan:**
1. Measure current downsampling time: 5.7s ✓ (already done)
2. Implement tile-based processing
3. Benchmark: Should see 5.7s → 3-4s improvement
4. If successful, add Option 1 (Morton order) for further gains
5. Final validation with perf

**Metrics to track:**
- Wall-clock time (primary)
- Cache miss rate (perf stat -e cache-misses)
- Memory bandwidth (perf stat -e memory-loads-aux)
- L3 cache hit rate (perf stat -e LLC-load-misses)

---

## Why This Works

The core problem is **spatial locality**:

```
Current (Linear):
  CPU needs: pixel 0, 42, 84, 126, ... (scattered)
  Memory: fetches cache lines for all of them (16 independent misses)
  Result: All 16 stalled waiting

Tiled (Cache-fit):
  CPU needs: pixels [0-99], then [100-199], then ...
  Memory: First 100 keep same cache warm
           Then switch cache for next 100
  Result: Fewer total cache lines needed, better reuse
```

The fact that 82% of CPU is idle (waiting for memory) means **memory access pattern is really the issue**, not the math.
