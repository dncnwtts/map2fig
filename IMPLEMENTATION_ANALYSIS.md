# Implementation Analysis: Dipole/Monopole Removal & map_editor

## Current Status Summary

### ✅ Already Implemented: Adaptive Resolution (ud_grade)
map2fig **already has intelligent ud_grade implementation**:
- In `src/pipeline.rs`: `load_and_process_data()` automatically downgrades high-resolution maps
- Logic:
  1. If `nside > HIGH_RES_NSIDE_THRESHOLD` (12288)
  2. Calculate `target_nside` based on output resolution
  3. Downgrade map if needed to avoid aliasing
  4. Special case: Gnomonic projections force `effective_width = 32768` to preserve resolution

**User insight**: This means `--width` smaller than the map resolution will automatically downsample - exactly the behavior you implemented!

---

## Dipole/Monopole Removal Analysis

### Three Implementations Compared

#### 1. **Cosmoglobe/healpy (Python)**
```python
# From plottools.py:680-687
if remove_dip:
    if remove_dip == 'auto':
        gal_cut = 30  # Galactic cut in degrees
    m = hp.remove_dipole(m, gal_cut=gal_cut, nest=nest, copy=True)

elif remove_mono:
    if remove_mono == 'auto':
        gal_cut = 30
    m = hp.remove_monopole(m, gal_cut=gal_cut, nest=nest, copy=True)
```

**Implementation**: Thin wrapper around healpy functions
- Uses `gal_cut` parameter to exclude Galactic plane from fit
- Fits monopole/dipole to remaining pixels
- Subtracts fitted values from full map

#### 2. **Cosmotools/map_editor (Fortran 90)**
```fortran
! From map_editor.f90:640-665
else if (trim(operation) == 'subtract_mono_dipole') then
    if (iargc() /= 4 .and. iargc() /= 8) then
       write(*,*) 'Usage: map_editor subtract_mono_dipole [input] [mask] [output]'
    endif
    
    call subtract_mono_dipole(mapname_in1, maskfile, nside, ordering, map, header, md)
    ! Optional: Output monopole/dipole components as md(1:4)
    ! md(1) = monopole
    ! md(2:4) = dipole components (x, y, z)
endif
```

**Features**:
- Takes explicit mask file (binary file with masked pixels)
- Optionally outputs fitted monopole/dipole values
- High-performance Fortran implementation
- Works on FITS files directly

#### 3. **Proposed for map2fig (Rust)**

---

## Architecture Recommendations

### Option A: Minimal (CLI-only dipole removal)
**Effort**: Low (~200 lines)
**Benefits**: Simple to implement, lightweight

Add to CLI:
```rust
--dipole        // Remove dipole (optional mask file)
--monopole      // Remove monopole
--dip-mask      // Mask file (FITS or binary)
--dip-gal-cut   // Galactic cut in degrees (default: 30)
```

Implementation in `src/pipeline.rs`:
1. After reading map, compute monopole/dipole via least-squares fit
2. Subtract from map before rendering
3. Optional: Output fitted values to stderr

### Option B: Full-featured (like map_editor)
**Effort**: Medium (~500 lines)
**Benefits**: Complete flexibility, matches Cosmotools capability

Add to CLI:
```rust
--dipole-fit [mask_file]     // Output monopole/dipole to separate file
--output-dipole [file]       // Save fitted dipole components
--include-galactic-plane     // Include Galactic plane in fit (default: exclude)
```

---

## map_editor Integration Strategy

### What map_editor Does Well
From the help text, it supports:
- **Simple operations**: scale, add, multiply, log, exp, sqrt, asinh, etc.
- **Complex operations**: smooth, ud_grade, subtract_mono_dipole, fit_gain_offset
- **Analysis**: print_stats, compute_spectral_index, statistical operations
- **Mask operations**: expand_mask, apply_mask, create source masks
- **Advanced**: beam operations, noise generation, polarization handling

### Why Add it to map2fig?
map_editor is powerful but **not designed for visualization**:
- Creates intermediate FITS files
- Requires manual chaining of operations
- CLI is cryptic and not user-friendly

**Better approach**: Add most-used operations **selectively** to map2fig:
1. Dipole/monopole removal (commonly needed before plotting)
2. Smooth (already have FWHM)
3. ud_grade (already have, but could expose as CLI option)
4. Scale/add_offset (already have as `--scale`)

**Don't add**: Statistical analysis, advanced Fortran operations - those stay in map_editor

---

## Recommended Implementation Path

### Phase 1: Dipole/Monopole Removal (IMMEDIATE)
**Benefit**: High (common preprocessing)
**Effort**: Low (~2 hours)

```rust
// In src/pipeline.rs

pub fn subtract_monopole(
    map: &mut [f64], 
    mask: Option<&[bool]>
) -> f64 {
    // Least-squares fit of monopole to unmasked pixels
    // Return fitted monopole value
}

pub fn subtract_dipole(
    map: &mut [f64],
    mask: Option<&[bool]>,
    meta: HealpixMeta
) -> [f64; 3] {
    // Fit dipole in 3D (or decomposed to x/y/z)
    // Return dipole components
}
```

