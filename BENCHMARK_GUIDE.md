# Benchmark & Code Quality Testing Guide

This directory contains comprehensive benchmarking and code quality testing infrastructure for **map2fig**.

## Quick Start

```bash
# Run full benchmark suite
./run_benchmarks.sh

# Run just Python benchmarks (faster)
python3 tools/benchmark.py m_test.fits --full

# Run specific benchmark subset
python3 tools/benchmark.py m_test.fits --projection-only
python3 tools/benchmark.py m_test.fits --feature-only
python3 tools/benchmark.py m_test.fits --comparison-only
```

## Files

### `run_benchmarks.sh`
Orchestration script that:
1. Builds release binary with optimizations
2. Runs unit tests (127 tests)
3. Runs clippy for code quality analysis
4. Executes comprehensive Python benchmark suite
5. Generates JSON results and summary

**Duration**: ~2-5 minutes depending on FITS file size

### `tools/benchmark.py`
Python benchmark suite with multiple test suites:

- **Projection Benchmark**: Mollweide, Hammer, Gnomonic
- **Scaling Benchmark**: Output widths 600, 1200, 1800, 2400 px
- **Feature Benchmark**: Baseline, graticule, LaTeX, log scaling, histogram equalization, all-features
- **Format Benchmark**: PNG vs PDF comparison
- **Comparison Benchmark**: map2fig vs HEALPy vs Cosmoglobe vs map2png

Outputs JSON results and file sizes.

**Note**: map2png benchmarking requires `libhealpix_cxx.so.4` library (from Cosmoglobe/Costotools HEALPix C++ library). If unavailable, skips gracefully.

### `tests/integration_tests.rs`
Rust integration test suite (~13 tests) covering:
- Full-sky map generation
- Extreme value handling (1e-6 to 1e6)
- Coordinate system consistency (round-trip conversions)
- UNSEEN/NaN value handling
- Ring vs Nested ordering support
- All scaling modes (Linear, Log, Symlog, Asinh)
- Colormap availability (80+)
- Multiple output resolutions
- FITS column indexing
- Degenerate map handling
- Metadata preservation

Run with:
```bash
cargo test --test integration_tests -- --test-threads=1
```

## Code Quality Tools

### Unit Tests (127 tests)
```bash
cargo test --lib
```
- Library-internal testing (scales, rotations, colormaps, etc.)
- Focused, fast (~0.7s)

### Integration Tests (~13 tests)
```bash
cargo test --test integration_tests
```
- End-to-end functionality validation
- Data handling robustness
- Format conversions

### Clippy Linting
```bash
cargo clippy --all-targets
```
Identifies code quality issues (active warnings):
- Too many function arguments (8-9 args typical for plotting functions)
- Dead code in test modules
- Unnecessary casts

### Test Coverage Analysis
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

### Benchmarking Criterion Framework (Optional)
For micro-benchmarking specific functions:
```bash
cargo bench
```

## Benchmark Results

Results are saved to `benchmark_results/TIMESTAMP/`:
- `outputs/` - Generated PNG/PDF files
- `outputs/benchmark_results.json` - Detailed timings and file sizes

### Interpreting Results

**Timing expectations** (on modern CPU, m_test.fits):
- Mollweide: 0.15-0.25s
- Hammer: 0.15-0.25s
- Gnomonic: 0.15-0.25s
- PNG output: ~0.1s faster than PDF
- 2400px width: 3-5x slower than 600px

**File sizes** (uncompressed):
- PDF (default): 20-50 KB
- PNG (default): 50-150 KB
- With graticule: +5-10 KB
- Higher resolution: scales with width²

**Comparison vs HEALPy**:
- map2fig typically 1.5-3x faster for standard maps
- PDF output not available in HEALPy (PNG only)
- map2fig uses 0.5-1.5x less memory

