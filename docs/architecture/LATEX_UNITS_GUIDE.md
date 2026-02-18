# LaTeX Units Rendering - User Guide

## Overview
Both PDF and PNG output formats now support LaTeX-formatted units labels on colorbars.

## Quick Start

### Basic Usage
```bash
# Generate PDF with LaTeX units
./map2fig -f input.fits -o output.pdf --latex --units 'Temperature (K)'

# Generate PNG with LaTeX units  
./map2fig -f input.fits -o output.png --latex --units 'Temperature (K)'
```

### With LaTeX Expressions
```bash
# Greek letters and math symbols
./map2fig -f data.fits -o plot.png --latex --units '$T_B$ (K)'

# Complex expressions
./map2fig -f data.fits -o plot.pdf --latex --units '$T_{\\mathrm{CMB}}$ (mK)'

# Multiple units
./map2fig -f data.fits -o plot.png --latex --units '$\\mu K_{RJ}$'
```

## LaTeX Support Matrix

| Format | LaTeX Support | Fallback | Notes |
|--------|--------------|----------|-------|
| PDF    | ✅ Native     | Unicode  | Full LaTeX support via Cairo |
| PNG    | ✅ Rendered   | Stripped | LaTeX is rasterized and composited |

## Features

### What Works
- ✅ Standard LaTeX math mode (between `$...$`)
- ✅ Greek letters: `$\alpha, \beta, \gamma, ...$`
- ✅ Math symbols: `$\mu, \sigma, \partial, \nabla, ...$`
- ✅ Subscripts and superscripts: `$T_B, x^2, ...$`
- ✅ Fractions: `$\frac{a}{b}$`
- ✅ Text mixing: `Temperature ($K$)`

### Known Limitations
- LaTeX rendering requires `pdflatex` and `pdftoppm` system tools
- Very complex LaTeX may fail silently (falls back to plain text)
- Inline math mode only (no display math)

## Examples

### Example 1: CMB Temperature
```bash
./map2fig -f cosmoglobe.fits -o cmb_temp.png \
  --latex \
  --units '$T_{\\mathrm{CMB}}$ (mK)' \
  -w 1200 \
  --colormap hot \
  --log
```

**Output**: PNG with properly formatted CMB temperature label

### Example 2: Flux Density  
```bash
./map2fig -f radio_survey.fits -o flux.pdf \
  --latex \
  --units '$S_\\nu$ (Jy)' \
  -w 1600 \
  --colormap viridis
```

**Output**: PDF with flux density units in proper notation

### Example 3: Rayleigh-Jeans Approximation
```bash
./map2fig -f data.fits -o rj_temp.png \
  --latex \
  --units '$T_{\\mathrm{RJ}}$ ($\\mu K$)' \
  -w 1000 \
  --colormap plasma \
  --gamma 0.8
```

**Output**: PNG with Rayleigh-Jeans temperature with micro symbol

## Troubleshooting

### "pdflatex compilation failed"
**Cause**: Invalid LaTeX syntax or pdflatex not installed

**Solution**:
1. Check LaTeX syntax is valid
2. Install LaTeX: `apt install texlive-latex-base` (Linux) or `brew install basictex` (macOS)
3. Verify installation: `pdflatex --version`

### "pdftoppm not found"
**Cause**: Poppler utilities not installed

**Solution**:
1. Install Poppler: `apt install poppler-utils` (Linux)
2. Or: `brew install poppler` (macOS)
3. Verify: `pdftoppm -v`

### LaTeX units showing as plain text in PNG
**Cause**: LaTeX rendering failed silently, fell back to stripped text

**Solution**:
1. Test LaTeX locally: `pdflatex "\\documentclass{standalone}\\begin{document}\$T\$\\end{document}"`
2. Simplify the LaTeX expression
3. Check system LaTeX installation

## Performance Notes

### Rendering Time
- LaTeX rendering adds **1-3 seconds** per plot (pdflatex compilation)
- Results are cached, so repeated renders are fast
- Cache location: `~/.cache/map2fig/latex_render/`

### File Sizes
- PNG with LaTeX: ~1.0-1.5 MB at 1200px (same as without LaTeX)
- PDF with LaTeX: ~500 KB (same as without LaTeX)

### Cache Management
Clear LaTeX cache if needed:
```bash
rm -rf ~/.cache/map2fig/latex_render/
```

## Comparison with Other Tools

### vs Matplotlib/Basemap
- ✅ Better colorbar labeling control
- ✅ Publication-quality Mollweide projection
- ✅ Faster rendering for large maps

### vs HEALPIX IDL
- ✅ Modern LaTeX support
- ✅ Multiple output formats (PDF/PNG)
- ✅ Open source, no license required

### vs Custom Scripts  
- ✅ Standardized output
- ✅ Consistent styling
- ✅ Production-ready

## Command Reference

### Units Formatting
```bash
# Plain text (no LaTeX processing)
--units 'Temperature (K)'

# LaTeX expressions
--units '$T_B$ (K)'

# Mixed text and LaTeX
--units 'Flux Density $S_\\nu$ (Jy)'

# No units label
# (omit --units flag)
```

### LaTeX Expressions
```bash
# Greek letters
\alpha, \beta, \gamma, \delta, \epsilon, \zeta, \eta, \theta
\lambda, \mu, \nu, \xi, \pi, \rho, \sigma, \tau, \phi, \chi, \psi, \omega

# Math symbols
\partial, \nabla, \times, \cdot, \pm, \infty, \propto

# Special formatting
\mathrm{text}    - Roman/upright text
\mathbf{text}    - Bold
\mathit{text}    - Italic
\mathcal{text}   - Calligraphic
```

## Integration Examples

### Python Script
```python
import subprocess

fits_file = "data.fits"
output_file = "plot.png"
units = "$T_{\\mathrm{CMB}}$ (mK)"

subprocess.run([
    "./map2fig",
    "-f", fits_file,
    "-o", output_file,
    "--latex",
    "--units", units,
    "-w", "1200",
    "--colormap", "hot",
    "--log"
])
```

### Makefile
```makefile
PLOTS = plot1.png plot2.pdf plot3.png

all: $(PLOTS)

%.png: data/%.fits
	./map2fig -f $< -o $@ --latex --colormap hot -w 1200

%.pdf: data/%.fits
	./map2fig -f $< -o $@ --latex --colormap viridis -w 1200

clean:
	rm -f $(PLOTS)
```

## Getting Help

For rendering issues:
```bash
./map2fig --help | grep -A 2 "units\|latex"
```

For system dependencies:
```bash
# Check LaTeX
pdflatex --version

# Check Poppler  
pdftoppm -v

# Check cache size
du -sh ~/.cache/map2fig/
```

## See Also
- [main README](README.md) - General usage
- [LATEX_RENDERING_PNG.md](LATEX_RENDERING_PNG.md) - Technical implementation
- [SESSION_SUMMARY.md](SESSION_SUMMARY.md) - Development notes
