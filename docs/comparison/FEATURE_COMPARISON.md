# Feature Comparison: map2png vs map2fig

## Executive Summary

map2fig is **feature-rich superset** of map2png with significantly more advanced capabilities. While map2png offers a simpler CLI interface, map2fig provides extensive customization, multiple projections, sophisticated scaling options, coordinate system support, and advanced masking/labeling features.

## Quick Comparison Table

| Feature | map2png | map2fig | Notes |
|---------|---------|---------|-------|
| **Projections** | Mollweide, Gnomonic | Mollweide, Gnomonic, **Hammer** | Hammer is map2fig exclusive |
| **Scaling Methods** | Linear, Log, Histogram | Linear, Log, SymLog, **Asinh**, Histogram, **Planck-Log** | map2fig has more options |
| **Color Schemes** | 8 built-in | **80+ colormaps** | map2fig has comprehensive coverage |
| **Coordinate Systems** | None | **Galactic, Equatorial, Ecliptic** | map2fig only |
| **Graticule/Grid** | Basic grid | **Advanced with styling** | map2fig: labels, overlays, configurable spacing |
| **Colorbar** | Basic On/Off | **On/Off, Extend, Tick Style** | map2fig: arrows, inward/outward ticks |
| **Masking** | None | **Threshold, File-based, Coordinated** | map2fig only |
| **Text Labels** | None | **Custom title, corners, auto-scaled** | map2fig only |
| **Data Transform** | None | **Scaling factor for units** | map2fig only |
| **LaTeX Support** | None | **Yes, for labels/units** | map2fig only |
| **Transparency** | None | **Yes** | map2fig only |
| **Background Control** | Color only | **Color, transparency, precise control** | map2fig more flexible |
| **Font Control** | None | **Font sizes for ticks, units, labels** | map2fig only |
| **Output Formats** | PNG only | **PNG, PDF (2 backends)** | map2fig supports PDF |
| **Performance Mode** | None | **Fast-render mode** | map2fig only |
| **Command Syntax** | Simple flags | **Detailed long options** | map2png easier for quick use |

---

## Detailed Feature Breakdown

### 1. **Core Projections**

#### map2png
- `-mollweide` (default)
- `-gnomonic`

#### map2fig
- `--projection mollweide` (default)
- `--projection gnomonic`
- `--projection hammer` ✨ **NEW**

**Status:** map2fig superior (additional Hammer projection)

---

### 2. **Scaling/Transformation Methods**

#### map2png
```
-linear           (default)
-logarithmic
-histogram
```

#### map2fig
```
--log             (logarithmic)
--symlog          (symmetric log)
--hist            (histogram)
--asinh           (inverse hyperbolic sine) ✨ NEW
--planck-log      (Planck logarithmic) ✨ NEW
(linear is default)
```

**Status:** map2fig superior (5 vs 3 methods)
- SymLog: Better for data with sign changes
- AsinH: Smoother behavior for small values
- Planck-Log: CMB-specific optimization

---

### 3. **Color Schemes/Colormaps**

#### map2png
```
-color planck    (Planck)
-color wmap      (WMAP)
-color gray      (Grayscale)
-color viridis   (Viridis)
-color plasma    (Plasma)
-color magma     (Magma)
-color cividis   (Cividis)
-color inferno   (Inferno)
-color turbo     (Turbo)
-color afmhot    (AFM Hot)
```
**Total: ~10 colormaps**

#### map2fig
```
-c/--cmap <NAME>  (default: viridis)
Supports 80+ colormaps including:
  - All matplotlib colormaps
  - Planck-specific colormaps
  - Custom colormaps in extended database
```
**Total: 80+ colormaps**

**Status:** map2fig superior (80+ vs 10)

---

### 4. **Coordinate System Support** ⭐

#### map2png
- ❌ **Not supported**
- Fixed to implicit galactic coordinates

#### map2fig
- ✅ `--input-coord gal|eq|ecl` 
- ✅ `--output-coord gal|eq|ecl`
- ✅ `--rotate-to LON,LAT` (rotate view center)
- ✅ `--roll ANGLE` (rotation around center)

**Status:** map2fig only feature (major advantage for multi-survey work)

---

### 5. **Grid/Graticule**

#### map2png
```
-grid NUM         (lat/lon grid spacing in degrees)
-glat NUM         (latitude grid spacing)
-glon NUM         (longitude grid spacing)
-lw NUM           (grid line width)
-nogrid           (disable, default)
```
**Features:** Basic regular grid, no styling

