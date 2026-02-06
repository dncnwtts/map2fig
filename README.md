# HEALPix Plotter (`map2fig`)

A fast, publication-quality Rust tool for visualizing HEALPix sky maps in Mollweide and Gnomonic projections. Reads FITS files and generates PDF or PNG plots with customizable colormaps, scaling, and coordinate transformations.

## Features

- **Multiple Projections**: Mollweide (full-sky) and Gnomonic (local) projections
- **80+ Colormaps**: Matplotlib colormaps plus custom scientific colormaps
- **Advanced Scaling**: Linear, log, symlog, asinh, histogram equalization, and Planck-specific scaling
- **Coordinate Systems**: Galactic, Equatorial (FK5), and Ecliptic with automatic transformations
- **Graticules**: Customizable coordinate grid overlays with color support
- **Output Formats**: PDF (via Cairo) and PNG (via image crate)
- **High Quality**: Publication-ready figures with configurable resolution and styling

## Installation

### Requirements
- Rust 1.70+ (uses 2024 edition)
- Cairo development libraries (for PDF output)

### Build
```bash
git clone <repository>
cd healpix_plotter
cargo build --release
```

The compiled binary will be at `target/release/map2fig`.

## Basic Usage

```bash
./map2fig -f data.fits -o output.pdf
```

## Common Use Cases & Examples

### 1. Basic All-Sky Map (Mollweide Projection)

**Use Case**: Quick visualization of a full-sky HEALPix map

```bash
./map2fig -f cosmoglobe.fits -o map.pdf
```

This creates a default Mollweide projection with:
- Viridis colormap
- Linear scaling
- Galactic coordinates
- 1200×600 pixel output
- Colorbar showing data range

**Output**: PDF with professional formatting and colorbar.

---

### 2. Log-Scale Map with Custom Color Limits

**Use Case**: Visualizing data with extreme dynamic range (e.g., instrumental sensitivity maps)

```bash
./map2fig -f sensitivity_map.fits \
  --log \
  --min 1e-6 --max 1e-3 \
  --cmap plasma \
  -o sensitivity.pdf
```

The `--log` flag applies logarithmic scaling; `--min` and `--max` set the color scale limits. The Plasma colormap is good for perceptually uniform maps.

---

### 3. Zoomed Gnomonic Projection with Local Graticule

**Use Case**: Detailed view of a specific sky region (e.g., Galactic center, Crab nebula)

```bash
./map2fig -f sky_map.fits \
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

**Output**: PNG zoomed on region of interest with visible coordinate grid.

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
./map2fig -f galactic_map.fits \
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

**Important**: The overlay coordinate system (e.g., ecliptic) must intersect the visible field of view. For very small FOVs (e.g., < 1 degree), overlay lines may not appear if they don't pass through that region. To ensure visibility, use a larger FOV or position the map center where the overlay coordinates are present.

Example: Ecliptic overlays work best when the center is near the ecliptic plane (around galactic longitude 0° or 180°).

---

### 4b. Gnomonic with Image Rotation

**Use Case**: Rotate the gnomonic projection around the center for custom viewing angles

```bash
./map2fig -f sky_map.fits \
  --projection gnomonic \
  --lon 0 --lat 0 \
  --fov 600 \
  --roll 45 \
  -o rotated_view.png
```

**Parameters**:
- `--roll 45`: Rotate the image 45° counterclockwise around the projection center
- Works with both graticule modes (local and overlay)

---

### 5. Multi-Panel Comparison: Mollweide with Dual Graticules

**Use Case**: Publication figure comparing coordinate systems on full-sky map

```bash
./map2fig -f combined_map.fits \
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

---

### 6. High Dynamic Range with Histogram Equalization

**Use Case**: Revealing subtle features in maps with non-Gaussian distributions

```bash
./map2fig -f survey_map.fits \
  --hist \
  --min 0.1 --max 0.9 \
  --cmap inferno \
  -w 1600 \
  --bg-color "#1a1a1a" \
  -o survey_equalized.pdf
```

**Parameters**:
- `--hist`: Use histogram equalization instead of linear scaling
- `--min/--max`: Define the percentile range (0.1 to 0.9 = 10th to 90th percentile)
- Dark background for astronomy publication style

---

### 7. Coordinate Rotation: Ecliptic View

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

### 8. Negative/Invalid Data Handling

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

### 9. Batch Processing Multiple FITS Files

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

### 10. Asymmetric Scaling for Bipolar Data

**Use Case**: CMB temperature maps or velocity fields (symmetric around zero)

```bash
./map2fig -f cmb_temperature.fits \
  --symlog \
  --linthresh 10 \
  --min -300 --max 300 \
  --cmap RdBu_r \
  --units "Temperature [$\mu$K]" \
  --latex \
  -o cmb_symlog.pdf
```

**Parameters**:
- `--symlog`: Symmetric logarithmic scaling (preserves sign)
- `--linthresh 10`: Linear region around zero (±10 μK) to avoid log(0)
- RdBu reversed colormap (Red-Blue): Red for hot, Blue for cold

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
| `--projection` | `mollweide` (default) or `gnomonic` |
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
| `--grat-par, --grat-mer` | Spacing for parallels/meridians (°) (mollweide) |
| `--grat-labels` | Show lat/lon values on grid lines |
| `--local-graticule` | Local grid for gnomonic (gnomonic only) |
| `--local-grat-dlat, --local-grat-dlon` | Local grid spacing (°) (gnomonic) |

### Styling
| Option | Description |
|--------|-------------|
| `--no-cbar` | Disable colorbar |
| `--no-border` | Disable map border |
| `--transparent` | Transparent background |
| `--bg-color` | Background color |
| `--bad-color` | Color for masked/invalid pixels |
| `--units` | Colorbar units string |
| `--latex` | Enable LaTeX rendering for labels |

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

For batch processing, use shell loops (embarrassingly parallel).

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

Contributions welcome! Common areas:
- New colormaps (see `tools/generate_colormaps.py`)
- Additional projections
- Graticule improvements
- Performance optimizations

## License

[Your license here]

## Citation

If you use this tool in your research, please cite:

```
@software{map2fig2025,
  title={map2fig: Fast HEALPix Visualization in Rust},
  author={Watkins, D.},
  year={2025}
}
```

## See Also

- [HEALPix Documentation](https://healpix.sourceforge.io/)
- [Planck Legacy Archive](https://pla.esac.esa.int/)
- [Cosmoglobe](https://cosmoglobe.uio.no/) - Full-sky maps
