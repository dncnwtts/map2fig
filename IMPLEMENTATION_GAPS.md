# Implementation Gaps & Feature Requests

## Features map2fig Could Adopt from map2png

### 1. **Multi-Panel Output** (High Priority)
**Status:** Not implemented in map2fig

map2png's `-ncol` parameter allows outputting multiple signal columns in one command:
```bash
map2png -ncol 2 -signal 1 -signal 2 input.fits output.png
# Produces 2-panel output automatically
```

**Potential map2fig Implementation:**
```bash
./map2fig -f input.fits -o output.pdf --columns 0,1 --layout 2x1
```

**Use Case:** Quickly compare multiple maps in one figure (e.g., temperature + polarization, different frequencies)

**Effort:** Medium (layout engine needed)

---

### 2. **Multi-Map Selection Interface**
**Status:** Not fully exposed in map2fig

map2png explicitly supports HDF multi-map selection:
```bash
map2png -map 1 -map 2 input.hdf output.png
```

**Current map2fig approach:** Use `-i/--col` for single selection, would need extension for explicit multi-map support

**Use Case:** Quickly select subsets from HDF files with multiple maps

**Effort:** Low (parsing already exists)

---

### 3. **Quantile-Based Auto-Scaling**
**Status:** Simple auto-scaling works, quantile version not exposed

map2png's `-auto NUM` parameter:
```bash
map2png -auto 0.1  # Use 10th-90th percentile for auto-range
```

**Current map2fig:** Auto-scaling always finds min/max, no quantile option

**Potential Implementation:**
```bash
./map2fig -f input.fits -o output.pdf --auto-scale-quantile 0.1
```

**Use Case:** Better handling of outliers in data with extreme noise/artifacts

**Effort:** LOW (min/max finding already abstracted)

---

### 4. **Simplified Short Command Option**
**Status:** map2fig uses longer, more explicit options

map2png philosophy: `map2png -option VALUE input.fits output.png` is more discoverable

map2fig tends toward: `./map2fig --long-option-name VALUE -f input.fits -o outfile.pdf` 

**Potential:** Add optional short flags in addition to long options (already partially done with `-f`, `-i`, `-c`, `-w`, `-o`)

**Effort:** Medium (clap rework)

---

## Features map2fig Has That Would Enhance map2png

### 1. **Coordinate System Support**
map2png lacks:
- No way to work with equatorial coordinates
- Fixed to galactic projection
- Cannot overlay different coordinate systems

map2fig offers:
```bash
./map2fig --input-coord eq --output-coord gal --graticule --grat-coord-overlay ecl
```

**Impact:** Essential for cross-survey work (Planck, WMAP, ACT, SPT mixing different coord systems)

---

### 2. **Advanced Graticule with Overlays**
map2png's grid is basic and unstyleable
map2fig offers:
- Primary + secondary graticule overlays
- Custom colors for overlays
- Coordinate labels
- Flexible spacing

---

### 3. **Data Masking**
map2fig supports masking; map2png doesn't

**Applications:**
- Mask galactic plane
- Mask point sources
- Mask bad pixels from mask file
- Clean visualization

---

### 4. **PDF with Vector Graphics**
map2fig can output publication-quality PDFs
map2png outputs PNG only

**Advantage:** Smaller files, scalable graphics, editable in Inkscape/Illustrator

---

## Known Differences in Behavior

### 1. **Column Indexing**
| Tool | Indexing | Example |
|------|----------|---------|
| map2png | 1-based | `-signal 1` selects first column |
| map2fig | 0-based | `-i 0` selects first column |

**Recommendation:** Document prominently or add compatibility flag

---

### 2. **Resolution Units**
| Tool | Parameter | Unit | Behavior |
|------|-----------|------|----------|
| map2png | `-resolution NUM` | arcmin/pixel | Applied to gnomonic projection |
| map2fig | `--res NUM` | arcmin/pixel | Same, but clearer variable name |

**Status:** Compatible

---

### 3. **Colormap Defaults**
| Tool | Default |
|------|---------|
| map2png | planck |
| map2fig | viridis |

**Reason:** map2fig uses matplotlib defaults; map2png tuned for Planck CMB data

**Compatibility Note:** User should explicitly specify for consistent results

---

### 4. **Grid Spacing**
| Tool | Parameter | Default | Unit |
|------|-----------|---------|------|
| map2png | `-grid NUM` | None (disabled) | Degrees |
| map2fig | `--grat-par`, `--grat-mer` | 15° each | Degrees |

**Note:** Different defaults and explicit lat/lon separation in map2fig

---

## Recommended Enhancements for map2fig

