# Phase 2 Decision: Actual vs Theoretical (v0.3.0 Profiling Results)

## Profiling Results After Cairo Batching (v0.3.0)

**Measured with perf record (F=1000Hz) on n=128 map (51,456 pixels):**

```
43.64% ————> cairo_surface_finish (PDF encoding/compression)
 |
 └─21.71% Cairo internals (PDF page rendering, zlib deflate)
 
 8.64% ————> sample_healpix_batch_simd (HEALPix sampling)
 |
 ├─4.43% __sincos_fma (trigonometric math)
 └─2.88% __atan2 (inverse tangent)
 
 4.21% ————> render_projection_to_grid
 
 3.99% ————> draw_colorbar_pdf (colorbar rendering)
 └─2.66% cairo_fill (Cairo fill operations - down from 16.88% in v0.2.0!)
```

## Key Discovery: Cairo Batching Succeeded, But PDF Encoding is Still Heavy

**Before Cairo batching (v0.2.0)**:
- `CairoImageSink::draw_pixel`: 16.88% (per-pixel fills)
- Total Cairo time: ~60% of run

**After Cairo batching (v0.3.0)**:
- `cairo_fill` (colorbar only): 2.66% 
- `cairo_surface_finish` (PDF encoding): 21.71%
- Total remaining Cairo time: ~22% in observable functions

**What happened**: 
- ✅ Reduced fill() calls from 51,456 to ~256 (99.5% reduction)
- ✅ Individual cairo_fill time dropped from 16.88% to 2.66% (84% reduction)
- 📈 BUT Cairo PDF encoder still takes 21.71% of time

**Root cause**: The bottleneck shifted from **pixel rendering** to **PDF page synthesis/compression**
- Cairo must build the PDF structure with all paths, colors, text, etc.
- zlib deflate compression (12.41% of time per perf output)
- PDF page header/metadata processing

---

## Strategic Decision: Phase 2B Before 2A

### Why Image Pre-rendering (2B) is Now Higher Priority

**Option A (SIMD Vectorization - Phase 2A)**:
- Targets HEALPix sampling math: 8.64% of time
- sin/cos/atan2 operations: 7.31% combined
- Best case: 50% speedup on math (3.66% saved total)
- Result: 470 ms → 453 ms (3.6% improvement, **below target**)

**Option B (Image Pre-rendering - Phase 2B)**:
- Eliminates PDF encoding overhead by using hybrid approach:
  - Render pixels to in-memory image (fast)
  - Embed image in PDF as single surface (avoids Cairo path manipulation)
- Available since v0.3.0: Cairo can paint from ImageSurface efficiently
- Target: Reduce 21.71% Cairo overhead by 50-70%
- Potential: 10-15% of 21.71% = 2-3% raw time savings
- **BUT: More importantly, avoids 43.64% total Cairo_surface_finish overhead by reducing PDF complexity**
- Realistic expectation: 8-12% additional improvement
- Result: 470 ms → 415-430 ms (12-15% improvement, **meets/exceeds target**)

### Why 2B Before 2A Makes Sense Architecturally

Current pipeline with Cairo batching:
```
Projection (raster grid) → Scale → Colormap → Cairo rasterize (256 fill calls) → PDF encode (21% time)
```

The architecture itself is limiting:
- Every pixel goes through Cairo's unified coordinate system
- Cairo must build internal representation for "page"
- PDF encoder then serializes and compresses

Image pre-rendering breaks this:
```
Projection (raster grid) → Scale → Colormap → ImageBuffer.put_pixel (FAST mem) → Cairo: paint image surface
```

**In this model**:
- No per-pixel Cairo operations at all
- No path manipulation overhead
- Single surface paint = single PDF object reference
- Dramatically reduces what Cairo must process

---

## Implementation: Image Pre-rendering (Phase 2B)

### Architecture

**Current (v0.3.0)**:
```rust
// src/render/pdf.rs: blit_raster()
let cr = cairo::Context::new(&surface).unwrap();
let mut sink = BatchedCairoImageSink::new(&cr);

for py in 0..raster.height() {
    for px in 0..raster.width() {
        let [r, g, b, a] = raster.get_pixel(px, py);
        sink.draw_pixel(px, py, image::Rgba([r, g, b, a]));
    }
}
sink.flush();  // 256 cairo_fill calls
```

