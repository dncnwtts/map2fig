# HEALPy vs HEALPix Plotter: Graticule Implementation Comparison

## Executive Summary

Both implementations solve the same problem—rendering coordinate grids on HEALPix sky maps—but with fundamentally different architectural approaches. **HEALPy emphasizes interactive matplotlib integration**, while **HEALPix Plotter prioritizes high-quality publication-ready output** with explicit control over coordinate systems and rendering pipelines.

---

## 1. Architecture & Design Philosophy

### HEALPy (Python)

**Primary Design Goal:** Interactive matplotlib visualization framework for astronomers
- **Layer Stack:** 
  - User API (mollview, orthview, graticule functions)
  - matplotlib Axes subclasses (MollweideAxes, OrthographicAxes, etc.)
  - Projection classes (MollweideProj, OrthographicProj)
  - Rotator (coordinate transformation backend)

- **Integration Pattern:** Tightly coupled to matplotlib
  - Graticule is a matplotlib Artists collection (lines, text)
  - Uses matplotlib's projections/transforms system
  - Live editing of projection parameters possible

- **Key Classes:**
  - `SphericalProjAxes`: Base class for all projections, implements graticule via `graticule()` method
  - `CreateRotatedGraticule()`: Helper to generate rotated grid lines
  - Individual projection axes: `MollweideAxes`, `OrthographicAxes`, `GnomonicAxes`

### HEALPix Plotter (Rust)

**Primary Design Goal:** Scientific publication pipeline with explicit format control
- **Layer Stack:**
  - CLI interface (argument parsing)
  - Plot coordinator (scene composition)
  - Graticule module (coordinate transformation + rendering)
  - Pixel sinks (PNG, PDF/Cairo rasterization)

- **Integration Pattern:** Functional pipeline
  - Graticule is data-first: generates polylines or rasterizes to pixel buffer
  - No assumption about output format until final render stage
  - Supports both rasterized (PNG) and vectorized (PDF) outputs

- **Key Structures:**
  - `GraticuleTransform`: Encapsulates multi-step coordinate system conversions
  - `GraticulePolyline`: Represents continuous lines (for PDF)
  - `GraticuleLineSegments`: Collection of polylines
  - Separate rendering functions for rasterized vs vectorized output

---

## 2. Coordinate System Handling

### HEALPy Approach

**Single Rotation Matrix:**
```python
# projaxes.py: line 530-541
vec0 = R.Rotator(coord=self.proj.mkcoord(coord=coord)).I(vec)
# Creates rotation from target coord system to current
```

- Uses `rotator.py` Rotator class with string-based coordinate system names ('G', 'E', 'C')
- Automatic detection of map's native coordinate system
- Implicit coordinate transformation via matrix inversion

**Strengths:**
- ✅ Automatic, user-transparent
- ✅ Handles "map in Galactic, show Ecliptic graticule" automatically
- ✅ Integrates with user's rotation parameter

**Limitations:**
- ❌ Harder to debug ("why is this line here?")
- ❌ Limited visibility into transformation steps
- ❌ Single fixed approach for all cases

### HEALPix Plotter Approach

**Explicit, Chainable Transformations:**
```rust
// graticule.rs: GraticuleTransform
pub struct GraticuleTransform {
    source: CoordinateSystem,
    target: CoordinateSystem,
    rotation_matrices: Vec<[[f64; 3]; 3]>,
}

// User explicitly specifies: "show Ecliptic graticule on Galactic map"
```

- Explicit source/target coordinate systems
- Precomputed rotation matrices (GAL_TO_EQ, EQ_TO_ECL, etc.)
- Can chain multiple transformations if needed

**Strengths:**
- ✅ Crystal clear what coordinates are being shown
- ✅ Easy to add new coordinate system combinations
- ✅ Deterministic and testable

**Limitations:**
- ❌ Requires user to specify both source and target
- ❌ More parameters to configure via CLI

---

## 3. Graticule Line Generation

### HEALPy: Interval Optimization Algorithm

**Key Function:** `_get_interv_graticule()` (projaxes.py lines 641-663)

```python
# Adaptive interval selection based on plot bounds
max_n_par = 18  # Maximum 18 parallels
max_n_mer = 36  # Maximum 36 meridians

if n_par > max_n_par:
    # Reduce spacing if too many lines
    dpar = set_prec((pmax - pmin) / dtor, max_n_par / 2) * dtor

# Ensures ratio between dmer/dpar stays in [0.2, 5.0]
if dmer / dpar < 0.2 or dmer / dpar > 5.0:
    dmer = dpar = max(dmer, dpar)
```

