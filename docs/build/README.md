# Build & Compilation Optimization

Documentation for build-time optimizations and compilation techniques.

## Contents

- **[COMPILATION_OPTIMIZATION.md](COMPILATION_OPTIMIZATION.md)** - Comprehensive guide to faster Rust compilation
  - Mold linker configuration
  - CPU-native optimizations
  - Parallel front-end compilation
  - Full profile optimizations
  - Expected speedups and trade-offs

- **[THIN_LTO_VERIFICATION.md](THIN_LTO_VERIFICATION.md)** - Link-time optimization results
  - Thin LTO implementation and verification
  - Runtime performance impact analysis
  - Comparison with fat LTO

## Quick Start

To apply the compilation optimizations:

1. **Check your current setup**: `cargo build --release 2>&1 | grep Finished`
2. **Enable mold linker**: Follow [COMPILATION_OPTIMIZATION.md](COMPILATION_OPTIMIZATION.md)
3. **Verify improvements**: Run `cargo clean && cargo build --release`

## Performance Targets

- **Baseline**: ~4m 09s (before any optimizations)
- **With mold**:  ~3m 04s (26% improvement)
- **With thin LTO**: ~2m 33s (38% total improvement from baseline)

## Key Techniques

| Technique | Impact | Setup Difficulty |
|-----------|--------|------------------|
| Mold linker | 26% faster | Easy |
| Thin LTO | +12% | Easy |
| Parallel codegen | ~5% | Automatic |
| CPU-native flags | ~2% | Easy |

See [COMPILATION_OPTIMIZATION.md](COMPILATION_OPTIMIZATION.md) for detailed configuration.
