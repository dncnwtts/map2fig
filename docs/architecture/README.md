# Architecture & Design Documentation

This directory contains design documents and technical specifications for the HEALPix Plotter's core systems.

## System Design Documents

### Projections
- **[GNOMONIC_GRATICULE_DESIGN.md](GNOMONIC_GRATICULE_DESIGN.md)** - Detailed specification for gnomonic projection graticule calculations
- **[GNOMONIC_TEXT_LABELS.md](GNOMONIC_TEXT_LABELS.md)** - Text positioning and rendering in gnomonic projections

### Output Formats & Rendering
- **[SVG_IMPLEMENTATION.md](SVG_IMPLEMENTATION.md)** - SVG output format design and implementation strategy
- **[SVG_IMPLEMENTATION_SUMMARY.md](SVG_IMPLEMENTATION_SUMMARY.md)** - Summary of SVG feature design
- **[LATEX_RENDERING_PNG.md](LATEX_RENDERING_PNG.md)** - LaTeX text rendering approach for PNG output
- **[OVERLAY_IMPLEMENTATION.md](OVERLAY_IMPLEMENTATION.md)** - Overlay feature design and API

### Layout & Scaling
- **[LAYOUT_SCALING.md](LAYOUT_SCALING.md)** - Figure layout calculations and DPI scaling reference
- **[LATEX_UNITS_GUIDE.md](LATEX_UNITS_GUIDE.md)** - Units system documentation for layout calculations

### Performance
- **[TILE_PARALLELIZATION_DESIGN.md](TILE_PARALLELIZATION_DESIGN.md)** - Design for parallel tile rendering and GPU optimizations

## Design Philosophy

These documents capture:
- **Mathematical foundations** - Coordinate transformation equations, graticule algorithms
- **API design** - Public interfaces and builder patterns
- **Performance considerations** - Parallelization strategies, memory layout
- **Implementation choices** - Why specific approaches were selected

## For Contributors

When implementing new features:
1. Check relevant design documents first
2. Ensure implementation matches documented API
3. Update design documents if requirements change
4. Document edge cases and limitations

## Recent Updates

- Updated SVG implementation design with latest feature set
- Added tile parallelization design for GPU work
- Documented graticule algorithms in detail

---

**Last Updated**: February 2026  
**See Also**: [../README.md](../README.md) for full documentation hub
