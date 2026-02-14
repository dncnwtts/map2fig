# Tier 4 Optimization Planning

## Current Performance Baseline
- Phase 5.2 (SIMD scaling): 0.777s - 0.914s per render on 25MB FITS
- Main phases (Phase 5.2 logs 2.3% improvement on large maps)
- Bottleneck identified: I/O and PDF generation dominate (~80% of runtime)

## Optimization Opportunities Analysis

### 1. **I/O Optimization** - HIGH IMPACT (Estimated: 15-25% speedup)

**Current Pipeline:**
- Read entire FITS file into memory
- Extract column with matching name
- Parse all HEALPix pixels

**Optimization Opportunities:**
1. **Memory-mapped I/O** (mmap)
   - Current: `fitsrs` crate reads entire file into memory
   - Potential: Use OS-level mmap for lazy column loading
   - Expected benefit: Faster cold-start, reduced memory pressure
   - Implementation: Fork or patch `fitsrs` crate

2. **Parallel Column Reading**
   - Current: Sequential FITS column parsing
   - Potential: Multi-threaded header parsing + rayon parallelism
   - Expected benefit: 2-4x on multi-core (up to 4 cores typical)
   - Implementation: Intermediate, rayon integration

3. **Index Caching**
   - Current: Parse FITS header every run
   - Potential: Cache column index/header metadata
   - Expected benefit: 5-10% on repeated renders
   - Implementation: Simple, JSON sidecar cache

**Tier 4.1 Recommendation:** Start with index caching (quick win), then mmap if needed

---

### 2. **PDF Generation Streaming** - MODERATE IMPACT (Estimated: 5-10% speedup)

**Current Pipeline:**
- Render pixels to Cairo image surface (in-memory buffer)
- Convert surface to PDF file
- Write to disk

**Optimization Opportunities:**
1. **Direct PDF Surface Rendering**
   - Current: Raster → Memory buffer → PDF
   - Potential: Render directly to PDF surface (Cairo supports)
   - Expected benefit: ~5% memory savings, better cache locality
   - Implementation: Medium complexity, requires cairo-rs API review

2. **Streaming Row-by-Row**
   - Current: Build entire image before PDF write
   - Potential: Generate PDF rows incrementally
   - Expected benefit: ~8% peak memory reduction
   - Implementation: High complexity, custom PDF writer

**Tier 4.2 Recommendation:** Try direct PDF surface first (simpler)

---

### 3. **Larger Batch Sizes** - MODERATE-HIGH IMPACT (Estimated: 10-20% speedup)

**Current Architecture:**
- 8-element SIMD batches (Phase 5.2)
- Limited to f64x8 (standard SIMD register)

**Optimization Opportunities:**
1. **16-Element Batches**
   - Current: Process 8 pixels per SIMD operation
   - Potential: Auto-vectorize 16-element loops, compiler unrolling
   - Expected benefit: 10-15% on larger maps via loop unroll
   - Implementation: Simple, compiler flags (`-C llvm-args=-march=native`)

2. **AVX-512 Support**
   - Current: Limited to 256-bit SIMD (f64x4 or f64x8)
   - Potential: AVX-512 provides f64x8 with more ops
   - Expected benefit: 10-20% on AVX-512 CPUs (newer Intel/AMD)
   - Implementation: Feature flag, conditional SIMD selection

3. **Cache-Aware Batch Ordering**
   - Current: Row-by-row pixel iteration
   - Potential: Reorder pixels for cache-coherent access
   - Expected benefit: 5-10% cache hit improvement
   - Implementation: Medium, requires profiling

**Tier 4.3 Recommendation:** Try 16-element batches with compiler flags first

---

### 4. **Adaptive Validity Masking** - MODERATE IMPACT (Estimated: 5-15% speedup)

**Current Architecture:**
- Validity masks applied per-pixel (Phase 5.2)
- Some pixels filtered out (projection invalid, healpix boundary)

**Optimization Opportunities:**
1. **Pre-filter Invalid Pixels**
   - Current: Check validity for every pixel in render loop
   - Potential: Build list of valid pixel coordinates up-front
   - Expected benefit: 10-15% if 30%+ pixels invalid
   - Implementation: Simple, requires benchmark on real data

