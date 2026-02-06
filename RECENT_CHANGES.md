# Recent Changes - Gnomonic Projection Improvements

## Issues Fixed

### 1. **Default Gnomonic Field of View Too Small**
**Problem**: When using gnomonic projection without specifying `--fov` and `--res`, the output map was tiny (~60×60 pixels).

**Solution**: Increased default `--fov` from 60 arcmin to 300 arcmin (1 degree to 5 degrees).
- Old default: 60×60 pixel map with 60 arcmin field of view
- New default: 300×300 pixel map with 300 arcmin field of view
- Much more usable without requiring manual parameter tuning

**Updated Command**: Now this works well without extra parameters:
```bash
./map2fig -f data.fits --projection gnomonic --lon 0 --lat 0 -o map.png
# Produces a reasonable 300×300 pixel map instead of tiny 60×60
```

### 2. **Graticule Overlay Rendering Incorrect on Gnomonic**
**Problem**: When using `--local-graticule` with `--grat-coord-overlay`, the local graticule was being rendered in the overlay color instead of black, and no actual overlay was shown.

**Root Cause**: Coordinate system transformation for gnomonic overlays is not implemented.

**Solution**: 
- Reverted to always rendering local graticule in black
- Added clear warning message when user attempts overlay: 
  ```
  Warning: Coordinate system overlay (--grat-coord-overlay) is not yet implemented for gnomonic projections.
  The local graticule is shown, but the overlay graticule is not rendered.
  For dual-coordinate visualization, use --projection mollweide instead.
  ```
- Updated documentation to clarify this limitation

**Current Behavior**:
```bash
# This now renders only the local graticule (in black), with a warning
./map2fig -f data.fits --projection gnomonic --local-graticule \
  --grat-coord-overlay eq --grat-overlay-color "#FF6B6B" -o map.png
```

## Technical Details

### Changes Made

1. **src/cli.rs**
   - Changed `fov` default from `60.0` to `300.0` arcmin
   - Updated default validation check

2. **src/plot.rs** (two locations)
   - Removed conditional colored graticule rendering
   - Always render local grid in black
   - Added warning when overlay is requested
   - Suppressed unused variable warnings

3. **src/gnomonic_graticule.rs**
   - Kept `render_gnomonic_local_grid_colored()` for future use
   - Function is available but not currently used in main plotting pipeline

4. **README.md**
   - Updated Example 3 (Gnomonic projection) with detailed resolution info
   - Clarified that Example 4 (overlay) is not yet implemented on gnomonic
   - Added troubleshooting section for both issues
   - Provided zoom level examples

## Future Work

**Gnomonic Overlay Rendering** (TODO):
Implement coordinate system transformation from overlay coordinates to the gnomonic tangent plane:
1. Generate graticule lines in overlay coordinate system
2. Transform points via `ViewTransform` to gnomonic view frame
3. Project points using gnomonic projection formula
4. Render transformed lines in specified color

This would allow:
```bash
# Future: Show Equatorial grid overlay on Galactic local grid
./map2fig -f data.fits --projection gnomonic \
  --local-graticule --grat-coord-overlay eq --grat-overlay-color "#FF6B6B" \
  -o dual_coords.png
```

## Testing

Test commands to verify the fixes:

```bash
# Test 1: Default FOV is now reasonable
./target/release/map2fig -f npipe_nodip.fits \
  --projection gnomonic --lon 0 --lat 0 \
  --local-graticule -o test_default_fov.png

# Test 2: Overlay warning is shown
./target/release/map2fig -f npipe_nodip.fits \
  --projection gnomonic --lon 0 --lat 0 \
  --local-graticule --grat-coord-overlay eq \
  -o test_overlay_warning.png 2>&1 | grep Warning

# Test 3: Still works with manual FOV
./target/release/map2fig -f npipe_nodip.fits \
  --projection gnomonic --lon 0 --lat 0 \
  --fov 100 --res 0.5 \
  --local-graticule -o test_manual_fov.png
```

## Impact Summary

- ✅ Gnomonic projections are now much more usable by default
- ✅ Better user experience without needing to figure out FOV/resolution
- ✅ Clear warning instead of silent incorrect behavior
- ✅ Documented limitations and workarounds
- ✅ No breaking changes to API (just better defaults)
