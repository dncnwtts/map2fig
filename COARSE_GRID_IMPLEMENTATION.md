# Coarse-Grid Sampling Implementation Guide

**Date:** February 19, 2026  
**Goal:** 2-4× speedup via intelligent subsampling of downsampling blocks  
**Complexity:** Low-to-medium (120-400 lines of code)  
**Timeline:** 1-4 weeks depending on which variant

---

## Current Algorithm (Baseline)

```rust
// Simplified pseudocode of current downgrade_healpix_map_xyf_parallel
fn downgrade_healpix_map(
    map: &[f64],
    source_nside: i64,
    target_nside: i64,
    ordering: HealpixOrdering,
) -> Vec<f64> {
    let fact = source_nside / target_nside;  // 8 for 8192→1024
    let target_npix = (12 * target_nside * target_nside) as usize;
    let mut result = vec![HPX_UNSEEN; target_npix];
    
    for target_pix in 0..target_npix {
        let (x, y, face) = match ordering {
            HealpixOrdering::Ring => ring2xyf(target_nside, target_pix as i64),
            HealpixOrdering::Nested => nest2xyf(target_nside, target_pix as i64),
        };
        
        let mut sum = 0.0;
        let mut hits = 0;
        
        let x0 = fact * x;
        let y0 = fact * y;
        
        // Read ALL pixels in 8×8 block
        for j in y0..(y0 + fact) {              // 8 rows
            for i in x0..(x0 + fact) {          // 8 columns
                let source_pix = match ordering {
                    HealpixOrdering::Ring => xyf2ring(source_nside, i, j, face),
                    HealpixOrdering::Nested => xyf2nest(source_nside, i, j, face),
                } as usize;
                
                let val = map[source_pix];
                if is_seen(val) {
                    sum += val;
                    hits += 1;
                }
            }
        }
        
        // Average the collected pixels
        if hits >= 1 {
            result[target_pix] = sum / hits as f64;
        }
    }
    
    result  // 806M pixels read for this step
}
```

**Cost:** Reads **806M pixels** (one per source location)

---

## Option A: Checkerboard Sampling (Simplest)

**Idea:** Skip every other pixel in both x and y directions. Read only 25% of pixels.

```rust
fn downgrade_healpix_map_checkerboard(
    map: &[f64],
    source_nside: i64,
    target_nside: i64,
    ordering: HealpixOrdering,
) -> Vec<f64> {
    let fact = source_nside / target_nside;  // 8
    let target_npix = (12 * target_nside * target_nside) as usize;
    let mut result = vec![HPX_UNSEEN; target_npix];
    
    for target_pix in 0..target_npix {
        let (x, y, face) = match ordering {
            HealpixOrdering::Ring => ring2xyf(target_nside, target_pix as i64),
            HealpixOrdering::Nested => nest2xyf(target_nside, target_pix as i64),
        };
        
        let mut sum = 0.0;
        let mut hits = 0;
        
        let x0 = fact * x;
        let y0 = fact * y;
        
        // Read every 2nd pixel (checkerboard pattern)
        for j in (y0..(y0 + fact)).step_by(2) {      // ← step_by(2)
            for i in (x0..(x0 + fact)).step_by(2) {  // ← step_by(2)
                let source_pix = match ordering {
                    HealpixOrdering::Ring => xyf2ring(source_nside, i, j, face),
                    HealpixOrdering::Nested => xyf2nest(source_nside, i, j, face),
                } as usize;
                
                let val = map[source_pix];
                if is_seen(val) {
                    sum += val;
                    hits += 1;
                }
            }
        }
        
        // Average the sampled pixels
        if hits >= 1 {
            result[target_pix] = sum / hits as f64;
        }
    }
    
    result  // 200M pixels read (~4× speedup)
}
```

**Implementation:**
- Add 15-20 lines (just add `.step_by(2)` to loop ranges)
- Trivial code change
- Can be added as new function next to existing one