**Proposed (v0.4.0)**:
```rust
// src/render/pdf.rs: blit_raster() - Image pre-rendering path
let mut img_buffer = image::RgbaImage::new(raster.width(), raster.height());

for py in 0..raster.height() {
    for px in 0..raster.width() {
        let [r, g, b, a] = raster.get_pixel(px, py);
        img_buffer.put_pixel(px, py, image::Rgba([r, g, b, a]));
    }
}

// Now render the entire image as a single Cairo operation (no more fill() calls!)
let cairo_img_surf = cairo::ImageSurface::create_from_png_stream(
    /* convert buffer to PNG in-memory, or use new Cairo function */
).unwrap();

cr.set_source_surface(&cairo_img_surf, x, y);
cr.paint().unwrap();  // ← Single paint() instead of 256 fill() calls
```

### Trade-offs

| Aspect | Before (Batching) | After (Image) | Impact |
|--------|-------------------|---------------|--------|
| Memory | Pixel raster only | Raster + ImageBuffer (~4MB for 1200×741) | +4MB temp |
| Cairo operations | 256 fill() | 1 paint() | Huge reduction |
| PDF size | Same | May be smaller (single image object) | Neutral/positive |
| Colormap accuracy | Exact (per-pixel Cairo) | Exact (bit-identical) | No loss |
| Latency | 470 ms | Expected 415-430 ms | 10-15% faster |

### Implementation Steps

1. **Create ImageSurface from buffer** (30 min)
   - Option A: Use Rust image crate PNG encoding
   - Option B: Use Cairo's ImageSurface::create_from_rgb/argb + paint
   - Recommended: Option B (direct, no PNG serialization overhead)

2. **Modify `blit_raster()` in pdf.rs** (1 hour)
   - Allocate RgbaImage buffer
   - Loop through pixels (same as now, just different sink)
   - Create Cairo surface from buffer
   - Paint once instead of batching fills

3. **Test output quality** (30 min)
   - Verify PDF reads correctly
   - Visual inspection (should be identical)
   - File size comparison

4. **Benchmark** (1 hour)
   - Profile with perf again
   - Measure improvement (target: 415-430 ms)
   - Check if Phase 2A is still needed

---

## Risk Assessment: Phase 2B vs 2A

| Aspect | Phase 2B (Image) | Phase 2A (SIMD) |
|--------|------------------|-----------------|
| **Implementation** | Straightforward (replace sink) | Complex (SIMD intrinsics) |
| **Testing** | Easy (output should be identical) | Requires regression testing |
| **Risk** | Low (proven technique) | Medium (platform-specific) |
| **Fallback** | Keeps batching as fallback | Harder to roll back |
| **Time to implement** | 2-3 hours | 4-6 hours |
| **Confidence** | High (21% overhead clearly visible) | Medium (8% target for 7.3% overhead) |

---

## Recommendation: **Start with Phase 2B (Image Pre-rendering)**

### Reasoning

1. **Higher confidence**: Profiling clearly shows 21.71% in cairo_surface_finish
2. **Higher impact**: 10-15% potential vs 3-4% from SIMD
3. **Lower risk**: Image buffering is well-established technique
4. **Simpler testing**: Can't go wrong with bit-identical output
5. **Architecture aligned**: Works with existing batch loop structure
6. **Provides fallback**: Phase 2A can still be added afterward

### If Phase 2B succeeds
- Target PDF: 415-430 ms (from 470 ms)
- Exceeds v0.3 goals by 50+%
- Could release as v0.4 with "image pre-rendering optimization"

### If Phase 2B hits snags
- Fallback: Keep batching, pursue Phase 2A SIMD
- Can parallelize both improvements if needed

---

## Next Immediate Steps

1. **Reserve 3-4 hours** for Phase 2B implementation
2. **Check Cairo documentation** for ImageSurface creation from buffer
3. **Start with proof-of-concept**: Small test PDF with image surface
4. **Profile before/after** to confirm 10-15% improvement
5. **If successful**: Commit, document, prepare v0.4 release
6. **If unsuccessful**: Pivot to Phase 2A SIMD vectorization

**Timeline**: 3-4 hours implementation + testing → Ready for benchmarking tomorrow

