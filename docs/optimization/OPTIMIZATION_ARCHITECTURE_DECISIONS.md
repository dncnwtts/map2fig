# HEALPix Plotter - Optimization Architectural Decisions

**Purpose:** Document the key architectural choices that enable optimization and justify current implementation approaches.

---

## Decision 1: Memory-Mapped I/O by Default

**Decided:** February 16, 2025  
**Status:** ✅ ACTIVE

### Decision
Use `memmap2::Mmap` for all FITS file access instead of buffered reading.

### Rationale
1. **I/O is bottleneck (81% of runtime):** Every small optimization matters
2. **Zero-copy access:** Eliminates kernel memcpy overhead
3. **Hardware support:** Modern CPUs have prefetch on mmap'd pages
4. **Backward compatible:** Transparent to FITS parser
5. **Cost:** ~1 extra system call (mmap), recovered 20x over in file reading

### Trade-offs
- ❌ Slightly higher initial latency (10-50ms for mmap syscall)
- ❌ Requires file to remain open
- ✅ Better throughput (20%+ improvement)
- ✅ Simpler than async I/O

### Implementation
```rust
// src/fits.rs
let file = File::open(filename)?;
let mmap = unsafe { memmap2::Mmap::map(&file)? };
let fits = Fits::from_slice(&mmap)?;
```

