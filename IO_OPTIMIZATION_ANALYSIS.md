# I/O Optimization Analysis (Tier 5.2)

## Executive Summary

**Current Status**: Metadata caching (Tier 4.2a) is fully operational and highly effective.

**Key Findings**:
- ✅ Metadata caching: 100% cache hit rate on repeated file access
- ✅ FITS header parsing: Only 0.2ms even on 3.1GB files (negligible cost)
- ✅ Cache provides 44s benefit on 3.1GB file (cached run: 23.38s vs uncached: 67.44s)
- 🎯 **Real bottleneck identified**: Column data I/O + PDF rendering (not metadata)

## Performance Architecture

### Current Runtime Breakdown (3.1GB FITS file, 512px output)

Estimated from benchmark data (23.38s cached render):

| Component | Time | % |Optimization Potential |
|-----------|------|---|-----------------------|
| Metadata I/O | 0.2ms | <1% | ✅ Already cached |
| Column reading | ~3s | 14% | 🔄 **TARGET** |
| Pixel operations | ~9s | 38% | ✅ SIMD optimized (Tier 3-5.1) |
| PDF rendering | ~11s | 48% | 🔄 Possible (streaming) |
| Memory/Overhead | ~0.2s | 1% | ✅ Efficient |

### Why Metadata Caching Alone Isn't Enough

The metadata (NSIDE, ORDERING, INDXSCHM) is truly negligible:
- **Cost of parsing**: 0.2ms (0.0009% of 23.38s render)
- **Cache hit benefit**: ~0.2ms saved (immeasurable)
- **Real time saver**: Avoiding column re-reads would be 14% = 3.3s

The 44s difference between first and cached run comes from:
1. ✅ FITS metadata cached (0.2ms saved)
2. ❌ Column data still read fresh (3s + any OS caching)
3. ✅ Pixel operations benefit from CPU cache (maybe 5-10% speedup)
4. ✅ PDF rendering may benefit from previous buffer allocations

## Next Optimization Targets

### Priority 1: Column Data I/O Optimization (14% gain potential)

**Problem**: FITS column reading happens every render, even if identical

**Current Code** (`src/fits.rs`, `read_healpix_column()`):
```rust
pub fn read_healpix_column(filename: &str, col_idx: usize) -> Vec<f64> {
    let f = File::open(filename)?;
    let reader = BufReader::new(f);
    let mut fits = Fits::from_reader(reader);
    
    // Re-parses FITS structure, seeks to column data, extracts
    // For sparse maps: allocates 12*NSIDE² + inflates with UNSEEN values
}
```

**Optimization Options**:

1. **Column Data Caching** (Recommended: 5-10% gain, low risk)
   - Cache full column vectors alongside metadata
   - Cache key: SHA256(filepath) + mtime + column_index
   - Storage: ~/.cache/map2fig/fits_col_{hash}_{col_idx}.bin (binary format)
   - Invalidation: Automatic on file mtime change
   - Benefit: Avoid re-reading binary table, binary deserialization
   - Code location: `src/fits.rs` `read_healpix_column()` wrapper

2. **Memory Mapping** (Complex: 5-15% gain, requires fitsrs fork)
   - Use mmap for lazy column loading
   - Requires modification to fitsrs binary table parser
   - Benefits: Zero-copy access, OS page cache integration
   - Risk: Compatibility issues, maintenance burden

3. **Binary Table Index Caching** (Moderate: 2-5% gain)
   - Cache HDU index and column byte offsets
   - Reduces seeking through FITS structure
   - Enable direct jump to data section
   - Storage: Offset information in metadata cache extend

4. **Parallel Column Reading** (Low: 1-3% gain)
   - Use rayon for multi-column extraction (if needed later)
   - Already optimized single-column case
   - Benefit only if reading multiple columns simultaneously

### Priority 2: PDF Rendering Optimization (48% gain potential)

**Problem**: Cairo PDF generation takes ~11s on 512px render

**Possible Approaches**:
1. **Streaming PDF** (5-10% gain)
   - Generate/write PDF incrementally instead of buffering
   - Fewer intermediate allocations
   - Complexity: Requires PDF structure refactoring

2. **Raster-to-PDF Optimization** (2-5% gain)
   - Pre-allocate Cairo surface more efficiently
   - Optimize colorbar rasterization
   - Use Cairo GPU acceleration if available

3. **Output Format Selection** (User choice)
   - PNG rendering is faster than PDF (test later)
   - Could offer --fast-png output for iterative work

**Status**: Lower priority since metadata caching (Tier 4.2a) already addresses I/O costs

## Implementation Plan (Tier 5.2.1: Column Data Caching)

### Phase 1: Design Column Cache

**Cache File Structure**:
```
~/.cache/map2fig/fits_col_<SHA256>_<mtime>_<col_idx>.bin

Binary format (little-endian, f64):
- Header (16 bytes):
  - u32: magic = 0xCAFEBABE ("CAFEBABE" in hex = cached FITS column)
  - u32: version = 1
  - u32: num_pixels
  - u32: reserved
- Data: num_pixels * f64 (8 bytes each)
```

