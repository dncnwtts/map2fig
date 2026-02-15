# Tier 3b: Cache-Aware Loop Buffer Pre-allocation

## Problem
The `render_projection_to_grid()` function (lines 246-799) allocates 10+ arrays **per iteration** in its hot path:
- ~55,000 iterations per image 
- Each iteration allocates: px_array_lo[8], py_array_lo[8], thetas_lo[8], validity_mask_lo[8], px_array_hi[8], py_array_hi[8], thetas_hi[8], validity_mask_hi[8], healpix_values_16[16], validity_mask_16[16], etc.
- This causes **stack thrashing** and poor L1/L2/L3 cache utilization
- perf c2c analysis showed 100% LLC misses to DRAM (working set > L3 capacity issue)

## Root Cause
Stack allocation churn in tight rendering loop. These are **fixed-size arrays** that should be pre-allocated once and reused.

## Solution  
Move array allocations **outside the loop** (lines 257-261), then reuse via:
- Direct assignment: `py_array_lo = [py; 8];`
- Clear-then-fill pattern: `thetas_lo = [0.0_f64; 8]; for i in 0..8 { thetas_lo[i] = ... }`
- Array slice operations: `healpix_values_16[..8].copy_from_slice(&healpix_values_lo);`

## Exact Changes Required

### 1. Add pre-allocation block BEFORE `for py in 0..height` (line 261)

```rust
    // Tier 3b: Pre-allocate hot-path arrays to eliminate stack allocation churn
    // These 8+ arrays are allocated ~55K times/image causing L1/L2 cache pressure
    let mut px_array_lo = [0u32; 8];
    let mut py_array_lo = [0u32; 8];
    let mut px_array_hi = [0u32; 8];
    let mut py_array_hi = [0u32; 8];
    let mut thetas_lo = [0.0_f64; 8];
    let mut thetas_hi = [0.0_f64; 8];
    let mut validity_mask_lo = [false; 8];
    let mut validity_mask_hi = [false; 8];
```

### 2. Remove `let mut px_array_lo = ...` allocations in loop (line ~269)
   - **BEFORE**: `let mut px_array_lo = [0u32; 8];`
   - **AFTER**: Use pre-allocated, just fill: `py_array_lo = [py; 8]; for (i, item) in px_array_lo.iter_mut()...`

### 3. Remove `let mut thetas_lo = ...` allocations (line ~280)
   - **BEFORE**: `let mut thetas_lo = [0.0_f64; 8];`
   - **AFTER**: Clear and reuse: `thetas_lo = [0.0_f64; 8]; for i in 0..8 { thetas_lo[i] = ... }`

### 4. Remove `let validity_mask_lo: [bool; 8] = [...]` array literal (line ~298)
   - **BEFORE**: `let validity_mask_lo: [bool; 8] = [proj_mask_lo[0] && ..., proj_mask_lo[1] && ..., ...]`
   - **AFTER**: Loop-based fill: `validity_mask_lo = [false; 8]; for i in 0..8 { validity_mask_lo[i] = proj_mask_lo[i] && healpix_mask_lo[i]; }`

### 5. Repeat steps 2-4 for the `_hi` batch (px_array_hi, py_array_hi, thetas_hi, validity_mask_hi)

### 6. Update merge section (line ~367)
   - **BEFORE**: `let mut healpix_values_16 = [0.0; 16]; let mut validity_mask_16 = [false; 16];` etc
   - **AFTER**: Direct reuse: `healpix_values_16[..8].copy_from_slice(&healpix_values_lo);` etc

### 7. Remove `let unseen_mask: [bool; 16] = [!is_seen(...), ...]` array literal (line ~391)
   - **BEFORE**: `let unseen_mask: [bool; 16] = [!is_seen(values[0]), ...]`
   - **AFTER**: Loop fill: `unseen_mask = [false; 16]; for i in 0..16 { unseen_mask[i] = !is_seen(healpix_values_16[i]); }`

### 8. Remove `let pixel_values: [PixelValue; 16] = if ...` if-expression (line ~410)
   - **BEFORE**: `let pixel_values: [PixelValue; 16] = if matches!(...) { ... } else { ... };`
   - **AFTER**: Fill pre-allocated: `if matches!(...) { ... pixel_values[i] = ... } else { ... pixel_values[i] = ... }`

## Expected Benefits
- **Cache misses**: 31.85% → <25% (reduced L1/L2 evictions from reuse)
- **Performance**: 10.14s → 9.7-9.9s (3-5% improvement from cache locality)
- **Working set**: Reduced stack frame size per iteration

## Validation
1. Compile: `cargo build -r`
2. Run on cosmoglobe: `time ./target/release/map2fig -f cosmoglobe_clipped.fits`
3. Measure cache misses: `perf stat -e cache-misses,cache-references ./target/release/map2fig ...`
4. Expected: Cache miss rate drops 2-4 percentage points

## Risk Level
**LOW** - No algorithmic changes, only moving allocations to change memory access patterns.

