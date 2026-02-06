# Issue Resolution Summary

## Issues Identified and Fixed

### Issue 1: Gnomonic Maps Too Small by Default
**Status**: ✅ **FIXED**

**Problem**: When using gnomonic projection without explicit `--fov` and `--res` parameters, output maps were extremely small (~60×60 pixels), making them barely visible.

**Root Cause**: Default field of view was set to 60 arcmin (1 degree), which combined with 1 arcmin/pixel resolution produced a tiny map area.

**Solution**: Increased default `--fov` from 60 to 300 arcmin (5 degrees), resulting in a 300×300 pixel map that's actually usable without parameter tuning.

**File Changed**: `src/cli.rs` line ~133
```rust
// Before:
#[arg(long, alias = "gnom-width", default_value_t = 60.0)]
pub fov: f64,

// After:
#[arg(long, alias = "gnom-width", default_value_t = 300.0)]
pub fov: f64,
```

**Result**: Users now get reasonable output immediately:
```bash
$ ./map2fig -f data.fits --projection gnomonic -o map.png
# Output: 348×386 pixels (visible and usable)
# Before: would have been ~60×60 (barely visible)
```

---

### Issue 2: Graticule Overlay Rendering Incorrectly
**Status**: ✅ **FIXED** (with clear documentation of limitation)

**Problem**: When using `--local-graticule` with `--grat-coord-overlay`, the local graticule was being rendered in the overlay color instead of black, and no actual overlay was shown.

**Root Cause**: The coordinate system transformation for overlays on gnomonic projections is complex and not yet implemented. The code was attempting to use a colored version of the local graticule when an overlay was requested.

**Solution**: 
1. Reverted to always rendering local graticule in black
2. Added clear warning message when users attempt overlay
3. Updated documentation to clarify the limitation
4. Suggested using Mollweide projection for coordinate overlays

**Files Changed**: `src/plot.rs` (two locations - PNG and PDF rendering)

**New Behavior**:
```rust
// Always render local grid in black
render_gnomonic_local_grid(&mut grid, &proj, grat_dlon, grat_dlat);

// Clear warning if overlay requested
if grat_overlay.is_some() {
    eprintln!("Warning: Coordinate system overlay (--grat-coord-overlay) is not yet implemented for gnomonic projections.");
    eprintln!("The local graticule is shown, but the overlay graticule is not rendered.");
    eprintln!("For dual-coordinate visualization, use --projection mollweide instead.");
}
```

**Result**: Users get clear feedback instead of silent incorrect behavior:
```bash
$ ./map2fig -f data.fits --projection gnomonic \
  --local-graticule --grat-coord-overlay eq -o map.png

Warning: Coordinate system overlay (--grat-coord-overlay) is not yet implemented for gnomonic projections.
The local graticule is shown, but the overlay graticule is not rendered.
For dual-coordinate visualization, use --projection mollweide instead.
```

---

## Documentation Updates

### Updated README.md
- **Example 3** (Gnomonic projection): Added detailed resolution information with examples of different zoom levels
- **Example 4** (Overlay): Clarified that overlays are not yet supported on gnomonic; suggested using Mollweide
- **Troubleshooting**: Added sections for:
  - Gnomonic maps being too small (explains FOV/resolution relationship)
  - Graticule overlay not showing (explains limitation and workaround)
  - Other common issues

### New Documentation Files
- **RECENT_CHANGES.md**: Detailed technical explanation of changes and future work
- **FIXES_SUMMARY.md**: Before/after comparison with command examples

---

## Testing & Verification

### Verified Working:
✅ Default gnomonic maps now produce 348×386 pixel output  
✅ Local graticule renders in black (correct color)  
✅ Clear warning shown when overlay attempted  
✅ Build succeeds with zero warnings  
✅ All command-line options still work  

### Test Commands:
```bash
# Test 1: Simple gnomonic with defaults - should be ~348×386
./map2fig -f npipe_nodip.fits --projection gnomonic -o simple.png

# Test 2: With graticule - should show grid in black
./map2fig -f npipe_nodip.fits --projection gnomonic \
  --local-graticule -o with_grat.png

# Test 3: Overlay warning - should show warning message
./map2fig -f npipe_nodip.fits --projection gnomonic \
  --local-graticule --grat-coord-overlay eq -o overlay_test.png 2>&1

# Test 4: Mollweide overlay - should work correctly
./map2fig -f npipe_nodip.fits --projection mollweide \
  --graticule --grat-coord-overlay eq -o mollweide_overlay.png
```

---

## Impact Summary

### User Experience Improvements
- ✅ Gnomonic projections work well with default parameters
- ✅ No more "tiny map" surprise when FOV/resolution aren't specified
- ✅ Clear error messages guide users to solutions
- ✅ Documentation clearly explains limitations and workarounds

### Code Quality
- ✅ No compiler warnings
- ✅ Clean, maintainable code
- ✅ Functions preserved for future enhancement (e.g., `render_gnomonic_local_grid_colored`)
- ✅ Comprehensive documentation for future developers

### Backward Compatibility
- ✅ No breaking changes to API
- ✅ Users can still override defaults with explicit parameters
- ✅ Existing Mollweide functionality unchanged
- ✅ All examples still work

---

## Future Work (Not Blocking)

**Gnomonic Coordinate System Overlays** (TODO):
When implemented, would allow showing multiple coordinate systems on gnomonic projection. This requires:
1. Transforming overlay coordinate system points via ViewTransform
2. Projecting transformed points using gnomonic projection formula
3. Rendering lines in specified color

This is documented in code as TODO for future implementation.

---

## Files Modified Summary

| File | Changes | Lines |
|------|---------|-------|
| src/cli.rs | Default FOV: 60 → 300 | 1 main + 1 validation |
| src/plot.rs | Graticule rendering logic, warnings | 2 locations updated |
| README.md | Examples updated, troubleshooting expanded | ~50 lines |
| RECENT_CHANGES.md | NEW: Technical documentation | 100+ lines |
| FIXES_SUMMARY.md | NEW: Before/after examples | 100+ lines |

**Total**: 4 files modified, 2 files created, ~0 breaking changes