#### map2fig
```
--graticule                 (enable primary grid)
--grat-coord gal|eq|ecl     (primary coordinate system)
--grat-coord-overlay ...    (secondary overlay)
--grat-overlay-color #RRGGBB (custom overlay color)
--grat-labels               (show lat/lon labels)
--grat-par DEG              (parallel spacing, default 15°)
--grat-mer DEG              (meridian spacing, default 15°)
--grat-line-width PIXELS    (line width, default 1)
--local-graticule           (for gnomonic)
--local-grat-dlat DEG
--local-grat-dlon DEG
```
**Features:** Advanced styling, coordinate overlays, labels, multiple reference systems

**Status:** map2fig superior (basic vs advanced)

---

### 6. **Colorbar Control**

#### map2png
```
-bar              (enable, default)
-nobar            (disable)
-ticks INT        (color bar sub-tick levels)
-digits INT       (precision of numeric labels)
```

#### map2fig
```
--no-cbar                   (disable colorbar)
--extend none|min|max|both  (arrow extensions)
--tick-direction inward|out (tick direction)
--tick-font-size POINTS     (auto-scaled default 12pt)
--units-font-size POINTS    (auto-scaled default 16pt)
```
**Status:** Different approaches - map2png focuses on tick precision, map2fig on visual styling

---

### 7. **Color Range/Scaling**

#### map2png
```
-minimum NUM
-maximum NUM
-range NUM        (symmetric range)
-norange          (auto, default)
-auto NUM         (quantiles for auto scaling)
```

#### map2fig
```
--min NUM           (lower limit, can be negative)
--max NUM           (upper limit, can be negative)
--gamma FLOAT       (gamma correction, default 1.0)
--scale FACTOR      (multiply data for unit conversion)
```

**Status:** Similar core functionality, map2fig adds gamma correction and unit scaling

---

### 8. **Text & Labels** ⭐

#### map2png
- ❌ **Not supported**

#### map2fig
- ✅ `--title TEXT`           (custom title, default: gnomonic center coords)
- ✅ `--no-title`             (disable title)
- ✅ `--no-text`              (disable all labels)
- ✅ `--llabel TEXT`          (top-left corner label)
- ✅ `--rlabel TEXT`          (top-right corner label)
- ✅ `--label-font-size PT`   (corner label font size)
- ✅ `--no-scale-text`        (constant text size for gnomonic)

**Status:** map2fig only feature

---

### 9. **Data Masking** ⭐

#### map2png
- ❌ **Not supported**

#### map2fig
- ✅ `--mask-file PATH`       (binary FITS mask: 0=masked, 1=valid)
- ✅ `--mask-below THRESHOLD` (mask values below this)
- ✅ `--mask-above THRESHOLD` (mask values above this)
- ✅ `--maskfill-color COLOR` (color for masked regions)
- ✅ `--mask-coord SYSTEM`    (mask coordinate system)

**Status:** map2fig only feature (powerful for clean visualization)

---

### 10. **Units & LaTeX** ⭐

#### map2png
- ❌ **Not supported**

#### map2fig
- ✅ `--units TEXT`           (colorbar units string)
- ✅ `--latex`                (enable LaTeX rendering)
- ✅ `--units-font-size PT`   (auto-scaled default 16pt)

**Status:** map2fig only - supports mathematical rendering

---

### 11. **Output Formats**

#### map2png
```
Positional args: input.fits output.png
Output: PNG only
```

#### map2fig
```
-o/--out FILENAME
Supports: PNG, PDF
--pdf-backend cairo (default)
  - cairo: high-quality vector graphics with colorbar and overlays
```

**Status:** map2fig superior (PNG + PDF output)

---

### 12. **Background & Transparency**

#### map2png
```
-bg RRGGBB        (background color hex)
```
**Default:** gray (128,128,128)

#### map2fig
```
--bg-color COLOR          (auto, gray, r,g,b,a or #RRGGBB)
--transparent             (transparent background)
--bad-color COLOR         (bad pixel color)
--maskfill-color COLOR    (masked region color)
```

**Status:** map2fig superior (more control, transparency support)

---

### 13. **Image Sizing**

#### map2png
```
-xsz INT or -size INT     (width in pixels)
(height always width/2 for Mollweide)
-resolution NUM           (arcmin/pixel for gnomonic)
```

#### map2fig
```
-w/--width WIDTH          (output width, default 1200)
--fov NUM                 (field of view in arcmin, gnomonic only, default 300)
--res NUM                 (resolution arcmin/pixel, gnomonic only, default 1)
```

**Status:** Similar, but map2fig simpler naming convention

---

### 14. **Data Input**

#### map2png
```
-signal INT       (column selection, 1-indexed)
-map INT          (for HDF multi-map selection)
```

