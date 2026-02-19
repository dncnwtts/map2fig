# Floating-Point Conversion Hotspots Analysis

## Critical Path: Where f32→f64 Conversion Occurs

### Phase 1: FITS Reading (fits.rs line 849)
```rust
// ❌ HOTSPOT: 2.6 seconds for f32→f64 conversion
let fits_read_start = std::time::Instant::now();
let data_array: DataArray = read_healpix_column(...);  // Returns DataArray::Float32 or Float64
let fits_elapsed = fits_start.elapsed();

let convert_start = std::time::Instant::now();
let data = data_array.as_f64_vec().into_owned();  // ← THIS IS THE 2.6s COST
let convert_elapsed = convert_start.elapsed();

// data is now Vec<f64>, losing f32 precision info
```

### Phase 2: Scaling Loop (pipeline.rs lines 57-70)
```rust
// Input: Vec<f64> (already converted)
let mut map = read_healpix_column_cached(new_fits_path, col);  // Returns Vec<f64>

for v in &mut map {
    if !is_seen(*v) {
        continue;
    }
    if v.abs() < 1e-20 {
        *v = HPX_UNSEEN;
    } else {
        *v *= scale_factor;  // ← Operates on f64
    }
}
```

### Phase 3: Downsampling (pipeline.rs lines 92-103)
```rust
let downgraded_map = match quality_level {
    QualityLevel::Best => {
        downgrade_healpix_map(&map, meta.nside, target_nside, meta.ordering)
        //                    ^^^^
        //                    &[f64] - signature expects f64
    }
    // ... other quality levels
}
```

### Phase 4: Rendering (healpix.rs)
```rust
// Downsampling function signature
pub fn downgrade_healpix_map(
    map: &[f64],  // ← Needs to be f64 for arithmetic
    source_nside: i64,
    target_nside: i64,
    ordering: HealpixOrdering,
) -> Vec<f64> {
    // For each target pixel:
    // let (x, y, face) = ring2xyf(target_nside, target_pix as i64);
    // let x0 = fact * x;
    // let val = map[source_pix];
    // All arithmetic assumes f64
}
```

## Conversion Chain Analysis

| Step | Input Type | Output Type | Cost | Avoidable? |
|------|-----------|-----------|------|-----------|
| FITS decode | binary | `DataArray::Float32` | 1.6s | No |
| **f32→f64 conversion** | `DataArray::Float32` | `Vec<f64>` | **2.6s** | ❓ YES |
| Scaling loop | `Vec<f64>` | `Vec<f64>` | ~0.2s | Partial |
| Downsampling | `Vec<f64>` | `Vec<f64>` | 1.3s | Partial |
| Masking | `Vec<f64>` | `Option<PixelMask>` | ? | ? |
| Rendering | `Vec<f64>` | PNG/PDF | ~0.1s | No |

## Root Cause: Type System Design

### Problem
```rust
pub struct ProcessedData {
    pub map: Vec<f64>,  // ← Forces conversion immediately
    pub meta: HealpixMeta,
}
```

The `ProcessedData.map` field is hardcoded as `Vec<f64>`, forcing us to convert f32 data at load time.

### Why This Hurts
1. For 806M f32 pixel FITS file: `806M × 4 bytes = 3.1 GB` on disk
2. Must convert all 806M values: `for &x in v { x as f64 }` sequential single-threaded
3. Array too large to benefit from CPU prefetcher (working set exceeds L3 cache)
4. Memory bandwidth saturated: 700 MB/s read + conversion overhead

## Conversion Points Inventory

### In `data_array.rs`:
1. **Line 74**: `get()` - converts on-demand ✓ (fine)
2. **Line 83**: `iter()` - wraps iterator with conversion ✓ (lazy)
3. **Line 92-96**: `min_value()` - converts during iteration ✓ (fine)
4. **Line 101-105**: `max_value()` - converts during iteration ✓ (fine)
5. **Line 114**: `valid_f64_values()` - builds Vec<f64> ✓ (known cost)
6. **Line 137**: `as_f64_vec()` - **MAIN CULPRIT** ❌ (2.6s for 806M elements)

### In `fits.rs`:
1. **Line 849**: `data_array.as_f64_vec().into_owned()` - Full buffer copy of f32→f64

### In `healpix.rs`:
- All downsampling functions expect `&[f64]` - would need templating to generalize

### In `pipeline.rs`:
- `ProcessedData { map: Vec<f64> }` - forces upfront conversion

## Solutions Ranked by Impact

### Solution 1: Make ProcessedData Generic (HIGH IMPACT)
```rust
pub struct ProcessedData<T: Float> {  // or enum Mapped { Float32(...), Float64(...) }
    pub map: Vec<T>,
    pub meta: HealpixMeta,
}
```
**Pros**: Eliminates 2.6s conversion upfront
**Cons**: Must template all downstream code (big refactor)

### Solution 2: Lazy Conversion with Cow (MEDIUM IMPACT)
```rust
pub struct ProcessedData {
    pub map: std::borrow::Cow<'static, Vec<f64>>,  // Borrowed for f64, owned for f32 conversion
    pub meta: HealpixMeta,
}
```
**Pros**: Only converts if/when actually accessed
**Cons**: Lifetime complexity, may still force conversion in downsampling

### Solution 3: Keep DataArray Through Pipeline (MEDIUM IMPACT)
```rust
pub struct ProcessedData {
    pub map: DataArray,  // Keep f32 native!
    pub meta: HealpixMeta,
}

// Update all consumers to work with DataArray
downgrade_healpix_map_generic(&data.map, ...)  // Generic version
```
**Pros**: Preserves type info throughout pipeline
**Cons**: Must refactor downsampling and all consumers

### Solution 4: Selective Reading (HIGH IMPACT, HIGH EFFORT)
Read only pixels needed for target resolution, skip the rest
**Pros**: Avoid reading 99.6% of file for nside 8192→512
**Cons**: Complex FITS random I/O, loses native map format, verification issues

## Recommended Approach

**Solution 3 (Keep DataArray)** offers best balance:
1. Minimal type system changes
2. Preserves f32 through entire pipeline  
3. Conversion only happens if/when rendering code demands f64
4. Can implement generic downsampling gradually

### Implementation Steps:
1. Change `ProcessedData.map: DataArray` instead of `Vec<f64>`
2. Add generic `downgrade_healpix_map_generic()` that works with DataArray
3. Update downsampling router to use generic version
4. Update consumer functions in `cli_builder.rs` to handle DataArray
5. Only convert to f64 in final rendering phase if needed

## Impact Estimate
- **Time saved** if fully implemented: 2.6s → 0s (100% of conversion cost)
- **Total speedup**: From 7.2s → 4.6s (36% faster)
- **Implementation effort**: Medium (affects ~10-15 functions)
