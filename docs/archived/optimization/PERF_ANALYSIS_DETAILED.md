# Performance Analysis: 3 GB FITS File (Feb 15, 2026)

**File:** `combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits` (3.0 GB)  
**Execution Time:** 22.58 seconds  
**Tool:** `perf stat` (CPU performance counters)

## Raw Performance Metrics

```
Cycles:                   61,507,761,850 (61.5B)
Instructions:             168,558,039,481 (168.6B)
IPC (Instructions/Cycle): 2.74
Branch Instructions:      29,138,503,373
Branch Misses:            17,641,080 (0.061% miss rate)
Cache References:         2,319,802,960
Cache Misses:             850,673,234 (36.67% miss rate)
LLC Loads:                279,593,653
LLC Load Misses:          74,329,328 (26.58% miss rate)

Time Breakdown:
- User time: 17.47s (77.3%)
- System time: 5.05s (22.4%)
- Total: 22.58s
```

## Key Performance Insights

### 1. Cache Efficiency: POOR (36.67% miss rate)

**Finding:** The L1-L3 cache miss rate of 36.67% is significantly higher than optimal.

**Analysis:**
- 2.32 billion cache references
- 850.7 million cache misses (every ~3 accesses miss cache)
- 74.3 million main memory accesses (LLC misses)
- At 200 cycles/miss penalty: **~15 billion stall cycles** (24% overhead)

**Why it matters:**
- Modern CPUs expect <15-20% cache miss rates for data-heavy code
- A miss rate of 36.67% means data access patterns are not cache-friendly
- Either working set is too large, or access patterns have poor spatial locality

**Opportunity:** 20-30% speedup possible with better cache utilization

### 2. Memory Bandwidth: Severely Underutilized (0.27%)

**Finding:** Reading at only 132.8 MB/s on modern hardware with 25-50 GB/s capability.

**Analysis:**
- File size: 3.0 GB
- Time to read: 22.58 seconds
- Current read rate: 3GB ÷ 22.58s = **132.8 MB/s**
- Modern DDR4: 25-50 GB/s
- Bandwidth utilization: **0.27%** (spectacularly inefficient!)

**Why it matters:**
- The CPU is spending most time waiting for data, not processing it
- Each memory miss costs 200+ CPU cycles idle
- Memory is the bottleneck, not the CPU

**Opportunity:** 10-20% speedup with better memory access patterns

### 3. Cache Line Efficiency: 16.8%

**Finding:** Loading 17.9 GB of cache lines to read 3 GB of data.

**Analysis:**
- LLC loads: 279.6 million
- Bytes per cache line: 64
- Total cache traffic: 279.6M × 64 = **17.9 GB**
- Actual data read: 3 GB
- Efficiency: 3GB ÷ 17.9GB = **16.8%**

**Interpretation:**
- Every cache line load brings in only ~10.8 bytes of useful data
- 53.2 bytes per 64-byte cache line are wasted (83.2% waste!)
- Likely causes: Sparse data access, random jumps, or algorithmic issues

### 4. Instruction Quality: Good (IPC 2.74)

**Finding:** Instruction-level parallelism is healthy at 2.74 IPC.

**Analysis:**
- Modern superscalar CPUs can reach 4-5 IPC
- Current 2.74 IPC is respectable (55% of theoretical max)
- Branch prediction accurate: 99.94% (17.6M misses out of 29.1B branches)

**Why it matters:**
- The bottleneck is NOT instruction scheduling
- The bottleneck is NOT branch prediction
- The bottleneck is clearly MEMORY ACCESS

### 5. Throughput: 7.46 Billion Instructions/Second

**Analysis:**
- 168.6B instructions ÷ 22.58s = 7.46 billion insn/s
- At 2.73 GHz, this is 7.46B / 2730M = 2.73 instructions per cycle
- This matches the IPC of 2.74 (consistent measurement)

## Theoretical Performance Limits

### Scenario A: Perfect L1 Cache (100% hit rate on L1 access)
**Assumption:** Eliminate all L1-L3 misses  
**Savings:** ~15 billion cycles eliminated (24% of total)  
**New time:** 22.58s × 0.76 = **17.2 seconds** (-23%)

### Scenario B: 2× Memory Bandwidth Utilization
**Assumption:** Reduce cache line waste by 50%  
**Savings:** 6-8 billion cycles  
**New time:** 22.58s × 0.85 = **19.2 seconds** (-15%)

