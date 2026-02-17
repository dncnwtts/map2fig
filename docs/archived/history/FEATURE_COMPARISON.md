# HEALPix Visualization Tool Feature Comparison

## Executive Summary

**map2fig (Rust)** is a modern, high-performance implementation with excellent scaling options and Mollweide/Gnomonic support. It excels in **mathematical transforms** (asinh, symlog) but lacks some higher-level convenience features found in Cosmoglobe/healpy.

### Speed Comparison (Subjective)
1. **map2png (C++)**: Fast (~seconds) - Basic features, minimal visual polish
2. **map2fig (Rust)**: Fast (~seconds) - Better visual output, modern architecture
3. **healpy (Python)**: Medium (~5-10s) - Good features, matplotlib foundation
4. **Cosmoglobe (Python)**: Slow (~20-60s) - Most features, polished output, publication-quality

---

## Feature Matrix

### PROJECTIONS

| Feature | map2fig | map2png | Cosmoglobe | healpy |
|---------|---------|---------|------------|--------|
| **Mollweide** | ✓ | ✓ | ✓ | ✓ |
| **Gnomonic** | ✓ | ✓ | ✓ | ✓ |

**Gap Analysis**:
- map2fig focus is on efficient all-sky and local projections with publication-quality output

---

### DATA SCALING/TRANSFORMATIONS

| Feature | map2fig | map2png | Cosmoglobe | healpy |
|---------|---------|---------|------------|--------|
| **Linear** | ✓ | ✓ | ✓ | ✓ |
| **Log** | ✓ | ✓ | ✓ | ✓ |
| **Symlog** | ✓ | ✗ | ✓ | ✓ |
| **Asinh** | ✓ | ✗ | ✗ | ✗ |
| **Power Law** | ✓ | ✓ | ✓ | ✓ |
| **Histogram EQ** | ✓ | ✓ | ✓ | ✓ |

**Advantage map2fig**: map2fig has **asinh scaling** which others lack - excellent for data with dynamic range spanning negative to positive values

---

### MAP MANIPULATION FEATURES

| Feature | map2fig | map2png | Cosmoglobe | healpy |
|---------|---------|---------|------------|--------|
| **Smoothing** | ✓ | ✓ | ✓ | ✓ |
| **Rotation/Roll** | ✓ | ✓ | ✓ | ✓ |
| **Dipole Removal** | ✗ | ✗ | ✓ | ✓ |
| **Monopole Removal** | ✗ | ✗ | ✓ | ✓ |
| **Ud_grade/Resize** | ✗ | ✗ | ✓ | ✓ |

**Gaps in map2fig**:
- **Dipole/Monopole removal**: Statistically remove mean (monopole) and dipole components
- **Ud_grade**: Resolution downsampling/upsampling
- These require statistical analysis on HEALPix data before rendering

---

### RENDERING & OUTPUT

| Feature | map2fig | map2png | Cosmoglobe | healpy |
|---------|---------|---------|------------|--------|
| **PNG Output** | ✓ | ✓ | ✓ | ✓ |
| **PDF Output** | ✓ | ✗ | ✓ | ✓ |
| **Graticule/Grid** | ✓ | ✗ | ✓ | ✓ |
| **Colorbar** | ✓ | ✓ | ✓ | ✓ |
| **LaTeX Support** | ✓ | ✓ | ✓ | ✓ |
| **High-Resolution** | ✗ | ✗ | ✓ | ✗ |

**map2fig Advantages**: PDF output and graticule support

**Gaps**: 
- **High-resolution export** (for publication): Cosmoglobe specifically optimizes for large output sizes
- May require custom tiling/stitching for very large output images

---

### COORDINATE SYSTEMS & CONVENTIONS

| Feature | map2fig | map2png | Cosmoglobe | healpy |
|---------|---------|---------|------------|--------|
| **Galactic** | ✓ | ✓ | ✓ | ✓ |
| **Equatorial (J2000)** | ✓ | ✓ | ✓ | ✓ |
| **Ecliptic** | ✓* | ? | ✓ | ✓ |
| **Custom Rotations** | ✓ | ✓ | ✓ | ✓ |

*map2fig has rot/rotation but may need verification for ecliptic

---

## Recommended Missing Features (Priority Order)

### HIGH PRIORITY
1. **Dipole/Monopole removal** - Common preprocessing step in CMB analysis
2. **Orthographic projection** - Useful for 3D globe visualization
3. **Hammer projection** - Better for full-sky equal-area maps

### MEDIUM PRIORITY
4. **Ud_grade** - Convenient for downsampling maps before plotting
5. **High-resolution tiling** - For publication-quality large images

### LOW PRIORITY
6. **Extended coordinate system support** - Ecliptic, custom frames
7. **More projections** - Stereographic, orthographic, etc.

---

## Code Architecture Comparison

### map2fig (Rust) - Strengths
- ✓ Modern, strongly-typed architecture
- ✓ Zero clippy warnings (excellent code quality)
- ✓ Deterministic PDF/PNG output
- ✓ Fast compilation and execution
- ✓ Advanced scaling options (asinh)

### map2fig (Rust) - Weaknesses
- ✗ Limited built-in data manipulation
- ✗ Must rely on external tools for preprocessing
- ✗ Smaller ecosystem than Python alternatives

### Cosmoglobe (Python) - Strengths
- ✓ Complete feature set
- ✓ Publication-ready quality
- ✓ Integrated data preprocessing
- ✓ Rich matplotlib customization

### Cosmoglobe (Python) - Weaknesses
- ✗ Very slow for large maps
- ✗ Heavy dependencies
- ✗ Less suitable for CLI/batch processing

### healpy (Python) - Strengths
- ✓ Standard HEALPix Python library
- ✓ Good balance of features and speed
- ✓ Well-documented and widely used
- ✓ Good coordinate system support

### healpy (Python) - Weaknesses
- ✗ Matplotlib-dependent (less suitable for high-quality output)
- ✗ No native PDF support
- ✗ Limited graticule customization

---

## Implementation Recommendations for map2fig

### Phase 1 (Easy wins)
1. **Add dipole/monopole removal** - Wrap HEALPix statistics functions
2. **Add Hammer projection** - Mathematical projection similar to Mollweide

### Phase 2 (Medium effort)
3. **Add ud_grade support** - HEALPix resolution resampling
4. **Add Orthographic projection** - Additional projection math

### Phase 3 (Polish)
5. **High-resolution tiling** - Render in sections and stitch
6. **Additional coordinate frames** - Ecliptic, custom rotations

---

## When to Use Each Tool

| Use Case | Recommended Tool | Reason |
|----------|------------------|--------|
| Fast batch processing | **map2fig** | Speed + quality balance |
| Publication-quality plots | **Cosmoglobe** | Best visual output |
| Scientific analysis + plotting | **healpy** | Good ecosystem |
| Simple, fast visualization | **map2png** | Minimal overhead |
| Integration in Rust projects | **map2fig** | Native Rust, no Python deps |

