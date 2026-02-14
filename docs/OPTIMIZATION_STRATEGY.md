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
- [ ] **NEXT**: Run detailed profiling with symbols, extract function names

### Phase 2: Tier 1 Optimization (Next Sprint)
Target: 5-15% speedup

#### Option A: Rasterization Redesign (High Impact, High Effort)
If flamegraph shows Cairo overhead is high:
```
Current:  Each pixel → Cairo rectangle call → PDF
Proposal: Pixels → local buffer → rasterize → Cairo path once
Expected: 20-30% speedup on render pass
```

Affected files:
- `src/render/cairo.rs` - Redesign pixel accumulation
- `src/render/mod.rs` - New pixel buffering strategy

#### Option B: SIMD Vectorization (Medium Impact, Medium Effort)
If flamegraph shows projection math is hot:
```
Current:  Sequential coordinate transforms
Proposal: Batch 4-8 pixels, vectorize with SIMD
Expected: 15-20% speedup on projection
```

Affected files:
- `src/projection.rs` - Add SIMD coordinate math
- `src/extensions/` - New SIMD utility functions

#### Option C: Scaling Optimization (Low-Medium Impact, Low Effort)
If flamegraph shows `scale_value()` is called frequently:
```
Current:  Computation per pixel
Proposal: Pre-compute lookup tables for common ranges
Expected: 5-10% speedup on scaling-heavy operations
```

Affected files:
- `src/scale.rs` - Add LUT caching

### Phase 3: Verification
- Re-run `./tools/scripts/profile.sh` after each optimization
- Track improvements in `docs/PERFORMANCE_TRACKING.md`
- Update flamegraph for before/after comparison

## Decision Point: Which Optimization to Pursue?

The flamegraph with debug symbols will answer:
1. **What consumes the 2.90% in `[map2fig]`?**
   - Function distribution tells us the priority
   - Top 3 functions = focus area

2. **How much of the time is Cairo?** (Currently measured at 2.10% visibly)
   - Single largest category → invest here
   - Small fraction → optimize Rust first

3. **Are there any surprise hotspots?**
   - Unexpected allocations
   - Surprising function call patterns
   - Data dependency inefficiencies

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
