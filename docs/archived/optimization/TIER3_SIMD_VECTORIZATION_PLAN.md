# Tier 3: SIMD Vectorization - Implementation Plan

## Overview
Build on Tier 2's batch architecture to implement actual vectorized operations using platform-specific SIMD intrinsics. Goal: +10-15% performance improvement through CPU vector parallelism.

## Current State (End of Tier 2)
- ✅ Batch projection functions (`pixel_to_ang_batch`) processing 8 pixels
- ✅ Batch HEALPix sampling (`sample_healpix_batch`) for 8 coordinates
- ✅ Main rendering loop integrated with 8-pixel batches
- ✅ Scalar fallback for boundary pixels
- ⏳ **Gap**: Loop unrolling without actual SIMD instructions

## Performance Bottlenecks (Profiling Results Needed)

### Expected Hotspots (by percentage of total time)
1. **Projection Transform** (~35-40% of loop time)
   - `pixel_to_ang()` repeated 8 times per batch
   - Trigonometry: `atan2`, `asin`, `sin`, `cos`
   - Memory operations: RasterGrid access

2. **HEALPix Sampling** (~25-30%)
   - Spherical → Cartesian conversion (trig)
   - View transformation (matrix multiply)
   - Cartesian → Spherical conversion
   - Array indexing

3. **Colormap Lookup** (~15-20%)
   - 256-entry LUT access (8x per batch)
   - RGB interpolation
   - Gamma correction (power function)

4. **Scaling & Normalization** (~10-15%)
   - Value scaling (min/max normalization)
   - Scale type operations (log, asinh, etc.)

## Tier 3 Strategy

### Phase 1: Vectorized Math Primitives (Week 1)
**Goal**: Build reusable SIMD math kernels

#### Task 1.1: Trigonometric SIMD Functions
```rust
// Vectorized trig for 8 f64 values at once
pub fn simd_sin_8(angles: [f64; 8]) -> [f64; 8]
pub fn simd_cos_8(angles: [f64; 8]) -> [f64; 8]
pub fn simd_atan2_8(y: [f64; 8], x: [f64; 8]) -> [f64; 8]
pub fn simd_asin_8(x: [f64; 8]) -> [f64; 8]
```

**Implementation Options**:
1. **Portable SIMD** (`core::simd`) - Stable Rust, good cross-platform
   - Widest compatibility (works on stable)
   - 8×f64 = 512-bit register (AVX-512 / 2×AVX on AVX2)
   - Some ops not vectorized (trig functions)

2. **x86-64 Intrinsics** (`core::arch::x86_64`)
   - `_mm256_*` for AVX2 (4×f64)
   - `_mm512_*` for AVX-512 (8×f64)
   - Manual implementation, best performance
   - Limited to x86-64

3. **Taylor Series** - Custom vectorizable implementations
   - sin/cos via polynomial approximation
   - Vectorizable across SIMD lanes
   - Controllable precision

**Decision**: Start with **Portable SIMD** + fallback to **Taylor Series** for trig

#### Task 1.2: SIMD Rotation Matrix Application
```rust
pub fn simd_rotate_8(
    vectors: [[f64; 3]; 8],
    rot_matrix: &[[f64; 3]; 3]
) -> [[f64; 3]; 8]
```
- Apply 3×3 matrix to 8 vectors in parallel
- 9 scalar products per vector → vectorize with SIMD

#### Task 1.3: SIMD Colormap Lookup
```rust
pub fn simd_colormap_sample_8(
    cmap: &Colormap,
    t_values: [f64; 8]  // 8 normalized values in [0, 1]
) -> [[u8; 3]; 8]      // 8 RGB colors
```
- Pack 8 LUT indices into vectors
- Vectorized gather operations if available
- Fallback: scalar 8x with cache locality

### Phase 2: Batch Projection Vectorization (Week 2)
**Goal**: Vectorize `pixel_to_ang_batch` with SIMD

#### Task 2.1: Vectorize Mollweide Projection
Replace current loop-unrolled version with SIMD:
```rust
impl Projection for MollweideProjection {
    fn pixel_to_ang_batch_simd(
        &self,
        px: &[u32; 8],
        py: &[u32; 8],
        grid: &RasterGrid,
    ) -> ([f64; 8], [f64; 8], [bool; 8]) {
        // Use SIMD primitives from Phase 1
        // Vectorized normalization
        // Vectorized trigonometry
        // Vectorized domain checking
    }
}
```

**Sub-tasks**:
- Vectorize u,v normalization (8 pixels at once)
- Vectorize inverse projection math
- Vectorize validity checking (8-wide comparison)

#### Task 2.2: Vectorize Hammer Projection
- Similar approach as Mollweide
- More complex math = larger speedup potential

### Phase 3: Batch HEALPix Vectorization (Week 2)
**Goal**: Vectorize `sample_healpix_batch` with SIMD

#### Task 3.1: Vectorized Spherical Conversions
```rust
fn simd_sph_to_vec_8(theta: [f64; 8], lon: [f64; 8]) -> [[f64; 3]; 8]
fn simd_vec_to_sph_8(vec: [[f64; 3]; 8]) -> ([f64; 8], [f64; 8])
```
- Simultaneous sin/cos of 8 angles
- Matrix-vector products for 8 vectors

#### Task 3.2: Vectorized View Transformation
```rust
fn simd_apply_rotation_8(
    vectors: [[f64; 3]; 8],
    transform: &ViewTransform
) -> [[f64; 3]; 8]
```
- 8 matrix-vector products in parallel
- 3×3 rotation matrix × 8 vectors

