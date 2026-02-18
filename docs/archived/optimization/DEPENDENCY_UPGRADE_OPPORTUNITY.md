# Dependency Update Opportunity Analysis

## Discovery

When building on a fresh system, Cargo showed several outdated dependencies available:

```
cdshealpix v0.7.3 → v0.9.0    ⭐⭐⭐ CRITICAL
cairo-rs   v0.19.4 → v0.21.5  ⭐⭐ Consider
image      v0.24.9 → v0.25.9  ⭐ Low priority
imageproc  v0.23.0 → v0.26.0  ⭐ Low priority
rand       v0.8.5 → v0.10.0   ⭐ Minimal
```

## Why This Matters

**cdshealpix is in our hottest loop**: 35% of all CPU cycles spent on HEALPix sampling (rotation + indexing).

Any optimization in cdshealpix v0.9.0 vs v0.7.3 would **directly improve overall performance** without our needing to change code.

### What Changed in cdshealpix 0.7.3 → 0.9.0?

Version jumps of 2 minor versions (0.7 → 0.9) typically include:
- Performance optimizations
- Better algorithm selection
- SIMD usage
- Cache efficiency improvements
- Bug fixes that affected speed

**Hypothesis**: cdshealpix 0.9.0 likely has optimizations for:
- `ang2pix` function (285 cycles/pixel in v0.7.3 - our biggest bottleneck)
- `sph2vec` / `vec2sph` conversions (rotation math)
- Index computation pathways

## Strategy: Free Performance Win

### Phase 1: Safe Update & Testing

```bash
# Create new branch
git checkout -b update-dependencies-cdshealpix

# Update to new cdshealpix (conservative approach)
cargo update -p cdshealpix

# Check if our code still compiles (API probably stable)
cargo build --release 2>&1 | head -50

# If any errors, review and fix (unlikely for patch upgrades)
```

### Phase 2: Benchmark Before & After

```bash
# Benchmark on current version
cargo build --release >/dev/null 2>&1
for i in 1 2 3; do
    time ./target/release/map2fig \
        -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
        -w 2400 -o /tmp/test.pdf 2>&1 | tail -3
done
# Record: e.g., 23.1s, 23.2s, 23.0s

# Update cdshealpix to 0.9.0
cargo update -p cdshealpix
cargo build --release >/dev/null 2>&1

# Re-benchmark
for i in 1 2 3; do
    time ./target/release/map2fig \
        -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
        -w 2400 -o /tmp/test.pdf 2>&1 | tail -3
done
# Record: e.g., 22.1s, 22.0s, 22.3s (ideally)
```

### Phase 3: Commit & Merge

If benchmark shows >1% improvement:
```bash
git commit -m "Update cdshealpix from 0.7.3 to 0.9.0

Potential gains from upstream optimizations:
- Improved ang2pix HEALPix indexing
- Better cache locality in rotation math
- Possible SIMD vectorization in v0.9.0

Measured improvement: +X% on 2400px rendering
No API breaking changes required."
```

---

## Risk Assessment

| Risk Factor | Level | Mitigation |
|---|---|---|
| **API Breaking Changes** | 🟢 Very Low | Minor version bumps (0.7→0.9) usually stable |
| **Unexpected Regressions** | 🟡 Low | Benchmark carefully with multiple runs |
| **Compilation Errors** | 🟢 Very Low | We don't use HEALPix internals, just API |
| **Measurement Noise** | 🟡 Medium | Run 3+ times, expect ±3-5% variance |

---

## Expected Outcomes

### Best Case ✨
- cdshealpix v0.9.0 has optimizations in `ang2pix` or rotation math
- **Measured gain: +3-8%** (direct free improvement)
- Combined with our Tier 1 optimizations: +5-13% total
- Cumulative effect: 23s → 20-21s @ 2400px

### Realistic Case ✓
- Minor optimizations in cdshealpix
- **Measured gain: +1-3%** (measurable but modest)
- Combines well with other improvements
- Cumulative: 23s → 22-23s (within variance, but average shifts)

