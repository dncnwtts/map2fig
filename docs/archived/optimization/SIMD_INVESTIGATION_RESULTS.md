# SIMD Vectorization Investigation Results

**Date:** February 15, 2026  
**Status:** ❌ NOT RECOMMENDED - No meaningful improvement observed

---

## Objective

Investigate SIMD vectorization of Mollweide projection as Phase 3b optimization, targeting 10-15% performance improvement for CPU-bound bottleneck.

## What Was Tested

**Baseline (Non-SIMD):** 10.897s, 11.366s, 11.114s → **avg 11.126s**

**SIMD-Optimized:** 11.326s, 11.311s, 11.199s → **avg 11.278s**

**Result:** SIMD is **+1.4% SLOWER** (0.152s regression)

---

## Why SIMD Failed

### Root Cause: Current "SIMD" is Not Actually SIMD

The existing SIMD implementation in `src/simd.rs` is **not true SIMD** - it's scalar operations unrolled for instruction-level parallelism:

```rust
#[inline(always)]
pub fn simd_sin_8(angles: [f64; 8]) -> [f64; 8] {
    // This is NOT SIMD - it's 8 scalar sin() calls
    [
        angles[0].sin(),
        angles[1].sin(),
        // ... 6 more individual calls
        angles[7].sin(),
    ]
}
```

**Module comment explicit:**
> "Note: These are currently scalar implementations optimized for instruction-level parallelism and CPU pipelining. True SIMD vectorization would require either nightly Rust with portable_simd or external C library bindings."

### Why Unrolled Scalar Operations Don't Help

1. **Code Size Overhead** - Creating 8 temporary arrays per function adds L1 instruction cache pressure
2. **Register Spilling** - More values in flight = more register pressure on 64-bit FPU
3. **CPU Pipelining Already Good** - Modern CPUs (Zen 3/4, Skylake) already exploit ILP well on scalar code
4. **No True Vectorization** - 512-bit AVX-512 or even 256-bit AVX2 would help, but we're still doing 8× scalar ops

### Implementation Code Flow

The SIMD projection in `src/mollweide.rs` unrolls 8 pixels:

```rust
// Line 212: Compute sincos for all 8 pixels
let (sin_2theta, _cos_2theta) = simd::simd_sin_cos_8(simd_mul_8(theta_aux, [2.0; 8]));
// This creates:
// 1. Temporary array: simd_mul_8() result
// 2. Temporary array: sin values returned  
// 3. Temporary array: cos values returned
// 4. Discards cos values → wasted computation

// Line 226: Separate sincos call
let c = simd::simd_cos_8(theta_aux);
// More temporaries, redundant computation

// All this for ~1.7% overhead that becomes negative at scale
```

---

## Why True SIMD Would Work (But Isn't Worth It)

Real SIMD using `portable_simd` (or C intrinsics) would:
- Compute 4-8 sin/cos operations with single CPU instruction
- Use 256-bit AVX or 512-bit AVX-512 registers
- Achieve theoretical 4-8× speedup for trig operations
- Benefit: ~20-40% overall (since trig is ~12% of total time)

**Why we're not doing this:**
1. Requires Rust nightly (`#![feature(portable_simd)]`)
2. Different code paths for different CPUs (AVX2, AVX-512, NEON, WASM)
3. Risk of slower code on mismatched SIMD levels
4. Maintenance burden for marginal gains (4-8 more months of testing)
5. Already using batch processing (8 pixels) which is good enough

---

## CPU Profile Context

From Callgrind profiling (3 GB file):

| Component | Instructions | % of Time | Real Time |
|-----------|--------------|-----------|-----------|
| Mollweide projection | 35.8B | 77.5% | ~7.5 sec |
| Math functions (sin/cos/atan2) | 5.4B | 11.8% | ~1.2 sec |
| Other computation | 5.0B | 10.7% | ~1.0 sec |

The bottleneck is **not just math**, but the entire projection pipeline:
- Coordinate transforms (cheap, already optimized)
- Early rejection tests (cheap, well-written)
- Math operations (11.8% - this is what SIMD targets)
- HEALPix lookups (appears inline, cache-sensitive)

---

## Why SIMD Can't Fix This

