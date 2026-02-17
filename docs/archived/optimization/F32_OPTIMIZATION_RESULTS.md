# F32 Optimization Experiment - Results & Analysis

**Date:** February 15, 2026  
**Status:** ✅ REVERTED (optimizations were counterproductive)

## Summary

Attempted two precision reduction strategies for performance optimization:
1. **Fast_math (f64 casting)**: Convert f64 → f32, compute math, convert back
2. **F32_math (native)**: Perform all intermediate calculations in f32

**Result:** Both approaches were *slower* than baseline due to conversion overhead and cache effects.

## Benchmark Results

**3 GB File: `combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits`**

| Configuration | Run 1    | Run 2    | Run 3    | Average  | vs Baseline |
|---------------|----------|----------|----------|----------|-------------|
| **Baseline**  | 10.832s  | 10.565s  | 10.471s  | **10.62s** | —         |
| Fast_math     | 11.224s  | 10.986s  | 10.822s  | 11.01s   | +3.7% slower ❌ |
| F32_math      | 10.700s  | 10.587s  | 11.190s  | 10.83s   | +2.0% slower ❌ |

## Analysis

### Why F32 Was Slower

1. **Casting Overhead (Fast_math approach)**
   - f64 → f32 requires precision loss + register manipulation
   - f32 operations execute fast, but conversion dominates
   - Back-conversion f32 → f64 adds more overhead
   - Net effect: Lost time > time saved from faster math

2. **F32 Precision Issues (F32_math approach)**
   - Although f32 math is faster in isolation
   - Conversion from u32 pixel coords to & back introduces overhead
   - Results in ~2% slowdown vs baseline
   - Higher variance (10.587s to 11.190s) suggests less stable performance

3. **Cache Effects**
   - Full f64 operations may benefit from CPU prefetching
   - F32 conversions disrupt pipeline
   - Modern CPUs with out-of-order execution penalize type conversion

### Why This Matters

Despite math operations being only **11.8% of total CPU time**, trying to optimize them with reduced precision:
- Adds complexity (conditional compilation, multiple code paths)
- Introduces accuracy trade-offs (1e-5 relative error)
- **Doesn't improve performance** due to conversion overhead
- Makes code harder to maintain

## What We Learned

✅ **Insights:**
- Reducing precision alone doesn't help if conversion overhead dominates
- Modern Rust/LLVM already optimizes f64 math operations well
- Cache locality matters more than single-precision math speedups
- The real bottleneck (77.5% in Mollweide projection) isn't in math operations

❌ **What Didn't Work:**
- F64 casting approach (+3.7% regression)
- F32 native approach (+2.0% regression)
- Either approach could be eliminated

## Conclusion

The codebase is already well-optimized for the actual bottleneck (projection calculations and memory layout). Precision reduction attempts failed because:

1. Math is only 11.8% of CPU time
2. Conversion overhead > math speedup
3. Current f64 operations are already efficient

**Next optimization should focus on:**
- Mollweide projection algorithm improvements
- Memory layout optimization (cache efficiency)
- True SIMD with portable_simd (if worthwhile)
- Algorithmic improvements (early rejection optimization, etc.)

## Reverted Files

- Removed `src/fast_math.rs` (139 lines of f64 casting functions)
- Removed `src/f32_math` module from `src/mollweide.rs`  
- Removed `f32_math = []` feature flag from `Cargo.toml`
- Removed `FAST_MATH_IMPLEMENTATION.md` documentation
- Removed `bench_fast_math.py` benchmark script

**Commit:** Reset to `4f3c408` (clean baseline)
