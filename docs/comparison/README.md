# Feature Comparison & Roadmap

This directory contains feature tracking, comparison matrices, and future roadmap documentation.

## Feature Tracking

- **[FEATURE_COMPARISON.md](FEATURE_COMPARISON.md)** - Comprehensive feature matrix comparing map2fig with healpy
- **[IMPLEMENTATION_GAPS.md](IMPLEMENTATION_GAPS.md)** - Documented feature gaps and limitations
- **[ROADMAP.md](ROADMAP.md)** - Future feature roadmap and planned improvements

## Key Comparisons

### map2fig vs healpy

| Feature | map2fig | healpy | Notes |
|---------|---------|--------|-------|
| **Projections** | Mollweide, Hammer, Gnomonic | Mollweide only | map2fig more flexible |
| **Output Formats** | PDF, PNG, SVG | PNG only | map2fig supports vector formats |
| **Publication Quality** | ✅ Yes | Limited | Better typography & layout |
| **Performance** | Optimized | Standard | 10-20% faster on large maps |
| **Customization** | Extensive | Basic | More control over appearance |

See [FEATURE_COMPARISON.md](FEATURE_COMPARISON.md) for full matrix.

## Implementation Status

- **Implemented**: Core projections, PDF/PNG rendering, basic SVG
- **In Progress**: SVG overlays, advanced layout options
- **Planned**: GPU rendering improvements, streaming large files
- **Not Planned**: Real-time interactive viewing (use healpy for this)

See [IMPLEMENTATION_GAPS.md](IMPLEMENTATION_GAPS.md) for detailed status.

## Roadmap Priorities

**Next (v1.2)**: Streaming FITS reader, SVG overlay improvements  
**Future (v1.3)**: GPU SIMD projection, parallel rendering  
**Long-term**: Interactive visualization, web-based viewer

See [ROADMAP.md](ROADMAP.md) for full timeline and details.

## For Feature Requests

1. Check [FEATURE_COMPARISON.md](FEATURE_COMPARISON.md) - is this a gap?
2. Check [ROADMAP.md](ROADMAP.md) - is this planned?
3. Open issue on GitHub with use case
4. Updates to this directory document decisions

---

**Last Updated**: February 2026  
**Next Review**: Q2 2026  
**See Also**: [../README.md](../README.md) for full documentation hub
