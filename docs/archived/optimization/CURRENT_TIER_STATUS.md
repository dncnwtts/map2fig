# Current Tier Status - Session 3

**Date Started:** February 15, 2026  
**Objective:** Tier 3b - Cache-Aware Loop Reordering

---

## Bottleneck Target

**Mollweide Algorithm Inefficiency**
- CPU time: 77.5% of total execution
- Current cache miss rate: 31.85% (too high)
- Issue: Loop structure causes poor memory locality
- Expected gain: 5-8% wall-clock improvement

---

## Investigation

### Step 1: Identify Hot Loops
Running `perf c2c` to find cache contention...

### Step 2: Analyze Memory Access Pattern
Looking for:
- Sequential vs. random access
- Cache line conflicts (same address, different cores)
- False sharing (same line, different variables)
- Poor loop nesting order

### Step 3: Reorder Loops
Candidate changes:
- Swap inner/outer loop order to improve stride
- Group related memory accesses
- Improve cache line utilization

### Step 4: Benchmark & Validate
- Measure execution time (target: 10.14s → ~9.7-9.9s)
- Verify cache misses drop (target: 31.85% → <25%)
- Ensure results correctness (compare pixel output)

---

## Implementation Notes

**Files to Examine:**
- `src/plot/mollweide.rs` - Main Mollweide rendering
- `src/projection.rs` - Projection math loops
- `src/render/target.rs` and `src/render/png.rs` - Pixel writing

**Key Functions:**
- `plot_mollweide_pdf()` / `plot_mollweide_png()` - Entry points
- `render_projection_to_grid()` - Main pixel generation loop
- `mollweide_inverse()` - Coordinate transformation

**Loop Targets:**
1. Pixel iteration loops (map height × map width = ~1M pixels)
2. Coordinate transformation per pixel
3. Color lookup per pixel

---

## Completed Work

None yet - just starting this tier.

---

## Blocked/Failed

- Tier 3 (SIMD): Blocked by F32 analysis showing math not bottleneck
- Tier 4: Too risky, defer until later

---

## Next Steps

1. Profile with `perf c2c` to identify exact cache issues
2. Examine mollweide loop structure
3. Propose reordering strategy
4. Implement and test
5. Document results in TIER3B_RESULTS.md