**Strategy:**
- User provides desired spacing (dpar, dmer) in degrees
- Algorithm checks if it would create too many lines
- Adjusts spacing to maintain 18-36 line density
- Ensures meridian/parallel spacing doesn't deviate too much

**Strengths:**
- ✅ Automatically prevents visual clutter
- ✅ Works well for interactive exploration
- ✅ Handles edge cases (very small or large intervals)

**Limitations:**
- ❌ Can change requested spacing without telling user
- ❌ Loss of control for publication-quality plots
- ❌ Hides what's actually being rendered

### HEALPix Plotter: Deterministic Generation with Guarantee

**Key Function:** `generate_graticule_degrees()` (graticule.rs lines 8-42)

```rust
// Strategy: Always include cardinal directions
// cardinal_lines: [0, ±90, 180]
// Then fill spacing evenly while respecting max_spacing

let mut lines = Vec::new();
lines.extend(&[0.0, 90.0, -90.0, 180.0]);  // Always include these

// Then add evenly-spaced lines up to max_spacing
let num_steps = (180.0 / max_spacing).ceil() as usize;
let step = 180.0 / num_steps as f64;

// Result: ["Clean" spacings like 30°, 45°, 60° 
//          PLUS guaranteed cardinal lines]
```

**Strategy:**
- Always include equator (0°) and meridian (0°)
- Always include poles (±90°) and antimeridian (180°)
- Fill remaining space with evenly-spaced lines
- User specifies maximum spacing, system guarantees cardinal lines

**Strengths:**
- ✅ Deterministic: exactly what you ask for
- ✅ Guaranteed key lines visible
- ✅ Transparent about what's being rendered (diagnostic tests show exact lines)

**Limitations:**
- ❌ Spacing may be uneven ("clean" vs "unclean")
- ❌ More lines than requested in some cases

---

## 4. Line Rendering Pipeline

### HEALPy: Matplotlib Artist System

**Data Flow:**
1. Generate line endpoints in spherical coordinates (theta, phi)
2. Transform via Rotator to target coordinate system
3. Project to 2D via projection class (vec2xy)
4. Create matplotlib Line2D artists
5. Add to axes

**Key Code** (projaxes.py lines 591-609):
```python
# For each parallel (constant latitude)
for t in theta_list:
    gratlines.append(
        self.projplot(
            phi * 0.0 + t, phi, fmt,
            coord=coord, direct=local, **kwds
        )
    )

# For each meridian (constant longitude)
for p in phi_list:
    gratlines.append(
        self.projplot(
            theta, theta * 0.0 + p, fmt,
            coord=coord, direct=local, **kwds
        )
    )
```

**Handling Discontinuities:**
- Not explicitly handled in legacy graticule
- `newvisufunc.py` attempts masking for rotated graticules:
```python
# Mask regions where lines jump too much
mask = np.where((np.abs(np.diff(g_lines[0]))) > np.deg2rad(45))
g_lines[0] = np.ma.array(g_lines[0])
g_lines[0][mask] = np.ma.masked
```

**Output:** Live, interactive matplotlib visualization

### HEALPix Plotter: Dual-Mode Rendering

**Two Rendering Pathways:**

#### A. Rasterized (PNG)
```rust
// render_graticule_mollweide() — lines 164-304
// 1. Sample graticule lines at high density
// 2. For each sample point:
//    - Project to 2D screen coordinates
//    - Detect discontinuities (Δu > 0.3 or Δv > 0.3)
//    - Break line if jump detected
// 3. Draw broken segments directly to pixel buffer
// 4. Result: PNG with crisp grid overlay
```

**Discontinuity Detection** (graticule.rs lines 226-247):
```rust
// Break lines at projection edges
const DISCONTINUITY_THRESHOLD: f64 = 0.3;

let du = (screen_x - prev_x).abs();
let dv = (screen_y - prev_y).abs();

if du > DISCONTINUITY_THRESHOLD || dv > DISCONTINUITY_THRESHOLD {
    // Large jump: line wraps or projection boundary
    // Break into separate segments
}
```

#### B. Vectorized (PDF via Cairo)
```rust
// render_graticule_cairo() — lines 464-501
// 1. Generate polylines via render_graticule_mollweide_vectorized()
// 2. Returns Vec<GraticulePolyline> (exact coordinate paths)
// 3. For each polyline:
//    - Create Cairo path
//    - Stroke with specified style
//    - Add to PDF surface
// 4. Result: PDF with infinitely-scalable vector grid
```

**Key Advantage:** Same coordinate transformation logic, different output:
- PNG: Raster (bitmap)
- PDF: Vector (scalable)

