# Tier 3b Pre-Allocation Optimization - FAILED

## Executive Summary

**Status**: ❌ **FAILED & REVERTED**

The Tier 3b optimization (pre-allocating 5 arrays outside the main rendering loop) resulted in a **71% performance regression**. The optimization backfired significantly and should be reverted.

## Benchmark Results (nside 8192, 3GB FITS file)

### Baseline (Original Code)
```
Wall-clock:      10.83 seconds
Cycles:          28.6 billion
Instructions:    55.6 billion (1.95 insn/cycle)
Cache misses:    23.44% of all cache refs
LLC load misses: 11.17% of all LL-cache accesses
```

### With Tier 3b Pre-allocation
```
Wall-clock:      18.57 seconds  ⚠️ +71.4% SLOWER
Cycles:          50.6 billion   ⚠️ +76.9% MORE
Instructions:    130.3 billion  ⚠️ +134.2% MORE  
Cache misses:    31.17% of all cache refs  ⚠️ +7.7pp WORSE
LLC load misses: 16.56% of all LL-cache accesses  ⚠️ +5.4pp WORSE
```

## Performance Delta Summary

| Metric | Baseline | Tier 3b | Change | % Change |
|--------|----------|---------|--------|----------|
| Wall-clock (sec) | 10.83 | 18.57 | +7.74 | +71.4% ❌ |
| Cycles (billions) | 28.6 | 50.6 | +22.0 | +76.9% ❌ |
| Instructions (billions) | 55.6 | 130.3 | +74.7 | +134.2% ❌ |
| Instructions/cycle | 1.95 | 2.58 | +0.63 | +32.3% |
| Cache misses (%) | 23.44% | 31.17% | +7.73pp | +33.0% ❌ |
| LLC misses (%) | 11.17% | 16.56% | +5.39pp | +48.3% ❌ |

## Analysis

### Why This Optimization Failed

1. **Compiler Optimization Inhibition**: The original loop-local arrays were likely optimized by LLVM's compiler in ways that pre-allocated outer-scope arrays prevent:
   - Stack-allocated arrays can be better optimized with stack frame analysis
   - Register allocation may be more efficient with smaller lexical scopes
   - The optimizer may aggressively inline/unroll with smaller arrays

2. **Register Pressure**: Moving arrays to outer scope increases their lifetime significantly, which can:
   - Prevent the compiler from reusing registers effectively
   - Force spills to memory in tight loops
   - Degrade instruction-level parallelism

3. **Cache Aliasing**: Pre-allocated outer-scope arrays may trigger:
   - More conservative memory aliasing assumptions
   - Reduced dead-code elimination opportunities
   - Worse memory access patterns due to array reuse

4. **Lost Loop Optimizations**: The compiler may have been performing:
   - Loop-invariant code motion within the tight inner loop
   - Better escape analysis with smaller scopes
   - More aggressive dead store elimination

### Evidence of Compiler Issues

The **134% increase in instructions** (55.6B → 130.3B) is the smoking gun:
- This massive inflation indicates the compiler is generating much more code
- Likely due to increased register pressure and spills
- More cache misses (31.17% → 23.44%) confirm memory bottlenecks

## Lessons Learned

### ❌ Do NOT Attempt Similar Optimizations
1. **Avoid pre-allocating arrays to reduce allocation churn** when:
   - Arrays are in tight, well-optimized loops
   - The compiler is already handling allocation efficiently
   - The array lifetime would expand significantly

2. **Trust modern compilers**: LLVM is sophisticated at:
   - Stack allocation optimization
   - Register allocation in tight loops
   - Cost of allocation vs. cost of spills

3. **Measure before and after**: This optimization seemed theoretically sound but was catastrophically wrong in practice

### ✅ What Might Work Instead
For stack allocation churn (if it's actually a bottleneck):
1. Use custom allocators (likely overkill for this workload)
2. Reduce array sizes where possible
3. Work at a coarser granularity (batch more pixels)
4. Profile to find actual bottleneck first

## Recommendation

**REVERT THIS OPTIMIZATION IMMEDIATELY**. The performance regression is severe and unambiguous.

### Revert Instructions
```bash
git revert <commit-hash-of-tier3b>
cargo build --release
```

## Conclusion

This is a textbook example of how even well-intentioned "low-level" optimizations can backfire when fighting against modern compiler optimizations. The lesson: **measure first, assume nothing, and trust the compiler's analysis of small allocations in tight loops.**
