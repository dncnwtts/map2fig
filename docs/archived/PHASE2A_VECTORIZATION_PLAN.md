# Phase 2A: SIMD Vectorization (Next Steps)

## Where We Are: Phase 2B Success

**Current performance (v0.4.0 after Phase 2B)**:
- PDF: 300ms (down from 617ms in v0.2.0)
- PNG: 160ms (stable)
- **Total improvement: 51.4% from baseline**
- Estimated efficiency: ~13.3 GFLOPs / 3.0 GHz = **~0.45× baseline scalar** (still very room for improvement!)

**What Phase 2B solved**:
- ✅ Eliminated per-pixel Cairo operations
- ✅ Eliminated path building overhead
- ✅ Used fast in-memory image buffer instead

**What remains**:
- HEALPix sampling math: 8.64% of runtime (sin/cos/atan2)
- Projection operations: 4.21% of runtime
- Scaling operations: implied in remaining time
- File I/O and initialization: ~20ms

---

## Phase 2A: SIMD Vectorization Strategy

### Remaining Optimization Potential

From profiling (perf record on v0.3.0, still applies):
```
8.64% ————> HEALPix sampling (sin/cos math)
 ├─4.43% __sincos_fma (trigonometric functions)
 └─2.88% __atan2 (inverse tangent)

4.21% ————> Projection rendering
```

**Realistic scope**: Vectorize trigonometric operations in:
1. HEALPix sampling (sincos, atan2)
2. Projection math if applicable

**Expected results**:
- Current scalar math cost: ~25-30ms (implied from profiling)
- With 4-8× SIMD parallelism: 6-10ms
- **Expected improvement: 15-20ms saved** → **280-285ms PDF time**
- This reaches **~54% total improvement** from v0.2.0

---

## Implementation Plan: Phase 2A

### Step 1: Identify Vectorization Targets (1-2 hours)

**Modules to analyze**:
- `src/healpix.rs`: `sample_healpix_batch_simd()` - uses sin/cos/atan2
- `src/mollweide.rs`: Mollweide inverse projection - uses sin/asin/cos
- `src/simd.rs`: Current SIMD wrapper functions (all scalar fallbacks)

**Current scalar bottleneck**:
```rust
// From src/healpix.rs: sample_healpix_batch_simd
let (sin_vals, cos_vals) = crate::simd::simd_sin_cos_8(angles);  // 8 calls, scalar
```

This is called for 8 HEALPix samples at a time. Each call does:
```rust
pub fn simd_sin_cos_8(angles: [f64; 8]) -> ([f64; 8], [f64; 8]) {
    let mut sines = [0.0; 8];
    let mut cosines = [0.0; 8];
    for i in 0..8 {
        (sines[i], cosines[i]) = angles[i].sin_cos();  // ← 8 scalar sin_cos calls
    }
    (sines, cosines)
}
```

With SIMD (SSE2 or AVX2), we'd replace this with a single vectorized operation.

### Step 2: Choose SIMD Implementation Strategy (30 minutes)

**Option A: `portable_simd` (Nightly Rust)**
- ✅ Latest, most portable
- ⚠️ Requires nightly compiler
- Availability: RFC accepted, good long-term solution

**Option B: `packed_simd` (Stable Rust)**
- ✅ Well-established, stable
- ✅ Should work on x86_64
- ⚠️ Less active development

**Option C: Inline x86_64 intrinsics**
- ✅ Direct control, no dependency
- ⚠️ Platform-specific, needs fallback
- ⚠️ More complex code

**Recommendation**: **Option A (`portable_simd`)** with fallback to scalar
- Modern, future-proof
- Can feature-gate nightly usage
- Fallback ensures stability

### Step 3: Vectorize sin_cos (2-3 hours)

**Current scalar** (`src/simd.rs`):
```rust
pub fn simd_sin_cos_8(angles: [f64; 8]) -> ([f64; 8], [f64; 8]) {
    let mut sines = [0.0; 8];
    let mut cosines = [0.0; 8];
    for i in 0..8 {
        (sines[i], cosines[i]) = angles[i].sin_cos();
    }
    (sines, cosines)
}
```

**With portable_simd** (simulated - exact syntax TBD):
```rust
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
pub fn simd_sin_cos_8(angles: [f64; 8]) -> ([f64; 8], [f64; 8]) {
    use std::simd::{f64x4, SimdFloat};
    
    // Process in two f64x4 chunks (SSE2)
    let chunk1 = f64x4::from_slice(&angles[0..4]);
    let chunk2 = f64x4::from_slice(&angles[4..8]);
    
    let (sin1, cos1) = chunk1.sin_cos();  // Vectorized!
    let (sin2, cos2) = chunk2.sin_cos();
    
    ([
        sin1[0], sin1[1], sin1[2], sin1[3],
        sin2[0], sin2[1], sin2[2], sin2[3],
    ], [
        cos1[0], cos1[1], cos1[2], cos1[3],
        cos2[0], cos2[1], cos2[2], cos2[3],
    ])
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
pub fn simd_sin_cos_8(angles: [f64; 8]) -> ([f64; 8], [f64; 8]) {
    // Fallback to current scalar implementation
    /* existing code */
}
```

