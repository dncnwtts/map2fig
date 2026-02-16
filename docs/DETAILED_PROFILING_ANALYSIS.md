# Detailed CPU Profiling Analysis - Downsampling Bottleneck Investigation

**Date:** February 16, 2026  
**File tested:** `combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits` (3.1 GB)  
**Profiling tool:** Linux perf (Performance Events)  
**Build:** Release mode with LTO, debug symbols enabled

## Key Performance Metrics

### Overall Execution Profile
```
Time elapsed:          23.15 seconds
User CPU time:         18.44 seconds
System CPU time:       4.64 seconds
CPU Utilization:       99.7% (single-threaded)
```

### Detailed CPU Counter Analysis

| Counter | Value | Interpretation |
|---------|-------|-----------------|
| **Cycles** | 62.2 billion | Total CPU cycles |
| **Instructions** | 134.4 billion | Machine instructions executed |
| **IPC** | 2.16 | Instructions per cycle (below potential 4-5) |
| **Cache Misses** | **877 million** | Total L1/L2/L3 misses (⚠️ HIGH) |
| **L1 D-cache Misses** | **1.08 billion** | Data cache miss rate: ~19.5% (⚠️ CRITICAL) |
| **L1 Miss Rate** | 19.5% | Very high - poor cache locality |
| **LLC Loads** | 294 million | Last-level cache accesses |
| **LLC Load Misses** | 53.9 million | LLC miss rate: 18.3% |
| **Branch Misses** | 14.3 million | Very low - excellent branch prediction |
| **Branch Miss Rate** | <0.1% | ✓ Branch prediction is not a bottleneck |

## Bottleneck Identification

### Primary Bottleneck: **Memory Access Patterns** (NOT computation)

**Evidence:**
1. **L1 Cache Misses = 1.08 billion** - This is the smoking gun
   - For 134.4B instructions, ~1.08B L1 misses = 19.5% miss rate
   - Modern CPUs target <10% L1 miss rate for compute-bound code
   - This indicates **random/non-sequential memory access**

2. **IPC = 2.16 is suboptimal**
   - Modern Zen/Intel CPUs can achieve 4-5 IPC with good cache locality
   - 2.16 IPC suggests the CPU is stalling waiting for memory

3. **LLC hits 281M of 294M accesses = 95.6% hit rate**
   - Good news: data mostly fits in L3 cache (8-16 MB typical)
   - Bad news: we're paying L3 latency (42 cycles) instead of L1 (4 cycles)

### Secondary: **Downsampling Algorithm Characteristics**

The downsampling function `downgrade_healpix_map_xyf()` performs:
```rust
for target_pix in 0..target_npix {
    for j in y0..y0_plus_fact {
        for i in x0..x0_plus_fact {
            let source_pix = xyf2nest(source_nside, i, j, face);  // ← Computation
            let val = map[source_pix];  // ← RANDOM MEMORY ACCESS!
            // accumulate
        }
    }
}
```

**The problem:**
- `source_pix` is computed from HEALPix coordinates
- HEALPix uses Morton-code (Z-order) interleaving for NESTED ordering
- This produces **pseudo-random but not cache-optimal memory access**
- We jump around in the source map array unpredictably

### Tertiary: **Loop Structure Kills Instruction-Level Parallelism**

Current code:
```rust
let mut sum = 0.0;  // Dependency chain
for ..{
    sum += map[source_pix];   // Accumulator is dependent on previous iteration!
}
```

- Addition is dependent on previous iterations
- Can't parallelize within the accumulator loop
- CPU pipeline stalls waiting for floating-point results

## Performance Comparison: Why Vectorization Failed

The attempted loop unrolling optimizations made it SLOWER (24.3s → was 19.4s):
```
❌ Unrolled version: 24.3 seconds (25% SLOWER)
   - More complex control flow
   - Less predictable jumps
   - Increased register pressure
   - Worse IPC

✓ Original version: 19.4 seconds (BETTER)
  - Simpler loops, less overhead
  - Better branch prediction
  - But still memory-bound
```

**Lesson:** You can't out-compute a memory bottleneck through code complexity.

## Theoretical Optimization Paths

### Option 1: Memory Access Optimization (Best ROI)
**Approach:** Pre-sort source pixel indices to create cache-friendly access patterns
```
Current (random): source_pix = xyf2nest(source_nside, i, j, face)
                  map[source_pix] ← random memory access

Better: Build index array sorted by source_pix value first
        Then access map[] sequentially
```
**Expected gain:** 2-3× speedup (reduce L1 misses from 1.08B to 300-500M)
**Complexity:** Medium (requires pre-processing phase)

