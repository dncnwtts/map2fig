# Gnomonic Projection Text Labels Feature

## Overview
Added resolution and pixel size labels to gnomonic projection output, similar to healpy's `hp.gnomview()` functionality.

## Feature Details

### What's Displayed
Text label shows the pixel size and grid dimensions:
```
{resolution:.2} '/pix,   {width}x{height} pix
```

Example output for a 200x200 pixel gnomonic map with 3 arcsmin/pixel resolution:
```
3.00 '/pix,   200x200 pix
```

### CLI Usage

**Show text label (default):**
```bash
cargo run -- -f data.fits --projection gnomonic -o map.png
```

**Hide text label:**
```bash
cargo run -- -f data.fits --projection gnomonic --no-text -o map.png
```

The `--no-text` flag works with both PNG and PDF output formats.

### Layout Adjustments
When text is enabled:
- Left margin is increased by `50.0 * (width / 1200.0)` pixels
- This reserves space for the vertical text label on the left side of the map
- When `--no-text` is used, layout remains compact

### Implementation Details

**Files Modified:**
- `src/cli.rs`: Added `--no-text` CLI argument
- `src/params.rs`: Added `show_gnomonic_text: bool` field to `GnomonicParams`
- `src/layout.rs`: Updated `compute_gnomonic_layout()` to accept `show_text` parameter
- `src/main.rs`: Pass `show_gnomonic_text` through parameter chain
- `src/plot.rs`: 
  - Updated `plot_gnomonic_png()` to render text labels
  - Updated `plot_gnomonic_pdf()` to render text labels with Cairo
  - Updated `plot_gnomonic_auto()` to pass parameters

**Text Rendering:**
- **PNG**: Uses imageproc to draw text in black (RGB 0,0,0)
- **PDF**: Uses Cairo text API to draw text in black
- **Position**: Left side of map, vertically centered
- **Font**: DejaVuSans (same font as colorbar labels)

## Testing

All 131 unit tests pass with this feature enabled.

### Manual Testing

Generate test images with different configurations:
```bash
# PNG with text (default)
cargo run -- -f cosmoglobe_clipped.fits --projection gnomonic --fov 10 -o test_with_text.png

# PNG without text
cargo run -- -f cosmoglobe_clipped.fits --projection gnomonic --fov 10 --no-text -o test_no_text.png

# PDF with text
cargo run -- -f cosmoglobe_clipped.fits --projection gnomonic --fov 10 -o test_with_text.pdf

# PDF without text
cargo run -- -f cosmoglobe_clipped.fits --projection gnomonic --fov 10 --no-text -o test_no_text.pdf
```

## Related Issues
- Phase 1: Gnomonic projection scale fix (FOV-aware percentile/histogram) - **COMPLETED**
- Phase 2: Text label feature - **COMPLETED**

Both phases are now fully functional and tested.
