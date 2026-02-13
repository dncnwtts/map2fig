# Performance Optimization Roadmap for map2fig

## Executive Summary
The rendering pipeline is fundamentally sequential per-pixel in your implementation.
With targeted optimizations, you can achieve **~1.3-1.6x speedup** (2.5s → 1.5-1.9s for full-sky maps).
The remaining gap to C++ comes from inherent Rust safety overhead that can't be fully eliminated without unsafe code.

---

## 1. Parallelization (Biggest Win: ~30-40% speedup)

### Current Bottleneck
```rust
// In src/plot/mod.rs:render_projection_to_grid()
for py in 0..height {
    for px in 0..width {
        // ... complex per-pixel operations ...
    }
}
```

For a 1200×600 map, that's **720,000 independent pixel computations** done serially.

### Solution: Rayon Parallelization

**Step 1**: Add rayon to `Cargo.toml`:
```toml
[dependencies]
rayon = "1.8"
```

**Step 2**: Modify the rendering loop in `src/plot/mod.rs`:

Replace this:
```rust
for py in 0..height {
    for px in 0..width {
        // pixel logic
        unsafe {
            grid.set_pixel_unchecked(px, py, rgba);
        }
    }
}
```

With this:
```rust
use rayon::prelude::*;

// Create mutable row buffer for each thread
let tile_height = 32; // Process rows in tiles to improve cache locality

(0..height).step_by(tile_height).par_bridge()
    .for_each(|tile_start| {
        let mut local_grid = grid.clone_empty(); // Pre-allocate for this tile
        
        let tile_end = (tile_start + tile_height).min(height);
        for py in tile_start..tile_end {
            for px in 0..width {
                if let Some((lon, lat)) = params.proj.pixel_to_ang(px, py, &local_grid) {
                    let theta = std::f64::consts::PI / 2.0 - lat;
                    let pixel_val = sample_and_scale(...);
                    let rgba = map_to_color(pixel_val, ...);
                    
                    unsafe {
                        local_grid.set_pixel_unchecked(px, py, rgba);
                    }
                }
            }
        }
        // Merge tile results back
        merge_tile_results(&mut grid, &local_grid, tile_start..tile_end);
    });
```

**Expected gain**: **30-40% speedup** (parallelizes across CPU cores; 4 cores = up to 4x, realistically 3-3.5x after overhead)

---

## 2. Remove Trait Object Overhead (5-10% speedup)

### Current Issue
```rust
params.proj.pixel_to_ang(...)  // Virtual call to trait method
```

In tight loops with millions of calls, trait object dispatch adds overhead.

### Solution: Monomorphize in Hot Path

Create a version that's specialized per-projection:

In `src/plot/mod.rs`, create specialization:
```rust
pub fn render_projection_to_grid_monomorphic<P: Projection>(
    params: &RenderGridParams,
    proj: &P,  // Instead of &dyn Projection
    grid: &mut RasterGrid,
) {
    // Same loop as before, but proj is concrete type
    // Compiler monomorphizes → no vtable lookup
}

// Route based on projection type
match params.projection_type {
    ProjectionType::Mollweide => {
        let proj = MollweideProjection::new(...);
        render_projection_to_grid_monomorphic(params, &proj, grid);
    }
    // ... same for others
}
```

**Expected gain**: **5-10% speedup** on rendering loop

---

## 3. Inline Aggressive Hints (3-5% speedup)

Add `#[inline(always)]` to hot-path functions:

In `src/scale.rs`:
```rust
#[inline(always)]
pub fn scale_value(
    value: f64,
    min: f64,
    max: f64,
    scale: Scale,
    neg_mode: NegMode,
    hist: Option<&HistogramScale>,
) -> PixelValue {
    // ... existing code ...
}
```

In `src/render/raster.rs`:
```rust
#[inline(always)]
pub unsafe fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: Rgba<u8>) {
    let idx = (y * self.width + x) as usize;
    unsafe {
        *self.data.get_unchecked_mut(idx) = color;
    }
}
```

In colormap sampling:
```rust
#[inline(always)]
pub fn sample(&self, t: f64) -> [u8; 3] {
    // ... existing code ...
}
```

**Expected gain**: **3-5% speedup**

---