### Phase 4: Render Loop Integration (Week 3)
**Goal**: Update main loop to use SIMD functions

#### Task 4.1: Conditional SIMD Dispatch
```rust
// In render_projection_to_grid()
if USE_SIMD && cfg!(target_cpu = "...") {
    let (lons, lats, proj_mask) = params.proj.pixel_to_ang_batch_simd(&px_array, &py_array, grid);
    let (samples, healpix_mask) = crate::healpix::sample_healpix_batch_simd(...);
    // SIMD colormap + scaling
} else {
    // Fall back to portable batch (current Tier 2)
}
```

#### Task 4.2: Benchmark & Validation
- Verify SIMD output matches scalar (within epsilon)
- Measure speedup on various hardware
- Profile to identify remaining bottlenecks

### Phase 5: Scaling & Colormap SIMD (Week 3)
**Goal**: Vectorize per-pixel operations

#### Task 5.1: Vectorized Scaling
```rust
fn simd_scale_value_8(
    values: [f64; 8],
    scale_params: &ScaleParams,
    cache: Option<&ScaleCache>
) -> [ScaledValue; 8]
```

#### Task 5.2: Vectorized Gamma Correction + Colormap
```rust
fn simd_apply_gamma_and_sample_8(
    scaled_values: [f64; 8],
    gamma: f64,
    cmap: &Colormap
) -> [[u8; 4]; 8]  // 8 RGBA colors
```

## Success Criteria

### Performance Targets
- **Conservative**: +5% speedup (0.94s → 0.89s on 1200×1200)
- **Expected**: +10-12% speedup (0.94s → 0.83s)
- **Aggressive**: +15% speedup (0.94s → 0.80s)

### Validation
- ✅ Batch SIMD output ≈ scalar output (fp64 epsilon)
- ✅ All 127+ unit tests passing
- ✅ Benchmark improvement measurable and consistent
- ✅ Fallback for non-AVX2 hardware

### Code Quality
- ✅ Zero unsafe code except in SIMD intrinsics (clearly marked)
- ✅ Feature gates for SIMD support
- ✅ Runtime CPU detection for optimal path
- ✅ Comprehensive documentation

## Architecture Decisions

### SIMD Width
- **Choice**: 8×f64 (512 bits)
  - Rationale: Matches Tier 2 batch size, fits in AVX-512 register
  - Fallback: Pair of 4×f64 (AVX2) registers
  - Most CPUs (2020+): AVX-512 or dual-AVX2 capable

### Portability
- **Tier 3a** (Portable): Use `core::simd` or Rust Portable SIMD
  - No unsafe code except standard library
  - Works on any target with 64-bit floats
  
- **Tier 3b** (Optimized): x86-64 intrinsics when available
  - `#[cfg(target_arch = "x86_64")]`
  - Fallback to portable SIMD
  - AVX2 minimum, AVX-512 preferred

### Feature Gates
```toml
# Cargo.toml
[features]
simd = ["packed_simd", "core_arch"]  # Enable SIMD
simd-avx2 = ["simd"]                 # Force AVX2
simd-avx512 = ["simd"]               # Force AVX-512
```

## Risk Assessment

### Technical Risks
| Risk | Impact | Mitigation |
|------|--------|-----------|
| Compiler SIMD codegen poor | Medium | Profile & inline hints |
| Trig accuracy loss | Low | Use high-precision Taylor series |
| CPU feature mismatch | Low | Runtime detection, fallback |
| Maintenance burden | Medium | Keep portable SIMD path as primary |

### Mitigation Strategy
1. Start with portable SIMD (larger maintenance benefit)
2. Benchmark each phase to verify gains
3. Keep scalar path as reference implementation
4. Extensive testing before feature gates

## Timeline & Milestones

**Week 1** (This session):
- [ ] Tier 3 Phase 1: Math primitives (sin, cos, atan2 via SIMD/Taylor)
- [ ] Unit tests for SIMD math (compare to scalar)
- [ ] Benchmark isolated functions

**Week 2** (Next session):
- [ ] Phase 2: Vectorized projection (Mollweide, Hammer)
- [ ] Phase 3: Vectorized HEALPix sampling
- [ ] Integration tests

**Week 3** (Future session):
- [ ] Phase 4: Conditional dispatch in render loop
- [ ] Phase 5: Scaling & colormap SIM
- [ ] End-to-end benchmarks & optimization

## Code References (Tier 2 Foundation)

**Batch Functions to Vectorize**:
- `src/mollweide.rs` lines 95-151 (`pixel_to_ang_batch`)
- `src/healpix.rs` lines 595-640 (`sample_healpix_batch`)
- `src/plot/mod.rs` lines 255-360 (batch rendering loop)

**Support Functions**:
- `src/healpix.rs` - `sph_to_vec`, `vec_to_sph`, `ang2pix`
- `src/rotation.rs` - ViewTransform, matrix operations
- `src/colormap.rs` - Colormap sampling

## Next Steps

1. **Immediate** (Next 30 min): Decision on SIMD strategy (portable vs intrinsics)
2. **Task 1**: Implement vectorized trig functions (sin, cos, atan2)
3. **Task 2**: Create SIMD math test suite
4. **Task 3**: Benchmark isolated math operations
5. **Task 4**: Integrate into projection path
6. **Task 5**: End-to-end benchmarking

---

**Goal**: Ship production-ready SIMD acceleration with +10% performance improvement while maintaining code clarity and cross-platform support.