---

## 5. Numerical Precision & Edge Cases

### HEALPy Observations

**Longitude Normalization:**
```python
# projector.py handles various projections
# Uses numpy's arctan2 for proper [-π, π] range
# Generally correct but details vary by projection class
```

**No Explicit Testing:**
- Test files exist (`test_graticule_rotation.py`) but focus on regression, not correctness
- Some tests check for exceptions, not output accuracy

### HEALPix Plotter Approach

**Explicit Longitude Normalization** (graticule.rs lines ~98-110):
```rust
fn vec_to_lonlat(v: [f64; 3]) -> (f64, f64) {
    let lat = v[2].asin();  // [-π/2, π/2]
    let lon = v[1].atan2(v[0]);  // [-π, π] from atan2
    (lon, lat)
}
```

**Comprehensive Diagnostics:**
- `debug_graticule_lines_original_coords()`: Show exact degree values in native system
- `debug_graticule_lines_ecliptic_on_galactic()`: Show coordinate transformation results
- Output: Exact Galactic coordinates for each sample point on each Ecliptic grid line

**Benefits:**
- ✅ Can verify correctness: "Ecl.Lon 0° should map to ~(96°, -60°) in Galactic"
- ✅ Rapid debugging of transformation issues
- ✅ Confidence in numerical accuracy

---

## 6. Performance Characteristics

### HEALPy

**Advantages:**
- Matplotlib rendering is cached and fast for interactive use
- Efficient numpy operations for line generation
- Live panning/zooming possible

**Disadvantages:**
- Intermediate Python object creation (lists of Line2D artists)
- Full redraw on any change
- Memory overhead for matplotlib figure management

**Typical Use Case:** Interactive notebook exploration, real-time parameter adjustment

### HEALPix Plotter

**Advantages:**
- Single-pass rendering pipeline
- No intermediate data structures for display
- Memory efficient (direct pixel writing)
- Parallelizable (each graticule line independent)

**Disadvantages:**
- Rust compilation time (few seconds)
- No live interactivity (write→view→adjust→recompile cycle)
- CLI parameter adjustment less fluid

**Typical Use Case:** Publication-quality batch rendering with reproducible settings

---

## 7. Feature Comparison Table

| Feature | HEALPy | HEALPix Plotter |
|---------|--------|-----------------|
| **Coordinate Systems** | G, E, C (implicit auto-detection) | G, E, C (explicit specification) |
| **Line Spacing** | Adaptive (optimized for viewing) | Deterministic (user-specified) |
| **Guaranteed Key Lines** | No (depends on spacing) | Yes (equator, meridian always shown) |
| **Output Formats** | Interactive matplotlib | PNG (raster), PDF (vector) |
| **Discontinuity Handling** | Limited (some masking in newvisufunc) | Explicit threshold-based breaking |
| **Rotation Parameters** | Automatic from user rot | Explicit source/target coords |
| **Graticule Labeling** | Yes (via matplotlib) | Not yet implemented |
| **Configuration Method** | Function parameters | CLI arguments + function API |
| **Testing** | Regression tests | Unit tests + diagnostics |
| **Vectorized Output** | No (always raster) | Yes (native PDF support) |

---

## 8. Code Quality & Maintainability

### HEALPy

**Strengths:**
- Well-established codebase (decades of astronomy use)
- Extensive matplotlib ecosystem integration
- Large community for issue reporting

**Challenges:**
- Mixed Python/C code (complex build)
- Multiple projection implementations with duplication
- Some deprecated patterns still in use

**Example Duplication:** Each projection class re-implements graticule generation with projection-specific tweaks.

### HEALPix Plotter

**Strengths:**
- Monolithic Rust codebase (no FFI complexity)
- Strong type system prevents coordinate confusion
- Comprehensive diagnostics built in

**Challenges:**
- Smaller ecosystem (newer code)
- Rust learning curve for contributions
- Fewer pre-built astronomical libraries

**Code Organization:** Clean separation of concerns
- `graticule.rs`: Coordinate transformation + rendering logic
- `plot.rs`: Integration with plotting pipeline
- `cli.rs`: User interface
- `tests`: Diagnostic output for verification

---

## 9. Practical Example: "Ecliptic Grid on Galactic Map"

### HEALPy Approach
```python
import healpy as hp
import numpy as np

# Create map in Galactic coordinates
m = hp.read_map('galactic_map.fits')

# Display in Galactic
hp.mollview(m, coord='G')

# Add Ecliptic graticule (automatic transformation)
hp.graticule(dpar=30, dmer=30, coord='E')

# Result: Ecliptic grid automatically rotated to overlay on Galactic
```