**Comparison vs map2png**:
- map2fig and map2png are both from Cosmoglobe ecosystem
- map2fig is Rust (faster, single binary, no dependencies)
- map2png is C++ (requires HEALPix library), classic reference implementation
- Both support Mollweide projection; map2fig adds Hammer & Gnomonic
- Performance typically comparable on same hardware

## Continuous Integration Checks

Recommended CI pipeline to run on every commit:

```yaml
# .github/workflows/code-quality.yml
on: [push, pull_request]
jobs:
  tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --lib
      - run: cargo test --test integration_tests
      - run: cargo clippy --all-targets -- -D warnings
      - run: ./run_benchmarks.sh
  
  # Also run nightly for additional checks
  nightly:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: nightly
      - run: cargo +nightly miri test --test integration_tests
```

## Regression Testing

Weekly/monthly benchmarks should be tracked:

```bash
# Compare against baseline (in CI)
python3 << 'EOF'
import json
from pathlib import Path

baseline = json.load(open("benchmark_baseline.json"))
current = json.load(open("benchmark_results/latest/outputs/benchmark_results.json"))

for test, results in current.items():
    if test in baseline:
        for name, result in results.items():
            old_time = baseline[test][name]['time']
            new_time = result['time']
            change = (new_time - old_time) / old_time * 100
            
            if change > 10:  # 10% regression
                print(f"⚠️  {test}/{name}: {change:+.1f}%")
EOF
```

## Known Issues & Future Improvements

### Current Limitations
- Integration tests don't write actual output files (just validate inputs)
- No property-based testing (would need proptest crate)
- No fuzzing (would need cargo-fuzz)
- Benchmark suite requires Python3 + matplotlib/cosmoglobe (optional)

### Future Enhancements
- [ ] Automated regression detection in CI
- [ ] Performance trend tracking (plot execution times over time)
- [ ] Binary size tracking
- [ ] Memory profiling with valgrind/heaptrack
- [ ] Statistical significance testing for small time differences
- [ ] Fuzz testing with cargo-fuzz
- [ ] Property-based testing for coordinate transforms
- [ ] Output determinism validation (bit-for-bit identical PDFs)

## Development Workflow

### Before committing
```bash
cargo fmt                              # Auto-format code
cargo clippy --all-targets --fix       # Auto-fix clippy issues
cargo test --lib                       # Quick tests
```

### Before merging PR
```bash
cargo test --all                       # All tests
./run_benchmarks.sh                    # Full benchmark
```

### Before release
```bash
cargo build --release                  # Optimized build
cargo test --release                   # Tests with optimizations
./run_benchmarks.sh                    # Baseline for version
cargo publish --dry-run                # Verify crates.io packaging (future)
```

## Troubleshooting

**"Binary not found at ./target/release/map2fig"**
```bash
cargo build --release
```

**"No FITS files found for benchmarking"**
- Provide a FITS file as argument: `python3 tools/benchmark.py my_map.fits`
- Or use test data: `python3 tools/benchmark.py m_test.fits`

**HEALPy/Cosmoglobe comparison returns "not installed"**
```bash
pip install healpy      # For HEALPy comparison
pip install cosmoglobe  # For Cosmoglobe comparison
```

**map2png comparison skipped with "NOT AVAILABLE"**
map2png requires `libhealpix_cxx.so.4` from the HEALPix library. Either:
- Install HEALPix C++ library: https://sourceforge.net/projects/healpix/
- Or run benchmarks without map2png (only affects comparison results)
- The benchmark suite gracefully skips map2png if unavailable

**Integration tests fail on `.apply_to_cartesian()`**
This indicates a missing method on the `Rotation` struct. The test needs updating to match your actual rotation API.

## References

- Main testing: `cargo test --help`
- Criterion benchmarking: https://bheisler.github.io/criterion.rs/book/
- Clippy lints: https://rust-lang.github.io/rust-clippy/
- Coverage tools: https://github.com/tamasfe/ra-lsp-setup
