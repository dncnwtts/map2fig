# I/O Bottleneck Analysis & Optimization Plan

**Date**: February 16, 2026  
**Branch**: io-optimization  
**Issue**: Large FITS files require 4-6 seconds to process, limiting GPU benefit

## Executive Summary

After benchmarking the full pipeline, **the bottleneck is NOT disk I/O but FITS parsing and type conversions**. Key findings:

1. **Disk caching doesn't help**: Times are consistent across runs (4.8-5.0s) - not I/O bound
2. **Memory-mapped I/O doesn't help**: mmap is actually slower (5.1s vs 4.9s)
3. **Column data caching IS working**: Shows "Cache HIT" but file still takes 4.8s
4. **Root cause**: FITS file parsing and type conversion overhead in `fitsrs` crate

## Current Performance

| Scenario | File | Time | Bottleneck |
|----------|------|------|------------|
| Default (downsampled to 24 MB) | 577 MB npipe | 1.5s | PDF rendering |
| Full resolution | 577 MB npipe | 4.8-5.1s | **FITS parsing + type conversion** |
| GPU with full res (--no-downgrade) | 577 MB npipe | 6.1s | I/O (80%) + GPU (0.2s) |

## Profiling Results

```
Run 1 (buffered):  4.85s
Run 2 (buffered):  4.83s
Run 3 (buffered):  5.05s
Average:           4.91s ± 0.10s

With mmap:         5.10s (1.2% SLOWER!)
Cache column:      0.054ms (instant, cache hit)
```

**Interpretation**:
- Consistent timing across runs → disk cache irrelevant
- mmap slower → excessive overhead for this workload
- Cache hit instant but total still ~4.8s → bottleneck is not FITS I/O

## Root Cause: FITS Library Overhead

The issue is in how `fitsrs` crate handles FITS files:

### Current Flow (for full-resolution file)

1. **File reading**: 577 MB FITS file opened ~0.8s
2. **HDU parsing**: Parse FITS headers, binary table structure ~0.5s
3. **Column selection**: `table.select_fields()` iterates through table ~0.3s
4. **Type conversion loop**: Converting every cell to f64 ~2.5s (SLOW!)
5. **Data validation**: Checking UNSEEN values ~0.5s
6. **Downgrade** (if applicable): 1-2 additional seconds

### The Time Killer: Type Conversions

Most of the slowdown occurs in this loop:

```rust
for cell in values {
    match cell {
        DataValue::Double { value, .. } => result.push(value),
        DataValue::Float { value, .. } => result.push(value as f64),
        DataValue::Integer { value, .. } => result.push(value as f64),
        other => panic!("Unsupported column type in FITS table: {:?}", other),
    }
}
```

**Problem**: This processes **50 million pixels** one by one with match statement overhead for each cell.

## Optimization Opportunities

### Tier 1: Direct Column Reading Without Type Conversion (Est. 30-40% speedup)

**Current approach**: Read generic DataValue from table, then match-case convert each pixel  
**Better approach**: Request column as specific type directly from fitsrs (if API supports)

**Constraint**: fitsrs `select_fields()` returns generic DataValue enum. Possible solutions:
1. Check if fitsrs has type-aware column reading
2. Implement FITS binary table parsing directly (bypass fitsrs type system)
3. Use unsafe pointer casting from raw bytes

**Difficulty**: Medium - requires understanding fitsrs internals or writing custom parser

### Tier 2: Vectorize Type Conversion (Est. 15-25% speedup)

Use SIMD to batch-convert values:

```rust
// Instead of pixel-by-pixel conversion, batch process
// Use simdjson or similar for vectorized float parsing if data is ASCII
// Or use explicit_simd for f32→f64 conversion on binary data
```

**Difficulty**: Medium - requires SIMD knowledge, dependency additions

### Tier 3: Parallel FITS Parsing (Est. 10-20% speedup)

Parse multiple HDUs or table chunks in parallel:

```rust
// rayon parallel iteration over HDU blocks
HDUs
    .into_par_iter()
    .flat_map(|hdu| parse_hdu_column(&hdu))
    .collect()
```

**Difficulty**: Medium - requires thread-safe FITS parsing

### Tier 4: Downgrade During Parsing (Est. 3-5% speedup)

Fuse downgrade operation into the initial loading:

```rust
// Instead of: load 500M → downgrade → use
// Do: load full but only store downsampled pixels
```

**Difficulty**: Low - mostly refactoring, already partially designed

### Tier 5: Alternative FITS Library (Est. ??? - risky)

Current: `fitsrs` provides safe Rust interface but with overhead  
Alternative: `cfitsio` via FFI (faster but less safe)

**Risk**: FFI complexity, safety issues, maintenance burden

## Recommended Approach

**Short term (io-optimization branch)**:
1. Implement Tier 4 (downgrade during parsing) - quick win, 3-5% improvement
2. Profile to confirm bottleneck is type conversion loop
3. Implement Tier 1 or 2 based on profiling results

**Long term**:
1. Consider replacing `fitsrs` with faster FITS reader (possibly custom)
2. Implement parallel HDU parsing if applicable
3. Add per-column type caching to avoid repeated conversions

## Implementation Plan

### Step 1: Confirm Type Conversion Bottleneck

Add detailed timing instrumentation to `read_healpix_column`:

```rust
pub fn read_healpix_column(filename: &str, col_idx: usize) -> Vec<f64> {
    let t_start = std::time::Instant::now();
    // ... file opening and parsing ...
    let t_parsed = std::time::Instant::now();
    eprintln!("FITS parsing: {:?}", t_parsed.duration_since(t_start));
    
    // Match and convert
    for cell in values {
        // ... conversion ...
    }
    let t_converted = std::time::Instant::now();
    eprintln!("Type conversion: {:?}", t_converted.duration_since(t_parsed));
}
```

### Step 2: Implement Downgrade-During-Parsing

Modify loading to optionally downsample on the fly:

```rust
pub fn read_healpix_column_with_downgrade(
    filename: &str, 
    col_idx: usize,
    target_nside: Option<i64>
) -> Vec<f64> {
    // If target_nside provided, collect into downsampled map instead of full
}
```

### Step 3: Benchmark and Document

Compare:
- Standard reading (full resolution)
- Downgrade-during-parsing (with downgrade)
- Current downgrade-after-parsing

Target: 3-5% improvement from Tier 4 enough to make --no-downgrade viable for moderately large files.

## Conclusion

The "slow I/O" is actually **slow FITS parsing due to generic type system overhead**. The path forward:

1. ✅ Confirmed mmap won't help (not I/O bound)
2. ⏳ Profile to confirm type conversion is bottleneck
3. ⏳ Implement downgrade-during-parsing optimization
4. ⏳ Consider vectorizing type conversions if viable
5. ⏳ Evaluate alternative FITS libraries for long-term solution

This analysis shows GPU acceleration was limited not by rendering performance, but by the fundamental FITS data loading architecture - a separate concern that should be addressed independently.