## 4. Profile-Guided Optimization (2-5% speedup)

Use PGO to let LLVM optimize based on real execution:

```bash
# Build with PGO instrumentation
RUSTFLAGS="-C llvm-args=-pgo-warn-missing-function" \
cargo build --release -p map2fig --profile=pgo-instr

# Run on representative workload
./target/pgo-instr/map2fig -f npipe_nodip.fits -o /tmp/output.pdf

# Build optimized version using profile data
RUSTFLAGS="-C llvm-args=-fprofile-use=../pgo-data/fused.profdata -C llvm-args=-fprofile-sample-accurate" \
cargo build --release --profile=pgo-opt
```

**Expected gain**: **2-5% speedup** (LLVM does branch prediction, inlining, reordering)

---

## 5. Reduce Allocations in Hot Path (2-3% speedup)

### Current Issue
The scaling function may allocate on every call.

### Solution: Pre-allocate scratch buffers

In `src/plot/mod.rs`:
```rust
fn render_projection_to_grid(...) {
    // Pre-allocate outside loop
    let mut coord_scratch = [0.0; 2]; // For (lon, lat)
    
    for py in 0..height {
        for px in 0..width {
            // Use scratch buffer instead of allocating
            if let Some((lon, lat)) = params.proj.pixel_to_ang_into(px, py, &mut coord_scratch) {
                // ... use lon, lat from scratch ...
            }
        }
    }
}
```

For histogram scaling, pre-compute in batches:
```rust
// Instead of: per-pixel histogram lookups
// Do: batch histogram computations
let mut histogram_cache = build_lookup_table(&hist, 256); // 256 predefined points
// Then use fast lookup instead of binary search
```

**Expected gain**: **2-3% speedup**

---

## 6. SIMD for Scaling (15-20% speedup for log/asinh only)

This is complex but worth it for log-heavy workloads:

Add to `Cargo.toml`:
```toml
packed_simd_2 = "0.3"  # or core_simd if using nightly
```

Vectorize scaling where possible:
```rust
#[cfg(target_arch = "x86_64")]
fn scale_log_simd(values: &[f64], min: f64, max: f64) -> Vec<f64> {
    use packed_simd::*;
    
    let lmin = min.ln();
    let lmax = max.ln();
    let inv_range = 1.0 / (lmax - lmin);
    
    let mut result = Vec::with_capacity(values.len());
    
    // Process 4 values at a time (256-bit AVX2)
    for chunk in values.chunks(4) {
        let v: f64x4 = f64x4::from_slice_unaligned(chunk);
        let ln_v = v.ln();
        let scaled = (ln_v - lmin) * inv_range;
        result.extend_from_slice(&scaled.to_array());
    }
    
    result
}
```