**Testing**: Exact bit-identical output verification
```rust
#[test]
fn test_simd_sin_cos_vs_scalar() {
    let angles = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
    let (sins, coss) = simd_sin_cos_8(angles);
    let (sins_scalar, coss_scalar) = simd_sin_cos_8_scalar(angles);
    
    for i in 0..8 {
        assert!(
            (sins[i] - sins_scalar[i]).abs() < 1e-15,
            "sin[{}] mismatch: SIMD={}, scalar={}",
            i, sins[i], sins_scalar[i]
        );
    }
}
```

### Step 4: Vectorize atan2 (1-2 hours)

Similar approach for `simd_atan2_8()`:
```rust
pub fn simd_atan2_8(y: [f64; 8], x: [f64; 8]) -> [f64; 8] {
    [
        y[0].atan2(x[0]),
        y[1].atan2(x[1]),
        // ... 8 scalar atan2 calls
    ]
}
```

Could replace with vectorized atan2 if available in portable_simd, or use approximation.

### Step 5: Profile and Benchmark (1-2 hours)

**Measure improvement**:
```bash
cargo build --release
./tools/scripts/profile.sh
# Expected: 300ms → 280-285ms for PDF
```

**Verify correctness**:
```bash
# Generate output before/after
./target/release/map2fig -f test.fits -o before.pdf
./target/release/map2fig -f test.fits -o after.pdf
# Visually inspect (should be pixel-identical)
```

---

## Stretch Goal: v0.5.0 Targets

| Optimization | Time | vs v0.2.0 | Notes |
|---|---|---|---|
| v0.2.0 Baseline | 617ms | - | Cairo per-pixel |
| v0.3.0 (batching) | 470ms | -23.8% | Reduced fill() calls |
| v0.4.0 (image pre-rendering) | 300ms | -51.4% | Eliminated Cairo overhead |
| v0.5.0 (SIMD) | ~285ms | -54% | Vectorized math |
| **Theoretical min** | ~130ms | -79% | I/O bound limit |

**Remaining gaps** (if Phase 2A doesn't hit 285ms target):
- File I/O: ~20ms (hard to optimize)
- Initialization/setup: ~10ms (minor)
- Remaining unoptimized math: ~25ms (Phase 2A target)

**If Phase 2A doesn't deliver expected gains**:
- Alternative: Multi-threading with Rayon (8-core parallelism)
- Caveat: Cairo is single-threaded, so limited benefit for PDF rendering
- Would help PNG rendering more: 160ms → 40-50ms potential

---

## Risk & Contingency

### Risk: Vectorization Doesn't Deliver Expected Gain
**Mitigation**:
- Keep scalar fallback always available
- Integration testing catches regressions
- Can disable SIMD with feature flag

### Risk: Exact FP Precision Issues
**Mitigation**:
- Test against scalar with tolerance (< 1e-15)
- Mollweide projection tests already exist, can validate

### Risk: Platform Incompatibility
**Mitigation**:
- Feature-gate SIMD to x86_64 SSE2+ only
- Fallback to scalar for other platforms
- Use portable_simd for future compatibility

---

## Next Phase: Integration with Phase 2B

Since Phase 2B is complete:
1. **Merge Phase 2B** to main branch (done ✓)
2. **Start Phase 2A** in next session
3. **Measure cumulative effect**: Phase 2A on top of 2B

---

## Success Criteria for Phase 2A

- ✅ Vectorize simd_sin_cos_8 with SIMD
- ✅ Achieve **285ms PDF rendering** (target)
- ✅ Maintain **bit-identical output** vs scalar
- ✅ Work on Linux/macOS x86_64
- ✅ Graceful fallback for other platforms
- ✅ Update PERFORMANCE_TRACKING.md with v0.5.0 results

---

## Files to Modify

1. `Cargo.toml`
   - Add `portable_simd` dependency (feature-gated if needed)

2. `src/simd.rs`
   - Add `#[cfg(...)]` feature gates
   - Implement SIMD versions of sin_cos, atan2, etc.
   - Keep scalar fallbacks

3. `src/healpix.rs`
   - Possibly adjust how simd::simd_sin_cos_8 is called (no changes expected)

4. `docs/PHASE2A_IMPLEMENTATION.md`
   - Document the vectorization approach and results

---

## Estimated Timeline

- **Step 1**: 1-2 hours (analysis)
- **Step 2**: 30 minutes (decide strategy)
- **Step 3**: 2-3 hours (implement sin_cos)
- **Step 4**: 1-2 hours (implement atan2)
- **Step 5**: 1-2 hours (benchmark)
- **Buffer**: 2-3 hours for debugging/testing
- **Total**: ~8-12 hours (1 work day)

---

## References

- Rust portable_simd: https://github.com/rust-lang/portable-simd
- packed_simd_2: https://docs.rs/packed_simd_2/
- x86_64 intrinsics: https://doc.rust-lang.org/stable/core/arch/x86_64/