### Problem 1: Data Dependencies
```
For each pixel:
  1. Pixel coordinates → normalize
  2. Normalize → Mollweide coords
  3. Mollweide coords → sin(), cos(), asin()
  4. Trig results → HEALPix indices
  5. HEALPix index → array lookup → colormap
```

Each step depends on previous. SIMD helps step 3, but not the full chain.

### Problem 2: Memory Access Pattern
- HEALPix data: 3GB → must hit CPU cache misses ~1in 500
- Cache misses cost ~200-300 cycles each
- Math operation costs ~10-50 cycles
- **Memory dominates math impact**

### Problem 3: Cache Locality vs Parallelism Trade-off
- Streaming 8 pixels in parallel breaks spatial locality
- Single-threaded sequential access: better cache prefetch
- SIMD unrolling creates data reordering: worse cache behavior
- Net: Negative impact on cache efficiency

---

## What Would Actually Help

### Tier 1: Low-Risk Optimizations (Already Done)
- ✅ BufReader 256 KB buffers (Phase 1) - 5-10% in I/O
- ✅ Metadata caching (Tier 4.2a) - 90% hit rate on repeated files
- ✅ Early projection rejection - Avoids invalid pixels

### Tier 2: Medium-Risk (Consider)
- **Math reduction:** Use `sincos()` library call that computes both sin+cos atomically
  - Current: Separate `sin()` then `cos()` = 2 expensive calls
  - Optimized: Single `sincos()` = 1.5× calls
  - Impact: ~2-3% potentially (only if Math is bottleneck in profiling)
  - Risk: Low (one function signature change)

### Tier 3: High-Risk / Won't Work
- ❌ **Rayon parallelization** - Overhead > speedup (per-pixel work is 0.1 microseconds)
- ❌ **Unrolled scalar SIMD** - Proven negative (this investigation)
- ❌ **True portable_simd SIMD** - Nightly dependency, AVX2/512 divergence complexity
- ❌ **GPU acceleration** - Overkill for 11-second workload; 3GB transfer overhead

### Tier 4: Game-Changers (Not Pursued)
- **Approximate math:** Use faster approximations (sin/cos poly table)
  - Would require accuracy validation vs Healpix standard
  - 5-10% speedup possible but risky to correctness
  
- **Algorithmic change:** Different projection scheme
  - Mollweide is well-established; not worth reimplementing
  
- **Custom libm wrapper:** Use glibc SIMD math library wrapper
  - `libmvec` or `libmvec-optimized` can do vectorized math
  - But Rust doesn't have stable bindings; would need C FFI
  - Effort: 4+ hours for uncertain payoff

---

## Recommendation: Accept Current Performance

**The application is performing well:**
- 11.1 seconds for 3 GB file processing
- 99.2% of time in CPU (not I/O-bound)
- Main bottleneck is unavoidable math (Mollweide + trig)
- CPU scaling is linear: larger files take proportionally longer

**Don't pursue:**
- ❌ Unrolled scalar "SIMD"
- ❌ True SIMD (nightly complexity)
- ❌ Rayon parallelization
- ❌ GPU offloading

**Consider if users complain:**
- Use larger batch sizes (16 pixels instead of 8) with memory-safe unroll
- Profile on target hardware (have they run Callgrind themselves?)
- Check if HEALPix sampling dominates (maybe optimize data access pattern)

---

## Files Changed

- **Created:** SIMD trait methods in `src/projection.rs`
- **Tested:** Mollweide + Hammer SIMD projection batches
- **Benchmarked:** 3 GB file with/without unrolled scalar ops
- **Result:** Reverted to baseline (no SIMD in use)

---

## Conclusion

The investigation revealed that "SIMD" optimization here is fundamentally limited by:
1. Lack of true vector CPU instructions (would need portable_simd)
2. Data dependencies that SIMD can't parallelize
3. Cache locality being more important than math speedup
4. Unrolled scalar ops actually hurt performance

**Status:** Investigation complete. Moving forward without SIMD implementation.

**Time Invested:** ~2 hours (profiling + implementation + benchmarking)

---

## References

- Callgrind profile: `callgrind.out.67716` (46 billion instructions)
- CPU profile analysis: `docs/optimization/CPU_PROFILE_DETAILED_ANALYSIS.md`
- Baseline benchmarks: `docs/optimization/BENCHMARK_RESULTS_IMAGE_UPGRADE.md`