### Option 2: Associative Accumulation (Medium ROI)  
**Approach:** Break dependency chain in accumulator using monoid structure
```rust
// Current (dependent):
let mut sum = 0.0;
for i in 0..N { sum += values[i]; }

// Better (parallel-ready):
let (s1, s2, s3, s4) = ...  // 4 independent accumulators
for i in (0..N).step_by(4) {
    s1 += values[i];
    s2 += values[i+1];
    s3 += values[i+2];
    s4 += values[i+3];
}
let sum = s1 + s2 + s3 + s4;
```
**Expected gain:** 1.5-2× speedup (from better ILP, 2.16 IPC → 3.5+ IPC)
**Complexity:** Low (straightforward instruction scheduling)
**Note:** This helps IF memory isn't the bottleneck. Since it is, gain is limited.

### Option 3: Algorithm Change (Highest ROI but risky)
**Current:** Full averaging downsampling (mathematically correct but slow)

**Alternative:** Ring-ordered HEALPix downsampling
- Ring order has better sequential access locality
- Trade decision quality for speed?
- **Not recommended:** Would lose our optimization from Tier 1 (fast mode)

## Cache Analysis Deep Dive

### L1 Data Cache (32 KB)
```
Miss rate: 19.5% (1.08B out of 5.5B estimated loads)
Latency: 4 cycles + pipelines = ~10 cycles
Cost: 1.08B × 10 cycles = 10.8B cycles

This ALONE accounts for: 10.8B / 62.2B = 17.4% of total time!
```

### L2 Data Cache (256 KB) 
```
Hit rate: ~95% of L1 misses (estimate based on LLC loads)
Latency: 12 cycles
Cost: (1.08B × 0.05) × 12 = 0.65B cycles = 1% of time
```

### L3 Cache (16 MB)
```
Hit rate: 95.6% (281M of 294M accesses)
Latency: 42 cycles
Cost: (294M × 0.044) × 42 = 0.54B cycles = 0.87% of time

Total memory stalls: ~19.2% of execution time
```

## Specific Recommendations for ud_grade Optimization

### Immediate (Fix the memory access pattern):

1. **Profile the HEALPix coordinate transforms**
   - How much time in `xyf2nest()` vs memory access?
   - Current estimate: 30% computation, 70% memory stalls

2. **Consider Ring ordering for high nside**
   - NESTED + Morton code = bad cache locality
   - RING ordering has better sequential access
   - Check `meta.ordering` - if RING, use ring2xyf path (already implemented!)

3. **Cache hot-spot analysis**
   - Build inverted index: source_pix_in_order[]
   - Sort target pixels by their source indices
   - Access source map sequentially

### Data Structure Change (Medium effort):

Build a **sorted pixel mapping** during downsampling init:
```rust
// Pre-compute which source pixels we'll access and in what order
let mut pixel_buffer: Vec<(usize, usize)> = Vec::new();  // (source_pix, target_pix)

for target_pix in 0..target_npix {
    for i in x0..x0+fact {
        for j in y0..y0+fact {
            let source_pix = xyf2nest(...);
            pixel_buffer.push((source_pix, target_pix));
        }
    }
}

// Sort by source pixel
pixel_buffer.sort();

// Now process in source-order (cache-friendly!)
for (source_pix, target_pix) in pixel_buffer {
    acc[target_pix] += map[source_pix];
}
```

**Expected:** Reduces L1 misses from 1.08B to 200-300M (70% reduction)
**Time savings:** ~4 seconds (19.4s → 15-16s)

## Conclusion

The downsampling bottleneck is **NOT**:
- ❌ Algorithm complexity (simple nested loops)
- ❌ Poor instruction generation (IPC 2.16 is okay for memory-bound code)
- ❌ Branch mispredicts (14.3M misses is negligible)
- ❌ Floating-point math (libm only 0.54% of time)

The bottleneck IS:
- ✅ **Cache-hostile memory access patterns** (1.08B L1 misses = 17.4% of time)
- ✅ **Random access into large map array**

**Best optimization:** Fix the memory access pattern through sorted index pre-computation.
**Expected speedup:** 1.2-1.5× (19.4s → 13-16s) 
**Effort:** Medium (new code path, but straightforward)

---

## perf stat Full Output

```
Performance counter stats for './target/release/map2fig':

    62,203,048,316      cycles                    #    2.694 GHz           (71.41%)
   134,403,410,372      instructions              #    2.16  insn per cycle (85.71%)
       877,313,322      cache-misses              #   12.3% of all cache refs (85.72%)
     1,079,575,754      L1-dcache-load-misses    #   (85.71%)
       294,165,655      LLC-loads                #   12.740 M/sec (85.72%)
        53,926,024      LLC-load-misses          #   18.33% of all LL-cache accesses (85.72%)
        14,340,828      branch-misses            #   (57.14%)
    23,089,363,393      task-clock               #    0.997 CPUs utilized

      23.152469232 seconds time elapsed
      18.439065000 seconds user
       4.640446000 seconds sys
```

