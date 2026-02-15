# HEALPix Plotter (`map2fig`)

A fast, publication-quality Rust tool for visualizing HEALPix sky maps in Mollweide, Hammer, and Gnomonic projections. Reads FITS files and generates PDF or PNG plots with customizable colormaps, scaling, and coordinate transformations.

## Features

- **Multiple Projections**: Mollweide (full-sky), Hammer (full-sky), and Gnomonic (local) projections
- **80+ Colormaps**: Matplotlib colormaps plus custom scientific colormaps
- **Advanced Scaling**: Linear, log, symlog, asinh, histogram equalization, and Planck-specific scaling
- **Coordinate Systems**: Galactic, Equatorial (FK5), and Ecliptic with automatic transformations
- **Graticules**: Customizable coordinate grid overlays with color support
- **Masking**: Mask regions by value range or binary FITS file with custom fill colors
- **Output Formats**: PDF (via Cairo) and PNG (via image crate)
- **High Quality**: Publication-ready figures with configurable resolution and styling

## Installation

### Requirements
- **Rust 1.80+** (uses 2024 edition for let chains and other modern features) — [Install from rustup.rs](https://rustup.rs)
- **LaTeX/pdflatex** (for equation rendering) — *Optional but recommended*
  - Ubuntu/Debian: `sudo apt-get install texlive-latex-base texlive-latex-extra`
  - macOS: `brew install basictex`
  - Fedora: `sudo dnf install texlive-latex`
- **Tectonic** (for superior LaTeX rendering) — *Optional; pdflatex used as fallback*
- **Cairo development libraries** (for PDF output)

### Quick Install (Recommended)
```bash
git clone https://github.com/dncnwtts/map2fig
cd map2fig
./install.sh
```

The `install.sh` script will:
1. Verify Rust is installed
2. Check for LaTeX (pdflatex) and guide installation if needed
3. Offer to install **tectonic** for improved LaTeX rendering
4. Build the release binary

### Manual Build
```bash
git clone https://github.com/dncnwtts/map2fig
cd map2fig
cargo build --release
```

The compiled binary will be at `target/release/map2fig`.

### Enabling Tectonic (Optional)
For superior LaTeX rendering with better spacing control:
```bash
cargo install tectonic
```

Once installed, tectonic will be automatically used for all LaTeX rendering. If tectonic is not available, the system pdflatex will be used as a fallback.

**Note**: Tectonic provides more reliable PDF generation with self-contained compilation. The build script (`build.rs`) checks for tectonic availability and provides guidance during the build process.

**⚠️ Build Issues?** See [TECTONIC_INSTALL.md](docs/development/TECTONIC_INSTALL.md) for detailed troubleshooting, including:
- Missing harfbuzz development headers (common on Linux)
- Platform-specific dependency installation
- Fallback behavior if tectonic doesn't work

## Basic Usage

```bash
./map2fig -f data.fits -o output.pdf
```

## Common Use Cases & Examples

### 1. Basic All-Sky Map (Mollweide Projection)

**Use Case**: Quick visualization of a full-sky HEALPix map

```bash
./map2fig -f npipe_nodip.fits -o map.pdf
```

This creates a default Mollweide projection with:
- Viridis colormap
- Linear scaling
- Galactic coordinates
- 1200×600 pixel output
- Colorbar showing data range

**Output**: Generates `map.pdf`—full-sky map with professional formatting, colorbar, and axis labels. The Mollweide projection displays the entire sky in an oval format suitable for astronomical publications. [View full resolution PDF](examples/outputs/example1_mollweide.pdf)

---

### 2. Log-Scale Map with Custom Color Limits

**Use Case**: Visualizing data with extreme dynamic range (e.g., instrumental sensitivity maps)

```bash
./map2fig -f npipe_nodip.fits \
  --log \
  --min 1e-6 --max 1e-3 \
  --cmap plasma \
  -o sensitivity.pdf
```

The `--log` flag applies logarithmic scaling; `--min` and `--max` set the color scale limits. The Plasma colormap is good for perceptually uniform maps.

**Output**: Generates `sensitivity.pdf`—full-sky map with logarithmic intensity scaling and Plasma colormap. Colorbar shows the 1e-6 to 1e-3 range with log scale labels. [View full resolution PDF](examples/outputs/example2_log_scale.pdf)

---

### 3. Zoomed Gnomonic Projection with Local Graticule

**Use Case**: Detailed view of a specific sky region (e.g., Galactic center, Crab nebula)

```bash
./map2fig -f npipe_nodip.fits \
  --projection gnomonic \
  --lon 266.5 --lat -28.9 \
  --local-graticule \
  --local-grat-dlat 2 \
  --local-grat-dlon 2 \
  -o galactic_center.png
```

**Parameters**:
- `--projection gnomonic`: Use local projection instead of all-sky
- `--lon 266.5 --lat -28.9`: Center on Galactic center
- `--local-graticule`: Show local coordinate grid (1° spacing by default)
- `--local-grat-d{lat,lon} 2`: 2° spacing for cleaner appearance

**Default Resolution**: 
- Default field of view: 300 arcmin (5 degrees)
- Default resolution: 1 arcmin/pixel = 300×300 pixel map area
- Adjust `--fov` (arcmin) and `--res` (arcmin/pixel) to change zoom level

[![Example 3: Gnomonic view with local graticule](examples/outputs/example3_gnomonic_graticule_preview.png)](examples/outputs/example3_gnomonic_graticule.png)

**Output**: Generates `galactic_center.png`—zoomed 300×300 pixel PNG centered on the Galactic center region with visible 2° coordinate grid. Black lines show local tangent-plane coordinates.

**Examples of different zoom levels**:
```bash
# Very zoomed (0.5°)
./map2fig -f data.fits --projection gnomonic --fov 30 -o zoomed_tight.png

# Standard zoom (5°) - default
./map2fig -f data.fits --projection gnomonic -o standard.png

# Wide view (10°)
./map2fig -f data.fits --projection gnomonic --fov 600 -o wide_view.png

# Ultra-high resolution (0.1 arcmin/pixel)
./map2fig -f data.fits --projection gnomonic --res 0.1 -o high_res.png
```

---

### 4. Gnomonic with Overlay Graticule (Multiple Coordinate Systems)

**Use Case**: Compare coordinates in different systems on a zoomed gnomonic view (e.g., show Equatorial coordinates over Galactic map)

Show local graticule with an overlay from a different coordinate system:

```bash
./map2fig -f npipe_nodip.fits \
  --projection gnomonic \
  --lon 0 --lat 0 \
  --fov 600 \
  --local-graticule \
  --grat-coord-overlay eq \
  --grat-par 30 \
  -o gal_with_eq_overlay.pdf
```

This renders:
- **Black grid**: Local tangent-plane coordinates at projection center (Galactic)
- **Yellow grid**: Equatorial (FK5) coordinates overlaid for reference

The overlay graticule shows how different coordinate systems relate on the same sky patch. The overlay lines may curve or appear distorted due to the projection, which is physically correct—it shows the true relationship between the coordinate systems on the curved sky.

**Output**: Generates `gal_with_eq_overlay.pdf`—600×600 pixel PDF with black local graticule and yellow equatorial overlay at 30° spacing, showing coordinate system relationships. [View PDF](examples/outputs/example4_overlay_graticule.pdf)

---

### 4b. Gnomonic with Image Rotation

**Use Case**: Rotate the gnomonic projection around the center for custom viewing angles

```bash
./map2fig -f npipe_nodip.fits \
  --projection gnomonic \
  --lon 0 --lat 0 \
  --fov 600 \
  --roll 45 \
  -o rotated_view.png
```

**Parameters**:
- `--roll 45`: Rotate the image 45° counterclockwise around the projection center
- Works with both graticule modes (local and overlay)

[![Example 4b: Gnomonic with 45° roll](examples/outputs/example4b_roll_preview.png)](examples/outputs/example4b_roll.png)

**Output**: Generates `rotated_view.png`—600×600 pixel PNG of the gnomonic projection rotated 45° counterclockwise. The projection center remains fixed while the map rotates around it.

---

### 4c. Graticule Customization (Line Width & Overlay Colors)

**Use Case**: Create publication-quality figures with customizable graticule appearance

```bash
./map2fig -f npipe_nodip.fits \
  --projection gnomonic \
  --lon 266.5 --lat -28.9 \
  --fov 120 \
  --local-graticule \
  --grat-line-width 2 \
  --grat-coord-overlay eq \
  --grat-overlay-color "#FF6B6B" \
  -o detailed_graticule.png
```

**Graticule Overlay Features**:
- **`--grat-coord-overlay`**: Add secondary coordinate system overlay
  - `eq` or `equatorial`: Show equatorial (FK5/ICRS) coordinates
  - `gal` or `galactic`: Show galactic coordinates  
  - `ecl` or `ecliptic`: Show ecliptic coordinates
  - Works for both Mollweide and Gnomonic projections
  
- **`--grat-overlay-color`**: Custom hex color for overlay lines
  - Format: `#RRGGBB` (e.g., `#FF0000` for red, `#00FF00` for green)
  - Default: `#FFFF00` (yellow) for good contrast on dark maps
  
- **`--grat-line-width`**: Control thickness of graticule lines
  - Default: 1 pixel (thin lines)
  - Increase for high-resolution plots: 2-4 pixels recommended
  - Example: `--grat-line-width 3` for 3-pixel thick lines
[![Example 4c: Customized graticule with thick red lines](examples/outputs/example4c_graticule_customization_preview.png)](examples/outputs/example4c_graticule_customization.png)

  - Applies to both local graticule and overlay coordinates

**Output**: Generates `detailed_graticule.png`—high-resolution 120×120 arcmin PNG with black local graticule and red equatorial overlay, both with 2-pixel line thickness.

**Advanced Example** - Publication figure with multiple overlays:

```bash
./map2fig -f npipe_nodip.fits \
  --projection gnomonic \
  --lon 0 --lat 0 \
  --fov 360 \
  --local-graticule \
  --grat-line-width 2 \
  --grat-coord-overlay eq \
  --grat-overlay-color "#00D9FF" \
  --grat-par 15 --grat-mer 15 \
  --cmap twilight \
  --width 2048 \
  -o graticule_demo.png
```

This creates:
- Local graticule in black (15° spacing)
- Equatorial overlay in cyan (15° spacing)
- 2-pixel thick lines for visibility at high resolution
- 2048×2048 pixel output

**Important Notes**:
- Overlay coordinate lines only appear where they intersect the visible map region
- For small field-of-view gnomonic plots (< 1°), overlay lines may not appear if they don't pass through the region—use a larger FOV or reposition the center
- Thick lines (width > 2) work best at resolution ≥ 0.1 arcmin/pixel

---

### 5. Multi-Panel Comparison: Mollweide with Dual Graticules

**Use Case**: Publication figure comparing coordinate systems on full-sky map

```bash
./map2fig -f npipe_nodip.fits \
  --graticule \
  --grat-coord gal \
  --grat-coord-overlay eq \
  --grat-overlay-color "#00D9FF" \
  --grat-par 30 --grat-mer 45 \
  --cmap coolwarm \
  --width 2400 \
  --latex \
  --units "Temperature [$\mu$K]" \
  -o publication_figure.pdf
```

**Features**:
- Mollweide (default) shows full-sky map
- Primary graticule in Galactic, secondary in Equatorial (cyan)
- 30° parallels, 45° meridians for clean appearance
- Higher resolution (2400 px width)
- LaTeX-formatted colorbar with units
- Publication-ready PDF output

**Output**: Generates `publication_figure.pdf`—high-resolution 2400×1200 pixel PDF with Coolwarm colormap, black Galactic graticule, cyan Equatorial overlay, and LaTeX-formatted colorbar showing temperature in µK. [View PDF](examples/outputs/example5_dual_graticules.pdf)


---

### 6. High Dynamic Range with Histogram Equalization

**Use Case**: Revealing subtle features in maps with non-Gaussian distributions

```bash
./map2fig -f npipe_nodip.fits \
  --hist \
  --cmap inferno \
  -w 1600 \
  --bg-color "26,26,26,255" \
  -o survey_equalized.pdf
```

**Parameters**:
- `--hist`: Use histogram equalization instead of linear scaling (flattens the histogram to reveal subtle features)
- `--bg-color "26,26,26,255"`: Dark background (RGB + alpha) for astronomy publication style (note: format is RGBA values, not hex)
- Optional: `--min/--max`: Explicit data range limits for histogram; omit to auto-detect full dynamic range

**Output**: Generates `survey_equalized.pdf`—1600×800 pixel PDF with histogram-equalized Inferno colormap and dark charcoal background, revealing subtle features across the full dynamic range. [View PDF](examples/outputs/example6_histogram_equalization.pdf)


---

### 7. Arcsinh Scaling for Symmetric Data

**Use Case**: Bipolar data (e.g., CMB temperature maps, velocity fields) with asymmetric noise properties

```bash
./map2fig -f npipe_nodip.fits \
  --asinh \
  --scale 1e6 \
  --min -1000 --max 1000 \
  --cmap rdbu-r \
  -o cmb_asinh.pdf
```

**Parameters**:
- `--asinh`: Inverse hyperbolic sine scaling (smooth transition between linear and logarithmic)
- `--scale 1e6`: Unit conversion factor (e.g., μK × 1e6 for sensitivity maps)
- `--min/--max`: Symmetric data range limits
- `--cmap rdbu-r`: Red-Blue reversed colormap (red for positive, blue for negative)

Arcsinh scaling is ideal for data where you need to see both small and large deviations from zero without artificial discontinuities. It's mathematically defined as `sinh⁻¹(x) = ln(x + √(1 + x²))`.

**Output**: Generates `cmb_asinh.pdf`—full-sky map with arcsinh-scaled Red-Blue colormap, ideal for publication-quality CMB or other symmetric data visualizations. [View PDF](examples/outputs/example7_asinh_scaling.pdf)


---

### 8. Symmetric Logarithmic Scaling with Linear Core

**Use Case**: Data with extreme dynamic range around a well-defined center (e.g., bipolar CMB, residual maps)

```bash
./map2fig -f npipe_nodip.fits \
  --symlog \
  --linthresh 10 \
  --scale 1e6 \
  --min -1000 --max 1000 \
  --cmap rdbu-r \
  -o cmb_symlog.pdf
```

**Parameters**:
- `--symlog`: Symmetric logarithmic scaling (preserves sign, linear near zero)
- `--linthresh 10`: Linear region threshold (±10 μK) to avoid log(0) singularities
- `--scale 1e6`: Unit conversion factor
- `--min/--max`: Symmetric data range limits
- `--cmap rdbu-r`: Red-Blue reversed colormap

Symlog creates a linear region in the middle (±linthresh) and logarithmic scaling beyond, allowing visualization of both tiny residuals near zero and extreme outliers. Perfect for differential maps showing deviations from a reference.

**Output**: Generates `cmb_symlog.pdf`—full-sky map with symlog-scaled Red-Blue colormap showing both small-scale structure and extreme features. [View PDF](examples/outputs/example8_symlog_scaling.pdf)


---

### 9. LaTeX Units & Mathematical Notation in Colorbar

**Use Case**: Publication-quality figures with proper mathematical symbols and formatting in labels

```bash
./map2fig -f npipe_nodip.fits \
  --latex \
  --units '$\mathrm{\mu K_{CMB}}$' \
  --cmap viridis \
  -o cmb_with_units.pdf
```

**Parameters**:
- `--latex`: Enable LaTeX rendering for units and labels
- `--units '$\mathrm{\mu K_{CMB}}$'`: Display units with proper formatting
  - `\mathrm{...}`: Roman (upright) text font
  - `\mu`: Greek letter μ (mu)
  - `_{CMB}`: Subscript notation
- `--cmap viridis`: High-quality perceptually uniform colormap

**Supported LaTeX Features**:
- Greek letters: `\mu`, `\sigma`, `\pi`, `\omega`, etc.
- Superscripts: `10^{-6}`, `x^{2}`
- Subscripts: `K_{CMB}`, `x_{1}`
- Text formatting: `\mathrm{...}`, `\mathbf{...}`, `\text{...}`
- Mathematical operators: `\pm`, `\times`, `\leq`, `\infty`, etc.

**Output**: Generates `cmb_with_units.pdf`—full-sky map with colorbar showing properly formatted units μK_{CMB} using Unicode mathematical symbols. [View PDF](examples/outputs/example9_latex_units.pdf)


---

### 10. Coordinate Rotation: Ecliptic View

**Use Case**: Transform map to show ecliptic coordinates (e.g., for zodiacal light studies)

```bash
./map2fig -f galactic_map.fits \
  --input-coord gal \
  --output-coord ecl \
  --graticule \
  --grat-coord ecl \
  --cmap twilight \
  -o ecliptic_view.pdf
```

The tool automatically rotates the data and graticule to ecliptic coordinates.

---

### 11. Negative/Invalid Data Handling

**Use Case**: Maps with masked regions (e.g., point source masks, bad pixels)

```bash
./map2fig -f masked_map.fits \
  --neg-mode unseen \
  --bad-color "200,200,200" \
  --min 0.01 --max 100 \
  --log \
  -o masked_clean.pdf
```

**Parameters**:
- `--neg-mode unseen`: Treat negative/masked pixels as UNSEEN (not included in scale calculation)
- `--bad-color`: Render masked pixels in light gray (RGB: 200,200,200)

**Alternative**: `--neg-mode zero` treats negatives as 0.

---

### 12. Batch Processing Multiple FITS Files

**Use Case**: Generate plots for all frequency bands of a survey

```bash
#!/bin/bash
for freq in 30 44 70 100 143 217 353; do
  input="planck_${freq}GHz.fits"
  output="planck_${freq}GHz.pdf"
  
  ./map2fig -f "$input" \
    --cmap viridis \
    --units "I [$\mu$K]" \
    --latex \
    --graticule \
    -w 1200 \
    -o "$output"
  
  echo "Generated $output"
done
```

Generate consistent publication figures for all data products.

---

### 13. Asymmetric Scaling for Bipolar Data

**Use Case**: CMB temperature maps or velocity fields (symmetric around zero)

```bash
./map2fig -f cmb_temperature.fits \
  --symlog \
  --linthresh 10 \
  --min -300 --max 300 \
  --cmap rdbu-r \
  --units "Temperature [$\mu$K]" \
  --latex \
  -o cmb_symlog.pdf
```

**Parameters**:
- `--symlog`: Symmetric logarithmic scaling (preserves sign)
- `--linthresh 10`: Linear region around zero (±10 μK) to avoid log(0)
- RdBu reversed colormap (Red-Blue): Red for hot, Blue for cold

---

### 14. Masking Regions by Value Range

**Use Case**: Hide pixels below/above certain thresholds or fill masked regions with custom colors (e.g., mask Galactic plane in CMB analysis)

```bash
./map2fig -f cmb_temperature.fits \
  --mask-below 0 \
  --maskfill-color gray \
  --cmap viridis \
  -o masked_by_threshold.pdf
```

**Parameters**:
- `--mask-below 10`: Mask all pixels with values < 10
- `--mask-above 1000`: Mask all pixels with values > 1000
- `--maskfill-color`: Fill color for masked pixels
  - `transparent` (default): Masked regions transparent
  - `gray`: Light gray (RGB 200,200,200)
  - Hex color: `#FF6B6B` (custom red)
  - RGB tuple: `128,128,128` (explicit gray)

**Output**: Generates `masked_by_threshold.pdf`—map with pixels outside the range hidden and filled with gray background.

---

### 15. Masking from Binary FITS File

**Use Case**: Apply external pixel masks (e.g., point source masks, galactic foreground masks, bad pixel lists)

```bash
./map2fig -f cmb_temperature.fits \
  --mask-file galactic_mask.fits \
  --maskfill-color "128,128,128" \
  --cmap viridis \
  -o cmb_with_mask.pdf
```

**Parameters**:
- `--mask-file`: Path to FITS file containing binary mask
  - File format: Binary table with single column containing pixel data
  - Data values: 0 or near-0 = masked, >0.5 = valid
  - NSIDE auto-detected from array size
  - Supports float, double, or integer data types
- `--mask-coord`: Coordinate system of mask
  - `gal` or `galactic`: Galactic coordinates (default)
  - `eq` or `equatorial`: Equatorial coordinates
  - `ecl` or `ecliptic`: Ecliptic coordinates

**Example mask file structure** (created with Python):
```python
import numpy as np
from astropy.io import fits

# Create a mask for Galactic plane (example)
nside = 128
npix = 12 * nside * nside
mask = np.ones(npix)  # Start with all valid (1)
# Mask pixels within ±10° of galactic equator
for pixel in range(npix):
    # Calculate pixel latitude
    # Set to 0 if within ±10° of equator
    pass

# Write to FITS
hdu = fits.BinTableHDU.from_columns([
    fits.Column(name='mask', format='D', array=mask)
])
hdu.writeto('galactic_mask.fits')
```

**Easiest way to generate mask FITS files:** Use the included Python utility:
```bash
python3 tools/create_mask_example.py --nside 128 --mask-galactic-plane 10 -o mask.fits
python3 tools/create_mask_example.py --nside 256 --mask-uniform 0.8 -o partial_mask.fits
python3 tools/create_mask_example.py --nside 512 --mask-random 0.2 -o random_mask.fits
```

See all available mask generation options:
```bash
python3 tools/create_mask_example.py --help
```

**Output**: Generates `cmb_with_mask.pdf`—map with pixels masked by external FITS file, useful for applying pre-computed masks across multiple visualizations.

---

### 16. Combined Masking and Scaling

**Use Case**: Apply both value-range masking and non-linear scaling for publication figures

```bash
./map2fig -f sensitivity_map.fits \
  --log \
  --min 1e-6 --max 1e-3 \
  --mask-below 0 \
  --maskfill-color transparent \
  --cmap magma \
  --graticule \
  -o sensitivity_masked.pdf
```

Combines logarithmic scaling for dynamic range with masking to hide invalid pixels. The mask is applied after scaling, ensuring consistent color limits across unmasked regions.

**Output**: Generates `sensitivity_masked.pdf`—map with log-scaled colors and transparent masked regions, ideal for publication figures.

---

## Command-Line Reference

### Input/Output
| Option | Description | Default |
|--------|-------------|---------|
| `-f, --fits` | Input FITS file | Required |
| `-i, --col` | FITS column index | 0 |
| `-o, --out` | Output filename | output.pdf |
| `-w, --width` | Output width (pixels) | 1200 |

### Colormaps & Scaling
| Option | Description |
|--------|-------------|
| `-c, --cmap` | Colormap name (80+ available) |
| `--min, --max` | Color scale limits |
| `--log` | Logarithmic scaling |
| `--symlog` | Symmetric log (for bipolar data) |
| `--asinh` | Inverse hyperbolic sine scaling |
| `--hist` | Histogram equalization |
| `--gamma` | Gamma correction factor |

### Projections
| Option | Description |
|--------|-------------|
| `--projection` | `mollweide` (default), `hammer`, or `gnomonic` |
| `--lon, --lat` | Center coordinates (degrees) |
| `--fov` | Field of view in arcmin (gnomonic only) |
| `--res` | Resolution in arcmin/pixel (gnomonic only) |
| `--roll` | Rotation angle in degrees around center (gnomonic only) |

### Graticules
| Option | Description |
|--------|-------------|
| `--graticule` | Enable coordinate grid (mollweide) |
| `--grat-coord` | Primary system: gal, eq, ecl |
| `--grat-coord-overlay` | Secondary system (e.g., show both gal+eq) |
| `--grat-overlay-color` | Color for secondary (hex #RRGGBB) |
| `--grat-line-width` | Thickness of graticule lines in pixels (default: 1) |
| `--grat-par, --grat-mer` | Spacing for parallels/meridians (°) (mollweide) |
| `--grat-labels` | Show lat/lon values on grid lines |
| `--local-graticule` | Local grid for gnomonic (gnomonic only) |
| `--local-grat-dlat, --local-grat-dlon` | Local grid spacing (°) (gnomonic) |

### Masking
| Option | Description |
|--------|-------------|
| `--mask-below` | Mask pixels with values below threshold |
| `--mask-above` | Mask pixels with values above threshold |
| `--mask-file` | Path to binary FITS mask file |
| `--maskfill-color` | Fill color for masked regions (transparent/gray/#RRGGBB/R,G,B,A) |
| `--mask-coord` | Coordinate system of mask: gal/eq/ecl |

### Styling
| Option | Description |
|--------|-------------|
| `--no-cbar` | Disable colorbar |
| `--no-border` | Disable map border |
| `--transparent` | Transparent background |
| `--bg-color` | Background color |
| `--bad-color` | Color for masked/invalid pixels |
| `--units` | Colorbar units string (LaTeX supported with `--latex`) |
| `--latex` | Enable LaTeX rendering for colorbar labels and units |

### Coordinates
| Option | Description | Values |
|--------|-------------|--------|
| `--input-coord` | Input coordinate system | gal, eq, ecl |
| `--output-coord` | Output coordinate system | gal, eq, ecl |
| `--scale` | Unit conversion factor | e.g., 1000 for mK→μK |

### Utilities
| Option | Description |
|--------|-------------|
| `-h, --help` | Print help message |
| `-V, --version` | Print version |
| `--verbose` | Detailed logging |

## Tips & Tricks

### File Format Support
- Reads any HEALPix map in FITS format (single column or multi-column)
- Specify column with `-i` flag: `./map2fig -f multi_col.fits -i 3`

### Colormap Selection
- **Perceptually uniform**: viridis, plasma, inferno, magma
- **Sequential**: Blues, Greens, Reds, YlOrRd
- **Diverging**: RdBu, PiYG, coolwarm, RdYlBu
- See all 80+ available: `./map2fig --help`

### Output Quality
- PNG for web/presentations: `-o map.png`
- PDF for publication: `-o map.pdf` (scalable)
- Higher pixel width for detail: `-w 2400` (default 1200)

### LaTeX Support for Units & Labels

When using the `--latex` flag, colorbar units and labels are rendered using your system's **LaTeX/TeX installation** for publication-quality mathematical notation.

#### How It Works
1. **LaTeX Compilation**: LaTeX strings are compiled to PDF using `pdflatex`
2. **Vector Conversion**: PDFs are converted to SVG (vector format) using `pdf2svg` or `convert`
3. **Rasterization**: SVG/PDF content is rendered to PNG for embedding in the final PDF
4. **Caching**: Rendered images are cached in `~/.cache/map2fig/latex/` to avoid re-rendering
5. **Fallback Chain**: SVG → High-DPI PNG (300 DPI) → Standard PNG (150 DPI) → Unicode

#### Requirements
For full LaTeX rendering, you need:
- `pdflatex` (usually part of TeX Live or MacTeX)
- **Either** `pdf2svg` **or** ImageMagick `convert` for vector PDF conversion
- `pdftoppm` (from poppler-utils) as final fallback

Install with:
```bash
# Ubuntu/Debian
sudo apt-get install texlive-latex-base poppler-utils pdf2svg imagemagick

# macOS (Homebrew)
brew install basictex poppler pdf2svg imagemagick

# Or TeX Live directly + system package manager
```

#### Rendering Pipeline
```
LaTeX → pdflatex → PDF → [pdf2svg | convert] → SVG/PNG → Cairo PDF
```

The vector-to-raster conversion preserves mathematical quality through careful scaling and high-DPI rendering.

#### Supported LaTeX Features
- **Full math mode**: All LaTeX mathematical environments and packages
- **Greek letters**: `\mu`, `\sigma`, `\pi`, `\omega`, `\Gamma`, etc.
- **Superscripts/Subscripts**: `10^{-6}`, `K_{CMB}`, `T_{\mathrm{eff}}`
- **Text formatting**: `\mathrm{text}`, `\mathbf{bold}`, `\mathit{italic}`, `\text{...}`
- **Mathematical symbols**: `\pm`, `\times`, `\leq`, `\approx`, `\propto`, `\infty`
- **Advanced packages**: Supports `amsmath`, `amssymb`, `mathtools`

#### Examples
```bash
# Perfect subscripts and text formatting (uses SVG pipeline if available)
./map2fig -f map.fits --latex --units '$K_{\mathrm{CMB}}$' -o map.pdf

# Scientific notation with Greek letters
./map2fig -f map.fits --latex --units '$10^{-6}\,\mu\mathrm{Jy}$' -o map.pdf

# Temperature with subscript and units
./map2fig -f map.fits --latex --units '$T_{\mathrm{eff}} = 5800\,\mathrm{K}$' -o map.pdf

# Complex expressions
./map2fig -f map.fits --latex --units '$\sigma = 10^{-3}\,K_{\mathrm{CMB}}$' -o map.pdf
```

**Tip**: Wrap LaTeX strings in single quotes `'...'` in bash to prevent shell interpretation:
```bash
# ✓ Correct (single quotes)
./map2fig -f map.fits --latex --units '$K_{CMB}$' -o map.pdf

# ✗ Wrong (double quotes will expand $ and \)
./map2fig -f map.fits --latex --units "$K_{CMB}$" -o map.pdf
```

#### Performance
- **First render**: Takes ~1-2 seconds per unique LaTeX string (depends on system and PDF conversion tool)
- **Subsequent renders**: Instant (cached from `~/.cache/map2fig/latex/`)
- **Runtime impact**: Negligible for typical plots with 1-3 units labels

#### Example Output
Run the example commands to generate publication-quality figures with rendered units.

### Coordinate Systems
- **Galactic (gal)**: Standard for CMB/ISM science
- **Equatorial (eq, FK5)**: Standard for source catalogs
- **Ecliptic (ecl)**: For solar system / zodiacal light studies

### Best Practices
1. Use `--graticule` for publication figures (helps readers orient)
2. Use `--latex` and `--units` for proper unit labels
3. Test with `--verbose` flag to debug coordinate transformations
4. For dark colormaps, use `--bg-color` to set plot background
5. Use `--hist` for non-Gaussian data distributions

## Performance

Typical performance on a modern CPU:
- Full-sky Mollweide (1200×600): ~500ms
- Gnomonic zoom (1248×1248): ~200ms
- Full pipeline with graticules: <1s

**Performance Optimization Tips**:
- The binary includes automatic downgrading of high-NSIDE maps for better performance at standard resolutions (1200-2400px)
- For benchmarking or ultra-high-resolution output (4000px+), disable downgrading with `--no-downgrade`
- Batch processing is embarrassingly parallel—use shell loops to process multiple files

### Comparison with map2png

map2fig can be benchmarked against the widely-used `map2png` tool from the HEALPix community. Libraries are automatically discovered from standard build locations:

```bash
# Run the comparison benchmark (auto-detects libhealpix_cxx and libhdf5)
./tools/scripts/compare_map2png_vs_map2fig.sh

# Or set custom library paths if needed
export MAP2PNG_LIBPATH="/custom/lib:$LD_LIBRARY_PATH"
./tools/scripts/compare_map2png_vs_map2fig.sh
```

The benchmark script gracefully handles missing map2png dependencies and will report map2fig standalone PNG performance metrics if comparison is unavailable. See [tools/README.md](tools/README.md) for detailed setup instructions and the map2png command-line interface.

#### Feature Comparison

**Benchmark Results:** map2fig is **27% faster** overall than map2png, with better performance on larger datasets.

**Feature Differences:** See detailed analysis documents:

| Document | Contents | Users |
|----------|----------|-------|
| [**FEATURE_COMPARISON.md**](docs/comparison/FEATURE_COMPARISON.md) | Complete side-by-side feature matrix with detailed explanations | Users comparing tools, deciding which to use |
| [**IMPLEMENTATION_GAPS.md**](docs/comparison/IMPLEMENTATION_GAPS.md) | Missing features, enhancement opportunities, and migration guide | Users transitioning from map2png, looking for feature parity |

**Quick Summary:**
- ✅ **map2fig advantages:** Faster, more projections (Hammer), coordinate systems, data masking, PDF output, LaTeX support, 80+ colormaps
- ✅ **map2png advantages:** Simpler CLI, multi-panel output in single command



## Troubleshooting

**Gnomonic map is too small**
- Default field of view is now 300 arcmin (5 degrees). This gives a reasonable 300×300 pixel map.
- For smaller regions: `--fov 30` (0.5 degree)
- For wider views: `--fov 600` (10 degrees)
- Default resolution is 1 arcmin/pixel; adjust with `--res`

**Graticule overlay not showing on gnomonic with small FOV**
- With very small fields of view (< 2 degrees), overlay coordinate systems may not intersect the visible region
- The overlay graticule (e.g., ecliptic) may genuinely not pass through that tiny patch of sky
- **Solution**: Use a larger FOV (`--fov`), or position the center where the overlay coordinates are present
- Example: Ecliptic overlays work best near ecliptic coordinates (galactic longitude 0°, 180°, or ±90°)
- Test with equatorial overlay first (`--grat-coord-overlay eq`) as it's more widely distributed

**Roll parameter not rotating the image**
- The `--roll` parameter works for gnomonic projections and rotates the image around the projection center
- Example: `--projection gnomonic --roll 45` rotates 45° counterclockwise
- Works independently (no need for `--rotate-to`)

**"Could not parse FITS file"**
- Ensure file is valid FITS with HEALPix metadata
- Check column index with `fitsinfo file.fits`

**"Invalid colormap: viridis"**
- Colormap names are case-insensitive, but must be valid
- Run with `--help` to see available colormaps

**Graticule lines look thick/thin**
- Adjust resolution with `--res` for gnomonic
- Adjust graticule spacing with `--grat-par`/`--grat-mer` (Mollweide)
- Adjust graticule spacing with `--local-grat-dlat`/`--local-grat-dlon` (gnomonic)

**Colors appear washed out**
- Try `--gamma 0.8` for brighter appearance
- Use different colormap better suited to your data
- Check output format (PNG vs PDF rendering differs slightly)

## Example FITS Files

The repository includes sample FITS files for testing:
- `cosmoglobe_clipped.fits`: Simulated Cosmoglobe data
- `npipe_nodip.fits`: Planck NPipe map (dipole removed)
- Other test files for various scenarios

## Contributing

**Development Roadmap:** See [ROADMAP.md](docs/comparison/ROADMAP.md) for planned features and priorities.

**High-Impact Contributions:**
- **Multi-panel output** ⭐ (Priority 1: Eliminate critical map2png use case)
- **Quantile-based auto-scaling** (Priority 2)
- **Additional short CLI flags** (Priority 2)
- **Batch processing mode** (Priority 3)

**Optional Improvements:**
- New colormaps (see `tools/generate_colormaps.py`)
- Additional projections
- Graticule improvements
- Performance optimizations
- Interactive/TUI mode

See [ROADMAP.md](docs/comparison/ROADMAP.md) for detailed implementation estimates and testing strategies.

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.

MIT License © 2024 Duncan Watts

## Citation & Acknowledgments

**map2fig** builds on the excellent work of the HEALPix, HEALPy, and Cosmoglobe communities, as well as the broader astronomical software ecosystem.

For detailed citations, references, and attribution, see:
- [**ACKNOWLEDGMENTS.md**](docs/history/ACKNOWLEDGMENTS.md) – Comprehensive credits and scientific references
- [**CITATION.cff**](CITATION.cff) – Machine-readable citation metadata

### Quick Citation

If you use **map2fig** in your research, please cite:

```bibtex
@software{map2fig2024,
  title={map2fig: Fast HEALPix Visualization in Rust},
  author={Watts, Duncan},
  year={2024},
  url={https://github.com/dncnwtts/map2fig}
}
```

**Key references to also cite:**
- Górski et al. (2005) – HEALPix framework
- Zonca et al. (2019) – HEALPy library
- Planck Collaboration (2020) – Data and colormaps
- Hunter & Droettboom – Matplotlib colormaps

## See Also

- [HEALPix Documentation](https://healpix.sourceforge.io/)
- [Planck Legacy Archive](https://pla.esac.esa.int/)
- [Cosmoglobe](https://cosmoglobe.uio.no/) - Full-sky maps
