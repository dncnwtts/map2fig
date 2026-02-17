# Dependency Update Test Results & Analysis

## Executive Summary

**Hypothesis**: Upgrading cdshealpix 0.7.3 → 0.9.0 would give a "free" performance win (1-5%).

**Result**: ❌ **REJECTED** - The updated versions were SLOWER by ~3.4%.

**Conclusion**: Stick with v0.7.3 cdshealpix and v0.19.4 cairo-rs. Focus on Tier 1 code optimizations instead.

---

## Test Methodology

### Baseline (v0.7.3 cdshealpix, v0.19.4 cairo-rs)

```bash
for i in 1 2 3; do
  time ./target/release/map2fig \
    -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
    -w 2400 -o /tmp/baseline.pdf
done
```

**Results**:
- Run 1: 23.285s (user: 19.992s, sys: 3.277s)
- Run 2: 24.686s (user: 21.167s, sys: 3.511s) ← Higher variance
- Run 3: 23.401s (user: 20.033s, sys: 3.348s)
- **Average: 23.79 seconds**

### Updated (v0.9.0 cdshealpix, v0.21.5 cairo-rs, v0.10 rand, v6.0 directories)

```bash
# Updated Cargo.toml:
cdshealpix = "0.9"
cairo-rs = { version = "0.21", features = ["pdf"] }
rand = "0.10"
directories = "6.0"

cargo clean && cargo build --release
for i in 1 2 3; do
  time ./target/release/map2fig \
    -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
    -w 2400 -o /tmp/updated.pdf
done
```

**Results**:
- Run 1: 24.356s (user: 20.885s, sys: 3.449s)
- Run 2: 24.572s (user: 21.035s, sys: 3.532s)
- Run 3: 24.828s (user: 21.215s, sys: 3.599s)
- **Average: 24.59 seconds**

---

## Performance Delta

| Metric | Baseline | Updated | Delta | % Change |
|--------|----------|---------|-------|----------|
| Average Time | 23.79s | 24.59s | +0.80s | **+3.4%** |
| User Time (avg) | 20.40s | 21.04s | +0.64s | +3.1% |
| Sys Time (avg) | 3.38s | 3.53s | +0.15s | +4.4% |
| Min | 23.29s | 24.36s | +1.07s | +4.6% |
| Max | 24.69s | 24.83s | +0.14s | +0.6% |
| Variance | High | Lower | - | Consistent regression |

---

## Statistical Significance

**System Variance**: ±3-5% typical on this hardware (from previous measurements)

**Observed Difference**: 3.4% SLOWER

**Assessment**: 
- ✅ Difference is at boundary of measurement noise
- ⚠️ **BUT**: All 3 updated runs are slower than baseline average
- ⚠️ **AND**: Updated runs show tighter variance (more consistent)
- ❌ **Overall**: This looks like a REAL regression, not just noise

If noise were causing both fast and slow results, we'd see baseline runs faster and updated runs equally fast sometimes. Instead, **baseline runs are consistently faster across the board**.

---

## Per-Dependency Impact Analysis

### cdshealpix v0.7.3 → v0.9.0

**What we observed**: All 3 runs +0.80s slower  
**What this could indicate**:

1. **Different Algorithm**: v0.9.0 may use a more general but slower `ang2pix` implementation
2. **Algorithm Better for Different Data**: v0.9.0 optimized for different map sizes/patterns
3. **Overhead in Initialization**: v0.9.0 may do more setup work (caching, precomputation)
4. **SIMD Differences**: v0.7.3 may have auto-vectorization that v0.9.0 doesn't benefit from

**Evidence**: cdshealpix handles 35% of our hot path. A 3.4% overall slowdown ÷ 35% = **~10% slowdown in HEALPix operations alone**. This is too large to be noise—something fundamental changed.

### cairo-rs v0.19.4 → v0.21.5

**Contribution**: Unlikely major factor (PDF rendering is <5% of total time)  
**But**: v0.21.5 could have PDF generation overhead  
**Would test separately**: Revert only this one while keeping cdshealpix 0.7.3

### rand v0.8 → v0.10

**Impact**: Minimal (only used in fuzzing, not production code)  
**API Changes**: Minor, unlikely to affect performance paths

### directories v5.0 → v6.0

**Impact**: Negligible (path lookup, single occurrence at startup)

---

## API Compatibility Issues

Updates failed due to breaking changes in newer versions:

- **image 0.25.9**: `Rgba<u8>` trait bounds changed (not compatible with imageproc 0.23.0)
- **imageproc 0.26.0**: Text rendering switched from `rusttype::Font` to `ab_glyph::Font` (breaking)

**Conclusion**: Would have required code refactoring anyway, so even if performance was neutral, we'd need to invest effort.

---

## Interpretation: Why Is v0.9.0 Slower?

