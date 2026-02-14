# Performance Optimization Strategy - v0.3 Planning

## Current Status (v0.2.0)

✅ **Baseline Established**:
- Flamegraph profiling in place
- Debug symbols enabled for function-level analysis
- Hot spots identified (Cairo 2.10%, Rust code 2.90%)
- Automated profiling script ready

## Optimization Roadmap

### Phase 1: Profile Analysis (This Session)
- [x] Baseline flamegraph with system view
- [x] Enable debug symbols for Rust function profiling
- [x] Identify major cost centers (Cairo, projection, scaling)
- [x] **CRITICAL FINDING**: Compare PNG vs PDF output speeds
  - **Result**: PDF is 3.57× slower than PNG (617 ms vs 173 ms)
  - **Conclusion**: Cairo rasterization is PRIMARY bottleneck
  - **Gap**: 427 ms = Cairo overhead, NOT projection/scaling

### Phase 2: Tier 1 Optimization - CAIRO RASTERIZATION (Revised)
Target: 25-50% speedup (Cairo optimization alone)

**Why Cairo is the bottleneck:**
- PNG uses simple image raster: 173 ms (projection + scaling + color + PNG write)
- PDF uses Cairo: 617 ms total
- Difference: **427 ms dedicated to Cairo rendering** (3.57× multiplier!)
- Per-pixel cost: 8.4 µs/pixel vs 2-3 µs for projection

#### Strategy: Batch Cairo Operations

Current approach:
```rust
// Draw pixel-by-pixel
for (x, y, color) in pixels {
    cairo_rectangle(x, y, 1, 1);
    cairo_set_source_rgb(color);
    cairo_fill();  // ← 51,000 individual calls!
}
```

Optimized approach:
```rust
// Batch similar colors, reduce fill calls
let grouped = group_pixels_by_color(pixels);
for (color, pixel_group) in grouped {
    cairo_set_source_rgb(color);
    for (x, y) in pixel_group {
        cairo_rectangle(x, y, 1, 1);
    }
    cairo_fill();  // ← ~256 calls instead of 51,000!
}
```

**Expected improvement**: 50% reduction in Cairo overhead
- From: 617 ms → **~504 ms** (18% overall)
- Exceeds v0.3 target of 10-15%

### Phase 2b: Parallel Option - SIMD Vectorization (Secondary)
Target: 8-12% additional speedup if Cairo already optimized

Only pursue AFTER Cairo optimization, not before.

### Original Phase 2 Options (Archived)

~~Option A: Rasterization Redesign~~ - Replace with Cairo batching
~~Option B: SIMD Vectorization~~ - Tier 2 after Cairo
~~Option C: Lookup table caching~~ - Tier 3

### Phase 3: Verification
- Re-run `./tools/scripts/profile.sh` after each optimization
- Track improvements in `docs/PERFORMANCE_TRACKING.md`
- Update flamegraph for before/after comparison

## Decision Point: Which Optimization to Pursue?

✅ **DECISION MADE BY EMPIRICAL DATA** (not theory)

```
PDF time: 617 ms
PNG time: 173 ms
Difference: 427 ms = Cairo overhead

Conclusion: Cairo rasterization MUST be optimized first.
```

This eliminates the need to guess - we can see exactly where the time is spent:
- **201 ms** (44%) of PNG time is projection + scaling + colormapping + PNG write
- **427 ms** (69%) is Cairo rendering the same data differently
- **11 ms** (2%) PDF file write overhead

**Action**: Focus Phase 2 on batching Cairo calls to reduce per-pixel overhead.

## Tools & Workflow

### Next Steps (Immediate):
```bash
# 1. With debug symbols enabled, re-profile for function names
./tools/scripts/profile.sh

# 2. Analyze flamegraph_v0.2.0.svg visually:
#    - Expand largest blocks
#    - Look for:
#      * repeat function calls (optimization opportunity)
#      * wide cairo blocks (consider buffering)
#      * math libraries in hot path (SIMD candidate)
firefox flamegraph_v0.2.0.svg

# 3. Document findings in PERFORMANCE_ANALYSIS_v0.2.0.md
#    - Which specific functions ≥ 1% of CPU?
#    - What's the call depth?
#    - Are there redundant operations?
```

### Measuring Success:
```bash
# Before optimization:
./tools/scripts/profile.sh
# → perf_report_v0.2.0.txt (already have this)

# Make optimization:
# → modify src/*.rs according to strategy

# After optimization:
cargo build --release
./tools/scripts/profile.sh
# → perf_report_v0.3.0-rc1.txt
# Compare times with baseline
```

## Risk & Feasibility

| Optimization | Risk | Feasibility | ROI | Priority |
|---|---|---|---|---|
| Cairo rasterization | Medium | Medium | Very High | 1 |
| SIMD vectorization | Low | Medium | High | 2 |
| LUT caching | Low | High | Medium | 3 |
| Rayon tuning | Low | High | Low | 4 |

## Timeline Estimate

- **Phase 1 (profiling)**: Done ✅
- **Phase 2a (rasterization)**: 2-4 hours coding + testing
- **Phase 2b (SIMD)**: 3-6 hours coding + testing
- **Phase 3 (verification)**: Continuous

## Success Criteria for v0.3

- [ ] Baseline times documented in PERFORMANCE_TRACKING.md
- [ ] At least one Tier 1 optimization implemented
- [ ] 10-15% speedup on average case
- [ ] No regression on edge cases
- [ ] Flamegraph shows reduced `[map2fig]` percentage
- [ ] Release notes include performance improvements

## Long-term (v0.4+)

- GPU acceleration exploration (if CPU gains plateau)
- Parallel FITS reading
- Advanced caching strategies
- Benchmark suite CI integration

---

## Notes for Next Session

If resuming optimization work:
1. Check `flamegraph_with_symbols_v0.2.0.svg` for function breakdown
2. Read PERFORMANCE_ANALYSIS_v0.2.0.md for context
3. Reference this document for optimization strategy
4. Update PERFORMANCE_TRACKING.md with new baseline timings
5. Implement highest-ROI option from Phase 2 options