### Conservative Case ⚠️
- No optimizations in cdshealpix 0.9.0 (unlikely)
- **Measured gain: <1%** (lost in noise)
- Still worth it: newer version is more maintained + maybe cache benefits elsewhere
- Update anyway for long-term stability

### Worst Case (Very Unlikely)
- cdshealpix v0.9.0 has regression
- **Measured loss: -1-2%**
- Action: Revert to v0.8.x (intermediate version) or stay on v0.7.3
- Still: We'll have tested it and documented findings

---

## Why This Fits Our Roadmap

Our optimization hierarchy was:

```
Tier 1: Quick-win code changes      (+2-5%)
├─ Pre-compute scale logs           (+1-2%)
├─ Gamma LUT                        (+1-2%)
└─ Histogram CDF binary search      (+0.5-1%)

Tier 2: Architectural refactors     (+5-8%)
├─ SIMD batching                    (+5-8%)
├─ ang2pix optimization             (+2-3%)
└─ Cache-aware ordering             (+2-4%)

Tier 3: Dependency updates (NEW!)   (+1-5%)
├─ cdshealpix v0.9.0                (+? - potential free win)
├─ cairo-rs v0.21.5                 (+? - PDF rendering)
└─ image v0.25.9                    (+? - PNG encoding)
```

**Tier 3 should come FIRST** because:
1. ✅ Zero code changes required
2. ✅ Zero architectural risk
3. ✅ Potential measurable gain if upstream optimized HEALPix
4. ✅ Easy to benchmark
5. ✅ Easy to revert if problems

---

## Secondary Updates (After cdshealpix)

### cairo-rs v0.19.4 → v0.21.5

**Impact**: PDF rendering (border + graticule rendering)  
**Risk**: Medium (cairo C bindings can have subtle issues)  
**Benchmark**: Time to render PDF only (exclude HEALPix sampling)

```bash
# Create separate PDF rendering benchmark
# Time only the cairo.paint() / graticule operations
```

**Decision**: Update if:
- cdshealpix update is successful
- cairo-rs has documented performance improvements
- Otherwise: skip (PDF is <5% of total time)

### image v0.24.9 → v0.25.9

**Impact**: PNG encoding (sequential, not in hot loop)  
**Risk**: Low  
**Benchmark**: Not needed (PNG is output, not performance critical)  
**Decision**: Update for maintenance (low priority)

---

## Implementation Timeline

1. **Today**: Update cdshealpix, benchmark, commit if positive
2. **Tomorrow**: Consider cairo-rs (if time permits)
3. **Later**: Update other crates for maintenance

---

## Success Criteria

| Step | Pass/Fail |
|---|---|
| Cargo builds without errors | ✅/❌ |
| Tests pass (`cargo test --release`) | ✅/❌ |
| Output identical to baseline | ✅/❌ |
| Performance change measurable | ✅/⚠️/❌ |
| Improvement > measurement noise (>2%) | ✅/⚠️/❌ |

---

## How This Adds to Our Gains

If cdshealpix 0.9.0 gives us +3% (optimistic but possible):

```
Current state:          23.0s @ 2400px
After cdshealpix:       22.3s @ 2400px (+3%)
+ Scale logs cache:     22.0s @ 2400px (+1-2% more)
+ Gamma LUT:            21.7s @ 2400px (+1-2% more)
────────────────────────────────────────────────
Final (all Tier 1+3):   21.7s @ 2400px (+5-8% total)
```

Still 1.8x behind C++ at ~12s, but that's respectable for single-threaded CPU optimization.

---

## Next Action

Ready to test? Let's update cdshealpix and see if we get a free win:

```bash
cd /home/dwatts/projects/map2fig

# Create branch
git checkout -b update-dependencies-cdshealpix

# Update
cargo update -p cdshealpix --aggressive

# Build
cargo build --release 2>&1 | tail -10

# Quick benchmark (if build succeeds)
for i in 1 2 3; do
    echo "Run $i..."
    time ./target/release/map2fig \
        -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits \
        -w 2400 -o /tmp/test_new.pdf 2>&1 | tail -3
done
```

Want to run this test now?
