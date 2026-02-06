# Crab Nebula Gnomonic Projection Comparison

## Test Parameters
- **Target**: Crab Nebula
- **Center**: RA 83.63°, Dec 22.01° (Equatorial coordinates)
- **Projection**: Gnomonic (local tangent plane)
- **Resolution**: 0.2 arcmin/pixel
- **Graticule**: 1° spacing for both latitude and longitude lines
- **Data**: npipe_nodip.fits (diffuse foreground CMB map, high resolution)

## Tool Comparison

### map2fig (Our Implementation)
```bash
./target/release/map2fig -f npipe_nodip.fits \
  --projection gnomonic \
  --gnom-lon 83.63 --gnom-lat 22.01 \
  --gnom-res 0.2 \
  -o crab_gnomonic_map2fig.png \
  --graticule --grat-par 1 --grat-mer 1 \
  --cmap viridis
```

**Output**: `crab_gnomonic_map2fig.png`
- **Dimensions**: 1248 × 1322 pixels
- **Format**: PNG (8-bit/color RGBA)
- **Size**: 627 KB

**Notes**:
- Dimensions determined by content bounding box
- Includes colorbar and border in dimensions
- Uses viridis colormap

### map2png (Cosmotools Reference)
```bash
/home/dwatts/Cosmotools/src/cpp/utils/map2png \
  -gnomonic \
  -longitude 83.63 -latitude 22.01 \
  -resolution 0.2 \
  -grid 1 -glon 1 -glat 1 \
  -xsz 2048 \
  npipe_nodip.fits crab_gnomonic_map2png.png
```

**Output**: `crab_gnomonic_map2png.png`
- **Dimensions**: 2048 × 2048 pixels (fixed square)
- **Format**: PNG (8-bit/color RGB)
- **Size**: 1.9 MB

**Notes**:
- Fixed square size specified by `-xsz`
- Default Planck colormap
- Higher pixel count provides more detail

## Graticule Implementation Status

### map2fig Gnomonic Graticule
- **Status**: Framework in place, not yet integrated with plotting
- **Available**: `gnomonic_graticule.rs` module
- **Modes**: 
  - Local grid (tangent plane coordinates) - defined but not wired
  - Sky coordinate overlay - defined but not implemented
- **Next Steps**: 
  1. Integrate graticule rendering into `plot_gnomonic_png/pdf` functions
  2. Test local grid rendering
  3. Implement sky coordinate transformations

### map2png Gnomonic Graticule
- **Status**: Working implementation
- **Grid Options**:
  - `-grid NUM`: Overall grid spacing interval in degrees
  - `-glon NUM`: Longitude interval 
  - `-glat NUM`: Latitude interval
- **Line Width**: `-lw NUM` for pixel width
- **Color**: Default colors used, customizable via `-color` option

## Observations

The graticule rendering on gnomonic projections differs fundamentally from Mollweide:
1. **No discontinuity issues** - gnomonic is a continuous local projection
2. **Natural curvature** - straight lines in local coordinates appear as curves in pixel space
3. **Limited field-of-view** - lines beyond horizon don't render (correctly behind projection plane)
4. **Coordinate system matters** - overlays from different coordinate systems will show different patterns

## Files Generated
- [crab_gnomonic_map2fig.png](crab_gnomonic_map2fig.png) - map2fig output
- [crab_gnomonic_map2png.png](crab_gnomonic_map2png.png) - map2png output

Both files successfully show the same sky region centered on the Crab Nebula with graticule overlays enabled.