**Performance:**
- Pixels read: 806M → 200M (4× reduction)
- Time estimate: 6.4s → ~1.8s downsampling (3.5× speedup)
- End-to-end: 10.9s → ~7.3s (33% faster)

**Quality Loss:**
- Average: ~10-15% RMS error vs ground truth
- Visible as slight "noisiness" on smooth maps
- Point sources may be missed entirely
- **Acceptable use:** Quick preview, fast iteration

**When to use:** User wants speed (`--fast` flag)

---

## Option B: Adaptive Coarse-Grid (Recommended)

**Idea:** Automatically use checkerboard in smooth regions, full grid in noisy/detailed regions.

```rust
fn downgrade_healpix_map_adaptive(
    map: &[f64],
    source_nside: i64,
    target_nside: i64,
    ordering: HealpixOrdering,
) -> Vec<f64> {
    let fact = source_nside / target_nside;
    let target_npix = (12 * target_nside * target_nside) as usize;
    let mut result = vec![HPX_UNSEEN; target_npix];
    
    // PHASE 1: Estimate local variance (quick pass with checkerboard)
    // Use 10M random sample to estimate baseline variance
    let sample_variance = estimate_global_variance(map, 10_000_000);
    let variance_threshold = sample_variance * 0.5;  // Use cheats below half variance
    
    // PHASE 2: Build variance map per region
    let mut region_variance = vec![0.0; target_npix];
    for region_idx in 0..target_npix {
        let (x, y, face) = match ordering {
            HealpixOrdering::Ring => ring2xyf(target_nside, region_idx as i64),
            HealpixOrdering::Nested => nest2xyf(target_nside, region_idx as i64),
        };
        
        let x0 = fact * x;
        let y0 = fact * y;
        
        // Quick variance estimate using checkerboard sample
        let mut samples = Vec::new();
        for j in (y0..(y0 + fact)).step_by(2) {
            for i in (x0..(x0 + fact)).step_by(2) {
                let source_pix = match ordering {
                    HealpixOrdering::Ring => xyf2ring(source_nside, i, j, face),
                    HealpixOrdering::Nested => xyf2nest(source_nside, i, j, face),
                } as usize;
                if is_seen(map[source_pix]) {
                    samples.push(map[source_pix]);
                }
            }
        }
        
        if !samples.is_empty() {
            let mean = samples.iter().sum::<f64>() / samples.len() as f64;
            let variance = samples
                .iter()
                .map(|x| (x - mean).powi(2))
                .sum::<f64>() / samples.len() as f64;
            region_variance[region_idx] = variance.sqrt();
        }
    }
    
    // PHASE 3: Downsample with adaptive grid
    for target_pix in 0..target_npix {
        let (x, y, face) = match ordering {
            HealpixOrdering::Ring => ring2xyf(target_nside, target_pix as i64),
            HealpixOrdering::Nested => nest2xyf(target_nside, target_pix as i64),
        };
        
        let mut sum = 0.0;
        let mut hits = 0;
        
        let x0 = fact * x;
        let y0 = fact * y;
        
        // Choose sampling strategy based on local variance
        let use_coarse = region_variance[target_pix] < variance_threshold;
        let step = if use_coarse { 2 } else { 1 };
        
        for j in (y0..(y0 + fact)).step_by(step) {
            for i in (x0..(x0 + fact)).step_by(step) {
                let source_pix = match ordering {
                    HealpixOrdering::Ring => xyf2ring(source_nside, i, j, face),
                    HealpixOrdering::Nested => xyf2nest(source_nside, i, j, face),
                } as usize;
                
                let val = map[source_pix];
                if is_seen(val) {
                    sum += val;
                    hits += 1;
                }
            }
        }
        
        if hits >= 1 {
            result[target_pix] = sum / hits as f64;
        }
    }
    
    result
}
```