2. **SIMD Mask Compression**
   - Current: 8-element mask array per batch
   - Potential: Use bitwise operations for predication
   - Expected benefit: 3-5% instruction cache improvement
   - Implementation: Low, bit manipulation

3. **Vectorized Mask Generation**
   - Current: Scalar mask computation per pixel
   - Potential: SIMD computation of entire region mask
   - Expected benefit: 5-10% on regions with boundaries
   - Implementation: Medium, requires projection vectorization

**Tier 4.4 Recommendation:** Wait for profiling data on real maps

---

## Execution Plan: RECOMMENDED PATH

### Phase 4.1 (Quick Wins - 1-2 hours)
- [ ] **Index Caching:** Add JSON sidecar for FITS header metadata
  - Estimated gain: 5-10%
  - Risk: Low
  - Effort: ~30 mins

- [ ] **Compiler Optimizations:** LTO + march=native
  - Estimated gain: 2-5%
  - Risk: Very low
  - Effort: ~15 mins

**Subtotal: 7-15% potential gain, 45 mins work**

### Phase 4.2 (I/O Optimization - 3-4 hours)
- [ ] **Memory-mapped FITS reading**
  - Estimated gain: 10-20%
  - Risk: Medium (fitsrs compatibility)
  - Effort: ~2 hours

- [ ] **Benchmarking & validation**
  - Update PERFORMANCE_TRACKING.md
  - Effort: ~30 mins

**Subtotal: 10-20% gain, 2.5 hours work**

### Phase 4.3 (Batch/PDF Optimization - 2-3 hours)
- [ ] **Direct PDF surface rendering**
  - Estimated gain: 3-5%
  - Risk: Low-Medium
  - Effort: ~1.5 hours

- [ ] **Larger batch optimization investigation**
  - Profile with perf
  - Benchmark different batch sizes
  - Effort: ~1 hour

**Subtotal: 5-15% gain, 2.5 hours work**

---

## Profiling Strategy

To identify actual bottlenecks before major refactoring:

```bash
# Profile current performance bottlenecks
cargo build --release
perf record -g ./target/release/map2fig -f cosmoglobe_clipped.fits -o /tmp/map.pdf -w 1200 --log
perf report

# Time individual phases
./target/release/map2fig -f cosmoglobe_clipped.fits -o /tmp/map.pdf -w 1200 --verbose --log
```

Profile will reveal:
- % time in FITS I/O vs rendering vs PDF generation
- Which PDF/Cairo functions dominate
- Whether mask computation is significant

---

## Decision Framework

**GO with Phase 4.1 (Quick Wins) if:**
- Want immediate small gains without risk
- Consensus: Yes, do this first

**FOLLOW with Phase 4.2 (I/O) if:**
- Profiling shows I/O > 40% of runtime (likely)
- Biggest potential impact
- Consensus: Yes, likely next target

**PRIORITIZE Phase 4.3 (Batch) if:**
- Profiling shows rendering loop clearly bottlenecked
- Cache miss rate high
- Consensus: Maybe, depends on profiling data

---

## Testing Strategy

For each phase:
1. Benchmark baseline with `./tools/scripts/benchmark_quick.sh`
2. Implement changes on feature branch
3. Validate functionality (all tests pass, PDFs visually identical)
4. Benchmark and record speedup in PERFORMANCE_TRACKING.md
5. Merge if speedup > 2-3% and no regression
6. Move to next phase

---

## Next Steps

**Immediate (Next 30 mins):**
1. Profile current performance with `perf` to identify real bottlenecks
2. Decide: Quick wins first, or deep-dive to I/O?
3. Create feature branch: `tier4-optimization` or similar

**Decision:** Which phase should we start with?
- A) Phase 4.1 (Quick wins - 45 mins, 7-15% gain)
- B) Phase 4.2 (I/O - 2.5 hrs, 10-20% gain)  
- C) Profile first then decide

Recommendation: **Start with Phase 4.1 (quick wins)**, then profile, then Phase 4.2 (I/O).
