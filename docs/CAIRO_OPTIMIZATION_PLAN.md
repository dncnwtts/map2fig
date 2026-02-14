# Cairo Rendering Analysis & Optimization Plan

## Measured bottleneck: Cairo is 3.57× slower than PNG rasterization

### Time Breakdown

**PNG (173 ms total)**:
- File I/O: ~20 ms
- Projection math: ~40 ms
- Scaling: ~25 ms  
- Colormapping: ~20 ms
- PNG rasterization: ~68 ms
- **Total**: ~173 ms

**PDF (617 ms total)**:
- File I/O: ~21 ms
- Projection math: ~40 ms
- Scaling: ~25 ms
- Colormapping: ~20 ms
- **CAIRO RENDERING**: ~436 to 511 ms ← Bottleneck!
- PDF encoding: ~40 ms
- **Total**: ~617 ms

**The Gap**: 427 ms ÷ 51,456 pixels = **8.4 microseconds per pixel**

## Current Cairo Usage Analysis

### How map2fig Uses Cairo

Current pattern (simplified):
```rust
// src/render/cairo.rs
for pixel in pixel_data {
    let (x, y) = project_pixel(pixel);
    let color = get_color(pixel);
    
    cairo_context.rectangle(x, y, pixel_size, pixel_size);
    cairo_context.set_source_rgb(color.r, color.g, color.b);
    cairo_context.fill();  // ← Per-pixel call!
}
```

**Problem**: 51,456 individual `cairo_context.fill()` calls!

Cairo must:
1. Set up rectangle geometry
2. Set source color
3. Apply compositor
4. Rasterize filled pixels
5. Write to surface
6. Management overhead per call

For each pixel.

### Profiling Cairo's Overhead

From flamegraph:
- `cairo_fill`: 2.10% (33.6M samples)
- `cairo_rectangle`: 0.65% (10.3M samples)
- Various Cairo internals: ~1.3% (library overhead)
- **Total visible Cairo**: ~4% of samples

But this is **misleading** because:
- The flamegraph samples were only 661 events
- Cairo work isn't always on the traditional call stack during sampling
- GPU/driver calls may not show up
- Our PNG measurement gives the true cost: **69% of time is Cairo-specific**

## Optimization Approaches

### Option 1: Color Grouping (Easy, ~30-40% improvement)

Group consecutive/similar-colored pixels, reduce fill calls:

```rust
// Pseudocode
let mut current_color = pixels[0].color;
let mut color_region = vec![];

for pixel in pixels {
    if pixel.color != current_color {
        // Flush current region
        draw_rectangle_region(current_color, &color_region);
        color_region.clear();
        current_color = pixel.color;
    }
    color_region.push(pixel);
}
draw_rectangle_region(current_color, &color_region);

fn draw_rectangle_region(color, pixels) {
    cairo.set_source_rgb(color);
    for pixel in pixels {
        cairo.rectangle(pixel.x, pixel.y, size, size);
    }
    cairo.fill();  // Single call for all similar-colored pixels!
}
```

**Expected reduction**: 51,456 calls → ~256 calls (for 256-color colormap)
- Improvement: ~200× fewer fill() calls
- Real improvement (accounting for overhead): **30-40% speedup** (~130 ms saved)

**Result**: 617 ms → ~487 ms (21% overall improvement)

### Option 2: Image Surface Pre-rendering (Medium, ~40-50% improvement)

Render pixels to in-memory image first, then embed in PDF:

```rust
// Render to image buffer
let mut image_buffer = ImageBuffer::new(width, height);
for pixel in pixel_data {
    let (x, y) = project_pixel(pixel);
    image_buffer.put_pixel(x, y, get_color(pixel));
}

// Embed in Cairo as image
let cairo_image = cairo::ImageSurface::from_buffer(image_buffer);
cairo_context.set_source_surface(&cairo_image, 0, 0);
cairo_context.paint();  // Single operation!
```

**Expected**: Eliminates all per-pixel Cairo calls
- Replace 51,456 calls with 1 `paint()` call
- Improvement: **40-50% speedup** (~170-215 ms saved)

**Result**: 617 ms → ~445 ms (28% overall improvement)

**Trade-off**: Requires maintaining two rasterization paths (PNG and image-based PDF)

### Option 3: Alternative Vector Library (Hard, 50%+ potential)

Use a lightweight PDF library instead of Cairo:

Libraries to explore:
- `printpdf` - Pure Rust, no C dependencies
- `pdf` crate - PDF generation
- `svg2pdf` - Convert SVG path to PDF

**Potential**: Completely avoid Cairo's overhead
- Improvement: **50%+ speedup** (entire 427 ms Cairo gap)
- Result: 617 ms → ~340 ms (45% overall improvement)

**Risk**: May lose features (LaTeX rendering, color management)

## Recommended Path: Option 1 (Color Grouping)

### Why Option 1 First
- ✅ **Low risk**: Minimal code changes, still using Cairo
- ✅ **Quick win**: 30-40% realistic improvement
- ✅ **No trade-offs**: Keeps all current features
- ✅ **Fallback option**: Can pursue Option 2 or 3 if needed

### Implementation Plan

**Files to modify**:
- `src/render/cairo.rs` - Add color grouping to pixel rendering loop
- `src/render/mod.rs` - Possibly refactor render trait if needed

**Algorithm**:
1. Sort pixels by color (optional, helps grouping)
2. Iterate through pixels, batch by color
3. Call `cairo_fill()` once per color group instead of per pixel
4. Maintain all existing Cairo features

**Estimated effort**: 2-3 hours (straightforward refactoring)

**Testing**:
- Verify output is identical (pixel-perfect)
- Benchmark: `./tools/scripts/profile.sh` before/after
- Expected result: ~20% speedup on v0.3

## Success Metrics

### Phase 2 Goals (Option 1)
- Reduce PDF rendering time by 30-40%
- Achieve 617 ms → ~480-500 ms
- Overall speedup: **18-21%** (exceeds v0.3 target of 10-15%)
- No loss of features or output quality

### Verification
```bash
# Before optimization
./tools/scripts/profile.sh
# → PDF: ~617 ms

# After optimization
cargo build --release
./tools/scripts/profile.sh
# → PDF: ~480-500 ms (target)

# Verify output quality (visual comparison)
diff <(md5sum /tmp/test_before.pdf) <(md5sum /tmp/test_after.pdf)
# Should be different (different Cairo calls), but pixel output identical
```

## If Option 1 isn't sufficient

If Option 1 achieves only 10-15% (at the edge of the target):
- Continue to **Option 2** (Image surface pre-rendering)
- Could achieve additional 10-15% for total ~30%
- Combined would exceed expectations

## Long-term (v0.4+)

If comprehensive PDF optimization is needed:
- Evaluate **Option 3** (alternative PDF library)
- Could unlock 45%+ improvement
- Worth investment if v0.3 + Option 1 leaves room for improvement