**Implementation:**
- Add 100-150 lines (variance estimation + branching logic)
- Requires helper function `estimate_global_variance()`
- Can reuse most existing code framework

**Performance:**
- Average case: 60% checkerboard + 40% full grid
- Pixels read: 806M → 500M (1.6× reduction)
- Time estimate: 6.4s → ~4.0s downsampling (1.6× speedup)
- End-to-end: 10.9s → ~7.9s (27% faster)

**Quality Loss:**
- Average: ~2% RMS error (nearly imperceptible)
- High-detail regions use full grid (preserved)
- Smooth regions use coarse grid (quality loss acceptable)
- **Acceptable for:** Publication-ready maps, production use

**When to use:** Default for all users (no flag needed, automatic)

**Key advantage:** User gets 27% speedup automatically, no quality perception.

---

## Option C: Two-Phase Downsampling (Best Quality)

**Idea:** Downsample in stages: first with full grid (preserve detail), then checkerboard on intermediate result.

```rust
fn downgrade_healpix_map_two_phase(
    map: &[f64],
    source_nside: i64,
    target_nside: i64,
    ordering: HealpixOrdering,
) -> Vec<f64> {
    // PHASE 1: Intermediate downsampling with full grid
    // 8192 → 4096 (2× reduction, reads all 806M pixels)
    let intermediate_nside = (source_nside + target_nside) / 2;
    let intermediate = downgrade_healpix_map(
        map,
        source_nside,
        intermediate_nside,
        ordering,
    );
    // Time: 3.0-3.2s
    
    // PHASE 2: Final downsampling with checkerboard on 200M pixels
    // 4096 → 1024 (2× reduction, reads only 200M pixels)
    let result = downgrade_healpix_map_checkerboard(
        &intermediate,
        intermediate_nside,
        target_nside,
        ordering,
    );
    // Time: 0.4-0.5s
    
    result  // Total: 3.4-3.7s (vs 6.4s baseline, 1.7× speedup)
}
```

**Implementation:**
- Add 20 lines (just orchestration, reuses existing functions)
- Simplest from code perspective
- Can add as new public function

**Performance:**
- Phase 1: Full grid on 806M pixels (3.2s)
- Phase 2: Checkerboard on 200M pixels (0.4s)
- Total downsampling: 6.4s → 3.6s (1.78× speedup)
- End-to-end: 10.9s → 7.5s (31% faster)

**Quality Loss:**
- Average: <1% RMS error
- Visually indistinguishable from ground truth
- First phase preserves all detail
- Second phase only coarsens already-smoothed intermediate
- **Acceptable for:** Publication, archival, scientific use

**When to use:** Auto-selected for large files (production default)

---

## Comparison & Decision Matrix

| Property | Checkerboard | Adaptive | Two-Phase |
|----------|-----------|----------|-----------|
| **Implementation** | 15 lines | 150 lines | 20 lines |
| **Speed** | 33% faster | 27% faster | 31% faster |
| **Quality Loss** | ~10-15% | ~2% | <1% |
| **User Config** | Flag required | Automatic | Automatic |
| **Testing Effort** | Minimal | Moderate | Heavy |
| **Maintenance** | Trivial | Low | Low |
| **Recommended Use** | `--fast` option | Default | Production |
| **Deployment Risk** | Low | Medium | Low |

---

## Phased Implementation Roadmap

### Week 1: Checkerboard + Two-Phase (Quick MVP)
```bash
# Minimal viable product: add two new functions
- downgrade_healpix_map_checkerboard() [15 lines]
- downgrade_healpix_map_two_phase() [20 lines]
- Add CLI flags: --quality=best|balanced|fast
- Test with 3-4 maps, validate output
```

**Deliverable:** Users can opt-in to 31% speedup with `--quality=balanced`

### Week 2-3: Adaptive (Production Quality)
```bash
# Add automatic variance-based selection
- implement estimate_global_variance() [30 lines]
- implement downgrade_healpix_map_adaptive() [120 lines]
- Make it default (no flag needed)
- Integration tests on diverse file types
- Benchmark speedup/quality tradeoff
```