**What's Happening:**
- HEALPy detects map is in Galactic ('G')
- User requests Ecliptic graticule ('E')
- Internally: G→E rotation applied
- Lines drawn in interactive pyplot figure

**User Benefit:** Simple, one-line addition
**Drawback:** Opacity about what's actually being rendered

### HEALPix Plotter Approach
```bash
# Command line
./map2fig \
  -f galactic_map.fits \
  -o output.pdf \
  --graticule \
  --grat-coord ecl \
  --grat-par 30 \
  --grat-mer 30
```

**What's Happening:**
- Parser identifies: map file, output format, graticule settings
- Reads FITS file metadata → confirms Galactic
- Graticule module:
  - Generates Ecliptic grid (12 meridians × 7 parallels at 30° spacing)
  - Transforms via E→G rotation matrix
  - Projects to Mollweide 2D
  - Detects discontinuities, breaks lines
- Renders to PDF with vectorized output
- Or PNG with rasterized output (same code path)

**User Benefit:** Explicit, reproducible, verifiable
**Trade-off:** More parameters to specify

### Verification via Diagnostics

```bash
# Show exactly which graticule lines are being used (E→G transform)
cargo test debug_graticule_lines_ecliptic_on_galactic -- --nocapture

# Output:
# Ecl.Lon 0.0°: G(-83.6°,-59.8°) → G(96.3°,-60.2°) → G(96.4°,-0.2°)
# Ecl.Lon 30.0°: G(-108.6°,-53.7°) → G(145.6°,-48.7°) → G(110.9°,3.1°)
# ... etc
```

This diagnostic output lets you verify: "Yes, the Ecliptic meridian at 0° is being rendered correctly in Galactic coordinates."

---

## 10. When to Use Which

### Use HEALPy When:

- 🎯 **Interactive exploration** of sky maps
- 📊 **Quick visualization** of astronomical data
- 🔧 **Rapid prototyping** with parameter changes
- 👥 **Collaborative analysis** in Jupyter notebooks
- 🎨 **Live tuning** of appearance settings

**Example:** Astrophysicist exploring CMB data in a notebook, trying different scalings and projections

### Use HEALPix Plotter When:

- 📄 **Publication-quality output** needed
- 🎯 **Batch processing** of many maps
- 🔬 **Reproducible results** required
- 📊 **Vector output** (scalable PDFs)
- ✅ **Verifiable coordinates** (diagnostic confirmation)
- ⚙️ **Deterministic pipeline** (no surprises)

**Example:** CMB researchers generating 50+ publication figures with identical settings and vectorized output

---

## 11. Summary: Design Trade-offs

| Aspect | HEALPy | HEALPix Plotter |
|--------|--------|-----------------|
| **Philosophy** | "Make it easy" | "Make it right" |
| **Coupling** | Tight (matplotlib) | Loose (functional) |
| **Transparency** | Implicit (automatic) | Explicit (user-controlled) |
| **Flexibility** | High (interactive) | Moderate (batch-focused) |
| **Reproducibility** | Depends on matplotlib version | Deterministic (Rust output) |
| **Verification** | Runtime testing | Unit tests + diagnostics |
| **Output** | Raster (pixels) | Raster + Vector (choice) |
| **Learning Curve** | Gentle (familiar matplotlib) | Steep (Rust + CLI) |

---

## 12. Recommendations for Future Development

### For HEALPix Plotter:
1. **Add graticule labels** (currently missing, HEALPy has this)
2. **Implement local graticules** (gnomonic projection mode)
3. **Support interactive mode** (via web server or GUI) for exploratory use
4. **Add more coordinate systems** (J2000, other ecliptic definitions)
5. **Performance optimization** if handling very large maps

### For HEALPy:
1. **Improve discontinuity handling** in legacy `graticule()` (currently has gaps at boundaries)
2. **Make interval adaptation explicit** (inform user when spacing is being modified)
3. **Add diagnostic output** for verification (similar to HEALPix Plotter approach)
4. **Vector output support** (currently matplotlib-only, PDF limited)
5. **Stricter type hints** for coordinate system specifications

---

## Conclusion

Both implementations solve the graticule rendering problem effectively but with different priorities:

- **HEALPy:** Optimized for **ease of use and interactivity**, trades some explicit control
- **HEALPix Plotter:** Optimized for **reproducibility and quality**, trades convenience for determinism

The ideal workflow might combine both: **Use HEALPy for exploration, HEALPix Plotter for publication.**
