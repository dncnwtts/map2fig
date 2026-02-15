# map2fig v0.5.0 Release Notes

**Release Date:** February 15, 2026

## Major Features & Improvements

### 🚀 Performance Optimizations (Tiers 1-3)

This release includes substantial performance improvements across the entire rendering pipeline:

- **Tier 1: Buffer Elimination** - Removed intermediate `Vec<DataValue>` buffer in sparse FITS column extraction
  - **Result:** 30-35% speedup on data loading
  
- **Tier 2: Memory-Mapped I/O** - Enabled mmap-based FITS file reading
  - **Result:** 20-21% additional speedup via zero-copy kernel I/O
  - **Combined with Tier 1:** 51.5% total improvement on data loading (22.58s → 10.94s)

- **Tier 3: SIMD Scale Vectorization** - Vectorized scaling operations for non-linear transformations
  - Added `simd_symlog_scale_8()` for symmetric logarithmic scaling
  - Added `simd_asinh_scale_8()` for inverse hyperbolic sine scaling  
  - Added `simd_plancklog_scale_8()` for PlanckLog transformations
  - **Result:** 1.2-1.3% improvement for Symlog/Asinh scales, ~0.05% overall (already dominated by Mollweide projection at 77.5% CPU)

### Overall Performance Impact

For a typical 3.1GB HEALPix file (nside 8192):
- **Before:** 22.58 seconds
- **After:** 10.87 seconds
- **Total Improvement:** 51.6% speedup

## Code Quality

### ✅ Code Cleanup & Safety
- Fixed all clippy warnings (uninit_vec, let_unit_value)
- Removed unsafe `set_len()` in buffer initialization, replaced with safe `vec![0u8; ...]`
- All code formatted with `cargo fmt`
- Zero compiler warnings

### ✅ Test Suite
- 171/173 unit tests passing (2 ignored Hammer roundtrip tests - expected)
- All benchmark and integration tests verified
- Performance regression testing completed

## Technical Details

### Memory-Mapped I/O (Tier 2)
- Eliminated kernel memcpy overhead by using `MmapFitsReader`
- Synergistic with Tier 1 buffer elimination for maximum effect
- Cache miss reduction: 36.67% → 27.67% (24.5% improvement)

### SIMD Scale Vectorization (Tier 3)
- Processes 8 values in parallel using instruction-level parallelism (ILP)
- Unrolled loops for CPU pipelining
- Fallback to scalar path for Histogram scale (binary search cannot be vectorized)

### Data Loading Architecture
- Sparse FITS maps with EXPLICIT indexing now use Rayon parallelization
- Dense maps optimized for cache-friendly sequential access patterns
- Zero-copy buffer handling throughout pipeline

## Architecture Insights

### Current Performance Bottleneck
- **Mollweide Projection:** 77.5% of CPU time (algorithmic bottleneck)
- **Data Loading:** 32% of pipeline time (now heavily optimized)
- **Scaling Operations:** <1% of CPU time (tier 3 target)
- **Cairo/PNG Rasterization:** 3.57× slower than native PNG rendering

### Optimization Roadmap
Future optimizations with projected impact:
- **Tile-based Parallelization:** 1.78-2.46× speedup (8-25 hours work, GPU-quality ROI)
- **GPU Acceleration (CUDA):** 2.5-2.8× speedup (40 hours work, best long-term option)
- **Tier 4-5 CPU Work:** Negligible ROI (diminishing returns due to Mollweide bottleneck)

## Dependencies
- `cdshealpix` 0.6.x - HEALPix coordinate mathematics
- `fitsrs` 0.5.x - FITS binary table reading
- `cairo-rs` 0.19.x - PDF rendering
- `image` 0.25.x - PNG and image processing
- `rayon` 1.7.x - Work-stealing parallelism
- `clap` 4.5.x - Command-line argument parsing

## Breaking Changes
None. This is a fully backward-compatible release.

## Installation

```bash
cargo install map2fig@0.5.0
```

Or build from source:
```bash
git clone https://github.com/dncnwtts/map2fig.git
cd map2fig && git checkout v0.5.0
cargo build --release
```

## Usage Examples

```bash
# Basic Mollweide projection
map2fig -f cosmoglobe.fits -o map.pdf

# With custom scaling and colormap
map2fig -f data.fits --log --min 1e-6 --max 1e-3 -c plasma --gamma 0.8

# Hammer projection with symlog scaling
map2fig -f data.fits --hammer --symlog

# Gnomonic projection with native resolution
map2fig -f data.fits --gnomonic --native --width 2000
```

## Documentation
See [docs/optimization/](docs/optimization/) for detailed performance analysis including:
- Tier 1-3 implementation details
- Benchmarking methodology and results
- Performance ceiling analysis
- Memory profiling data

## Contributors
- Duncan Watts (@dncnwtts) - Core development and optimization

## License
MIT

---

**Next Steps:** The next major optimization target is tile-based parallelization for multi-core speedup on 4+ core systems. GPU acceleration via CUDA is also planned for maximum speedup potential.