#### map2fig
```
-i/--col COLUMN   (column index, 0-indexed)
(No explicit map selection, implicit)
```

**Status:** map2png more flexible for multi-signal/multi-map FITS, but map2fig simpler for single selection

---

### 15. **Output Control**

#### map2png
```
-ncol INT         (number of columns in layout grid)
(Allows multi-panel output with single command)
```

#### map2fig
```
(No built-in multi-panel support)
```

**Status:** map2png advantageous for batch output, map2fig requires separate commands

---

### 16. **Performance Options**

#### map2png
- ❌ **Not available**

#### map2fig
```
--fast-render             (skip graticule, colorbar, labels for iteration)
--no-downgrade            (disable auto-downgrading for high-NSIDE maps)
--no-border               (disable Mollweide border)
```

**Status:** map2fig only

---

### 17. **Verbosity**

#### map2png
```
-verbose          (timing information)
-quiet            (default, suppress output)
```

#### map2fig
```
--verbose         (detailed execution info)
```

**Status:** Similar

---

## Missing in map2png (map2fig Exclusive)

1. **Hammer projection** - Alternative full-sky projection
2. **SymLog & AsinH scaling** - Better handling of signed/wide-range data
3. **Planck-Log scaling** - CMB-optimized transformation
4. **Coordinate systems** - Galactic/Equatorial/Ecliptic support
5. **Coordinate rotation** - `--rotate-to` and `--roll`
6. **Advanced graticule** - Overlays, labels, styling
7. **Custom text labels** - Title, corner labels with auto-scaling
8. **Data masking** - Threshold and file-based masking
9. **LaTeX support** - Mathematical rendering in labels
10. **Transparency** - Transparent background support
11. **PDF output** - Vector graphics with 2 backends
12. **Unit conversion** - `--scale` factor for data
13. **Font control** - Customizable font sizes
14. **Fast-render mode** - Iteration mode
15. **Better border control** - Disable Mollweide border

---

## Missing in map2fig (map2png Exclusive)

1. **Multi-panel output** - `--ncol` for grid layouts in single command
2. **Multi-signal handling** - Explicit `-signal` for column selection interface
3. **Multi-map selection** - `-map` for HDF selection
4. **Quantile auto-scaling** - `-auto` parameter

---

## Command Simplicity Comparison

### map2png - Simple One-Liner Examples
```bash
# Basic Mollweide with custom colors
map2png -color plasma -minimum 1e-6 -maximum 1e-3 input.fits output.png

# Log scale with grid
map2png -logarithmic -grid 10 input.fits output.png

# Multiple output panels
map2png -ncol 2 -signal 1 -signal 2 input.fits output.png
```

### map2fig - More Explicit Options
```bash
# Basic Mollweide with custom colors
./map2fig -f input.fits -o output.pdf -c plasma --min 1e-6 --max 1e-3

# Log scale with labeled graticule
./map2fig -f input.fits -o output.pdf --log --graticule --grat-labels

# Multiple separate commands needed for multi-panel
./map2fig -f input.fits -o panel1.pdf -i 0
./map2fig -f input.fits -o panel2.pdf -i 1
```

**Note:** map2png short flags are more discoverable; map2fig's long options are more self-documenting.

---

## Performance Comparison

From benchmark results:
- **map2fig**: 27% faster overall than map2png
- map2fig: 157ms (small), 167ms (medium), 1104ms (large)
- map2png: 191ms (small), 333ms (medium), 1430ms (large)

**Advantage:** map2fig

---

## Recommendation for Users

### Use **map2png** if you:
- Need **simple, quick visualizations** without deep customization
- Want **multi-panel output** from single command
- Prefer **short command-line syntax**
- Only need basic Mollweide/Gnomonic projections

### Use **map2fig** if you:
- Need **publication-quality output** with fine control
- Work with **multiple coordinate systems** (cosmic surveys, cross-survey comparison)
- Want **PDF output** with vector graphics
- Need **data masking** or **overlays**
- Require **LaTeX support** for labels and units
- Want **better performance** (27% faster)
- Need **advanced scaling** (SymLog, AsinH, etc.)
- Want **transparency** or custom styling

---

## Summary

**map2fig is a superset of map2png** with:
- ✅ Better performance (27% faster)
- ✅ More projections (Hammer)
- ✅ More scaling methods (5 vs 3)
- ✅ 80+ colormaps (vs 10)
- ✅ Coordinate systems (vs none)
- ✅ Advanced graticule with overlays
- ✅ Data masking & transparency
- ✅ PDF output
- ✅ Custom labels & LaTeX

But map2png remains simpler for quick workflows and offers multi-panel output in single commands.
