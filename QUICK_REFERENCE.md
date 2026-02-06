# Quick Reference: Gnomonic Projection After Fixes

## TL;DR - What Changed

1. **Maps are now bigger by default** (300×300 instead of 60×60)
2. **Overlay warning is clearer** (tells you to use Mollweide instead)
3. **Documentation explains everything** (see README.md)

## Common Commands

### Basic Gnomonic View
```bash
./map2fig -f data.fits --projection gnomonic -o map.png
# Now produces reasonable ~348×386 pixel map
```

### Zoomed on Region
```bash
# 5° zoom (default)
./map2fig -f data.fits --projection gnomonic --lon 266 --lat -29 -o map.png

# More zoomed (0.5°)
./map2fig -f data.fits --projection gnomonic --lon 266 --lat -29 --fov 30 -o zoomed.png

# Less zoomed (10°)
./map2fig -f data.fits --projection gnomonic --lon 266 --lat -29 --fov 600 -o wide.png
```

### With Graticule
```bash
./map2fig -f data.fits --projection gnomonic \
  --local-graticule \
  --local-grat-dlat 2 --local-grat-dlon 2 \
  -o map_with_grid.png
```

### With Scaling
```bash
./map2fig -f data.fits --projection gnomonic \
  --log --min 0.001 --max 0.5 \
  -o map_log_scale.png
```

### For Coordinate Overlay
```bash
# Use Mollweide instead (overlays not yet implemented on gnomonic)
./map2fig -f data.fits --projection mollweide \
  --graticule --grat-coord-overlay eq \
  --grat-overlay-color "#FF6B6B" \
  -o mollweide_overlay.png
```

## FOV/Resolution Guide

**Field of View (--fov)** is in arcminutes (60 arcmin = 1 degree)
**Resolution (--res)** is in arcmin/pixel

| Use Case | --fov | --res | Result |
|----------|-------|-------|--------|
| Ultra-zoomed (nebula) | 30 | 0.1 | ~300×300 px, very detailed |
| Super-zoomed (cluster) | 60 | 0.5 | ~120×120 px |
| Heavily zoomed | 120 | 0.5 | ~240×240 px |
| **Default** zoom | **300** | **1** | **~300×300 px** ✨ |
| Wide view | 600 | 1 | ~600×600 px |
| Full supergalactic | 1200 | 1 | ~1200×1200 px |

## Default Behavior

### Before Fix ❌
```bash
./map2fig -f data.fits --projection gnomonic -o map.png
# Result: 60×60 pixel map (tiny, barely visible)
# Required: --fov 300 --res 0.1 to get reasonable size
```

### After Fix ✅
```bash
./map2fig -f data.fits --projection gnomonic -o map.png
# Result: 348×386 pixel map (visible, good quality)
# Optional: --fov and --res only if you want different zoom
```

## Known Limitations

**Coordinate overlays on gnomonic**: Not yet implemented  
**Workaround**: Use `--projection mollweide` to show multiple coordinate systems

```bash
# This will show warning:
./map2fig -f data.fits --projection gnomonic \
  --grat-coord-overlay eq -o map.png
# Output: "Warning: Coordinate system overlay is not yet implemented..."

# Use this instead:
./map2fig -f data.fits --projection mollweide \
  --grat-coord-overlay eq -o map.png
# Output: Both Galactic and Equatorial grids shown
```

## Documentation

- **README.md**: Full usage guide with 10 examples
- **RESOLUTION_SUMMARY.md**: This issue's resolution in detail
- **FIXES_SUMMARY.md**: Before/after comparison
- **RECENT_CHANGES.md**: Technical implementation details

## Questions?

See the relevant documentation file above, or check `./map2fig --help` for all options.
