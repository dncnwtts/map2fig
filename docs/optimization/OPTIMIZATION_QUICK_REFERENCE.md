# HEALPix Plotter - Optimization Quick Reference

**Current Status:** 13.79 seconds (64.8% improvement from baseline)

---

## What's Been Optimized ✅

| Tier | Technique | Speedup | Status | Notes |
|------|-----------|---------|--------|-------|
| **1** | Direct float32 binary reading | 3.4× | ✅ ACTIVE | Elimnates enum overhead |
| **1.1** | Memory-mapped I/O | 1.21× | ✅ ACTIVE | Single-line change |
| **1.2** | Streaming percentile computation | 1.5× | ✅ ACTIVE | 79% memory reduction |
| **4** | Rayon parallelization | 1.36× | ✅ ACTIVE | Multi-threaded downsampling |
| **2** | SIMD (wide crate f64x2) | 1.04× | ✅ ACTIVE | Transcendental vectorization |

**Total Combined:** 64.8% improvement (39.2s → 13.79s)

---

## What Failed ❌

| What | Result | Why | Status |
|------|--------|-----|--------|
| F32 precision reduction | -2-3% slower | Conversion overhead | ❌ REVERTED |
| Pre-allocation (Tier 3b) | -71% slower | Fought compiler optimization | ❌ REVERTED |
| Downgrade-during-parsing | -25% slower | Coordinate conversion overhead | ❌ ABANDONED |
| std::portable_simd (f64x8) | +2-3% potential | SLEEF incompatible, nightly only | ⚠️ DEFERRED |

---

## Current Bottleneck

**Wall-Clock Breakdown (13.79s):**
- FITS Reading: 11.2s (81%) ← **PRIMARY BOTTLENECK**
- Projection + Scaling: 1.9s (14%)
- Rendering: 0.7s (5%)

**Key Finding:** 80% of runtime is I/O or rendering. Further CPU optimizations yield <2% returns.

---

## Before You Optimize Something

### ✅ DO

- [ ] **Profile first:** Use `cargo flamegraph` to identify actual bottleneck
- [ ] **Measure baseline:** Run benchmark 3+ times, report mean
- [ ] **Measure after:** Same benchmark, same conditions, report comparison
- [ ] **Document decision:** Why you think this will work
- [ ] **Check copilot-instructions.md:** See what's been tried before
- [ ] **Check OPTIMIZATION_AUDIT_2026.md:** See failed attempts
- [ ] **Plan rollback:** If it's slower, revert in <5 minutes
- [ ] **Test on multiple file sizes:** Small (6 MB), medium (200 MB), large (3+ GB)

### ❌ DON'T

- ❌ Don't assume "allocation churn" is a problem (it's not for small arrays)
- ❌ Don't reduce float precision without measuring conversion cost
- ❌ Don't parallelize without 10K+ workload (Rayon overhead not worth it)
- ❌ Don't use nightly Rust features for <5% gain (restricts user base)
- ❌ Don't trust compiler optimization intuition (modern compilers are smart)
- ❌ Don't attempt SIMD beyond f64x2 (nightly only, incompatible libs)
- ❌ Don't fuse algorithms to "reduce allocations" (coordinate conversion overhead kills it)

---

## Recommended Next Optimizations

### Priority 1: Cache-Aware Loop Reordering (5-8% gain)
- **What:** Process pixels in Morton/Z-order instead of row-major
- **Effort:** 10-15 hours
- **Expected Result:** 13.79s → 12.7s
- **ROI:** Good
- **Difficulty:** Medium

### Priority 2: GPU Acceleration (3-15× gain)
- **What:** Offload Mollweide projection to GPU
- **Effort:** 40-80 hours
- **Expected Result:** 13.79s → 1-4s
- **ROI:** Excellent
- **Difficulty:** Hard

### Priority 3: Asynchronous I/O (10-15% gain)
- **What:** Pipeline FITS reading with rendering
- **Effort:** 15-30 hours
- **Expected Result:** 13.79s → 11.5s
- **ROI:** Good
- **Difficulty:** Medium

### NOT Recommended: Further SIMD Optimization
- f64x8 would require nightly Rust + SLEEF
- Expected gain: 2-3%
- Trade-off: Nightly dependency, maintenance burden
- **Verdict:** Not worth it

---

## Key Lessons Learned

