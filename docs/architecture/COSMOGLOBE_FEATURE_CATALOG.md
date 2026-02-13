# Cosmoglobe vs map2fig Feature Comparison

## Feature Catalog

### ✅ Already Implemented in map2fig

#### Projections
- ✅ Mollweide (full-sky equal-area)
- ✅ Hammer (full-sky equal-area, slightly different)
- ✅ Gnomonic (planar/telescopic view)
- Cosmoglobe also supports: Aitoff, Lambert, Polar, 3D, Cartesian

#### Scaling/Normalization
- ✅ Linear
- ✅ Log
- ✅ Symlog (symmetric log)
- ✅ Histogram equalization
- ✅ Asinh (inverse hyperbolic sine)
- ✅ Planck log (custom)
- Cosmoglobe also supports: symlog2 variant

#### Colormaps
- ✅ 80+ built-in colormaps (matplotlib + custom)
- ✅ Custom colormap support
- Cosmoglobe supports: Planck, cmasher, plotly qualitative, matplotlib

#### Labels & Rendering
- ✅ Left label (llabel) with LaTeX support
- ✅ Right label (rlabel) with LaTeX support
- ✅ Title label
- ✅ Colorbar with customizable units (LaTeX)
- ✅ Graticule (coordinate grid overlay)
- ✅ Dual graticule (primary + secondary coordinate system overlay)
- ✅ Graticule labels
- ✅ Graticule customization (line width, spacing, color)

#### Data Operations
- ✅ Map smoothing (Gaussian FWHM)
- ✅ UD-grading (resolution changes)
- ✅ Coordinate system transformations (Galactic, Equatorial, Ecliptic)
- ✅ Map rotation (lon, lat, psi)
- ✅ Remove dipole
- ✅ Remove monopole
- ✅ Value scaling by factor

#### Output Formats
- ✅ PDF (via Cairo)
- ✅ PNG
- ✅ Transparent background
- ✅ Width/resolution control
- Cosmoglobe also supports: Automatic filename with dark mode suffix

#### UI/Layout
- ✅ Colorbar (enable/disable)
- ✅ Border (enable/disable)
- ✅ Figure borders and padding
- ✅ Tick direction (inward/outward)
- ✅ Font size control (ticks, units, labels)
- ✅ Dark mode styling

---

## ❌ Missing Features (Priority Order)

### High Priority
1. **Masking**
   - Mask by file (FITS binary mask)
   - Mask by value range (threshold-based)
   - `maskfill` (color for masked pixels)

2. **Extended Colorbar Control**
   - `extend` parameter (extend beyond min/max with arrows)
   - Named color schemes (sunburst, ice, etc.)
   - Colorbar orientation: horizontal vs vertical

### Medium Priority
3. **Input Flexibility**
   - Direct FITS file path input
   - Component selection from multi-component FITS
   - Frequency-based scaling
   - Sample selection from array

4. **Axis/Grid Customization**
   - X/Y axis labels (xlabel, ylabel)
   - Custom tick labels
   - Grid spacing control (separate lon/lat)
   - Tick label colors
   - Flip convention ('astro' vs 'geo')
   - Phi convention (counterclockwise, clockwise, symmetrical)

5. **Advanced Metadata**
   - Title as separate parameter
   - Per-component automatic formatting
   - Override plot properties dict
   - Return figure object option

### Lower Priority
6. **Quality of Life**
   - Darkmode toggle
   - Figure width from page fraction
   - Style presets
   - Data-only return (no rendering)
   - Return both figure and parameters

---

## Implementation Roadmap

### Phase 1: Core Missing Features (Highest Impact)
- [ ] **3D Mollweide visualization**

### Phase 2: Input/Output Enhancements
- [ ] FITS direct input
- [ ] Component selection

### Phase 3: Fine-Tuning & Polish
- [ ] Axis customization (labels, tick colors)
- [ ] Style presets
- [ ] Dark mode toggle

---

## Feature Comparison Matrix

| Feature | map2fig | Cosmoglobe | Priority |
|---------|---------|------------|----------|
| Mollweide | ✅ | ✅ | - |
| Hammer | ✅ | ✅ | - |
| Gnomonic | ✅ | ✅ | - |

| 3D | ❌ | ✅ | 🔵 Low |
| Masking | ✅ | ✅ | - |
| Masking | ❌ | ✅ | 🔴 High |
| Remove dipole | ✅ | ✅ | - |
| Remove monopole | ✅ | ✅ | - |
| Graticule | ✅ | ✅ | - |
| Dual graticule | ✅ | ✅ | - |
| Linear norm | ✅ | ✅ | - |
| Log norm | ✅ | ✅ | - |
| Symlog norm | ✅ | ✅ | - |
| Histogram | ✅ | ✅ | - |
| Custom tick labels | ❌ | ✅ | 🟠 Medium |
| Dark mode | ⚠️ | ✅ | 🔵 Low |
| Axis labels | ⚠️ | ✅ | 🟠 Medium |
| Per-pixel export | ❌ | ❌ | 🔵 Low |

---

## Notes

- **Cosmoglobe is Python wrapper** around healpy's `projview`
- **map2fig is native Rust** - higher performance, but more effort for new features
