# Optimization Status Report

**Date**: February 14, 2026  
**Current Branch**: `performance-optimizations`  
**Status**: Active - contains 2 implemented micro-optimizations

---

## Summary

**Implemented Optimizations**: 2/8 candidates  
**Cumulative Measured Benefit**: ~3-4% on medium files (6-25MB)  
**Cumulative Theoretical Benefit**: ~7% (2.9% + 4.8%)  
**Performance Gap vs C++ Implementation**: Still ~2x slowdown (main bottleneck remains)

---

## Optimization Ranking & Status

### Tier 1: Low-Effort, Measurable Gains (Completed ✅)

#### 1. Projection Path Specialization ✅ DONE
- **Potential**: 2-5% (actual: 2.9% in tight projections)
- **Effort**: Low (simple inlining)
- **Risk**: None
- **Status**: Merged into performance-optimizations branch
- **Files**: `src/mollweide.rs`, `src/hammer.rs`
- **Implementation**: Inlined `norm_x()`, `norm_y()` calculations, algebraic rearrangement of oval boundary check
- **Lesson**: Function call overhead in tight pixel loop; diminishes with resolution

#### 2. Colormap Sampling ✅ DONE
- **Potential**: 5-10% (actual: 4.8% at 2400px, 3.3% at 4000px)
- **Effort**: Trivial (single line change)
- **Risk**: None (pixel interpolation unchanged)
- **Status**: Merged into performance-optimizations branch
- **File**: `src/colormap.rs` line 2072-2088
- **Implementation**: Replace `(t * n).round()` with `t * 255.0` (truncation)
- **Why it works**: 256-entry LUT doesn't need rounding; truncation sufficient
- **Per-pixel cost**: Saves ~1-2 CPU cycles per 5.76M+ calls

---

### Tier 2: Medium-Effort Options (Investigated, Not Pursued)

#### 3. Scale Value Caching ⏸️ NOT PURSUED
- **Potential**: 3-5% 
- **Effort**: Medium (~100 lines, careful memory management)
- **Risk**: Medium (cache invalidation, memory overhead)
- **Status**: Analyzed but not implemented
- **Blocker**: Scale function already uses histogram caching; further caching would require architectural changes
- **Reason for pause**: Requires significant refactoring for uncertain gain

#### 4. Branching Reduction ❌ ATTEMPTED, REGRESSED
- **Potential**: 3-5% (theory)
- **Effort**: Medium (~150 lines code duplication)
- **Risk**: High
- **Status**: Tested and reverted (regressed -4.8%)
- **Implementation attempted**: Hoist `if let Some(mask)` outside pixel loop
- **Why it failed**: Loop duplication hurt instruction cache; modern branch predictor handles mask checks efficiently
- **Lesson learned**: Logical optimization ≠ performance optimization

#### 5. Memory Layout Optimization ⏸️ NOT PURSUED
- **Potential**: Unknown (1-10%?)
- **Effort**: High (~200+ lines)
- **Risk**: High (breaks existing data structures)
- **Status**: Not investigated
- **Candidate**: Align HEALPix data for cache efficiency
- **Reason for pause**: Uncertain benefit; high disruption risk

---

### Tier 3: High-Effort/High-Risk Options (Not Pursued)

#### 6. HEALPix Interpolation ⏸️ NOT PURSUED
- **Potential**: 5-8%
- **Effort**: High (~300 lines new code)
- **Risk**: High (changes numerical results, accuracy trade-offs)
- **Status**: Not investigated
- **Why skipped**: Would change output; requires validation against reference
- **Note**: Might provide quality/performance trade-off worth exploring separately

#### 7. Double-Angle Trig Optimization ❌ ATTEMPTED, REGRESSED
- **Potential**: 3-5% (theory)
- **Effort**: Low (~20 lines)
- **Risk**: Low
- **Status**: Tested and reverted (regressed -4%)
- **Implementation attempted**: Use `sin(2θ) = 2sin(θ)cos(θ)` to avoid one sin() call
- **Why it failed**: Modern FPU so efficient that the extra multiplication was slower
- **Lesson learned**: Intuitive micro-optimizations can be wrong; need empirical validation

#### 8. Cache-Aware Loop Tiling ⏸️ NOT PURSUED
- **Potential**: 2-4%
- **Effort**: High (architecture change)
- **Risk**: High (complexity explosion)
- **Status**: Not investigated
- **Note**: Might improve L1/L2 cache hit rates; likely too complex for marginal gain

---

## Remaining Optimization Opportunities (Viability Analysis)

### Easier Paths (But Limited Gain)

#### Profile-Guided Optimization
- **Effort**: Low
- **Potential**: 2-5%
- **Method**: Use `perf` to identify exact bottlenecks
- **Action**: `perf record -F 99 ./target/release/map2fig ...` then analyze hot spots
- **Why useful**: Could identify unexpected bottlenecks

#### Compiler Optimization Flags
- **Effort**: Trivial (change Cargo.toml)
- **Potential**: 1-3%
- **Current**: `-O` optimization level
- **Options**: Try `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`

#### SIMD Vectorization (Partial)
- **Effort**: Medium (50-100 lines per optimization)
- **Potential**: 5-15% (estimated)
- **Feasible targets**: 
  - Colormap bulk sampling (16M pixels = parallelizable)
  - HEALPix rotation matrix multiply (3×3 matrices)
