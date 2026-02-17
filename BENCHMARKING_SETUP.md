# Benchmarking Infrastructure Setup

## Overview

Completed implementation of professional-grade benchmarking toolkit with three complementary tools:

1. **Hyperfine** - End-to-end statistical benchmarking with confidence intervals
2. **Criterion** - Detailed micro-benchmarks with HTML reports and regression detection
3. **Bencher** - CI/CD integration for continuous performance tracking

## Tools Comparison

| Tool | Purpose | Granularity | Time | Strength |
|------|---------|-------------|------|----------|
| **Hyperfine** | End-to-end | Binary execution | 10-15 min | Statistical rigor, variance analysis |
| **Criterion** | Micro benchmarks | Function-level | 5-10 min | HTML reports, per-run analysis |
| **Divan** | Quick micro benchmarks | Function-level | <1 min | Cycle-accurate, fast iteration |
| **Bencher** | CI regression tracking | Both | CI integration | Performance history, alerts |

## How to Use

### Local Benchmarking

Quick start:
```bash
# Run comprehensive test suite
./benches/run_benchmarks.sh all

# Or run individual suites
./benches/run_benchmarks.sh divan       # Fastest (30 sec)
./benches/run_benchmarks.sh criterion   # Detailed (5-10 min)
./benches/run_benchmarks.sh e2e         # Statistical (10-15 min)
```

### Manual Commands

```bash
# Divan (cycle-accurate, fast)
cargo bench --bench divan_benchmarks

# Criterion (with HTML reports)
cargo bench --bench criterion_benchmarks
# Output: target/criterion/report/index.html

# Hyperfine (end-to-end with statistical analysis)
bash benches/hyperfine_benchmarks.sh
# Output: /tmp/hyperfine_results.json and .md
```

### CI Integration (Bencher)

Bencher integration is configured in `.github/workflows/benchmarks.yml` and `bencher.toml`. 

To enable:
1. Set `BENCHER_API_TOKEN` secret in GitHub repo settings (sign up at bencher.dev)
2. Push to main or create PR - benchmarks run automatically
3. Results appear as PR comments with regression warnings

## Benchmark Targets

### Micro-benchmarks (Criterion/Divan)
- `pix2ang_ring` - Pixel to angle conversion (billions of calls per render)
- `ang2pix_ring` - Angle to pixel conversion (inverse transform)
- Downgrade operation - Nside reduction with parallelization

### End-to-End (Hyperfine)
Files ranging from 0.7 MB to 3.1 GB:
- Small (6.8 MB, nside=128)
- Medium (72 MB, nside=512)
- Large (576 MB, nside=512)
- Huge (3.1 GB, nside=8192)

## Key Metrics

### Hyperfine Output
- Wall-clock time (what users perceive)
- Mean, median, min, max
- 95% confidence intervals
- Variance and standard deviation

### Criterion Output
- Time per iteration
- Throughput (ops/sec)
- R² regression statistics
- Outlier detection

### Divan Output
- Cycles (CPU-independent measurement)
- Allocs per iteration
- Cache behavior hints

## Performance Book Reference

These tools implement best practices from the [Rust Performance Book](https://nnethercote.github.io/perf-book/):

- ✓ Multiple workloads (small to huge files)
- ✓ Realistic inputs (actual FITS files)
- ✓ Low-variance metrics (statistical analysis)
- ✓ Warmup runs (cache stabilization)
- ✓ Careful measurement (confidence intervals)
- ✓ CI tracking (regression detection)

## Next Steps

After benchmarking infrastructure is validated:

1. **Establish baseline** with current code
2. **SIMD aggregation** - Vectorize coordinate conversions (Tier 2)
3. **Coordinate caching** - Cache pix2ang results (Tier 2.1)
4. **Re-benchmark** - Validate improvements with statistical confidence

## Files Added

```
.github/workflows/benchmarks.yml    # CI/CD workflow
bencher.toml                         # Bencher config
benches/
  ├── criterio n_benchmarks.rs      # Criterion harness
  ├── divan_benchmarks.rs           # Divan harness
  ├── hyperfine_benchmarks.sh       # Shell-based E2E tests
  └── run_benchmarks.sh             # Convenience wrapper
```

## Notes

- All benchmarks use `black_box()` to prevent compiler optimizations from skewing results
- Criterion auto-detects sample size for stable measurements
- Divan uses inline profiling for cycle-level accuracy
- Hyperfine filters outliers and reports confidence intervals (95% by default)
