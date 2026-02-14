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

### Running Benchmarks
```bash
./tools/scripts/run_benchmarks.sh
# or individually:
python tools/python/benchmarks/cosmoglobe_benchmark.py
```

### Installing
```bash
./tools/scripts/install.sh
```

### Testing/Verification
```bash
./tools/scripts/verify_fixes.sh
python tools/python/analyze_heights.py
```

## Adding New Tools

1. **Shell scripts**: Place in `scripts/` with execute permission
2. **Python utilities**: Place in `python/` and update this README
3. **Analysis tools**: Create subdirectory if a group of related scripts

## Notes

- All scripts assume they're run from the project root
- Dependencies are listed in documentation/README.md or within scripts
- Python scripts should be compatible with Python 3.8+