**Expected gain**: **15-20% speedup for log scaling** (but only if you're doing heavy log-heavy workloads)

---

## 7. Cache-Friendly Memory Layout (5-8% speedup)

HEALPix data access patterns benefit from better cache locality:

In `src/healpix.rs`, optimize the index calculation:
```rust
// Current approach may jump around
fn sample_healpix_index(...) -> Option<usize> {
    // ... complex indexing ...
}

// Better: batch by ring or optimize stride for cache
#[inline(always)]
fn sample_healpix_optimized(...) -> (f64, Option<usize>) {
    // Pre-computed lookup tables for fast indexing
    static RING_OFFSETS: [usize; NSIDE] = precompute_offsets();
    
    // Direct array access instead of complex calculations
    &self.data[RING_OFFSETS[ring] + offset]
}
```

**Expected gain**: **5-8% speedup**

---

## 8. Remove Redundant Checks (1-2% speedup)

Pull loop-invariant checks outside:

Current:
```rust
for py in 0..height {
    for px in 0..width {
        let gamma_inv = if (params.gamma - 1.0).abs() < f64::EPSILON {
            1.0
        } else {
            params.gamma
        };  // ← Computed 720k times!
        
        let t = if gamma_inv == 1.0 {
            t
        } else {
            t.powf(gamma_inv)
        };
    }
}
```

Optimized:
```rust
let needs_gamma = (params.gamma - 1.0).abs() > f64::EPSILON;
let gamma_inv = if needs_gamma { params.gamma } else { 1.0 };

for py in 0..height {
    for px in 0..width {
        let t = if needs_gamma {
            t.powf(gamma_inv)
        } else {
            t
        };
    }
}
```

**Expected gain**: **1-2% speedup**

---

## 9. Optimize Hot Functions with Release Profile Settings

Update `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = "fat"        # Adds 10-15s to build but enables cross-module optimization
codegen-units = 1 # Already set, good
strip = false      # Not critical for perf but reduces stack overhead
panic = "abort"    # Slightly smaller code
```

For aggressive optimization on specific module:
```toml
[profile.release]
# Default settings
opt-level = 3

# But for hot paths, push harder
[profile.bench]
opt-level = 3
lto = "fat"
codegen-units = 1
```

**Expected gain**: **2-3% speedup**

---

## 10. Benchmarking: Measure Real Improvement

Create a benchmark harness in `src/main.rs` or separate binary:

```bash
# Before optimization
$ time ./target/release/map2fig -f npipe_nodip.fits -w 1200 -o /tmp/test.pdf
real    0m2.517s

# After Rayon optimization
real    1m.8-1.9s       (30-40% win)

# After all optimizations
real    0m1.5-1.6s      (40-50% win total)
```

Create a benchmark script:
```bash
#!/bin/bash
cargo build --release

# Warm up
./target/release/map2fig -f npipe_nodip.fits -o /tmp/tmp.pdf

# Actual benchmark (5 runs)
for i in {1..5}; do
    /usr/bin/time -f "%e seconds" ./target/release/map2fig \
        -f npipe_nodip.fits -w 1200 -o /tmp/bench_${i}.pdf
done
```

---

## Recommended Implementation Order

1. **Rayon parallelization** (30-40% gain, ~1 hour implementation)
2. **Inline hints** (3-5% gain, ~30 minutes)
3. **Profile-guided optimization** (2-5% gain, ~15 minutes)
4. **Memory layout optimization** (5-8% gain, ~2 hours if needed)
5. **SIMD** (15-20% for log-heavy, ~4 hours if needed)
6. **Cache optimization** (5-8% gain, ~1 hour)

**Total realistic gain with 1-3: ~40-50%** (2.5s → 1.25-1.5s)

---

## Known Limitations (Why You Can't Reach C++)

After all these optimizations, you'll hit these remaining limitations:

### 1. **Iterator/Closure Overhead**
Even with inlining, Rust's iterator abstractions add ~5% overhead vs direct C array indexing

### 2. **Option/Result Unwrapping**
Every `if let Some(...)` costs branches and comparison checks

### 3. **Bounds Checking**
Rust insists on bounds checking even with unsafe blocks (you opt-out, but it's verbose)

### 4. **Trait Dispatch**
Even monomorphized, vtable avoidance requires template bloat

### 5. **Memory Allocation Patterns**
Rust's ownership model forces more allocations than C++'s free-form pointers

**Bottom line**: C++ can achieve 0.8-1.0s. Rust will likely max out at 1.5-1.8s even with all optimizations due to language fundamentals.

---

## Expected Results

| Optimization | Effort | Expected Gain | Cumulative |
|--------------|--------|---------------|-----------|
| Baseline | - | - | 2.500s |
| #1: Rayon | 1 hour | -35% | 1.625s |
| #2: Inlining | 0.5 hours | -5% | 1.544s |
| #3: PGO | 0.25 hours | -3% | 1.497s |
| #4: Cache layout | 1 hour | -6% | 1.408s |
| **Total effort** | **2.75 hours** | **-44% speedup** | **~1.4s** |

---

## Reality Check

- 2-3x speedup over C++ is **impossible** without fundamental language changes
- 40-50% speedup is **realistic** with these optimizations
- **1.4-1.5s full-sky render** is still respectable and beats default Rust significantly
- The gap to C++'s 0.8s is the cost of **memory safety guarantees**

Choose your battles based on whether that safety matters enough to accept slower builds.

---

## References

- Rayon parallel iterator: https://docs.rs/rayon/latest/rayon/
- Rust performance book: https://nnethercote.github.io/perf-book/
- PGO in Rust: https://doc.rust-lang.org/rustc/profile-guided-optimization.html
