# Graticule Rendering Quality Improvements - Implementation Summary

## Issues Fixed

### 1. ❌ PNG Graticule Lines Looked Pixelated/Rough
**Problem:** PNG graticule lines were drawn using Bresenham algorithm directly to the pixel grid, resulting in aliased, jagged appearance. Border was smooth (using Cairo anti-aliasing) but graticules were crude.

**Solution:** Integrated graticule rendering into the same Cairo-based surface used for the border:
- Generate vectorized polylines via `render_graticule_mollweide_vectorized()`
- Render polylines to a Cairo ImageSurface (anti-aliased vector rendering)
- Composite the anti-aliased surface onto the main PNG image
- Result: Smooth, publication-quality graticule lines matching border quality

**Implementation:** Modified `plot_mollweide_png()` to share the Cairo rendering surface with the border.

### 2. ❌ Pole Latitude Lines Wrapping at Boundaries
**Problem:** Extreme latitude lines (near ±90° poles) were wrapping around boundaries, creating visible crossing artifacts.

**Root Cause:** At the poles, all longitudes mathematically converge to the same point, causing numerical instability. The discontinuity detection threshold (Δu > 0.3 or Δv > 0.3) was being triggered incorrectly by pole transformations, breaking the line.

**Solution:** Added special handling for poles:
- Detect when sampling latitudes near ±90°
- Skip discontinuity detection at poles (since all longitudes converge anyway)
- At non-pole latitudes, keep existing discontinuity detection to handle projection edges

**Implementation:** Added pole detection logic to both meridian and parallel sampling loops in `render_graticule_mollweide()`.

---

## Technical Details

### PNG Graticule Rendering Pipeline (IMPROVED)

**Before:**
```
Graticule lines (lon/lat) 
    ↓
Project to Mollweide [0,1]
    ↓
Bresenham rasterization (pixelated, no anti-aliasing)
    ↓
Draw directly to RgbaImage
    ↓
Result: Jagged grid lines
```

**After:**
```
Graticule lines (lon/lat)
    ↓
Project to Mollweide [0,1]
    ↓
Vectorized polylines
    ↓
Cairo ImageSurface (anti-aliased rendering)
    ↓
Composite to main PNG
    ↓
Result: Smooth, crisp grid lines
```

### Pole Handling Logic

```rust
// For each parallel (constant latitude):
let is_pole = (par_deg - 90.0).abs() < 0.1 || (par_deg + 90.0).abs() < 0.1;

if is_pole {
    // Skip: poles are points, not lines
} else {
    // Normal processing with discontinuity detection
    // Sample all longitudes along this latitude
}
```

This prevents the issue where numerical errors at poles cause false positive discontinuity detection.

---

## Unit Test Added

**Test:** `test_pole_graticule_no_wrapping()`

**Purpose:** Regression test for pole boundary crossing issue

**Verification Method:**
1. Sample an extreme latitude (±90°)
2. Transform through coordinate systems (E→G in test)
3. Project all sampled points to Mollweide
4. Verify that different longitudes at the pole project to approximately the same location
5. Assert no large jumps (Δu > 0.15 or Δv > 0.15)

**Result:** ✅ PASS

---

## Code Changes

### File: `src/plot.rs`

**Change 1:** Unified Cairo surface creation for both border and graticule
- Lines 455-510: Combined `if draw_border || show_graticule` block
- Creates single Cairo ImageSurface with padding
- Renders graticule first, then border (so border appears on top)
- Both use anti-aliased Cairo rendering

**Before:**
```rust
if draw_border { /* only border */ }
```

**After:**
```rust
if draw_border || show_graticule {
    // Shared Cairo surface
    // Render graticule (if enabled)
    // Render border (if enabled)
}
```

### File: `src/graticule.rs`

**Change 1:** Meridian sampling - skip discontinuity check at poles (lines 186-228)
- Added `is_pole` detection
- Skip discontinuity comparison when sampling across poles
- Prevents false positives from numerical instability

**Change 2:** Parallel sampling - skip poles entirely (lines 252-290)
- Detect poles explicitly
- Skip rendering for poles (they're points, not lines)
- Sample normally for non-pole latitudes

**Change 3:** New unit test `test_pole_graticule_no_wrapping()` (lines 1164-1220)
- Regression test for pole crossing issue
- Validates that all longitudes at ±90° latitude cluster together
- Prevents future regressions

---

## Testing Results

**All Tests:** 76 passing (70 unit + 6 integration)
- Previous: 75 tests
- New: 1 regression test for pole wrapping
- Status: ✅ All passing

**Manual Verification:**

```bash
# Test the problematic case (G→C with E graticule, par 30°)
./target/release/map2fig \
  -f cosmoglobe_DIRBE_06_I_n00512_DR2.fits \
  --input-coord G --output-coord C \
  --grat-coord E --graticule --grat-par 30 \
  --cmap binary --out test_pole_crossing.png

# Result: PNG with smooth, clean graticule lines ✅
# No pole wrapping artifacts ✅
```

---

## Visual Improvements

| Aspect | Before | After |
|--------|--------|-------|
| **Line quality** | Pixelated/jagged | Smooth/anti-aliased |
| **Consistency** | Graticule pixelated, border smooth | Both smooth (Cairo) |
| **Pole behavior** | Wrapping at ±90° | Clean without artifacts |
| **File size** | ~1.5 MB | ~1.5 MB (quality, not size) |

---

## Design Benefits

1. **Unified Rendering:** Border and graticule use identical Cairo pipeline
2. **Publication Quality:** Anti-aliased lines suitable for printing
3. **Robustness:** Pole detection prevents numerical edge cases
4. **Testability:** Regression test guards against future pole issues
5. **Maintainability:** Clearer code path, fewer special cases

---

## Status

✅ **All issues resolved:**
1. PNG graticule now smooth and anti-aliased
2. Pole wrapping issue eliminated

✅ **All tests passing:** 76/76 (including new regression test)

✅ **Ready for production use** with publication-quality output

The graticule rendering is now production-ready with both visual quality and numerical robustness at all latitudes including the poles.
