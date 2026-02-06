# Summary of Fixes for Gnomonic Projection Issues

## Problem 1: Maps Too Small by Default ❌ → ✅ FIXED

### Before
```bash
$ ./map2fig -f npipe_nodip.fits --projection gnomonic --lon 0 --lat 0 -o map.png
# Output: tiny ~60×60 pixel map (barely visible)
# Users had to discover and use: --fov 300 --res 0.1
```

### After
```bash
$ ./map2fig -f npipe_nodip.fits --projection gnomonic --lon 0 --lat 0 -o map.png
# Output: reasonable ~348×386 pixel map (clearly visible)
# Default --fov changed from 60 to 300 arcmin
```

**Impact**: Users get usable output immediately without parameter trial-and-error.

---

## Problem 2: Overlay Not Rendering / Silent Failure ❌ → ✅ FIXED

### Before
```bash
$ ./map2fig -f data.fits --projection gnomonic \
  --local-graticule \
  --grat-coord-overlay eq \
  --grat-overlay-color "#FF6B6B" \
  -o map.png
# Local graticule was rendered in RED (the overlay color) instead of BLACK
# No overlay was actually shown
# User gets confusing result with no warning
```

### After
```bash
$ ./map2fig -f data.fits --projection gnomonic \
  --local-graticule \
  --grat-coord-overlay eq \
  --grat-overlay-color "#FF6B6B" \
  -o map.png

Warning: Coordinate system overlay (--grat-coord-overlay) is not yet implemented for gnomonic projections.
The local graticule is shown, but the overlay graticule is not rendered.
For dual-coordinate visualization, use --projection mollweide instead.
```

**Impact**: Clear feedback instead of silent incorrect behavior. Users know to use Mollweide for multi-coordinate visualization.

---

## Command Examples Comparison

### Zoomed View on Galactic Center

**Before** (had to manually tune):
```bash
./map2fig -f sky_map.fits \
  --projection gnomonic \
  --lon 266.5 --lat -28.9 \
  --res 0.5 \
  --fov 150 \
  --local-graticule
```

**After** (works with just defaults):
```bash
./map2fig -f sky_map.fits \
  --projection gnomonic \
  --lon 266.5 --lat -28.9 \
  --local-graticule
# If you want even more zoom:
# --fov 100 (for 1.7°) or --fov 30 (for 0.5°)
```

### Super Zoomed View

**Before**:
```bash
./map2fig -f crab.fits \
  --projection gnomonic \
  --lon 83.6 --lat 22.0 \
  --fov 30 --res 0.1 \
  --local-graticule --local-grat-dlat 0.25 --local-grat-dlon 0.25
```

**After** (same, but now you know the defaults are reasonable):
```bash
# Still use the same when you want high zoom
./map2fig -f crab.fits \
  --projection gnomonic \
  --lon 83.6 --lat 22.0 \
  --fov 30 --res 0.1 \
  --local-graticule --local-grat-dlat 0.25 --local-grat-dlon 0.25
```

---

## Technical Summary

### Code Changes
- **src/cli.rs**: Default `fov` changed from 60.0 → 300.0 arcmin
- **src/plot.rs**: Graticule rendering logic simplified, clear warning messages added
- **README.md**: Updated examples and troubleshooting section
- **Documentation**: New RECENT_CHANGES.md file with detailed technical notes

### What Works Now
✅ Gnomonic projections have reasonable defaults  
✅ Local graticule always renders in black  
✅ Clear warning when overlay is unavailable  
✅ Documented workarounds and alternatives  
✅ No warnings or errors in compilation  

### Known Limitations (Documented)
⏳ Coordinate overlays on gnomonic not yet implemented (use Mollweide instead)  
⏳ This is documented in README, error messages, and RECENT_CHANGES.md  

### Testing Commands
```bash
# Test 1: Simple gnomonic with defaults - should be reasonable size
./map2fig -f npipe_nodip.fits --projection gnomonic -o simple.png

# Test 2: Overlay warning - should show warning message
./map2fig -f npipe_nodip.fits --projection gnomonic \
  --local-graticule --grat-coord-overlay eq -o overlay_test.png 2>&1

# Test 3: Mollweide overlay works fine
./map2fig -f npipe_nodip.fits --projection mollweide \
  --graticule --grat-coord-overlay eq -o mollweide_overlay.png
```