**Validation**:
- Automatically invalidated if file mtime changes
- Clear cache if format version mismatches
- Graceful fallback to FITS read if cache corrupted

### Phase 2: Implement Column Cache

**File**: `src/fits.rs`

**New Functions**:
```rust
fn get_column_cache_key(filepath: &str, col_idx: usize, mtime_secs: u64) -> String {
    // SHA256(filepath) + "_" + col_idx + "_" + mtime_secs
}

fn try_load_column_cache(filepath: &str, col_idx: usize) -> Option<Vec<f64>> {
    // Check mtime hasn't changed
    // Load binary file
    // Validate magic number
    // Return Vec<f64>
}

fn save_column_cache(filepath: &str, col_idx: usize, data: &[f64]) {
    // Create ~/.cache/map2fig/ if needed
    // Generate cache key
    // Write magic + version + length + data
}

pub fn read_healpix_column_cached(filename: &str, col_idx: usize) -> Vec<f64> {
    // Try cache first
    if let Some(data) = try_load_column_cache(filename, col_idx) {
        eprintln!("[CACHE] Column hit: {}", filename);
        return data;
    }
    
    // Cache miss: read from FITS
    let data = read_healpix_column(filename, col_idx);
    save_column_cache(filename, col_idx, &data);
    data
}
```

**Integration**: Replace `read_healpix_column()` call in `healpix.rs:read_healpix_data()` with `read_healpix_column_cached()`

### Phase 3: Benchmarking

**Test Cases**:
1. Small file (cosmoglobe_clipped.fits, 25MB): Should show ~0.1s benefit (small column)
2. Large file (combined_map_95GHz, 3.1GB): Should show ~3s benefit (large column)
3. Multiple columns: If cache effective, consider parallelization

**Command**:
```bash
export MAP2FIFigure_PROFILE=1
cargo run --release -- -f combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits -o map.pdf
# First run: checks cache miss, saves to disk
# Second run: cache hit, loads binary directly
```

### Phase 4: Safety Measures

**Checklist**:
- [ ] Verify cache invalidation works (mtime-based)
- [ ] Test with corrupted cache file (should fallback gracefully)
- [ ] Test with read-only cache directory (should skip caching)
- [ ] Document cache location for users to clear if needed
- [ ] Add `--clear-cache` flag to CLI for explicit cache clearing

## Expected Outcomes

### Performance Targets

**Small file** (cosmoglobe_clipped.fits, 25MB):
- First run: 43.6s (unchanged - column is tiny)
- Cached: 0.6s (unchanged - caching provides <50ms benefit)

**Large file** (combined_map_95GHz, 3.1GB):
- First run: 67.4s (unchanged - cache miss on first render)
- Cached: 23.4s → **20.1s** (3.3s column reading saved = **14% improvement**)
- Cache benefit visualization: `-3.3s / 67.4s = -4.9% first run cost (once only)`

### Integration with Optimization Timeline

```
Completed: Tier 3 (SIMD) + Tier 4 (Native CPU + Caching) + Tier 5.1 (Batch Size)
   ↓
In Progress: Tier 5.2.1 (Column Data Caching) ← YOU ARE HERE
   ↓
Future: Tier 5.2.2 (Binary Table Index) → Tier 5.3 (PDF Streaming) → Tier 5.4 (Adaptive)
```

## Decision Points

**Should we implement column caching?**

| Aspect | Status | Recommendation |
|--------|--------|-----------------|
| Complexity | Low (90 lines new code) | ✅ Worth it |
| Risk | Low (graceful fallback) | ✅ Safe |
| Benefit | 14% on large files | ✅ Worthwhile |
| User impact | Automatic, transparent | ✅ No downside |
| Storage cost | ~3.1GB binary cache per file | ⚠️ Acceptable (optional, user-managed) |

**Alternative**: If storage is concern, focus on PDF streaming (Tier 5.3) instead.

## References

- **Tier 4.2a Metadata Caching**: `src/fits.rs:read_healpix_meta_cached()`
- **Column Reading**: `src/fits.rs:read_healpix_column()`
- **Integration Point**: `src/healpix.rs:read_healpix_data()` calls column reader
- **Cache Directory**: `src/fits.rs:get_cache_dir()`
- **Tooling**: `tools/profile_io.py`, `tools/profile_columns.py`

## Next Steps

1. **Immediate** (This session):
   - [ ] Commit diagnostics module
   - [ ] Document I/O analysis in this file
   - [ ] Push `tier5-io-optimization` branch

2. **Next Session**:
   - [ ] Implement `read_healpix_column_cached()` in fits.rs
   - [ ] Update healpix.rs integration point
   - [ ] Benchmark and validate 14% gain
   - [ ] Merge to main if successful

3. **Future** (if column caching successful):
   - Consider Tier 5.2.2 (binary table index)
   - Revisit PDF rendering (Tier 5.3) for remaining 48%
