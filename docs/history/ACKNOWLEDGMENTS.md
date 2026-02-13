# Acknowledgments

## Scientific Foundation

**map2fig** is built on decades of astronomical data visualization research. We gratefully acknowledge the contributions of the following projects and people:

### HEALPix Framework
The hierarchical equal area isoLatiitude pixelization (HEALPix) mathematical framework is the foundation of all spherical pixelization and analysis in map2fig.

**Primary Reference:**
- **Górski, K. M., Hivon, E., Banday, A. J., Wandelt, B. D., Hansen, F. K., Reinecke, M., & Bartelmann, M.** (2005)
  - "HEALPix: A Framework for High-Resolution Discretization and Fast Analysis of Data Distributed on the Sphere"
  - *The Astrophysical Journal*, 622(2), 759–771
  - DOI: [10.1086/427976](https://doi.org/10.1086/427976)
  - [Full Paper](https://arxiv.org/abs/astro-ph/0409513)

The HEALPix scheme enables efficient operations on spherical data, which are essential for modern CMB, large-scale structure, and multi-wavelength surveys.

---

### HEALPy Library
The [HEALPy](https://healpy.readthedocs.io) Python library is the primary reference for map2fig's implementation, particularly for:
- **Graticule rendering** algorithms for coordinate grids
- **Mollweide and Hammer projection** mathematics
- **Gnomonic projection** implementation and text label positioning
- **Coordinate transformation** methodologies (healpy/astropy foundations)

**Reference:**
- **Zonca, A., Singer, L. P., Lenz, D., Reinecke, M., Rosset, C., Hivon, E., & Górski, K. M.** (2019)
  - "healpy: equal area pixelization and spherical harmonics transforms for data on the sphere in Python"
  - [GitHub: healpy/healpy](https://github.com/healpy/healpy)
  - [Documentation](https://healpy.readthedocs.io)

Specifically, map2fig's gnomonic projection text rendering (displaying resolution, pixel size, and zoom information) directly parallels HEALPy's `hp.gnomview()` functionality.

---

### Cosmoglobe Package
The [Cosmoglobe](https://github.com/Cosmoglobe/Cosmoglobe) CMB data analysis package provided:
- Feature catalog for comparison and functionality testing
- Performance benchmarking baseline
- Reference implementations for color scaling options
- Test data and validation datasets

**Reference:**
- **Cosmoglobe Collaboration**
  - [GitHub: Cosmoglobe/Cosmoglobe](https://github.com/Cosmoglobe/Cosmoglobe)
  - [Documentation](https://cosmoglobe.readthedocs.io)

---

## Colormaps & Scientific Visualization

### Matplotlib Colormaps
Over 50 colormaps from Matplotlib are integrated into map2fig as precomputed 256-entry RGB lookup tables (LUTs), including:
- **Perceptually Uniform:** Viridis, Plasma, Inferno, Magma
- **Sequential:** Blues, Greens, Reds, YlOrRd, Oranges
- **Diverging:** RdBu, PiYG, coolwarm, RdYlBu, Spectral

**Reference:**
- **Hunter, J. D., & Droettboom, M.** (2007–present)
  - "Matplotlib: Python plotting library"
  - [GitHub: matplotlib/matplotlib](https://github.com/matplotlib/matplotlib)
  - [Colormaps Documentation](https://matplotlib.org/stable/tutorials/colors/colormaps.html)

Matplotlib's colormaps are widely adopted in the scientific community for their perceptual uniformity and accessibility.

### CMasher (Perceptually Uniform Science Colormaps)
Additional science-focused colormaps from CMasher are included for:
- Better perceptual uniformity
- Improved accessibility for colorblind readers
- Specialized visualization needs

**Reference:**
- **van der Velden, E., & Breddels, M. A.** (2020–present)
  - "CMasher: Scientific colormaps for making accessible plots"
  - [GitHub: 1313e/CMasher](https://github.com/1313e/CMasher)
  - [Documentation](https://cmasher.readthedocs.io)

### Planck Mission Colormaps
Custom colormaps designed for Planck CMB temperature and polarization maps:
- **planck**: Primary colormap for intensity maps
- **planck_log**: Logarithmic variant for high dynamic range
- **wmap**: WMAP mission color scheme

**Reference:**
- **Planck Collaboration** (2020)
  - "Planck 2018 results. I. Overview and the legacy release"
  - *Astronomy & Astrophysics*, 641, A1
  - DOI: [10.1051/0004-6361/201833880](https://doi.org/10.1051/0004-6361/201833880)
  - [Full Results](https://www.esa.int/Science_Exploration/Space_Science/Planck/Planck_2018_results)

---

## Coordinate Systems & Rotations

Coordinate system transformations (Galactic ↔ Equatorial ↔ Ecliptic) follow standard astronomical conventions:

**Primary References:**
- **Hipparcos Catalog & FK5 Standard:** Equatorial coordinates with proper epoch handling
- **Equinox J2000.0** for all coordinate transformations
- **Rotation matrices** validated against HEALPy implementations

The coordinate transformations are essential for proper alignment with:
- Galactic plane surveys (CMB, ISM, foreground studies)
- Extragalactic source catalogs (Equatorial FK5/ICRS)
- Zodiacal light and dust analysis (Ecliptic coordinates)

---

## Comparison Tools & Validation

### map2png
The [map2png](https://github.com/Cosmoglobe/Cosmotools) tool (part of the Cosmotools repository, originally written by Sigurd Næss @amaurea) provided essential benchmarking and validation for:
- Output quality consistency
- Performance baseline comparisons
- Feature parity testing

---

## Software Dependencies

map2fig is built on excellent open-source libraries:

### Core Rust Crates
- **[cdshealpix](https://crates.io/crates/cdshealpix)** - HEALPix mathematics in Rust
- **[fitsrs](https://crates.io/crates/fitsrs)** - FITS file I/O
- **[cairo-rs](https://crates.io/crates/cairo-rs)** - Vector graphics & PDF rendering
- **[image](https://crates.io/crates/image)** - PNG and image processing
- **[imageproc](https://crates.io/crates/imageproc)** - Image manipulation (fonts, text)
- **[clap](https://crates.io/crates/clap)** - Command-line argument parsing
- **[rusttype](https://crates.io/crates/rusttype)** - Font rasterization

### Optional Tools
- **[Tectonic](https://tectonic-typesetting.org/)** - Modern LaTeX engine for mathematical rendering
- **[pdflatex](https://www.ctan.org/pkg/latex)** - Extended TeX (fallback)
- **[pdf2svg](https://github.com/jalios/pdf2svg)** - PDF to SVG conversion
- **[ImageMagick](https://imagemagick.org/)** - Image format conversion

---

## Astronomical Standards

map2fig adheres to:
- **FITS Standard** (Flexible Image Transport System) for data I/O
- **HEALPix ring & nest ordering conventions**
- **ICRS/FK5 Equatorial Coordinate System** (J2000.0 epoch)
- **IAU Galactic Coordinate Standard**

---

## Citation Guidelines

If you use **map2fig** in your research, please cite:

### BibTeX Format
```bibtex
@software{map2fig2026,
  Title   = {map2fig: Fast Rust Tool for HEALPix Sky Map Visualization},
  Author  = {Watts, Duncan},
  Url     = {https://github.com/dncnwtts/map2fig},
  Version = {0.1.0},
  Year    = {2026}
}
```

### CITATION.cff Format
See [`CITATION.cff`](CITATION.cff) for machine-readable citation metadata compatible with GitHub, Zenodo, and other archival platforms.

### If Using Specific Features
- **Mollweide/Hammer projections:** Cite HEALPix (Górski et al. 2005) and HEALPy (Zonca et al. 2019)
- **Matplotlib colormaps:** Cite Matplotlib (Hunter & Droettboom)
- **CMasher colormaps:** Cite van der Velden & Breddels (2020)
- **Planck colormaps:** Cite Planck 2018 (Planck Collaboration 2020)
- **Coordinate transforms:** Cite HEALPy (astropy foundations)

---

## Contributors

- **Duncan Watts** – Core development, Rust implementation, refactoring

## Community

Special thanks to the open-source astronomy and Rust communities for creating the ecosystem that makes tools like map2fig possible.

---

## Errata & Corrections

If you find any citations are incorrect or missing, please open an issue at:
[GitHub Issues](https://github.com/dncnwtts/map2fig/issues)

We take proper attribution seriously and will promptly correct any oversights.

---

**Last Updated:** February 2025
