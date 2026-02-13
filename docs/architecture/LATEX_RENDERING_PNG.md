# PDF vs PNG Rendering - Summary of Changes

## Problem Statement
The PDF and PNG outputs were rendering colorbar units labels differently:
- **PDF**: Properly rendered LaTeX-formatted units as vector text via Cairo
- **PNG**: Stripped LaTeX delimiters and showed raw text because it didn't support LaTeX rendering

This caused visual inconsistency between output formats.

## Solution Implemented

### Changes Made
1. **Modified `src/plot.rs` - Mollweide PNG rendering** (lines 755-818)
   - Added LaTeX rendering support for units labels
   - Uses `render_latex_to_png()` to convert LaTeX to raster image
   - Alpha-blends the rendered LaTeX image onto the main PNG
   - Falls back to stripped text if LaTeX rendering fails

2. **Modified `src/plot.rs` - Gnomonic PNG rendering** (lines 1284-1347)
   - Applied the same LaTeX rendering support as Mollweide
   - Consistent positioning and fallback behavior

### Implementation Details

The PNG rendering now:
1. Calls `latex_render::render_latex_to_png(units_str, 6)` to get the rendered image bytes
2. Loads the PNG bytes using the `image` crate
3. Composites the LaTeX image onto the main image with proper alpha blending:
   ```rust
   let alpha = pixel[3] as f32 / 255.0;
   let blended = Rgba([
       ((pixel[0] as f32 * alpha + existing[0] as f32 * (1.0 - alpha)) as u8),
       ((pixel[1] as f32 * alpha + existing[1] as f32 * (1.0 - alpha)) as u8),
       ((pixel[2] as f32 * alpha + existing[2] as f32 * (1.0 - alpha)) as u8),
       255,
   ]);
   ```
4. Falls back to stripped LaTeX text if rendering fails

## Rendering Pipeline

### PDF (Cairo-based)
```
LaTeX string → pdflatex → PDF
↓
PDF → pdftoppm (for rasterization to PNG at given DPI)
```

### PNG (RgbaImage-based with embedded LaTeX)
```
LaTeX string → pdflatex → PDF → pdftoppm → PNG bytes
↓
Load PNG bytes → Alpha-blend onto main image
```

## Comparison Results

**Note**: Perfect pixel-level identity between PDF and PNG is not achievable due to:
1. Different underlying rendering libraries (Cairo vs RgbaImage)
2. Different font rendering engines and anti-aliasing algorithms
3. Inherent differences in text baseline calculations

However, both formats now:
- ✅ Support LaTeX-formatted units labels
- ✅ Render them as rasterized images (not raw text)
- ✅ Center them properly below the colorbar
- ✅ Use the same layout calculations

## Testing

Generate test outputs:
```bash
# PDF version
./target/release/map2fig -f <fits_file> -o output.pdf --latex --units '$T_{CMB}$ (K)'

# PNG version  
./target/release/map2fig -f <fits_file> -o output.png --latex --units '$T_{CMB}$ (K)'
```

Compare outputs:
```bash
python3 compare_pdf_png.py output.pdf output.png /tmp/compare_output
```

This generates:
- `difference.png` - Heat map showing pixel differences
- `comparison.png` - Side-by-side comparison of the two images

## Files Modified
- [src/plot.rs](src/plot.rs) - Lines 755-818 (Mollweide) and 1284-1347 (Gnomonic)
- [compare_pdf_png.py](compare_pdf_png.py) - New comparison utility

## Build Status
✅ All changes compile successfully with `cargo build --release`
