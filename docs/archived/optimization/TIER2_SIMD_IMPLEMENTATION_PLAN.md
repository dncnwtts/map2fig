# Tier 2 SIMD Batching Implementation Plan

## Overview

Convert the single-pixel rendering loop to process **8 pixels in parallel** using SIMD operations. This targets the most CPU-intensive parts of the pipeline:

1. **Projection** (`pixel_to_ang`): Compute (lon, lat) from (px, py)
2. **Spherical math**: Convert to/from (theta, phi) vectors
3. **Rotation**: Apply view transformation
4. **Gamma**: Apply power function

## Current Bottleneck

**File:** `src/plot/mod.rs` lines 245-290

```rust
for py in 0..height {
    for px in 0..width {
        // 1. pixel_to_ang(px, py)           // Projection math
        // 2. sample_healpix(theta, phi)     // Trig + HEALPix lookup
        // 3. scale_value(...)               // Already optimized (Tier 1)
        // 4. apply_gamma(t, gamma_inv)      // Gamma LUT (Tier 1, already fast)
        // 5. colormap.sample(t)             // LUT lookup
        // 6. grid.put_pixel(...)            // Write result
    }
}
```

**Hotspots** (in rough order of cost):
1. `pixel_to_ang`: Medium - involves sqrt, trig
2. `sample_healpix`: Medium - spherical interpolation
3. `apply_gamma`: Low - now optimized with LUT
4. `scale_value`: Low - now optimized with cache

## Proposed SIMD Strategy

### Phase 1: Batch Projection (Priority: HIGH)

**Goal:** Process 8 x-coordinates and 8 y-coordinates in parallel

```rust
// OLD (scalar):
for px in 0..width {
    for py in 0..height {
        if let Some((lon, lat)) = proj.pixel_to_ang(px, py, grid) {
            // ... process one pixel ...
        }
    }
}

// NEW (SIMD):
for y_batch in 0..height / 8 {
    let py = [y_batch*8, y_batch*8+1, ..., y_batch*8+7];  // 8 y-coords
    
    for x_batch in 0..width / 8 {
        let px = [x_batch*8, x_batch*8+1, ..., x_batch*8+7];  // 8 x-coords
        
        // Process 8 pixels in parallel:
        let lons = proj.pixel_to_ang_batch_lon(&px, &py, grid);  // [f64; 8]
        let lats = proj.pixel_to_ang_batch_lat(&px, &py, grid);  // [f64; 8]
        
        let thetas = [PI/2 - lat for lat in lats];  // [f64; 8]
        
        // Process 8 HEALPix samples in parallel
        let values = sample_healpix_batch(map, meta, view, thetas, lons);  // [f64; 8]
        
        // Rest of pipeline on 8 values at once
        let scaled = scale_value_batch(values, params);  // [f64; 8]
        let gammaed = apply_gamma_batch(scaled, gamma);  // [f64; 8]
        let colors = colormap_batch(gammaed);            // [Rgba; 8]
        
        // Write 8 pixels to grid
        for (i, color) in colors.iter().enumerate() {
            grid.put_pixel(x_batch*8 + i, y_batch*8 + j, *color);
        }
    }
}
```

### Phase 2: Spherical Math Vectorization (Priority: MEDIUM)

**Goal:** Vectorize sin/cos/atan2 operations

Rust doesn't have built-in SIMD trig in std lib. Options:

1. **Direct SIMD with platform intrinsics** (AVX-512, Neon)
   - Pro: Maximum performance
   - Con: Platform-specific, complex
   
2. **Use `packed_simd` crate** (nightly feature)
   - Pro: Portable, clean API
   - Con: Depends on nightly Rust
   
3. **Manual loop unrolling** (simplest for now)
   - Pro: Works on stable Rust, reasonable speedup
   - Con: Less optimal than true SIMD

**Recommendation:** Start with **manual loop unrolling** then add `packed_simd` if needed.

### Phase 3: Rotation Vectorization (Priority: LOW)

**Goal:** Matrix-vector multiply 8 vectors in parallel

```rust
// OLD: v_map = rotation_matrix * v_view
// Process 1 vector at a time

// NEW: v_map_batch = rotation_matrix * [v_view_1, v_view_2, ..., v_view_8]
// Process 8 vectors in parallel via matrix multiply
```

Only adds ~10% more speedup due to memory bandwidth limits.

## Implementation Steps