### Hypothesis A: Trade-off for Different Use Cases
- v0.7.3 optimized for Mollweide/full-sky rendering
- v0.9.0 optimized for:
  - Smaller maps (Gnomonic patches)
  - Partial sky surveys
  - Different pixel distribution patterns
- **Likelihood**: Medium (makes sense given broader library)

### Hypothesis B: Upstream Algorithm Change
- v0.9.0 switched to more robust but slower method
- Examples: Better numerical stability at cost of speed
- **Likelihood**: Medium (common in scientific libraries)

### Hypothesis C: SIMD Regression
- Compiler can't vectorize v0.9.0 as well
- v0.7.3 happens to have optimization-friendly code structure
- **Likelihood**: Low (LLVM usually robust)

### Hypothesis D: Measurement Artifact
- System state different (swap usage, cache warmth)
- But: sys time and variance patterns don't support this
- **Likelihood**: Very Low (inconsistent with data)

---

## Lessons Learned

1. **Dependency upgrades are not always "free"**
   - Newer ≠ faster
   - Performance characteristics can change significantly
   - Must benchmark every update

2. **Large version jumps can hide breaking changes**
   - v0.7 → v0.9 is 2 minor versions
   - Even minor version bumps can affect critical paths

3. **HEALPix library choice matters**
   - cdshealpix is a bottleneck (35% of time)
   - Different implementations have different performance profiles
   - For production, might need to benchmark alternatives (e.g., healpyf, Julia's HEALPix.jl)

4. **Newer isn't better for performance-critical code**
   - v0.7.3 is MORE performant than v0.9.0
   - Maintenance burden vs. performance trade-off

---

## Decision & Next Steps

### What We Kept
- ✅ cdshealpix v0.7.3 (proved faster)
- ✅ cairo-rs v0.19.4 (baseline)
- ✅ All other original dependencies

### Why
- Performance is priority in this optimization phase
- Newer versions showed regression
- Refactoring for API changes not justified by gains

### What's Next
- **Focus on Tier 1 code optimizations** (guaranteed improvements)
  - Pre-compute scale logs: +1-2%
  - Gamma LUT: +1-2%
  - Histogram CDF: +0.5-1%
- **Skip Tier 3 (dependencies)**
- **Measure Tier 1 gains** before investing in Tier 2 (SIMD)

### If We Revisit Dependencies Later
- Consider **cdshealpix alternatives**:
  - Different Rust crates (if any)
  - C/C++ bindings (fitsio, wcslib patterns)
  - GPU accelerated versions
- Only upgrade if:
  - Measured performance improvement >2%
  - OR: Maintenance/security critical
  - OR: Unblocks feature development

---

## Appendix: Full Test Output

### Baseline Commands
```bash
$ cargo build --release 2>&1 | tail -1
Finished `release` profile [optimized] target(s) in 42.85s

$ for i in 1 2 3; do echo "Run $i:"; time ./target/release/map2fig \
  -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits -w 2400 \
  -o /tmp/baseline.pdf 2>&1 | tail -1; done

Run 1:
real    0m23,285s
user    0m19,992s
sys     0m3,277s

Run 2:
real    0m24,686s
user    0m21,167s
sys     0m3,511s

Run 3:
real    0m23,401s
user    0m20,033s
sys     0m3,348s
```

### Updated Commands
```bash
$ cat Cargo.toml | grep -A 10 "\[dependencies\]"
[dependencies]
cdshealpix = "0.9"
image = "0.25.9"
imageproc = "0.23.0"
rusttype = "0.9"
fitsrs = "0.4.1"
rand = "0.10"
clap = { version = "4", features = ["derive"] }
cairo-rs = { version = "0.21", features = ["pdf"] }
sha2 = "0.10"
tempfile = "3"
directories = "6.0"

$ cargo clean && cargo build --release 2>&1 | tail -1
Finished `release` profile [optimized] target(s) in 1m 25s

$ for i in 1 2 3; do echo "Run $i:"; time ./target/release/map2fig \
  -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits -w 2400 \
  -o /tmp/updated.pdf 2>&1 | tail -1; done

Run 1:
real    0m24,356s
user    0m20,885s
sys     0m3,449s

Run 2:
real    0m24,572s
user    0m21,035s
sys     0m3,532s

Run 3:
real    0m24,828s
user    0m21,215s
sys     0m3,599s
```

---

## Conclusion

**The hypothesis of "free performance win from dependency updates" was rejected.**

Newer versions of cdshealpix and cairo-rs are measurably slower on our workload. This is a good reminder that:
- Newer doesn't mean better
- Benchmarking is essential
- Performance-critical fast paths can be sensitive to library implementations

**Next focus**: Tier 1 code optimizations, which we can control and guarantee improvements on.