### Scenario C: Reach Peak IPC (4.0 instead of 2.74)
**Assumption:** Better instruction scheduling  
**Savings:** Execute same instructions 46% faster  
**New time:** 22.58s × (2.74/4.0) = **15.4 seconds** (-31%)
*Note: Limited by memory bottleneck, unlikely to achieve*

### Scenario D: All Optimizations Combined (Realistic)
**Assumptions:**
- 20% improvement in cache efficiency (3-5GB cache line waste reduction)
- 15% improvement in memory access patterns
- 5% improvement in instruction-level parallelism

**Estimated Time:** 22.58s × 0.70 = **15.8 seconds** (-30%)

## Actionable Optimization Opportunities

### Priority 1: Cache Efficiency (20-30% gain potential)

**Problem:** 36.67% miss rate is too high  
**Root Causes:**
- Data access patterns not sequential
- Working set larger than L3 cache chunks
- Possible false sharing or alignment issues

**Solutions:**
1. **Data Layout Optimization**
   - Profile which data structures cause cache misses
   - Reorganize data for better spatial locality
   - Consider struct-of-arrays (SoA) for SIMD alignment
   - Estimated impact: 10-15% speedup

2. **Prefetching Strategy**
   - Add explicit prefetch hints for known access patterns
   - Use `__builtin_prefetch()` in Rust
   - Prefetch next cache lines before they're needed
   - Estimated impact: 5-10% speedup

3. **Cache-Aware Algorithms**
   - Modify projection/scaling to work on cache-line-sized blocks
   - Reduce memory jumps in tight loops
   - Keep hot data in L1/L2
   - Estimated impact: 10-15% speedup

### Priority 2: Memory Bandwidth (10-20% gain potential)

**Problem:** Using only 0.27% of available bandwidth  
**Root Causes:**
- CPU waiting for memory more than computing
- Serialized data loading
- Single-threaded file I/O

**Solutions:**
1. **Batch Processing**
   - Read larger chunks at once
   - Process multiple rows/strips together
   - Better amortize I/O latency
   - Estimated impact: 5-10% speedup

2. **Parallel Processing**
   - Use rayon/tokio for parallel sections
   - Process multiple scanlines concurrently
   - Each thread has its own cache, less contention
   - Estimated impact: 20-40% speedup (but diminishing with 8 cores)

3. **Memory-Mapped I/O**
   - Use mmap for large files
   - Let OS handle prefetching automatically
   - Reduce system call overhead
   - Estimated impact: 3-8% speedup

### Priority 3: Instruction-Level Parallelism (5-10% gain potential)

**Problem:** IPC at 2.74 could reach 4.0  
**Limitation:** Memory stalls block instruction execution  
**Solutions:**
1. **Software Pipelining**
   - Overlap computation with memory latency
   - Unroll loops to hide memory delays
   - Estimated impact: 3-5% speedup (if not memory-limited)

2. **Vectorization (SIMD)**
   - true portable_simd for data parallelism
   - But only if memory access improves first
   - Estimated impact: 5-15% speedup (depends on bandwidth)

### Priority 4: Avoid Micro-Optimizations

**Already Tested (DO NOT RETRY):**
- ❌ F32 precision reduction: -2% to -3.7% (slower)
- ❌ Unrolled scalar SIMD: -1.4% (slower)
- ❌ True portable_simd: 8 hours for 2-3% (poor ROI)

## Recommendations for Optimization

**High Priority (start here):**
1. ✅ Profile cache behavior with perf (done)
2. 🔲 Identify hot data structures causing cache misses
3. 🔲 Improve spatial locality in projection algorithm
4. 🔲 Consider memory-mapped I/O for FITS reading

**Medium Priority:**
1. 🔲 Evaluate parallel processing (rayon for independent rows)
2. 🔲 Implement prefetching for known access patterns
3. 🔲 Benchmark against optimized versions

**Low Priority (defer):**
1. ❌ Precision reduction (F32) - already proven slower
2. ❌ Micro-optimizations - limited by memory
3. ❌ Complex SIMD - wait for memory optimizations first

## Summary

- **Current Performance:** 22.58 seconds for 3 GB file
- **Bottleneck:** Memory access (36.67% cache miss rate)
- **Bandwidth Utilization:** Only 0.27% of available
- **Realistic Ceiling:** 15-18 seconds (-30-35% improvement)
- **Key Insight:** Cache efficiency, not CPU speed, is the blocker

The application is **memory-bound, not CPU-bound**. Optimizations using precision reduction, instruction-level tricks, or micro-optimizations will not help. Focus on memory access patterns, cache locality, and I/O efficiency.