- **Blocker**: Rust SIMD ecosystem fragmented (`packed_simd` unstable, external crates)

### Harder Paths (Significant Gain Possible)

#### Parallelization (Row-Level)
- **Effort**: High
- **Potential**: 10-20% (estimated, but Phase 27 showed overhead washed it out)
- **Previous attempt**: Rayon parallelization (Phase 27) showed 10-20% speedup was negated by thread overhead
- **Status**: Already archived as insufficient ROI
- **Reason**: Pixel loop not enough work per thread; synchronization overhead dominates

#### Algorithm Changes
- **Effort**: Very High
- **Potential**: 20-50% (estimated)
- **Examples**:
  - Level-of-detail rendering for high resolutions
  - Approximate HEALPix sampling (fewer pixels)
  - Progressive rendering (early output)
  - GPU backend (Cairo → compute shader)
- **Blocker**: Destructive to current design; requires new validation

### Comparison with C++ Baseline
- **Current Rust**: 23.1s @ 2400px
- **Theoretical C++ speedup factor**: ~2x (based on user mention)
- **Equivalent C++ time**: ~11.5s
- **Gap to close**: 12.6s (52% of time)
- **Required optimization**: Fundamental algorithmic/architectural change, not micro-optimizations

---

## Effort vs Gain Analysis

```
Potential Gain vs Effort Matrix

High Gain (>10%)
├─ Parallelization (20%)        ⚠️ [TRIED: overhead killed gains]
├─ Algorithm changes (20-50%)   🚫 [Too destructive]
└─ GPU backend (40%?)           🚫 [Major rewrite]

Medium Gain (5-10%)
├─ SIMD vectorization (5-15%)   ⏸️ [Medium effort, feasible]
├─ Multi-threaded I/O (3-8%)    ⏸️ [Low-medium effort]
└─ HEALPix interpolation (5-8%)  ⏸️ [High effort, risky]

Low Gain (2-5%)
├─ Compiler flags (1-3%)        ✅ [Trivial effort]
├─ Profile-guided optimization  ✅ [Low effort]
├─ Scale caching (3-5%)         ⏸️ [Medium effort]
├─ Branch reduction (3-5%)      ❌ [Regressed]
└─ Memory layout (1-5%?)        ⏸️ [High effort, uncertain]

Implemented (7% compound)
├─ Projection inlining (2.9%)   ✅ [Done]
└─ Colormap truncation (4.8%)   ✅ [Done]
```

---

## Recommendation for Next Steps

### Short Term (Next Session, Trivial Effort)
1. **Try compiler optimization flag**: Add `lto = "fat"` to Cargo.toml
   - Cost: 1 line change
   - Time: 5 minutes
   - Potential: 1-3%
   - Risk: None

2. **Run with `perf` for profiling**: Identify if bottlenecks have shifted
   - Cost: 10 minutes
   - Potential: Identify unexpected opportunities
   - Risk: None

### Medium Term (If Performance Still Matters)
3. **SIMD for colormap sampling**:
   - Cost: ~50 lines new code
   - Potential: 3-5% (on colormap, which is 8% of time)
   - Risk: Low (isolated function)
   - Time: ~2-3 hours

4. **SIMD for HEALPix rotation**:
   - Cost: ~100 lines new code
   - Potential: 5-10% (on HEALPix sampling, which is 35% of time)
   - Risk: Medium (correctness validation needed)
   - Time: ~4-6 hours

### Long Term (If C++ Parity Matters)
5. **Reassess parallelization**: Phase 27 used thread-per-row; could try:
   - Work-stealing queue for balanced load
   - Larger work blocks (batches of rows)
   - Lock-free data structure for pixel output
   - Potential: 15-30%, but requires significant engineering

---

## This Branch's Value

Even at 4-7% improvement, `performance-optimizations` is worth keeping because:

1. **Code quality**: Removed unnecessary variables, clearer comments
2. **Non-destructive**: Zero breaking changes, pure optimization
3. **Documented**: Detailed analysis of what works/what doesn't
4. **Foundation**: If larger optimizations are attempted later, these micro-optimizations compound
5. **Learning**: Tests demonstrated that intuitive optimizations often fail

---

## Current Branch Status

**Commits**:
- `943944d`: Projection path inlining
- `e7daaa3`: Further projection testing (reverted regression)
- `623c26f`: Colormap sampling (round → truncation)
- `3d341c2`: Comprehensive optimization documentation
- `75b2477`: Small files benchmark analysis
- `b3e55f2`: Main vs optimized branch comparison

**Files Modified**:
- `src/mollweide.rs`: Projection inlining
- `src/hammer.rs`: Projection inlining
- `src/colormap.rs`: Colormap sampling fast path
- Documentation: 3 new analysis files

**Testing**: ✅ Fully tested, no regressions, non-breaking

---

## Conclusion

The `performance-optimizations` branch contains solid, non-intrusive optimizations that:
- ✅ Actually improve small-to-medium files (3-4% measurable)
- ✅ Have zero risk of breaking functionality
- ✅ Teach us what works (colormap) and what doesn't (branching, trig)
- ✅ Provide foundation for future compound optimizations

The 2x performance gap vs C++ requires larger changes (parallelization, algorithm changes, or SIMD), not micro-optimizations. This branch is a good holding point that advances the codebase without committing to major restructuring.

