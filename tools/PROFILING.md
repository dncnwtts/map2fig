# Profiling Guide

This document describes how to profile `map2fig` and identify performance bottlenecks.

## Prerequisites

### Linux (flamegraph + perf)
```bash
# Install flamegraph
cargo install flamegraph

# Install perf (if not already installed)
sudo apt-get install linux-tools-generic  # Ubuntu/Debian
sudo dnf install perf                      # Fedora
```

### macOS (Instruments)
Built into Xcode; use `cargo instruments` if installed.

### All Platforms (time + memory)
These use standard Rust tools.

## Profiling Methods

### 1. Flamegraph (Linux - Best for finding bottlenecks)

Generate a flamegraph to visualize where time is spent:

```bash
# Build with debug symbols for accurate profiling
cargo flamegraph --bin map2fig -- -f tests/data/class_dr1_40GHz_skymap_n128.fits -o /tmp/test.pdf

# Results in flamegraph.svg
# Open in browser: firefox flamegraph.svg
```

**What to look for:**
- Wide blocks = functions consuming lots of time
- Stack depth shows call hierarchy
- Colors are random, don't indicate anything (look at area/width)

### 2. Time Profiling (All platforms)

Compare rendering time across different inputs and scaling modes:

```bash
# Run the benchmark suite
python tools/python/benchmarks/cosmoglobe_benchmark.py

# Or run individual timing tests
time ./target/release/map2fig -f tests/data/class_dr1_40GHz_skymap_n128.fits -o /tmp/test.pdf
time ./target/release/map2fig -f tests/data/class_dr1_40GHz_skymap_n128.fits --log -o /tmp/test_log.pdf
time ./target/release/map2fig -f tests/data/class_dr1_40GHz_skymap_n128.fits --hist -o /tmp/test_hist.pdf
```

### 3. Memory Profiling (Linux - Valgrind)

```bash
# Install valgrind
sudo apt-get install valgrind  # Ubuntu/Debian
sudo dnf install valgrind       # Fedora

# Profile memory usage
valgrind --tool=massif ./target/release/map2fig -f examples/cosmoglobe_clipped.fits -o /tmp/test.pdf

# Analyze results
ms_print massif.out.<pid>
```

### 4. Custom Timing with --release

Always profile with release builds:

```bash
cargo build --release

# Time a specific operation
time ./target/release/map2fig -f large_map.fits --log -o output.pdf
```

## Performance Baseline

Before/after changes, run these standard benchmarks:

```bash
# Small map (quick baseline)
time ./target/release/map2fig -f tests/data/class_dr1_40GHz_skymap_n128.fits -o /tmp/test.pdf

# Medium map
time ./target/release/map2fig -f tests/data/cosmoglobe_DIRBE_06_I_n00512_DR2.fits -o /tmp/test.pdf

# Different scaling modes (all on same file)
time ./target/release/map2fig -f tests/data/class_dr1_40GHz_skymap_n128.fits --log -o /tmp/test_log.pdf
time ./target/release/map2fig -f tests/data/class_dr1_40GHz_skymap_n128.fits --hist -o /tmp/test_hist.pdf
time ./target/release/map2fig -f tests/data/class_dr1_40GHz_skymap_n128.fits --asinh -o /tmp/test_asinh.pdf
```

## Pre-Release Profiling Checklist

Before every release:

1. **Build release binary**
   ```bash
   cargo build --release
   ```

2. **Run flamegraph on representative data**
   ```bash
   cargo flamegraph --bin map2fig -- -f examples/cosmoglobe_clipped.fits -o /tmp/test.pdf
   # Save flamegraph.svg with version tag: flamegraph_v0.2.0.svg
   ```

3. **Run benchmark suite**
   ```bash
   python tools/python/benchmarks/cosmoglobe_benchmark.py | tee perf_v0.2.0.txt
   ```

4. **Compare against previous version**
   - Check if times have regressed significantly
   - Update PERFORMANCE_TRACKING.md

5. **Document findings**
   - Record any hotspots discovered
   - Note areas for future optimization

## Interpreting Results

### Flamegraph Analysis
- **Wide blocks** = high CPU time
- **Tall stacks** = deep call chains (consider inlining)
- **Fragmented blocks** = sporadic, small calls (overhead)

Common hotspots in map2fig:
- `project_pixel()` - Projection math
- `scale_value()` - Data scaling
- `render_pixel()` - Color mapping and rasterization
- `read_fits()` - File I/O (input overhead, not render time)

### Timing Analysis
- Linear scaling with Nside² = expected behavior
- Superlinear increase = cache misses or algorithm issues
- Sublinear = good parallelization

## Future Optimization Opportunities

Common optimization techniques to guide profiling:

1. **SIMD** - Vectorize projection math (portable_simd)
2. **Cache locality** - Better memory layout in pixel arrays
3. **Parallelization** - Fine-tune Rayon thread count
4. **Algorithm selection** - Use faster algorithms for specific cases
5. **Allocation reduction** - Minimize temporary allocations in hot paths

## Tools Reference

| Tool | Platform | Best For | Command |
|------|----------|----------|---------|
| flamegraph | Linux | Finding bottlenecks | `cargo flamegraph -- ...` |
| perf | Linux | Detailed CPU stats | `perf record` + `perf report` |
| Instruments | macOS | Comprehensive profiling | Xcode built-in |
| Valgrind | Linux | Memory profiling | `valgrind --tool=massif` |
| time | All | Quick timing | `time ./binary` |

## References

- [Flamegraph guide](https://www.brendangregg.com/flamegraphs.html)
- [Cargo flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [Rust Profiling Book](https://nnethercote.github.io/perf-book/)
