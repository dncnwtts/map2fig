# Tools & Utilities

This directory contains utility scripts and tools for development, testing, and analysis.

## Structure


### 📁 [scripts/](scripts/)
Shell scripts for common tasks:

| Script | Purpose |
|--------|---------|
| `install.sh` | Installation and setup helper |
| `run_benchmarks.sh` | Run comprehensive performance benchmarks |
| `benchmark_quick.sh` | Run quick performance benchmarks |
| `compare_map2png_vs_map2fig.sh` | Benchmark map2fig PNG output against map2png |
| `profile.sh` | Performance profiling and baseline measurements |
| `run_tests.sh` | Run test suite |
| `plot_rotation.sh` | Plotting utility for coordinate rotation |
| `verify_fixes.sh` | Verification test suite |

### 🐍 [python/](python/)
Python utility scripts for development and analysis:

| Script | Purpose |
|--------|---------|
| `benchmark.py` | Performance benchmarking utilities |
| `analyze_heights.py` | Triangle height geometry analysis |
| `compare_outputs.py` | Compare test outputs and results |
| `compare_pdf_png.py` | Compare PDF vs PNG rendering |
| `create_mask_example.py` | Generate example mask FITS files |
| `debug_coordinates.py` | Debug coordinate transformations |
| `HEIGHT_ANALYSIS.py` | Detailed height/geometry metrics |
| `verify_height_fix.py` | Verify triangle rasterization fixes |

### 📊 [python/benchmarks/](python/benchmarks/)
Performance comparison scripts:

| Script | Purpose |
|--------|---------|
| `cosmoglobe_benchmark.py` | Benchmark on Cosmoglobe maps |
| `mollview_benchmark.py` | Compare with healpy's mollview |

## Usage

### Using the Makefile (Recommended)
The easiest way to perform common tasks is via the Makefile in the project root:

```bash
make build          # Build the project (cargo build --release)
make test           # Run test suite
make test-fuzz      # Run fuzzing tests (requires Rust nightly)
make clean          # Remove build artifacts
make help           # Show available targets
```

### Running Benchmarks
```bash
python tools/python/benchmarks/cosmoglobe_benchmark.py
# or:
python tools/quick_bench.py
```

### Testing/Verification
```bash
./tools/scripts/verify_fixes.sh
python tools/python/analyze_heights.py
```

### Performance Profiling
```bash
./tools/scripts/profile.sh
# Generates perf_report_v*.txt with baseline timings and flamegraph_v*.svg (Linux)
```

For detailed profiling methodology, see [PROFILING.md](PROFILING.md).

## Benchmarking Against map2png

The `compare_map2png_vs_map2fig.sh` script compares map2fig PNG output performance against the legacy map2png tool. This is a valuable comparison baseline as map2png is widely used in the astronomy community.

### map2png Dependencies

**map2png requires the following C++ libraries:**
- `libhealpix_cxx` - HEALPix C++ library
- `libhdf5` - HDF5 storage format library

**Library paths (auto-configured from Cosmotools build):**
```bash
HEALPIX_LIB=/home/dwatts/Downloads/Healpix_3.83/lib
HDF5_LIB=/home/dwatts/local/hdf5-2.0.0/lib
```

### map2png Command-Line Interface

```bash
map2png [options] input.fits output.png
```

**Key options:**
| Option | Purpose |
|--------|---------|
| `-minimum NUM` | Color scale minimum |
| `-maximum NUM` | Color scale maximum |
| `-linear` | Linear color scale (default) |
| `-logarithmic` | Logarithmic color scale |
| `-histogram` | Histogram equalization |
| `-mollweide` | Mollweide projection (default) |
| `-gnomonic` | Gnomonic projection |
| `-bar` / `-nobar` | Show/hide color bar |
| `-grid NUM` | Grid spacing in degrees |
| `-color SPEC` | Colormap: planck, wmap, viridis, plasma, turbo, etc. |
| `-xsz INT` | Image width in pixels (height = width/2) |
| `-xsize INT` | Alternative to `-xsz` |

For complete options, run: `map2png` (no args)

### Running map2png Comparison

1. **Basic usage (auto-detects libraries):**
   ```bash
   ./tools/scripts/compare_map2png_vs_map2fig.sh
   ```

2. **Custom library paths (if needed):**
   ```bash
   export MAP2PNG_LIBPATH="/custom/path/lib:$LD_LIBRARY_PATH"
   ./tools/scripts/compare_map2png_vs_map2fig.sh
   ```

3. **Custom map2png binary location:**
   ```bash
   export MAP2PNG_PATH="/path/to/map2png"
   ./tools/scripts/compare_map2png_vs_map2fig.sh
   ```

If map2png dependencies are unavailable, the script gracefully skips comparison and generates standalone map2fig benchmarks.

## Adding New Tools

1. **Shell scripts**: Place in `scripts/` with execute permission
2. **Python utilities**: Place in `python/` and update this README
3. **Analysis tools**: Create subdirectory if a group of related scripts

## Notes

- All scripts assume they're run from the project root
- Dependencies are listed in documentation/README.md or within scripts
- Python scripts should be compatible with Python 3.8+