CLI additions:
```rust
#[arg(long, help="Remove monopole from map")]
dipole: bool,

#[arg(long, help="Remove monopole from map")]
monopole: bool,

#[arg(long, help="Mask file (FITS or binary) for dipole fit")]
dipole_mask: Option<String>,

#[arg(long, default_value="30", help="Galactic cut in degrees")]
dipole_gal_cut: f64,
```

### Phase 2: Extended Mask Support (OPTIONAL)
**Benefit**: Medium
**Effort**: Medium (~4 hours)

Allow explicit mask file input instead of just gal_cut:
- Support FITS binary mask files
- Support raw binary masks
- Document format

### Phase 3: Output Dipole Components (OPTIONAL)
**Benefit**: Low (mostly for analysis)
**Effort**: Low (~1 hour)

```rust
--output-dipole-values   // Print to stdout: mono, dip_x, dip_y, dip_z
```

---

## Cosmoglobe vs map2fig Comparison

### Cosmoglobe Approach
```python
# Remove dipole with automatic gal_cut
plot(..., remove_dip=True, ...)

# Or explicit mask
plot(..., remove_dip='auto', gal_cut=40, ...)
```
**Pros**: Simple, integrated
**Cons**: Only works in Python API

### map2fig Approach (Proposed)
```bash
# Remove dipole with default gal_cut
map2fig -f map.fits -o plot.pdf --dipole

# Remove dipole with custom gal_cut
map2fig -f map.fits -o plot.pdf --dipole --dipole-gal-cut 40

# Remove with explicit mask file
map2fig -f map.fits -o plot.pdf --dipole --dipole-mask mask.fits
```
**Pros**: CLI-friendly, flexible, reproducible
**Cons**: Requires separate arguments

---

## Implementation Code Sketch

### Least-Squares Dipole Fit
```rust
/// Fit and subtract dipole from map
pub fn subtract_dipole(
    map: &mut [f64],
    nside: u32,
    ordering: HealpixOrdering,
    mask: Option<&[bool]>,
) -> Result<[f64; 3], String> {
    use nalgebra::{Matrix, Vector};
    
    let npix = map.len() as u32;
    let mut A = Matrix::zeros(npix as usize, 4); // [1, x, y, z]
    let mut b = Vector::zeros(npix as usize);
    let mut count = 0;
    
    for i in 0..npix {
        // Skip masked pixels
        if let Some(m) = mask {
            if m[i as usize] { continue; }
        }
        
        // Skip bad pixels
        if !map[i as usize].is_finite() { continue; }
        
        // Get HEALPix pixel coordinates
        let (theta, phi) = pix2ang(nside, ordering, i);
        
        // Build design matrix row
        let (x, y, z) = sphere_coords(theta, phi);
        A[(count, 0)] = 1.0;
        A[(count, 1)] = x;
        A[(count, 2)] = y;
        A[(count, 3)] = z;
        
        b[count] = map[i as usize];
        count += 1;
    }
    
    // Solve: A^T A x = A^T b
    let solution = (A.transpose() * &A)
        .lu()
        .solve(&(A.transpose() * &b))
        .ok_or("Singular matrix in dipole fit")?;
    
    // Subtract fitted monopole+dipole from map
    for i in 0..npix as usize {
        let (theta, phi) = pix2ang(...);
        let (x, y, z) = sphere_coords(theta, phi);
        let fitted = solution[0] + solution[1]*x + solution[2]*y + solution[3]*z;
        map[i] -= fitted;
    }
    
    Ok([solution[1], solution[2], solution[3]])
}
```

---

## Decision Matrix

| Aspect | Implementation | Effort | Value |
|--------|---|---|---|
| Dipole/Monopole removal | Phase 1 | Low | High |
| Mask file support | Phase 2 | Medium | Medium |
| Output dipole values | Phase 3 | Low | Low |
| Full map_editor integration | N/A | Very High | Low |

**Recommendation**: Implement Phase 1 first (dipole/monopole removal), then evaluate user demand for Phases 2-3.

---

## Files to Modify

1. `src/pipeline.rs` - Add `subtract_monopole()` and `subtract_dipole()` functions
2. `src/cli.rs` - Add `--dipole`, `--monopole`, `--dipole-mask`, `--dipole-gal-cut` arguments
3. `src/main.rs` - Call dipole/monopole removal in pipeline
4. `Cargo.toml` - Add `nalgebra` for linear algebra (if implementing least-squares fit)

---

## Alternative: Use healpy Bindings

Could expose healpy's `remove_dipole/remove_monopole` via Rust bindings:
- Pros: Proven implementation, matches Cosmoglobe
- Cons: Adds Python dependency to Rust project (defeats purpose)

**Not recommended** - Rust implementation is better for standalone CLI tool.

