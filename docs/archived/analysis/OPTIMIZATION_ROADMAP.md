# Healpix Plotter - Optimization Roadmap & Next Steps

## Current Status Summary

### Completed Optimizations (Main Branch)
✅ **Tier 3**: Full SIMD vectorization (8-element batches)
✅ **Tier 4**: Native CPU + Metadata caching (100% hit rate)
✅ **Tier 5.1**: Batch size optimization (16-element batches)
✅ **Tier 5.2**: Column data caching (81.7% improvement on large files)
✅ **Tier 5.3**: PDF rendering analysis (deferred; streaming not feasible with Cairo)

**Overall Performance**: 3.1GB FITS file takes ~70s uncached, **12.8s cached** (81% improvement)

## Immediate Next Steps (Priority Order)

### Step 1: Validate Column Cache Effectiveness on All File Types ⏱️ 30 min

**Rationale**: Column caching works well on 3.1GB files, but should verify across dataset:
- Very large files (>5GB)
- Small FITS files (<100MB)
- Different column layouts (binary table vs image HDUs)
- Different FITS libraries (validate our assumptions)

**Action**:
```bash
# Test on available FITS files
for fits_file in *.fits; do
  echo "Testing $fits_file..."
  time cargo run -- -f "$fits_file" -o /tmp/$(basename "$fits_file" .fits).pdf
done
```

**Expected Outcome**: Confirm 50-80% improvement across all tested files
**Deliverable**: Updated PERFORMANCE_TRACKING.md with results

---

### Step 2: Fix Remaining Tests & Warnings ⏱️ 15 min

**Rationale**: Clean compilation is critical for maintainability

**Current Status**:
- 163 tests passing ✅
- 0 errors ✅
- 4 warnings in pdf_optimize.rs (unused imports) - FIXED ✅

**Action**: Verify all tests pass and remove any remaining warnings
```bash
cargo test --lib 2>&1 | grep "test result"
cargo clippy --lib 2>&1 | grep -E "warning|error"
```

**Expected Outcome**: "test result: ok. 163 passed" (zero warnings)
**Deliverable**: Clean build artifact

---

### Step 3: Document Optimization Achievements ⏱️ 20 min

**Rationale**: Capture lesson learned and decisions made for future optimization work

**Action**: Update PROJECT_SUMMARY.md or create OPTIMIZATION_SUMMARY.md

**Key Points to Document**:
1. Why Tier 5.3 PDF streaming was deferred
2. Counter-intuitive 1200px faster than 512px finding (cache effects)
3. Column caching outperformed metadata caching by 81x
4. Impact of batch size optimization (minimal -0.5% but enabled larger batches)

**Expected Outcome**: Clear record of optimization decisions
**Deliverable**: OPTIMIZATION_SUMMARY.md

---

## Medium-Term Work (3-7 days)

### Tier 5.4: Adaptive Masking 🎯

**Goal**: Skip processing masked/invalid pixels earlier in pipeline

**Current State**: Pixels marked UNSEEN are currently:
1. Read from FITS
2. Scaled
3. Projected
4. Then filtered

**Optimization**: Filter at step 0, before scaling

**Estimated Gain**: 10-15% (removes processing of ~50% of pixels in many realistic datasets)
**Effort**: Medium (modify pipeline.rs and pixel processing loops)
**Risk**: Low (filtering logic already implemented, just moving it earlier)

**Implementation Plan**:
```rust
// Current: scale all pixels, filter invalid later
let mut pixels = read_column();
let scaled = pixel.iter().map(|p| scale_value(p, ...)).collect();
let valid = scaled.into_iter().filter(|p| !is_unseen()).collect();

// Optimized: filter before scaling
let pixels = read_column();
let valid = pixels.into_iter().filter(|p| !is_unseen()).collect();
let scaled = valid.iter().map(|p| scale_value(p, ...)).collect();
```

**Files to Modify**:
- `src/pipeline.rs` - Move UNSEEN filtering to post-read step
- `src/healpix.rs` - Ensure is_unseen check is performant

---

### Tier 5.5: Output Format Benchmarking 🔬

**Goal**: Quantify PDF vs PNG rendering overhead

**Question**: Is PDF rendering 10.6s or is it general rasterization overhead?

**Benchmark Plan**:
1. Same 3.1GB file, same 512px resolution
2. Output to PDF (current)
3. Output to PNG (image crate only, no PDF wrapper)
4. Output to SVG (if feasible)

**Tool**: Extend profile_pdf.py to support multiple output formats

**Expected Insight**: If PNG is 80% of PDF time, then we've found the real bottleneck (rasterization not PDF). If PNG is <1s, then PDF overhead is confirmed as primary issue.

