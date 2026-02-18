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

## v0.3.0 (2026-02-15) - Cairo Batching Optimization

**System**: Linux, Rust 1.92.0, release profile  
**Test File**: `tests/data/class_dr1_40GHz_skymap_n128.fits` (Nside=128)

### Optimization: Cairo Call Batching
Reduced per-pixel Cairo `fill()` calls from 51,456 to ~256 by grouping pixels of the same color.

**Detailed Timing Results**:
| Format | v0.2.0 | v0.3.0 | Delta | % Improvement |
|--------|--------|--------|-------|---------------|
| PDF | 617ms | 470ms | -147ms | **+23.8%** |
| PNG | 173ms | 170ms | -3ms | +1.7% (minimal change expected) |

**Key Metrics**:
- Target improvement: 10-15% minimum
- **Actual achievement: 23.8% for PDF** ✓ Exceeds target
- PNG largely unaffected (uses image crate, not Cairo)
- Cairo fill() calls reduced by 99.5%: 51,456 → ~256 per frame

**Design**:
- New `BatchedCairoImageSink` struct: batches pixels by color in HashMap
- `flush()` method: groups rectangles per color, single `fill()` call per color
- Tradeoff: HashMap overhead minimal vs Cairo API overhead savings
- Output identical to original (same pixels, same colors, same layout)

**Implementation Details**:
- Modified `src/render/pdf.rs`: `blit_raster()` uses `BatchedCairoImageSink`
- Modified `src/plot/mollweide.rs`: main plotting uses `BatchedCairoImageSink`
- HashMap-based batching: O(1) color lookup, ~256 unique colors typical

**Testing**:
- ✅ Compiled successfully with all debug symbols enabled
- ✅ Output visually verified (PDF/PNG identical to baseline)
- ✅ Measurements taken with release build on same test data
- ✅ Improvement stable across multiple runs

**Analysis**:
- Cairo's per-pixel `fill()` calls were bottleneck (confirmed empirically)
- Grouping by color reduces painter/rasterizer work by ~200×
- Demonstrates importance of batching in graphics APIs
- Next opportunity: Consider Option 2 (image surface pre-rendering) for 40-50% improvement

## v0.4.0 (2026-02-15) - Image Pre-rendering Optimization

**System**: Linux, Rust 1.92.0, release profile  
**Test File**: `tests/data/class_dr1_40GHz_skymap_n128.fits` (Nside=128)

### Optimization: Image Pre-rendering (Phase 2B)
Eliminated remaining Cairo surface overhead by rendering pixels to in-memory image buffer first, then embedding as single surface operation.

**Dramatic Timing Results**:
| Format | v0.3.0 | v0.4.0 | Delta | % Improvement |
|--------|--------|--------|-------|---------------|
| PDF | 470ms | 300ms | -170ms | **+36.2%** |
| PNG | 170ms | 160ms | -10ms | +5.9% (unaffected, as expected) |
| **Combined v0.4 vs v0.2.0** | - | - | -317ms | **+51.4%** 🎉 |

**Key Metrics**:
- Target improvement: 10-15% additional (from Phase 2B)
- **Actual achievement: 36.2% for PDF** ✓✓ Far exceeds expectations
- PNG largely unaffected (uses different code path)
- **Total improvement from v0.2.0 to v0.4.0: 51.4%** (617ms → 300ms)

**Design**:
- Replace BatchedCairoImageSink with direct image buffer approach
- Create `RgbaImage` buffer (fast Rust memory, no Cairo)
- Use `PngSink` to write pixels directly (same as PNG rendering path)
- Convert buffer to Cairo surface with `ImageSurface::create_for_data()`
- Paint surface once (single operation, not 256 fill calls)
- No path management overhead at all

**Implementation Details**:
- Modified `src/render/pdf.rs`: `blit_raster()` uses PngSink + ImageSurface::create_for_data()
- Modified `src/plot/mollweide.rs`: main plotting uses PngSink instead of Cairo sink
- Removed dependency on BatchedCairoImageSink (though kept for compatibility)
- Memory: ~4MB temporary buffer for pixel data (negligible cost)

**Testing**:
- ✅ Compiled successfully, no warnings
- ✅ Output verified (PDF 513KB, PNG quality identical)
- ✅ Measurements stable across 3 runs (290-300ms consistent)
- ✅ PNG path unaffected (160ms, essentially same as v0.3.0)

**Why Phase 2B Was So Successful**:
1. **Identified actual bottleneck**: Profiling (perf record) showed cairo_surface_finish at 21.71%
2. **Root cause analysis**: Cairo PDF encoding/compression overhead, not just pixel operations
3. **Architectural insight**: Image pre-rendering bypasses entire Cairo path building
4. **Unexpected benefit**: 36% vs predicted 10-15% because we eliminated more than just fill() overhead
   - Path building overhead
   - Matrix transformation overhead
   - Color state management overhead
   - Compositor overhead

**Analysis**:
- Image pre-rendering was more effective than expected
- v0.4 exceeds v0.3 target by 2.4× (36% vs 15%)
- Combined improvement demonstrates power of empirical profiling
- Remaining opportunities: Phase 2A (SIMD math, ~5-8% more) could target 350ms target

## Future Releases

### v0.5.0: Phase 2A (SIMD Vectorization) - Recommended Next Step

**Opportunity**: HEALPix sampling math (sin/cos/atan2) at 8.64% per profiling
- Vectorize trigonometric operations
- Expected: 350ms target (additional 5-8% from 300ms)
- Effort: 4-6 hours implementation + testing

**Stretch goal**: 300ms PDF rendering (matching PNG for visual maps)

### Structure for tracking improvements:

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
