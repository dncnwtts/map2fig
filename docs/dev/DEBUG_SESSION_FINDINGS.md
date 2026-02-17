# F32 Optimization Debug Session - Findings & Status

## Session Summary
Investigation into why the f32 native reader optimization wasn't achieving expected performance improvements (60% faster, 50% memory reduction). Discovered critical bugs in FITS data offset calculation and byte order handling.

## Key Findings

### Bug #1: FITS Data Offset Calculation (FIXED)
**Problem**: For multi-HDU FITS files (primary HDU + extension), the offset calculation was finding the PRIMARY HDU's end marker instead of the BINTABLE extension's end marker.

**Impact**: Reading data from wrong file offset (2880 instead of 5760), resulting in garbage data being interpreted.

**Root Cause**: `find_binary_table_data_offset()` only looked for the FIRST "END" keyword, not the LAST one.

**Fix Applied**:
- Updated function to search all END keywords and use the LAST one
- Correctly calculate data offset as block after the BINTABLE extension's END keyword
- Now handles both single-HDU and multi-HDU files correctly

**File**: `src/fits.rs` lines ~320-365 (`find_binary_table_data_offset` function)

### Bug #2: FITS Byte Order (FIXED)
**Problem**: Reading float data as little-endian when FITS standard specifies big-endian (network byte order).

**Impact**: All float values were being byte-swapped, resulting in completely incorrect data (e.g., 1.0 read as -inf or garbage).

**Root Cause**: Used `f32::from_le_bytes()` and `f64::from_le_bytes()` instead of big-endian variants.

**Fix Applied**:
- Changed `from_le_bytes()` to `from_be_bytes()` in both f32 and f64 readers
- Matches FITS binary table standard (IEEE 754 in big-endian format)

**Files**: `src/fits.rs` lines ~190-195 (f32 reader), ~260-265 (f64 reader)

### Bug #3: Missing TOFFSET Handling (FIXED) 
**Problem**: TOFFSET keywords are optional in FITS (default to 0 for first column) but code returned None if not found.

**Impact**: Single-column dense maps couldn't use native readers because TOFFSET wasn't present.

**Root Cause**: Strict header parsing didn't handle FITS standard default values.

**Fix Applied**:
- For col_idx=0, default to offset=0 (FITS standard)
- For multi-column tables without TOFFSET, still return None (requires further work)

**Files**: `src/fits.rs` lines ~132-150 (f32), ~255-270 (f64)

### Issue #4: Test Compilation (FIXED)
**Problem**: Tests using `data[idx]` indexing failed because DataArray doesn't implement Index trait.

**Solution**: Changed test code to use `data.get(idx).expect("pixel out of bounds")` API.

**Files**: `src/fits.rs` lines 1058-1073 (various test methods)

## Current Status

### ✅ Completed
1. Fixed multi-HDU file offset calculation
2. Fixed byte order endianness (little-endian → big-endian)
3. Fixed missing TOFFSET handling for single-column tables
4. Fixed test compilation and indexing issues
5. All 180 unit tests passing ✓

### ❓ Investigation Needed
Despite fixes, benchmark shows files still being detected as `float64`:
```
Data Types Detected:
  • f32 (native): 0 files  ←  UNEXPECTED!
  • f64 (native): 4 files
```

**Debug Trace** (from temporary eprintln output):
```
[f32] Entering try_read_float32_column_native, col_idx=0
[f32] Found XBinaryTable
[f32] NSIDE=8192
[f32] TFORM="1024E"
[f32] Parsed: elem_count=1024, type_char=E
[f32] ✓ Type is float32 (E)
[f32] f32 reader returned None, trying f64  ← ISSUE IS HERE
```

The f32 reader correctly:
- Detects XBinaryTable ✓
- Reads NSIDE ✓
- Parses TFORM correctly (type='E' for float32) ✓
- But then returns None

**Likely Cause**: Issue somewhere after type check, probably in:
1. `find_binary_table_data_offset()` - still returning None?
2. Data reading loop - bounds check failing?
3. Unexpected exception in parsing logic?

## Next Steps

To complete the f32 optimization:

1. **Add detailed debug logging to native readers**:
   - Log each step in `try_read_float32_column_native()` (lines 130-200)
   - Log all return statements with reason
   - Check `find_binary_table_data_offset()` output

2. **Test `find_binary_table_data_offset()` independently**:
   - Verify it returns correct offset for each test file
   - Trace END keyword search for multi-HDU files

3. **Check data bounds**:
   - Ensure row_start/row_end calculations don't exceed mmap length
   - Verify row_size, num_rows, elem_count parsed correctly

4. **Verify sparse map detection**:
   - Check INDXSCHM handling
   - Confirm EXPLICIT indexing properly returns None

## Performance Expectations (After Fix)

Once native f32 reader is fully working:
```
Large file (3.1 GB combined_map):
- Current: 10.9s (fallback path)
- Expected: 3.4s (68% faster)
```

## Files Modified
- `src/fits.rs`: Multi-HDU offset fix, byte order fix, TOFFSET handling, test fixes
- `src/data_array.rs`: No changes (working correctly)
- Test results: 180/180 passing ✓

## References
- FITS Standard: IEEE 754 floats in big-endian byte order (network byte order)
- HEALPix FITS Format: Uses IMPLICIT indexing (dense) or EXPLICIT (sparse)
- fitsrs: Binary table parsing used for header extraction only

## Related Issues to Watch
- Sparse map handling with EXPLICIT indexing (may need separate testing)
- Multi-column tables without TOFFSET (currently unsupported)
- Performance regression detection if changes made