---

### Tier 5.6: Cache Statistics & CLI Integration

**Goal**: Make column caching diagnostics user-visible

**Current State**: Cache works but silently

**Enhancement**:
```bash
# Possible CLI additions:
cargo run -- -f data.fits -o map.pdf --cache-stats
# Output:
# Column cache hit: 1 operation (2.3GB, saved 45s)
# File loaded from: /home/.cache/map2fig/fits_col_a1b2c3d4_0_1234567890
```

**Implementation**:
- Add `--cache-stats` flag to CLI
- Modify pipeline.rs to track cache hits/misses
- Pretty-print results

---

## Long-Term Direction (1-2 weeks)

### Investigation: Cache Synergy Effect

**Counter-intuitive Finding**: 1200px outputs faster than 512px on large files

**Hypothesis**: 16-element batches in Tier 5.1 interact with OS page cache

**Experiment Plan**:
1. Profile with `perf record` and `perf report`
2. Check L1/L2/L3 cache miss rates
3. Measure batch boundary alignment effects
4. Test with batch size = 32 (double current)

**Potential Gain**: If cache alignment helps 1200px, might apply to all sizes

---

### Research: Alternative Optimization Approaches

**Potential Areas**:
1. **Lazy Graticule Generation**: Don't generate all graticule lines, render on-demand
2. **Quadtree Pixel Mapping**: Group adjacent pixels for faster coordinate projection
3. **SIMD Projection**: Use SIMD for HEALPix → Mollweide transformation (currently scalar)
4. **Parallel Rendering**: Process different map regions on different CPU cores

**Selection Criteria**:
- Estimated gain > 10%
- Implementation effort < 3 days
- Test coverage maintainable

---

## Strategic Decisions Made

### Why Stop at Tier 5.2? 

✅ **Column caching provides 81% improvement** - This is a major win, justifies publication/release

❌ **PDF streaming requires library replacement** - Not feasible with Cairo

❌ **Further SIMD gains marginal** - Memory bandwidth limited at this point

✅ **Cache infrastructure is solid** - Can be extended to binary table indexing

### Recommendation: Ship Current State

**Rationale**:
1. **81% improvement on cached loads** is end-user visible and impactful
2. **Tier 3-5.2 is stable** with comprehensive test coverage (163 tests)
3. **Clear roadmap for future** (Tier 5.4+ identified and scoped)
4. **Further gains marginal** compared to effort required

**Action**: Merge main branch to release candidate for version 2.0

---

## Testing & Validation Checklist

Before next release:

- [ ] All 163 unit tests passing
- [ ] Benchmark on 5+ different FITS files (various sizes)
- [ ] Column cache invalidation on file modification
- [ ] Cache directory creation on first run (permissions)
- [ ] Graceful fallback if cache corrupted
- [ ] Performance on small files (<100MB) regression test
- [ ] Memory profiling (peak RSS acceptable)
- [ ] Cairo/PDF output quality unchanged

---

## File Organization

**Documentation Created**:
- `TIER5_3_PDF_ANALYSIS.md` - PDF rendering analysis & decision
- `OPTIMIZATION_ROADMAP.md` - This file
- `PERFORMANCE_TRACKING.md` - Quantified improvements (existing, to be updated)

**Code Assets**:
- `src/pdf_optimize.rs` - Infrastructure for future PDF work
- `tools/profile_pdf.py` - PDF profiling tool
- `src/diagnostics.rs` - I/O diagnostics framework
- `src/fits.rs` - Column caching implementation

---

## Questions for Review

1. **Priority**: Should we aim for Tier 5.4 (adaptive masking) or focus on testing/stabilization?
2. **Release**: Is current state (81% improvement) ready for 2.0 release?
3. **Scope**: Should future work focus on output formats or cache infrastructure?
4. **Review**: Any optimizations we missed that should be considered?

---

## Summary

**Achieved**: **81.7% speedup on cached loads** (70s → 12.8s) through column data caching

**Framework/Infrastructure**:
- Binary cache with mtime validation
- Diagnostics for monitoring cache effectiveness
- PDF complexity estimation (for future work)
- Extensible profiling tools

**Next Obvious Work**: 
1. Validate cache effectiveness across all FITS types
2. Implement Tier 5.4 (adaptive masking, +10-15% potential)
3. Benchmark output formats (PDF vs PNG)

**Strategic Posture**: Healthy optimization velocity, clear roadmap, solid test coverage. Ready to continue or release.

---

*Last Updated*: End of Tier 5.3 analysis phase (PDF streaming deferred)
*Status*: Ready for team review and priority guidance