### Priority 1 (High Value)
1. ✅ **Quantile auto-scaling** - Adds robustness to outliers
2. ⬜ **Multi-panel output** - Major productivity gain
3. ⬜ **More short flags** - Easier discovery

### Priority 2 (Medium Value)
4. ⬜ **Explicit multi-map selection interface** - Better HDF support
5. ⬜ **Column name support** - For named FITS columns
6. ⬜ **Batch processing mode** - Process multiple files with patterns

### Priority 3 (Nice to Have)
7. ⬜ **Automatic colormap selection** - Based on data characteristics
8. ⬜ **Template/preset system** - Save common option combinations
9. ⬜ **Interactive mode** - Adjust parameters and re-render

---

## Feature Parity Scorecard

### Output Quality
| Feature | map2png | map2fig | Winner |
|---------|---------|---------|--------|
| Color accuracy | ✅ | ✅✅ | map2fig (80+ cmaps) |
| Border quality | ✅ | ✅✅ | map2fig (configurable) |
| Text rendering | ❌ | ✅✅ | map2fig (LaTeX) |
| PDF support | ❌ | ✅✅ | map2fig |

### Flexibility
| Feature | map2png | map2fig | Winner |
|---------|---------|---------|--------|
| Projections | ✅✅ | ✅✅✅ | map2fig (+Hammer) |
| Scaling | ✅✅ | ✅✅✅ | map2fig (+SymLog, AsinH) |
| Coordinate systems | ❌ | ✅✅ | map2fig |
| Grid/Graticule | ✅ | ✅✅✅ | map2fig (overlays) |
| Masking | ❌ | ✅✅ | map2fig |

### Usability
| Feature | map2png | map2fig | Winner |
|---------|---------|---------|--------|
| Command brevity | ✅✅✅ | ✅✅ | map2png |
| Option discovery | ✅✅ | ✅ | map2png (short flags) |
| Multi-panel | ✅✅ | ❌ | map2png |
| Help clarity | ✅ | ✅✅ | map2fig (detailed docs) |

### Performance
| Feature | map2png | map2fig | Winner |
|---------|---------|---------|--------|
| Speed | ✅ | ✅✅✅ | map2fig (27% faster) |
| Memory | ✅ | ✅ | Tie |

---

## CLI Syntax Comparison

### Quick Plot Example

**map2png:**
```bash
map2png input.fits output.png
```

**map2fig:**
```bash
./map2fig -f input.fits -o output.png
```

**Winner:** map2png (more intuitive positional args)

---

### Log Scale with Limits

**map2png:**
```bash
map2png -logarithmic -minimum 1e-6 -maximum 1e-3 input.fits output.png
```

**map2fig:**
```bash
./map2fig -f input.fits -o output.png --log --min 1e-6 --max 1e-3
```

**Winner:** Similar, slight brevity to map2png

---

### Advanced Visualization

**map2png:**
```bash
map2png -color plasma -grid 10 -glat 5 -glon 5 -logarithmic \
  -minimum 1e-6 -maximum 1e-3 -bar -ticks 2 \
  input.fits output.png
```

**map2fig:**
```bash
./map2fig -f input.fits -o output.pdf -c plasma \
  --graticule --grat-par 5 --grat-mer 5 --log \
  --min 1e-6 --max 1e-3 --extend both \
  --tick-direction inward
```

**Winner:** map2fig (more options available, clearer intent)

---

## Migration Path for map2png Users

### Minimal Equivalent
```bash
# map2png
map2png input.fits output.png

# map2fig
./map2fig -f input.fits -o output.pdf
```

### With Log Scaling
```bash
# map2png
map2png -logarithmic -minimum 1e-6 -maximum 1e-3 input.fits output.png

# map2fig
./map2fig -f input.fits -o output.pdf --log --min 1e-6 --max 1e-3
```

### With Grid
```bash
# map2png
map2png -grid 15 input.fits output.png

# map2fig
./map2fig -f input.fits -o output.pdf --graticule --grat-par 15 --grat-mer 15
```

### Note for Multi-Panel Users
map2png supports:
```bash
map2png -ncol 2 -signal 1 -signal 2 input.fits output.png
```

map2fig requires separate commands:
```bash
./map2fig -f input.fits -o panel1.pdf -i 0
./map2fig -f input.fits -o panel2.pdf -i 1
# Then combine with ImageMagick or similar
```

---

## Conclusion

**map2fig is the more advanced tool** with superior features and performance, but **map2png retains advantages in simplicity and multi-panel capability**. 

For users transitioning from map2png → map2fig:
1. Learn the long-form option names (more explicit)
2. Use 0-based column indexing instead of 1-based
3. Change default colormap from planck → viridis
4. For multi-panel: use separate commands instead of `-ncol`
5. Gain: coordinate systems, data masking, PDF output, LaTeX support

