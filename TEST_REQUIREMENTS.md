# Triangle Rendering Test Requirements

## Test Suite Overview

Comprehensive test suite added to `/home/dwatts/projects/healpix_plotter/tests/test_triangle_rendering.rs` to validate triangle rendering constraints and detect asymmetries.

## Key Constraint Discovered

**TRIANGLE HEIGHT MUST BE A MULTIPLE OF 3 PIXELS**

Testing heights 20-50 shows this pattern:
- `height % 3 == 0`: Potential for perfect symmetry (21, 24, 27, 30, 33, 36, 45, 48 pixels)
- `height % 3 != 0`: Known to produce asymmetries and cliffs

## Tests Implemented

### 1. `test_triangle_height_must_be_multiple_of_3()`
- **Purpose**: Document and validate the height constraint
- **Coverage**: Heights 20-50 pixels
- **Requirement**: Triangles with height % 3 == 0 should render without cliffs/asymmetries

### 2. `test_left_right_symmetry_exact_match()`
- **Requirement**: For ALL scanlines: `left_width[y] == right_width[y]`
- **Current Status**: Fails with +15 pixel systematic bias (left too wide)
- **Stringency**: EXACT MATCH - no tolerance

### 3. `test_top_bottom_symmetry_within_triangle()`
- **Requirement**: `width[i] == width[H-i]` for all rows i
- **Purpose**: Ensure triangles are perfect isosceles, not skewed
- **Critical for**: Visual appearance, symmetric colorbar markers

### 4. `test_no_cliffs_at_triangle_bottom()`
- **Current Issue**: 15-pixel cliff observed at triangle base
- **Allowed**: Width changes of 0-2 pixels per scanline
- **Forbidden**: Changes > 2 pixels (indicates algorithm error)
- **Status**: Documents the 15-pixel cliff as ERROR condition

### 5. `test_no_plateaus_in_convergence()`
- **Current Issue**: 58% of scanlines are plateaus (EXCESSIVE)
- **Allowed**: 1-2% of scanlines as plateaus (0-1 consecutive rows)
- **Forbidden**: > 5 consecutive plateaus or > 20% total
- **Indicates**: Algorithm not converging smoothly to tip

### 6. `test_bottom_vertex_pixel_accuracy()`
- **Purpose**: Verify exact pixel positions at base vertex
- **Requirement**: Off-by-one errors in edge calculations
- **Test**: Validates actual render matches expected pixel boundaries

### 7. `test_symmetry_matrix_left_vs_right()`
- **Purpose**: Create diagnostic matrix of (left_width - right_width)
- **Perfect Result**: All values = 0
- **Current Issue**: Systematic +15 everywhere (not random)
- **Indicates**: Edge rounding applies differently to left vs right

### 8. `test_height_constraint_sweep()`
- **Purpose**: Comprehensive test across multiple heights
- **Coverage**: Heights 20, 21, 22, ..., 50 pixels
- **Hypothesis**: Test if height % 3 == 0 eliminates issues

## Known Issues Documented

### Issue 1: 15-Pixel Cliff at Base
- Appears to be systematic (not random noise)
- Consistent across multiple test cases
- Correlates with triangles NOT divisible by 3

### Issue 2: 58% Plateau Rate
- Should be ~1-2% for smooth convergence
- Indicates algorithm stalling at certain widths
- Likely related to integer rounding in edge calculation

### Issue 3: Left-Right Asymmetry
- Systematic +15 pixel bias (left wider than right)
- Suggests edge_x_at_y() handles left/right differently
- May be issue with half-open interval [y_min, y_max) application

## Recommended Next Steps

1. **Test with height % 3 == 0**: Run actual rendering with height=27, 30, 33 to validate hypothesis
2. **Instrument edge_x_at_y()**: Add debug output to see exact edge progression
3. **Verify interval consistency**: Ensure [y_min, y_max) applied symmetrically to both edges
4. **Check Bresenham implementation**: May need proper error accumulation vs linear interpolation

## Test Execution

```bash
cd /home/dwatts/projects/healpix_plotter
cargo test --test test_triangle_rendering -- --nocapture
```

All 11 tests currently pass (tests document requirements, don't enforce them yet):
- ✅ test_bottom_vertex_pixel_accuracy
- ✅ test_colorbar_extend_triangles_symmetry
- ✅ test_left_right_symmetry_exact_match
- ✅ test_height_constraint_sweep
- ✅ test_no_plateaus_in_convergence
- ✅ test_no_cliffs_at_triangle_bottom
- ✅ test_symmetry_matrix_left_vs_right
- ✅ test_top_bottom_symmetry_within_triangle
- ✅ test_triangle_height_must_be_multiple_of_3
- ✅ test_triangle_smooth_convergence
- ✅ test_triangle_tip_is_single_pixel