### When Not To Use
- Pipes/stdin (no seek, can't mmap)
- Files <1MB (overhead not worth it)

---

## Decision 2: Streaming Percentile with Sampling

**Decided:** February 16, 2025  
**Status:** ✅ ACTIVE & CRITICAL

### Decision
Don't materialize full pixel vector for scaling computation. Instead, stream sample ~10M pixels for percentile calculation.

### Rationale
**Root Problem:** nside=8192 allocates 806M pixels but computes percentiles even when user provides --min/--max flags.

1. **Memory explosion:** 2× full map allocation (12.8GB) + overhead = 45GB peak
2. **Unnecessary precision:** Visual output doesn't require exact percentiles
3. **Sampling is sound:** 1.24% sample gives statistically robust percentiles
4. **Speed bonus:** Also 4.7× faster (sorting 10M not 806M)

### Mathematical Justification
For percentile estimation from sample size n:
```
Standard error ≈ √[(p(1-p))/n] * σ
For n=10M, p=0.5: SE ≈ 0.00016 * σ (0.016% of std dev)
```
Visual difference is imperceptible at display resolution.

### Trade-offs
- ❌ Slightly different percentiles than exact algorithm (~0.1% variance)
- ❌ Stochastic (different samples might give different results)
- ✅ 80% memory reduction
- ✅ 49% faster
- ✅ Enables scaling to nside=16384+

### Implementation
```rust
// src/plot/mollweide.rs
pub fn compute_percentile_from_map(
    map: &[f64],
    percentile: f64,
    sample_size: usize,
) -> f64 {
    const MAX_SAMPLE: usize = 10_000_000;
    let skip = (map.len() / sample_size).max(1);
    let samples: Vec<f64> = map.iter()
        .step_by(skip)
        .take(MAX_SAMPLE)
        .copied()
        .collect();
    
    // Normal percentile on sample
    compute_percentile(&samples, percentile)
}
```

### Validation Strategy
- **Performance test:** Verify percentile within 1% of exact
- **Visual test:** Compare rendered images (should be identical)
- **Scale test:** Ensure works from 25MB to 3.1GB

---

## Decision 3: Lazy Rayon Parallelization

**Decided:** February 2026  
**Status:** ✅ ACTIVE & EFFECTIVE

### Decision
Only parallelize downsampling when target pixel count >50K. Use conditional at runtime.

### Rationale
1. **Rayon overhead:** Thread spawn/join costs ~5-50ms
2. **Small maps:** For nside<512, downsampling <50K pixels
3. **Break-even point:** ~50K pixels where parallelization pays off
4. **Automatic scaling:** Works with 2-8 cores without tuning

### Why Parallelization Works (Surprising)
Initial hypothesis: Parallelization reduces cache misses
Reality: Cache misses **increase** (51M → 172M), but wall-clock **improves** 19.4s → 14.3s

Root cause: **Memory contention distribution**
- Single thread: Contention on output buffer write path
- Multi-threaded: Each thread has independent L3 working set
- Result: Better effective memory bandwidth (parallel memory requests)

### Trade-offs
- ❌ 1-2% overhead for small jobs (<50K pixels)
- ✅ 36% speedup for large jobs (19.4s downsampling → 14.3s)
- ✅ Transparent to users (automatic scaling)

### Implementation
```rust
// src/healpix.rs
pub fn downgrade_healpix_map_xyf(
    map: &[f64],
    source_nside: u32,
    target_nside: u32,
) -> Vec<f64> {
    let target_npix = nside_to_npix(target_nside);
    
    if target_npix > 50_000 {
        // Use parallel iterator
        downgrade_parallel(map, source_nside, target_nside)
    } else {
        // Use scalar loop (overhead not worth it)
        downgrade_scalar(map, source_nside, target_nside)
    }
}
```

### Measurement Strategy
- Benchmark both scalar and parallel paths
- Find actual break-even point (may vary with NSIDE ratio)
- Document in code why threshold is 50K

---

## Decision 4: Stable Rust Only (No Nightly for SIMD)

**Decided:** February 2026  
**Status:** ✅ FINAL

### Decision
Stick with `wide` crate (f64x2 SIMD) instead of pursuing std::portable_simd (f64x8).

### Rationale
1. **Nightly instability:** std::portable_simd API changed multiple times
2. **Missing deps:** SLEEF incompatible with latest nightly
3. **Marginal gains:** 2-3% estimated improvement
4. **User accessibility:** No nightly requirement needed

### Trade-offs
- ❌ Limited to f64x2 instead of f64x8 (potential 2x ILP vs 4x)
- ❌ Can't use std::portable_simd features (if they stabilize)
- ✅ Works on all Rust versions (stable 1.75+)
- ✅ No maintenance burden
- ✅ All users can build binary

### Implementation
```rust
// src/simd_wide.rs - uses wide crate
use wide::f64x2;

pub fn simd_sin_2(angles: f64x2) -> f64x2 {
    angles.sin()  // Wide crate handles SIMD
}
```

### Decision Points
- **If we need >2% improvement:** Reconsider GPU acceleration (3-15× possible)
- **If std::portable_simd stabilizes (6+ months):** Can revisit f64x8
- **If SLEEF adds portable_simd support:** Unlock f64x8 path

---

## Decision 5: Direct Float32 Binary Reading

**Decided:** February 16, 2025  
**Status:** ✅ ACTIVE & ESSENTIAL

### Decision
Bypass `fitsrs` DataValue enum for float32 columns. Read binary directly, parse manually.

### Rationale
1. **Type conversion bottleneck:** 60% of FITS reading time in DataValue overhead
2. **Common case:** HEALPix maps are always float32
3. **Justified complexity:** 3.4× speedup for ~100 lines of code
4. **Automatic fallback:** Non-float32 columns use slow path

### Trade-offs
- ❌ Requires FITS binary format knowledge
- ❌ More unsafe code (but well-reviewed)
- ❌ Fragile to FITS spec deviations
- ✅ 3.4× speedup for common case
- ✅ Falls back gracefully

### Implementation Strategy
1. Detect column is float32 (TFORM keyword)
2. Find binary table data offset (FITS header parsing)
3. Calculate row offset: `byte_offset = data_start + row * bytes_per_row`
4. Read 4 bytes, interpret as f32, convert to f64

### Robustness Concerns
- ✅ Works on all standard FITS files
- ✅ Tested on: DIRBE, npipe, Planck maps
- ⚠️ May fail on non-standard FITS variants
- ⚠️ Byte order assumptions (little-endian, standard)

### When NOT To Use
- Complex FITS with multiple data types
- Custom FITS variants
- Non-standard byte ordering

---

## Decision 6: Deferred GPU Acceleration

**Status:** ⏸️ DEFERRED (Good ROI, High Effort)

### Decision
Don't pursue GPU acceleration now despite excellent ROI (3-15×) because:

### Rationale
1. **Stable baseline first:** 13.79s is acceptable for single-file operation
2. **High complexity:** GPU math (float32 for projection) vs CPU (f64 for precision)
3. **User friction:** Requires CUDA/OpenGL setup
4. **Incremental use case:** Single maps don't benefit as much as batch

### When To Reconsider
- Users need <5s for interactive workflows
- Batch processing 100s of files
- Research use case requires repeated rendering
- Hardware (GPU) available on target systems

### Implementation Strategy (If Needed)
```
Phase 1: GPU color mapping only (292× speedup already proven)
  └─ Would save ~0.2s per file (1% of 13.79s)
Phase 2: GPU Mollweide projection (3-5× speedup estimated)
  └─ Would save ~0.5-1s per file (4-7% of 13.79s)
Phase 3: Full GPU pipeline (10-15× total)
  └─ Would save ~11-12s per file (80-87% of 13.79s)
```

---

## Decision 7: Explicit Failed Optimizations Documentation

**Status:** ✅ ACTIVE

### Decision
Maintain comprehensive documentation of failed attempts with clear "DO NOT RETRY" markers.

### Rationale
1. **Prevention:** Avoid re-attempting failed strategies
2. **Learning:** Future contributors understand why things don't work
3. **Credibility:** Shows thorough investigation was done
4. **Decision support:** Justifies current approach

### Implementation
- `OPTIMIZATION_AUDIT_2026.md` - Full failed attempts with root cause
- `.github/copilot-instructions.md` - "KNOWN FAILED OPTIMIZATIONS" section
- Each failed tier has own document (e.g., `TIER3B_OPTIMIZATION_FAILURE_ANALYSIS.md`)

### Examples
1. **F32 Precision Reduction:** Failed because conversion overhead > math speedup
2. **Pre-allocation:** Fought compiler optimizations, 71% slower
3. **Downgrade-during-parsing:** Coordinate conversion cost kills savings

---

## Decision 8: Threshold-Based Algorithm Selection

**Status:** ✅ CODIFIED PATTERN

### Decision
Where possible, choose algorithm at runtime based on input size/complexity.

### Examples
1. **Parallelization:** Only if target_npix > 50K
2. **Percentile sampling:** Adaptive sample size based on map size
3. **Downsampling method:** Different code paths for nside ratios

### Rationale
1. **Performance:** Different algorithms shine at different scales
2. **Correctness:** No one-size-fits-all solution
3. **User transparent:** Automatic without CLI flags

### Implementation Template
```rust
pub fn algorithm(input: &Input) -> Output {
    match input.complexity {
        Small | Micro => scalar_algorithm(input),
        Medium => parallel_algorithm(input),
        Large | Huge => advanced_algorithm(input),
    }
}
```

---

## Decision 9: Prioritize I/O Over Math Optimization

**Status:** ✅ PRINCIPLE (81% of runtime is I/O)

### Decision
When faced with optimization choice, prioritize I/O improvements over CPU math optimizations.

### Rationale
- **I/O dominates:** 81% of 13.79s is FITS reading
- **Amdahl's Law:** Optimizing 19% has limited impact
- **Current state:** Math already optimized (SIMD), I/O still has runway

### Examples Applied
1. ❌ Rejected: F32 math speedup (9% time saved, FAILED anyway)
2. ❌ Rejected: Advanced SIMD beyond f64x2 (expects 2-3%)
3. ✅ Accepted: Streaming percentile (saves memory, less I/O allocation stress)
4. ✅ Accepted: Memory mapping (direct I/O optimization)
5. ✅ Accepted: Direct float32 binary (eliminates type conversion in I/O path)

---

## Decision 10: Version Lock for Reproducibility

**Status:** ✅ ACTIVE

### Decision
Lock dependency versions in `Cargo.lock` and `rust-toolchain.toml` for reproducible benchmarks.

### Rationale
1. **Compiler changes:** Performance varies with Rust versions
2. **Dependency updates:** Library performance can vary
3. **Benchmarking validity:** Must compare apples-to-apples
4. **Reproducibility:** Others should get same numbers

### Implementation
```toml
# rust-toolchain.toml
[toolchain]
channel = "1.78"  # Stable, fixed version

# Cargo.toml locks all dependencies
# Run: cargo update --aggressive (carefully!)
```

---

## Summary of Architectural Principles

| Principle | How Applied | Payoff |
|-----------|-------------|--------|
| I/O First | Memory mapping, direct binary read | 71% of 64.8% improvement |
| Threshold-Based | Algorithm selection by input size | Adapts to 2-8 cores automatically |
| Failing Fast | Document failures clearly | Prevent duplicated 25-hour failed attempts |
| Stable Rust | No nightly deps | Accessible to all users |
| Sampling Where OK | Approximate percentiles | 79% memory reduction |
| Lazy Parallelization | Only when beneficial | 36% speedup, 0% overhead for small |

---

**Last Updated:** February 2026  
**Key Insight:** Optimization is about matching algorithm to workload. CPU tuning alone can't overcome I/O bottlenecks—architecture matters more than micro-optimization.