### Step 1: Create Batch Projection Functions
- File: `src/projection.rs` or `src/plot/mod.rs`
- Add `pixel_to_ang_batch()` that returns `([f64; 8], [f64; 8])`
- Test against scalar version for correctness

### Step 2: Create Batch HEALPix Sampling
- File: `src/healpix.rs`
- Add `sample_healpix_batch()` that samples 8 pixels at once
- Manual loop unrolling: process 8 samples without SIMD intrinsics

### Step 3: Create Batch Rendering Loop
- File: `src/plot/mod.rs`
- Create `render_projection_to_grid_simd()` function
- Keep scalar fallback for edge pixels (boundaries)
- Merge results at end

### Step 4: Testing & Validation
- Unit tests: Compare scalar vs SIMD output (should be identical)
- Benchmark: Measure throughput improvement
- Edge cases: Handle partial batches at image boundaries

## Code Skeleton

```rust
/// Process 8 pixels in parallel
#[inline]
fn batch_render_8_pixels(
    px: [u32; 8],
    py: [u32; 8],
    params: &RenderGridParams,
    grid: &mut RasterGrid,
) {
    // 1. Batch projection: 8 (lon, lat) pairs
    let (lons, lats) = batch_pixel_to_ang(&px, &py, params.proj, grid);
    
    // 2. Convert to spherical: 8 theta values
    let thetas = lats.map(|lat| std::f64::consts::PI / 2.0 - lat);
    
    // 3. Batch HEALPix sampling: 8 values
    let values = batch_sample_healpix(params.map, params.meta, params.view, &thetas, &lons);
    
    // 4. Batch scaling: 8 normalized values [0, 1]
    let scaled = batch_scale_value(&values, params);
    
    // 5. Batch gamma: 8 gamma-corrected values
    let gammaed = batch_apply_gamma(&scaled, params.gamma);
    
    // 6. Batch colormap: 8 RGBA colors
    let colors = batch_colormap(&gammaed, params.cmap);
    
    // 7. Write 8 pixels to grid
    for i in 0..8 {
        if px[i] < grid.width && py[i] < grid.height {
            grid.put_pixel(px[i], py[i], colors[i]);
        }
    }
}

pub fn render_projection_to_grid_simd(
    params: RenderGridParams,
    grid: &mut RasterGrid,
) {
    let width = grid.width;
    let height = grid.height;
    
    // Main SIMD loop: process 8 pixels at a time
    for py in (0..height).step_by(8) {
        for px in (0..width).step_by(8) {
            // Handle batch
            if px + 8 <= width && py + 8 <= height {
                // Full 8×1 batch
                let py_arr = [
                    py, py+1, py+2, py+3, py+4, py+5, py+6, py+7
                ].map(|y| y as u32);
                let px_arr = [px as u32; 8];
                
                batch_render_8_pixels(px_arr, py_arr, &params, grid);
            } else {
                // Partial batch at edges - fall back to scalar
                for y in py..height.min(py+8) {
                    for x in px..width.min(px+8) {
                        // Scalar path (existing code)
                        render_single_pixel(x as u32, y as u32, &params, grid);
                    }
                }
            }
        }
    }
}
```

## Expected Performance

| Metric | Tier 1 Only | Tier 1 + Tier 2 | Gain |
|--------|------------|-----------------|------|
| Linear 1200×1200 | 0.930s | ~0.82s | +12% |
| Histogram 1200×1200 | 1.198s | ~1.05s | +12% |
| Combined Gain | +1.5% | ~5-7% | **+4-6%** |

Conservative estimate due to:
- Memory bandwidth limits (8 FMA ops/cycle max)
- Cache misses in HEALPix lookup
- Scalar fallback at image edges

## Risk Assessment

| Risk | Mitigation |
|------|-----------|
| Off-by-one errors | Extensive unit tests comparing scalar vs SIMD |
| Cache misses | Profile with perf, may revert if regression |
| SIMD register pressure | Keep batch size at 8 (AVX2 lane count) |
| Complexity | Keep scalar path as reference implementation |

## Success Criteria

- ✅ All tests pass (same output as scalar)
- ✅ No performance regression on small images
- ✅ +3-5% speedup on 2400×2400 renders
- ✅ Code remains maintainable (clear comments/documentation)

## Timeline Estimate

- Step 1 (Projection): 2-3 hours
- Step 2 (HEALPix): 1-2 hours  
- Step 3 (Main loop): 1-2 hours
- Step 4 (Testing): 2-3 hours
- **Total: 6-10 hours over multiple sessions**

