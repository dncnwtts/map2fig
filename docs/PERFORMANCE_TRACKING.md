# Performance Tracking

Systematic performance metrics tracked across releases to monitor optimization progress.

## v0.2.0 (2026-02-15) - Initial Release Baseline

**System**: Linux, Rust 1.92.0, default profile  
**Test File**: `tests/data/class_dr1_40GHz_skymap_n128.fits` (Nside=128)

| Test | Time | Notes |
|------|------|-------|
| Default (Linear) | 0.623s | Baseline linear scaling |
| Log Scale | 1.176s | Logarithmic data transformation |
| Histogram Equalization | 0.626s | Adaptive scaling |
| Asinh Scale | 0.608s | Inverse hyperbolic sine |
| Symlog Scale | 0.506s | Symmetric log scaling |
| Medium Map (n=512) | 1.075s | Larger dataset test |

**Optimizations in Release**:
- Tier 5.4: Adaptive masking for unseen pixels (9% improvement on masked maps)
- Tier 5.2: Column data binary caching (81.7% improvement on repeated runs)
- Zero-pixel detection: Threshold-based vs exact comparison (stability improvement)

**Known Issues**: None blocking performance

## Future Releases

Structure for tracking improvements:

### vX.Y.Z (YYYY-MM-DD) - Release Notes

**System**: OS, Rust version, profile used  
**Key Changes**:
- Feature/optimization description
- Expected impact

| Test | Time | Delta vs v0.2.0 | Notes |
|------|------|-----------------|-------|
| Default (Linear) | TBD | TBD | |
| Log Scale | TBD | TBD | |

**Analysis**:
- Hotspots identified: [describe flamegraph findings]
- Optimization opportunities: [what was improved]
- Next focus: [what to optimize next]

---

## Performance Analysis Process

Before each release:

1. **Build Release**
   ```bash
   cargo build --release
   ```

2. **Run Profiling**
   ```bash
   ./tools/scripts/profile.sh
   ```

3. **Generate Flamegraph** (Linux)
   ```bash
   cargo flamegraph --bin map2fig -- -f cosmoglobe_clipped.fits -o /tmp/test.pdf
   ```

4. **Compare Against Previous**
   - Look for regressions (> 5% slowdown)
   - Document improvements

5. **Update This Document**
   - Add new version section
   - Document findings in Analysis

## Optimization Targets

Based on analysis, these are areas for potential improvement:

### High Priority
- [ ] Projection math (project_pixel) - SIMD vectorization
- [ ] Pixel rasterization loop - cache locality
- [ ] Memory allocation patterns in render path

### Medium Priority
- [ ] Scaling function optimization - lookup tables vs computation
- [ ] Colormap interpolation efficiency
- [ ] File I/O buffering

### Low Priority (Pre-GPU)
- [ ] Algorithm selection based on Nside
- [ ] Parallel chunk sizing tuning
- [ ] Compile-time specialization

## Tools Used

- **flamegraph** - Visualizes CPU time distribution
- **perf** - Linux CPU profiling
- **time** - Simple wall-clock timing
- **Valgrind** - Memory profiling

See [PROFILING.md](PROFILING.md) for detailed profiling instructions.

## Release Checklist

- [ ] Run `./tools/scripts/profile.sh`
- [ ] Compare timings against previous version
- [ ] Generate flamegraph (if Linux)
- [ ] Update PERFORMANCE_TRACKING.md
- [ ] Check for any regressions > 5%
- [ ] Commit performance summary