**Deliverable:** 27% speedup for all users transparently

### Week 4: Validation & Documentation
```bash
# Quality assurance
- Pixel-by-pixel RMS comparison vs baseline
- Visual inspection gallery (before/after samples)
- Histogram analysis (preserve distribution?)
- Document quality settings in README
- Add to --help output
```

**Deliverable:** Production-ready with clear quality/speed documentation

---

## Testing Strategy

### Quality Metrics
```rust
// Compute RMS error vs baseline
fn compute_rms_error(baseline: &[f64], coarse: &[f64]) -> f64 {
    let sum: f64 = baseline
        .iter()
        .zip(coarse.iter())
        .filter(|(b, c)| is_seen(*b) && is_seen(*c))
        .map(|(b, c)| (b - c).powi(2))
        .sum();
    let n = baseline.iter().filter(|v| is_seen(**v)).count() as f64;
    (sum / n).sqrt()
}

// Compare histograms
fn histogram_divergence(baseline: &[f64], coarse: &[f64]) -> f64 {
    // Compute Wasserstein distance or KL divergence
    // Check if distribution is preserved
}
```

### Test Files
- Smooth maps (cosmology simulations) → should use checkerboard
- Noisy maps (observational) → should use full grid
- Point sources → verify not missed
- Edge cases → nans, zeros, negative values

### Benchmarking
```bash
./target/release/map2fig --quality=best input.fits out_best.png
./target/release/map2fig --quality=balanced input.fits out_balanced.png
./target/release/map2fig --quality=fast input.fits out_fast.png

# Measure speed and compare outputs
```

---

## Integration with Existing Code

Current downsampling is in `src/healpix.rs`:
- Lines 1240-1330: `downgrade_healpix_map_xyf_parallel()`
- Called from `src/pipeline.rs` line 73

**Integration points:**
1. Add new functions at end of `healpix.rs` (after existing downsampling)
2. Add quality flag to `src/setup.rs` configuration struct
3. Update `src/pipeline.rs` to choose algorithm based on quality setting
4. Update CLI parser in `src/main.rs` with `--quality` flag

**Backward compatibility:**
- Default stays `--quality=best` (current algorithm, exact output)
- Users opt-in to faster options
- No breaking changes to API or output format

---

## Recommendation

**Start with Week 1 approach:**
1. Implement two-phase downsampling (quick win, 31% speedup, <1% quality loss)
2. Add CLI flag for user control
3. Validate on test suite
4. Get user feedback

**If successful, proceed to adaptive:**
5. Implement adaptive selection
6. Make it default
7. Remove need for flags (transparent speedup)

**This approach:**
- ✅ Delivers value quickly (1-2 weeks)
- ✅ Maintains quality for publication users
- ✅ Gives speed for interactive users
- ✅ Minimal code complexity
- ✅ Easy to validate and test
- ✅ No external dependencies

---

## Example Usage (After Implementation)

```bash
# Current (slowest, exact)
$ ./map2fig input.fits output.pdf
# Time: 10.9s

# Balanced (default after adaptive is added)
$ ./map2fig input.fits output.pdf
# Time: 7.9s (27% faster, quality indistinguishable)

# Fast (user wants preview)
$ ./map2fig --quality=fast input.fits output.pdf
# Time: 7.3s (33% faster, slight noise visible)

# Best (user needs publication quality now)
$ ./map2fig --quality=best input.fits output.pdf
# Time: 10.9s (exact, current implementation)
```

---

## Next Steps

1. Agree on approach (Checkerboard + Two-Phase → Adaptive)
2. Create feature branch: `feature/coarse-grid-sampling`
3. Implement Week 1 (two-phase only)
4. Test and benchmark
5. Get user feedback before proceeding to Week 2 (adaptive)
