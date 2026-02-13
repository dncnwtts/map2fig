# PNG Triangle Rendering Symmetry Fixes - Complete

## Problem Statement

PNG colorbar extend triangles had **two independent symmetry issues**:

1. **Left-Right Asymmetry**: Right triangles were consistently 1 pixel wider than left triangles on every scanline
2. **Top-Down Step Asymmetry**: Left and right edges didn't step inward synchronously - one edge would move while the other stayed fixed, creating a jagged appearance instead of clean diagonal edges

The user described it as: *"No point in the triangle"* and *"no top-down symmetry"*.

## Root Causes

### Issue 1: Left-Right Rasterization Asymmetry

**Root Cause**: Integer division rounding bias in `edge_x_at_y()`

```rust
// BROKEN:
let x = x1 + (dx * t_num + dy / 2) / dy;
```

The formula `dy / 2` uses integer division, which creates different rounding behavior:
- For positive numerators: rounds up at 0.5
- For negative numerators: truncates differently

Since left triangles have negative dx (pointing left) and right triangles have positive dx (pointing right), they experience different rounding, causing systematic 1-pixel bias.

**Fix**: Use symmetric rounding offset that works for both positive and negative slopes

```rust
// FIXED: Symmetric rounding for both slopes
let numerator = dx * t_num;
let offset = (dy + 1) / 2;  // Symmetric offset for both directions
let x = if numerator >= 0 {
    x1 + (numerator + offset) / dy
} else {
    x1 + (numerator - offset) / dy
};
```

### Issue 2: Top-Down Step Asymmetry

**Root Cause**: Independent edge calculations with different rounding accumulation

When calculating left and right edges independently:
- Edge slopes are different: (-50, 60) vs (50, 60)
- Rounding happens independently at each scanline
- Due to fractional accumulation, edges don't step in sync
- Result: One edge steps, then the other, creating 2-pixel jumps instead of smooth 1-pixel steps

**Fix**: Special case for isosceles triangles with symmetric linear interpolation

Instead of calculating edges independently, use the triangle's inherent symmetry:

```rust
// For isosceles triangles, use symmetric interpolation
let t = (y - tip_y) / (base_y - tip_y);  // Progress from tip to base
let left_x = base_left + (center - base_left) * (1 - t);
let right_x = base_right + (center - base_right) * (1 - t);
```

This ensures both edges move exactly symmetrically since they're calculated from the same parameters.

## Implementation

### Changes to `src/colorbar.rs`

1. **Fixed `edge_x_at_y()` rounding** (lines 425-445):
   - Separated positive and negative numerator handling
   - Both cases now use symmetric offset `(dy+1)/2`
   - Results in identical rounding behavior regardless of slope direction

2. **Added isosceles triangle detection** (lines 345-420):
   - Compares edge lengths to identify isosceles triangles
   - Detects which vertex is the "tip" (equidistant from other two)
   - Falls back to standard rasterization for non-isosceles triangles

3. **Implemented symmetric isosceles rasterization** (lines 352-416):
   - Uses linear interpolation from tip toward base
   - Both edges interpolate symmetrically from center point
   - Eliminates independent edge rounding issues

### New Tests

Added 4 critical tests to `tests/test_triangle_rendering.rs`:

1. **`test_actual_pixel_rendering_symmetry()`** - Renders left and right triangles, validates pixel counts match per scanline
2. **`test_triangle_base_to_tip_convergence()`** - Verifies smooth convergence from wide base to 1-pixel tip
3. **`test_left_right_edge_step_symmetry()`** - Validates both edges step inward by same amount each scanline ✓ **Key test for top-down symmetry**
4. **`test_isosceles_rasterization_simple()`** - Confirms isosceles triangles render perfectly symmetric around center

## Results

### Before Fix
```
LEFT-RIGHT ASYMMETRY:
  y=40: left=1 pixels, right=2 pixels
  y=41: left=4 pixels, right=5 pixels  
  ... (20 asymmetric rows)

TOP-DOWN ASYMMETRY:
  y=23: LEFT stepped 0, RIGHT stepped 1
  y=24: LEFT stepped 1, RIGHT stepped 0
  y=29: LEFT stepped 0, RIGHT stepped 1
  y=30: LEFT stepped 1, RIGHT stepped 0
  ... (20 asymmetric rows)
```

### After Fix
```
LEFT-RIGHT SYMMETRY:
  ✓ All scanlines have symmetric widths

TOP-DOWN SYMMETRY:
  y=21: Both edges stepped 1 ✓
  y=22: Both edges stepped 1 ✓
  y=23: Both edges stepped 1 ✓
  ... (all scanlines synchronized)
```

### Test Status
✅ All 19 tests passing
- 15 original coordinate/constraint tests
- 4 new pixel-level rendering tests

### Visual Result
- PNG triangles now have **perfect left-right symmetry**
- Edges step **cleanly and synchronously** without jitter
- Triangles converge to **proper points** (not offset)
- Overall rendering matches **PDF quality** (which uses Cairo backend)

## Colorbar Triangles Verification

Colorbar extend triangles (the main use case) are isosceles by design:
- Tip vertex: center of colorbar height
- Base vertices: left and right edges of colorbar

The symmetric isosceles rasterization ensures these render perfectly:
- Left triangle uses proper symmetric convergence
- Right triangle uses proper symmetric convergence
- Both triangles are visual mirror images

## Files Modified

1. `src/colorbar.rs`:
   - Fixed `edge_x_at_y()` rounding (lines 425-445)
   - Added isosceles detection (lines 345-350)
   - Added symmetric isosceles rasterization (lines 352-416)
   - Made `fill_triangle()` public for testing

2. `tests/test_triangle_rendering.rs`:
   - Added 4 new comprehensive pixel-level tests
   - Total tests now: 19 (all passing)

## Technical Significance

This fix demonstrates a key principle in rasterization:
- **Symmetric geometry requires symmetric algorithms**
- Independent edge calculations can't guarantee symmetry due to rounding
- Using the geometry's inherent symmetry in the algorithm ensures correct results

The isosceles triangle case is common in GUIs and charting applications, so this fix has broader applicability beyond colorbar rendering.

## Output Generated

- `test_final.png`: Full-resolution PNG with both symmetry fixes applied
- File size: ~1.4M (same as before, correct rendering)
- Colorbar triangles now render with perfect symmetry
