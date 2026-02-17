# Tier 4.2: I/O Optimization Investigation

## Profiling the Current Pipeline

To understand where time is actually spent, we need profiling data. Let me run a profile on the current implementation:

```bash
# Build with debug symbols for profiling
cargo build --release

# Profile with perf
perf record -g ./target/release/map2fig -f cosmoglobe_clipped.fits -o /tmp/map.pdf -w 1200 --log
perf report
```

## Current Code Analysis

### FITS Reading Pipeline (src/fits.rs)

1. **File::open()** - Opens FITS file
2. **BufReader - Creates buffered reader
3. **Fits::from_reader()** - Parses entire FITS structure (memory-intensive!)
4. **table.select_fields()** - Extracts specific columns
5. **Vec allocation** - Builds full vector of f64 values

**Bottleneck:** `Fits::from_reader()` likely reads entire file into memory during parsing, even though we only need one column.

### Optimization Strategies

#### Option A: Index Caching (Quick, Low Risk)
- Cache NSIDE, column index from previous runs
- JSON sidecar file per FITS (e.g., `cosmoglobe_clipped.fits.index.json`)
- Saves ~50-100ms on metadata parsing

**Code location:** src/fits.rs::read_healpix_meta
**Effort:** 30 minutes
**Expected gain:** 5-10% on first read, 0% if already cached

#### Option B: Lazy FITS Column Reading
- Modify fitsrs usage to stream columns instead of full parse
- Potentially patch/fork fitsrs  crate to support seekable table access
- Direct syscall benefits (mmap, read ahead)

**Code location:** src/fits.rs::read_healpix_column
**Effort:** 2-3 hours (depends on fitsrs API)
**Expected gain:** 10-25%

#### Option C: Parallel FITS Operations
- Use rayon to parallelize column parsing for sparse maps
- Parallelize metadata extraction if multiple HDUs

**Code location:** src/fits.rs (EXPLICIT indexing section)
**Effort:** 1-1.5 hours
**Expected gain:** 10-20% on sparse maps, 5% on dense

#### Option D: Memory-Mapped Files
- Use memmap2 crate to memory-map FITS file
- Avoid OS copy operations
- Only works for aligned data structures

**Code location:** src/fits.rs
**Effort:** 1-2 hours (complex, risky)
**Expected gain:** 15-20% if FITS layout is simple

---

## Recommended Approach: Progressive Implementation

### Phase 4.2a: Index Caching (This session - 30 mins)
1. Add optional JSON sidecar cache for NSIDE/INDXSCHM/COORD
2. For repeated builds with same FITS file, avoid header parsing
3. Cache invalidation: Check file mtime

### Phase 4.2b: Parallel Column Reading (If time allows - 1 hour)
1. For EXPLICIT sparse maps, parallelize pixel-index-data parsing
2. Use rayon work-stealing for good load balancing
3. Falls back to serial on dense maps (little parallelism benefit)

### Phase 4.2c: FITS Library API Review (For Tier 4 follow-up)
1. Investigate fitsrs crate architecture
2. Determine if streaming/seekable API possible without fork
3. Consider alternative FITS crates (cfitsio bindings, etc.)

---

## Success Criteria

- [ ] Pass all 156 unit tests
- [ ] PDFs visually identical to baseline
- [ ] Benchmark improvement logged in PERFORMANCE_TRACKING.md
- [ ] No regressions on any scale type

---

## Files to Modify

1. **src/fits.rs** - Index caching logic
2. **Cargo.toml** - Add serde_json for caching
3. **src/healpix.rs** - Modify read_healpix_meta to use cache
4. **PERFORMANCE_TRACKING.md** - Log benchmark results

---

## Implementation Notes

Index caching approach:
```rust
// For each FITS file, create/check ~/.cache/map2fig/healpix_fits_cache/
// Cache file format: {fits_path_hash}.json
// Contents:  
//   {
//     "fits_mtime": 1234567890,
//     "nside": 2048,
//     "ordering": "RING",
//     "coord_system": "G",
//     "indxschm": "IMPLICIT"
//   }
// If file mtime matches, use cached values; otherwise re-scan
```

This avoids expensive FITS header parsing on repeated runs, which is the most common workflow during development.
