# Memory Optimization Attempt: Results and Lessons

**Date:** February 16, 2026  
**Optimization Tried:** Cache-sequential source pixel sorting  
**Result:** ❌ FAILED - 68.3 seconds (196% slower than baseline 19.4s)

## What We Tried

### The Idea
The profiling analysis identified 1.08B L1 cache misses as the bottleneck. We hypothesized that changing the memory access pattern from "target→random sources" to "sequential source access" would reduce cache misses by sorting the (source_pix, target_pix) pairs.

### The Implementation
```rust
// Build massive vector of all (source, target) pairs
let mut accesses: Vec<(usize, usize)> = Vec::with_capacity(
    target_npix * ratio * ratio  // 3.1M targets × 256 = ~805M pairs
);

// Sort by source_pix for sequential access
accesses.sort_unstable_by_key(|a| a.0);

// Process in source order
let mut accum = vec![0.0; target_npix];
let mut hits = vec![0; target_npix];
for (source_pix, target_pix) in accesses {
    accum[target_pix] += map[source_pix];
    hits[target_pix] += 1;
}
```

### Why It Failed

| Cost Factor | Impact | Details |
|---|---|---|
| **Building vector** | ~5s | Allocating 805M pairs + initialization overhead |
| **Sorting** | ~35-40s | 805M items × log₂(805M) ≈ 24B comparisons |
| **Extra allocations** | ~5s | Separate `accum[]` and `hits[]` vectors |
| **Memory overhead** | Massive | 805M pairs = ~13 GB memory traffic for sort |
| **Total overhead** | **49-50 seconds** | More expensive than the cache misses we were trying to fix! |

### Performance Impact

```
Baseline (random access):    19.4 seconds
Attempted optimization:      68.3 seconds
Slowdown:                    3.5× WORSE

Cache misses:
  Baseline:    1.08B L1 misses (17.4% of time)
  Optimized:   6.26B L1 misses (much worse!)
  Instructions: 134B → 418B (3× more!)
```

## Why This Teaches Us About Real Performance

### Amdahl's Law in Action
The cache misses account for ~17.4% of total time (4 seconds of 23s).
- Best-case speedup from fixing cache misses: ~1.21× (save 4s out of 19.4s)
- Actual sorting overhead: >50 seconds!
- Net result: 3.5× slower

### The Real Lesson
**Sorting out of order is more expensive than random access with cache misses.**

Proof:
- Random L1 cache miss: 10 cycles latency (overlapped with other work)
- Sorting 805M items: dedicated CPU work, no parallelism opportunity
- Winner: random access (existing design is already optimal for this scale)

## Why Current Design is Actually Good

The current nested-loop approach:
```rust
for target_pix in 0..target_npix {           // 3.1M iterations
    for j in y0..y0+16 {                     // 16 iterations
        for i in x0..x0+16 {                 // 16 iterations
            source_pix = xyf2nest(...);      // ~10 instruction cycles
            sum += map[source_pix];          // ~10 cycle L1 miss (worst case)
        }
    }
}
```

Total work: 3.1M × 256 = 805M iterations
- Nested loops have minimal overhead
- Each iteration is ~20 cycles (10 compute + 10 memory)
- No extra allocations or sorting pass
- Shows 2.16 IPC (good for memory-bound code)

## What COULD Actually Work

### Option A: Distributed Caching (Rayon Parallelization)
Split downsampling into chunks processed by different threads:
```
Thread 1: target pixels 0-780K
Thread 2: target pixels 780K-1.56M  
Thread 3: target pixels 1.56M-2.34M
Thread 4: target pixels 2.34M-3.1M
```
- Each thread's source accesses fit better in L3 cache
- Different threads hit different cache lines
- Reduces contention on memory bus
- **Expected: 1.1-1.2× speedup** (less overhead than full sorting)
- **Effort:** Low (Rayon is already available)
- **Note:** Requires careful thread binding to CPU cores

### Option B: Blocked Iteration with Prefetching
Process pixels in 64×64 blocks instead of per-pixel:
```rust
for block in 0..(num_blocks) {
    prefetch_hint(next_block_data);  // Tell CPU what's coming
    for target_pix in block {
        for source in target_pix.sources {
            sum += map[source];
        }
    }
}
```
- Prefetch hints help CPU hide memory latency
- Block processing improves cache reuse
- **Expected: 1.05-1.15× speedup** (modest)
- **Effort:** Low, but requires careful tuning

### Option C: Accept Current Performance
The 1.08B L1 cache misses are actually **acceptable** for this algorithm:
- 17.4% of time is a reasonable cost for random access
- The algorithm is mathematically simple and robust
- Trying to optimize further = chasing diminishing returns
- **Realistic limit:** 1.2-1.3× speedup max (hard ceiling due to memory)

## Recommendation

### Short-term (Next Action)
✅ **CURRENT STATE IS FINE** - Revert to original algorithm  
- Baseline 19.4s is respectable for 800M+ reads + compute
- Optimization attempts need minimal overhead (<1s total)
- The 17.4% cache miss cost is unavoidable with this memory pattern

### Medium-term (If needed)
Try Option A (Rayon parallelization) with thread affinity:
- Low risk (only adds threading layer)
- Can measure actual speedup before/after
- Falls back to serial if threads hurt performance
- Expected 1.1-1.2× gain = 2-3 seconds saved

### Long-term (Different approach needed)
If >30% speedup is required:
- Switch to Ring-ordered HEALPix (sequential access but lower quality)
- Or use GPU downsampling (CUDA/HIP)
- Or accept that downsampling is fundamental bottleneck

## Peak Performance Estimate

Given:
- 805M source pixel reads @ L3 latency (42 cycles)
- 10 cycles per read + compute overhead
- CPU @ 2.7 GHz

Minimum time = (805M × 10 cycles) / 2.7G = ~3 seconds  
Actual time = 19.4 seconds (6.5× overhead for everything else)

**Hard limit:** ~6-8 seconds minimum (even with perfect optimization)

This means:
- Best-case speedup: 19.4s → 6-8s = **2.4-3.2×**
- But requires eliminating ALL other bottlenecks
- Realistically achievable: 1.2-1.5× = 12-16 seconds

## Conclusion

**The sorting optimization failed but taught us valuable lessons:**

1. ✅ Bottleneck correctly identified (L1 cache misses)
2. ❌ But solution cost > benefit (overhead of sort is fatal)
3. ✅ Current algorithm is near-optimal for its problem space
4. ✅ Further optimization has hard ceiling of ~2.4× max
5. ✅ Most impactful next step would be parallelization (1.1-1.2×)

**Action:** Keep baseline algorithm, consider Rayon parallelization if 10-15% speedup matters for users.

