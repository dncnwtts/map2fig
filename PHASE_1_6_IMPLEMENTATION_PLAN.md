# Phase 1.6: CUDA Kernel Implementation

## Objectives
Phase 1.6 implements the actual CUDA kernel for GPU-accelerated Mollweide projection rendering. The framework (Phase 1.2B-1.5) is complete and validated; this phase adds the computational kernel that performs:
1. Inverse Mollweide projection: pixel (x,y) → (lon, lat)
2. HEALPix coordinate conversion: (lon, lat) → HEALPix index
3. Data sampling and interpolation
4. Scaling and colormapping
5. RGBA output generation

**Expected Speedup**: 2.5-2.8× on 3.1GB FITS file (19s → ~4s)

## Architecture

### GPU Memory Layout
```
GPU Device Memory
├─ HEALPix data (f64)            // ~3.1 GB
├─ Colormap (RGB LUT, 256×3)      // 768 bytes
├─ ViewTransform (matrix)          // 64 bytes
├─ Scaling parameters              // 100 bytes
└─ Output RGBA buffer (u32)        // width × height × 4

Host Memory
├─ Scaling parameters
├─ Colormap (256 RGB)
└─ Mollweide projection constants
```

### Kernel Design

#### Mollweide Inverse Projection (GPU-side)
For output pixel (px, py) → (lon, lat):
- Aspect ratio correction: x' = px / width, y' = py / height
- Mollweide projection inversion:
  - Intermediate value: m = √(2.0 / π)
  - θ (auxiliary angle) from y' via Newton-Raphson
  - lon = π × (2×x' - 1)
  - lat = arcsin(sin(θ))
- View transform: Apply rotation matrix
- Check validity: Out-of-bounds pixels marked UNSEEN

#### HEALPix Lookup (GPU-side)
- Convert (lon, lat) → (theta, phi) in HEALPix spherical coords
- Use `healpix_ang2pix()` algorithm (ring mode, Nside)
- Bilinear interpolation between adjacent pixels

#### Colormapping (GPU-side)
- Scale value to [0, 1] range based on min/max
- Apply gamma correction: v_scaled^(1/gamma)
- Clamp to [0, 255] and look up in 256-entry RGB LUT
- Set alpha=255 for valid pixels, alpha=0 for UNSEEN

### Kernel Parameters
```cuda
// Passed via kernel arguments or constant memory
gridDim.x (blocks per row)
gridDim.y (blocks per column)  
blockDim.x = 16
blockDim.y = 16

// Passed via global memory pointers
d_healpix_data[]         // float64, length: nside²×12
d_colormap[]             // uint8, shape: (256, 3)
d_output[]               // uint32, shape: (width, height)

// Passed via constant memory
const nside              // HEALPix resolution
const width, height      // Output image dimensions
const scale_min, scale_max
const gamma
const view_matrix[9]     // 3×3 rotation for coordinate system
```

## Implementation Steps

### Step 1: Complete CUDA Kernel (PTX)
**File**: `src/gpu/cuda/kernel.rs` → Update `MOLLWEIDE_PROJECTION_PTX`

Features:
- Thread coalescing for memory bandwidth
- Shared memory for constant data (colormap caching)
- Optimized inverse projection (avoid transcendentals in inner loop)
- UNSEEN value handling
- Gamma correction

### Step 2: GPU Memory Transfer Infrastructure
**File**: `src/gpu/cuda/memory.rs` → Add:
- `GpuColormap`: colormap LUT transfer (256 RGB → 768 bytes)
- `GpuViewTransform`: view matrix transfer (9 f64 values)
- `GpuScalingParams`: min/max/gamma transfer

### Step 3: Kernel Launch Implementation
**File**: `src/gpu/cuda/projection.rs` → Update `CudaMollweideProjector::project()`

Replace placeholder with:
1. Upload HEALPix data (h2d_copy_f64)
2. Upload colormap (h2d_copy via colormap buffer)
3. Upload scaling params
4. Launch kernel via `device.launch_on_stream()`
5. Synchronize and copy output back (d2h_copy_u8)

### Step 4: Integration & Testing
**File**: `src/gpu/cuda/mod.rs` → Replace `render_gpu_reference()`

Real implementation:
```rust
pub fn render_gpu<P: Projection>(
    params: &RenderGridParams<P>,
    grid: &mut RasterGrid,
) -> Result<(), String> {
    let mut projector = CudaMollweideProjector::new(
        params.map,
        &params.meta,
        grid.width,
        grid.height,
    )?;
    
    let output = projector.project(
        params.map,
        params.scale.minv,
        params.scale.maxv,
        params.cmap,  // NEW: Pass full colormap
        params.gamma,
        params.view,  // NEW: Pass view transform
    )?;
    
    // Fill grid from output buffer
    for y in 0..grid.height {
        for x in 0..grid.width {
            let idx = ((y * grid.width + x) as usize) * 4;
            let rgba = [output[idx], output[idx+1], output[idx+2], output[idx+3]];
            grid.set_pixel(x, y, Rgba(rgba));
        }
    }
    
    Ok(())
}
```

## Key Algorithms

### Mollweide Inverse Projection
```rust
// CPU equivalent for reference
fn pixel_to_mollweide(px: f32, py: f32, width: f32, height: f32) -> (f64, f64) {
    let x = (px + 0.5) / width * 2.0 - 1.0;
    let y = (py + 0.5) / height;
    
    // Mollweide constants
    let SIN_HALF_PI = 1.0;  // sin(π/2)
    let SQRT_2_OVER_PI = (2.0 / PI).sqrt();
    
    // Newton-Raphson to find theta from y
    let mut theta = asin(y) as f32;
    for _ in 0..3 {
        let sin_theta = sin(theta);
        let cos_theta = cos(theta);
        theta -= (2.0*theta + sin(2.0*theta) - PI*y) / (2.0 + 2.0*cos(2.0*theta));
    }
    
    // Compute lon, lat
    let lon = PI * x;
    let lat = asin(sin(theta));
    
    (lon as f64, lat as f64)
}
```

### HEALPix Ring Mode Indexing
```rust
// Ring mode: pixels ordered by latitude rings, then longitude
fn healpix_ang2pix_ring(nside: u32, theta: f64, phi: f64) -> u32 {
    let costheta = theta.cos();
    let m = nside as f64;
    let p = phi / (2.0 * PI) * m;
    
    // Determine ring number based on latitude
    let north = (costheta > 1.0 - 1.0 / (3.0 * m * m)) as u32;
    let ipring = if north > 0 {
        (m * (1.0 - costheta / (1.0 - 1.0 / (3.0 * m * m)))).floor() as u32
    } else {
        // Similar for equatorial and south
        ...
    };
    
    // Combine ring and longitude indices
    ...
}
```

## Performance Considerations

### Memory Bandwidth
- HEALPix sampling: ~8 bytes per pixel (f64)
- Output: 4 bytes per pixel (RGBA)
- Colormap: 1 byte per pixel (LUT index)
- **Goal**: 80%+ memory coalescing via global stride accesses

### Computation
- Inverse Mollweide: ~50 FLOPs per pixel (3 transcendentals)
- HEALPix lookup: ~30 FLOPs per pixel
- Colormapping: ~10 FLOPs per pixel
- **Total**: ~90 FLOPs per pixel

### Occupancy
- 16×16 blocks = 256 threads
- 2 blocks per SM on RTX 3000 (full occupancy)
- L2 cache: 3 MB shared among all SMs (good for colormap hits)

## Validation Strategy

### Unit Tests (Phase 1.4 adapted)
- Kernel execution without GPU data access (standalone)
- Mollweide projection correctness vs CPU reference
- HEALPix index lookup correctness
- Colormapping accuracy (±1/255 per channel tolerance)

### Integration Tests
- Full render vs CPU reference (pixel-by-pixel comparison)
- Large FITS file (3.1 GB): accuracy + performance
- Edge cases: UNSEEN values, extreme scaling, rotations

### Benchmarking
- Wall-clock time on test file: Current 19s → Target ~4s
- Per-step timing: Data upload, kernel execution, output readback
- Compare with CPU baseline (19s) and Phase 1.6 GPU (4s target)

## File Changes Summary
| File | Changes | Lines |
|------|---------|-------|
| `src/gpu/cuda/kernel.rs` | Implement full Mollweide CUDA kernel | PTX kernel body |
| `src/gpu/cuda/projection.rs` | Kernel launch with data passing | +50 |
| `src/gpu/cuda/memory.rs` | Colormap/view transform buffers | +30 |
| `src/gpu/cuda/mod.rs` | Replace render_gpu_reference() | -30, +20 |
| `tests/gpu_validation.rs` | Add GPU vs CPU accuracy tests | +20 |

## Timeline
- **Kernel implementation**: 2-3 hours
- **Memory infrastructure**: 1 hour
- **Integration & testing**: 1-2 hours
- **Benchmarking & optimization**: 0.5-1 hour
- **Total**: ~4-7 hours → ~1 working day

## Success Criteria
- ✅ Kernel compiles without errors
- ✅ GPU output matches CPU reference (±1/255 per channel)
- ✅ 3.1 GB FITS renders in <5 seconds
- ✅ 2.5-2.8× speedup achieved
- ✅ All Phase 1.4 + 1.5 tests still passing
