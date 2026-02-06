# Gnomonic Graticule Design

## Overview
Graticule support for gnomonic projections requires two main rendering modes:

### Mode 1: Local Grid (Field-of-View Grid)
- Renders constant latitude/longitude lines in the **local tangent plane** coordinate system
- Lines are defined relative to the center point being observed
- Allows viewing the local coordinate grid with natural curvature as FOV increases
- Similar to healpy's `graticule()` for gnomonic projections
- **Input**: Local tangent-plane coordinates (lon/lat relative to center)
- **Behavior**: Straight lines near center, curves at edges for large FOV

### Mode 2: Sky Coordinate Overlay
- Renders constant latitude/longitude lines in a **specific celestial coordinate system** (Galactic, Equatorial, Ecliptic)
- Shows where celestial coordinates appear in the image
- Similar to astropy's grid overlay (WCSAxes)
- **Input**: Graticule coordinate system (G, C, E) that may differ from input map coordinates
- **Behavior**: Curves naturally through the gnomonic projection

## Implementation Strategy

### Architecture

```
GnomonicGraticule {
    mode: GraticuleMode
    spacing: f64  // in degrees
}

enum GraticuleMode {
    LocalGrid {
        // Lines at constant lat/lon in tangent plane coordinates
        dlon: f64,  // meridian spacing in degrees
        dlat: f64,  // parallel spacing in degrees
    },
    SkyCoordinate {
        // Lines in specified coordinate system
        coord_system: CoordSystem,
        dlon: f64,
        dlat: f64,
    },
}
```

### Rendering Pipeline

For **Local Grid**:
1. Generate constant lon/lat lines in tangent plane frame (±0.5 to ±0.5 in normalized coords)
2. Sample each line at fine resolution
3. For each sample point, use gnomonic **forward** projection: (local_lon, local_lat) → (pixel_x, pixel_y)
4. Draw line segments in image space

For **Sky Coordinate Overlay**:
1. Generate constant lon/lat lines in specified coordinate system (G/C/E)
2. Transform to map's input coordinate system
3. Transform to tangent plane local coordinates via ViewTransform
4. Use gnomonic **forward** projection to get pixel coordinates
5. Draw line segments in image space
6. (Optionally add tick labels with actual coordinate values)

### Key Differences from Mollweide

- **No discontinuity detection** needed for gnomonic (local projection, continuous)
- **Forward projection** is primary (pixel_to_ang in mollweide was inverse)
- Must handle coordinate system transformations for sky overlays
- Line curvature is implicit in the projection, not from coordinate wrapping

## Implementation Tasks

1. **GnomonicGraticule struct and enums** - Define mode and parameters
2. **Local grid renderer** - Draw tangent-plane aligned grid
3. **Sky coordinate renderer** - Draw overlays in different coordinate systems
4. **Integration with plot_gnomonic functions** - Add graticule to PNG and PDF output
5. **CLI arguments** - Add `--grat-local`, `--grat-sky-coord`, `--grat-spacing` to options

## Testing Strategy

- Unit tests for gnomonic forward projection with graticule points
- Visual tests with various coordinate systems and spacings
- Edge case tests: high-latitude centers, large FOV, small FOV
- Comparison with healpy (local mode) and astropy (sky coordinate mode)
