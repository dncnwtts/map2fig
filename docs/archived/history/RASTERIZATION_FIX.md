# Rasterization Asymmetry Fix

## Problem Identified

The PNG triangle rendering had a **systematic 1-pixel asymmetry** in left vs right triangles:
- Every scanline in the right triangle had 1 extra pixel compared to the left triangle
- This manifested as visually asymmetric triangle arrows in the colorbar
- The issue was **NOT** in the vertex coordinate calculations (those were correct)
- The issue was in the **pixel-level rasterization** algorithm

## Root Cause

The bug was in the `edge_x_at_y()` function in `src/colorbar.rs` (line ~420):

```rust
// BROKEN: Integer division bias toward positive slopes
let x = x1 + (dx * t_num + dy / 2) / dy;
```

The formula `(dx * t_num + dy/2) / dy` uses integer division `dy/2`, which:
- For `dy=20`: `dy/2 = 10` (correct)
- For positive numerators: `(20 + 10) / 20 = 1` ✓
- For negative numerators: `(-20 + 10) / 20 = -0.5 → 0` (floors toward zero, introduces bias)

This asymmetry is compounded because:
- **Left triangles** have `dx < 0` (pointing left), so numerators are often negative
- **Right triangles** have `dx > 0` (pointing right), so numerators are positive
- The different rounding rules for negative vs positive values cause the bias

## Solution

Fixed the `edge_x_at_y()` function to use **symmetric rounding** that treats positive and negative slopes identically:

```rust
// FIXED: Symmetric rounding for both positive and negative slopes
let numerator = dx * t_num;
let x = if numerator >= 0 {
    x1 + (numerator + dy / 2) / dy
} else {
    // For negative: subtract (dy-1)/2 for symmetric rounding
    x1 + (numerator - (dy - 1) / 2) / dy
};
```

This ensures that:
- Negative slopes round the same way as positive slopes
- Left and right triangles have identical pixel patterns (just mirrored)
- All scanlines have symmetric widths

## Test Results

### Before Fix
```
  y=40: left=1 pixels, right=2 pixels  ← Asymmetry!
  y=41: left=4 pixels, right=5 pixels  ← Off by 1
  y=42: left=7 pixels, right=8 pixels  ← Off by 1
  ...
  ⚠️  Row 0-19: asymmetric width!
  ❌ FOUND 20 ASYMMETRIC ROWS
```

### After Fix
```
  y=40: left=1 pixels, right=1 pixels  ✓ Symmetric
  y=41: left=4 pixels, right=4 pixels  ✓ Symmetric
  y=42: left=7 pixels, right=7 pixels  ✓ Symmetric
  ...
  ✓ All scanlines have symmetric widths
  test result: ok. 16 passed; 0 failed
```

## Files Changed

- `src/colorbar.rs`: Fixed `edge_x_at_y()` function (lines 390-436)
- `src/colorbar.rs`: Made `fill_triangle()` public for testing (line 334)
- `tests/test_triangle_rendering.rs`: Added `test_actual_pixel_rendering_symmetry()` test

## Impact

This fix eliminates the visual asymmetry in PNG colorbar extend triangles:
- ✅ Left and right triangles now perfectly symmetric
- ✅ No more "1-pixel off" bias in rasterization
- ✅ Triangle tips render as proper points (not offset)
- ✅ All colorbar rendering now matches PDF quality

## Why Previous Tests Passed

The earlier tests only checked **vertex coordinates in the code**, not the **actual rendered pixels**:
- Tests verified that `fill_triangle()` was *called* with correct vertices
- Tests did NOT check what pixels were actually written
- The bug was in the edge-crossing calculation inside `fill_triangle()`, not in the vertex calculation

The new `test_actual_pixel_rendering_symmetry()` test actually renders triangles and validates pixel output, catching this type of rasterization bug.