1. **Amdahl's Law is real:** Each optimization tier yielded smaller gains:
   - Tier 1: 3.4× (attacks 70% of bottleneck)
   - Tier 1.2: 1.5× (attacks memory)
   - Tier 4: 1.36× (parallelization)
   - Tier 2: 1.04× (SIMD)

2. **I/O dominates:** 81% of runtime is file reading. CPU optimizations hit hard ceiling.

3. **Trust the compiler:** Pre-allocation and math optimization failed because modern compilers are already doing these at the assembly level.

4. **Measure everything:** Intuition about which optimization "should work" is often wrong (F32 precision, pre-allocation both seemed good but failed).

5. **Memory bandwidth is the new limit:** Current system uses ~42 GB/s of ~55 GB/s available. Can't extract more without algorithm redesign.

---

## Development Workflow

### Building for Benchmarking
```bash
# Standard release build
cargo build --release

# Maximum optimization (with LTO)
cargo build --release -C target-cpu=native -C lto=fat

# Profile (with flamegraph)
cargo flamegraph --release -- -f file.fits -o /tmp/out.pdf
```

### Benchmarking a Change
```bash
# Baseline
time cargo run --release -- -f combined_map_95GHz_nside8192.fits -o /tmp/out.pdf

# Make change, rebuild
cargo build --release

# Measure again
time cargo run --release -- -f combined_map_95GHz_nside8192.fits -o /tmp/out.pdf

# Calculate improvement
# Example: was 13.79s, now 13.50s = 0.29s improvement = 2.1%
```

### Profiling with Perf
```bash
# Generate flame graph
perf record -g cargo run --release -- [args]
perf script | stackcollapse-perf.pl | flamegraph.pl > flamegraph.svg

# Or use cargo-flamegraph directly
cargo flamegraph --release -- [args]

# Detailed metrics
perf stat -r 5 cargo run --release -- [args]

# Cache analysis
perf record -e LLC-load-misses cargo run --release -- [args]
```

---

## File Organization

**Documentation:**
- `OPTIMIZATION_AUDIT_2026.md` - Comprehensive history (this is the canonical reference)
- `docs/MASTER_OPTIMIZATION_STATUS.md` - Detailed tier-by-tier breakdown
- `docs/NIGHTLY_PORTABLE_SIMD_INVESTIGATION.md` - Why f64x8 SIMD was abandoned
- `.github/copilot-instructions.md` - Project guidelines + failed optimizations

**Code:**
- `src/fits.rs` - Tier 1 optimizations (fast float32 reading)
- `src/healpix.rs` - Tier 4 optimization (parallel downsampling)
- `src/simd.rs` / `src/simd_wide.rs` - Tier 2 optimizations (SIMD math)
- `src/plot/mollweide.rs` - Tier 1.2 optimization (streaming percentile)

---

## Testing & Validation

When implementing a new optimization:

1. **Correctness:** Visual output must be identical to baseline
   - Compare PDF output pixel-by-pixel
   - Use existing test images for validation

2. **Performance:** Must show measurable improvement
   - Use multiple file sizes (small, medium, large)
   - Run 3+ times, report average
   - Accept >1% variation (system noise)

3. **Robustness:** Must not break edge cases
   - Empty maps
   - All-NaN maps
   - Single-pixel maps
   - Various NSIDE values

4. **Documentation:** Must document findings
   - Create `docs/optimization/TIERX_RESULTS.md`
   - Update `OPTIMIZATION_AUDIT_2026.md`
   - Update `.github/copilot-instructions.md` if major

---

## Performance Targets

| Scenario | Target | Status | Notes |
|----------|--------|--------|-------|
| Small file (6 MB, nside=128) | <0.6s | Current: 0.57s | ✅ MET |
| Medium file (193 MB, nside=512) | <2s | Current: 1.60s | ✅ MET |
| Large file (577 MB, nside=1024) | <3s | Current: 2.70s | ✅ MET |
| Extra-large file (3.1 GB, nside=8192) | <15s | Current: 13.79s | ✅ MET |
| PNG rendering | <1s | Current: 0.9s | ✅ MET |

---

## Contact & Questions

If you're planning an optimization:
1. Read this file first
2. Read `OPTIMIZATION_AUDIT_2026.md`
3. Check `.github/copilot-instructions.md` for avoided pitfalls
4. Profile before assuming what's slow
5. Document your findings (even failures are valuable)

---

**Last Updated:** February 2026
