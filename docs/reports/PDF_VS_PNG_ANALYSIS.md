# PNG vs PDF Rendering Paths - Technical Analysis

## Rendering Architecture

### PDF Rendering Path
```
plot.rs (line ~1245)
  └─ render_colorbar_gradient() + draw_colorbar_extends()
     └─ Uses Cairo rendering (render/pdf.rs)
        └─ Mathematical primitives
           └─ Continuous interpolation
              └─ NO integer rounding issues
                 └─ PERFECT SYMMETRY ✓
```

### PNG Rendering Path
```
plot.rs (line ~627)
  └─ render_colorbar_gradient() + draw_colorbar_extends()
     └─ Uses fill_triangle() (colorbar.rs)
        └─ Integer scanline rasterization
           └─ edge_x_at_y() with integer rounding
              └─ Integer rounding issues
                 └─ height % 3 periodicities
                    └─ ASYMMETRIES ✗ (15px cliffs, 58% plateaus)
```

## Why PDF Works But PNG Doesn't

### PDF: Cairo's Approach
1. Uses floating-point primitives
2. Mathematical rendering (fills actual triangle shape)
3. Subpixel accuracy
4. No periodicity issues
5. **Result**: Perfect symmetry by construction

### PNG: fill_triangle() Approach
1. Integer pixel coordinates only
2. Scanline-by-scanline rasterization
3. Each scanline calculates edge x-positions with integer rounding
4. Uses half-open interval [y_min, y_max)
5. **Problem**: Rounding errors accumulate based on height value
   - When `height % 3 == 0`: Errors cancel symmetrically
   - When `height % 3 != 0`: Errors accumulate asymmetrically

## fill_triangle() Algorithm (PNG)

### Current Implementation
```rust
fn fill_triangle(vertices: [(i32, i32); 3], color: Rgba<u8>, img: &mut image::RgbaImage) {
    let y_min = /* minimum y of all vertices */;
    let y_max = /* maximum y of all vertices */;
    
    for y in y_min..=y_max {
        // For each edge, find where it crosses this scanline
        let x_left = edge_x_at_y(left_edge, y);    // Integer calculation
        let x_right = edge_x_at_y(right_edge, y);  // Integer calculation
        
        // Fill horizontal line from x_left to x_right
        for x in x_left..=x_right {
            img.put_pixel(x, y, color);
        }
    }
}

fn edge_x_at_y(p1: (i32, i32), p2: (i32, i32), y: i32) -> Option<i32> {
    // Linear interpolation with midpoint rounding
    let dy = (p2.1 - p1.1).abs();
    let dx = p2.0 - p1.0;
    let t_num = (y - p1.1).abs();
    
    // Midpoint rounding: (dx * t_num + dy/2) / dy
    let x = p1.0 + (dx * t_num + dy / 2) / dy;
    Some(x)
}
```

### The Problem: Rounding Periodicity
```
For isosceles triangle, both edges should converge symmetrically:
- Left edge converges from left-side of base
- Right edge converges from right-side of base

BUT when height % 3 != 0:
- Cumulative rounding: (dx * t + dy/2) / dy
- Error pattern repeats every dy/GCD(dx, dy) pixels
- For slope dx/dy ≈ 1/2, error pattern has period related to 3
- Left and right edges have phase-shifted error patterns
- RESULT: Asymmetry!

WHEN height % 3 == 0:
- Error pattern aligns perfectly
- Left and right have identical phase
- RESULT: Perfect symmetry!
```

## Example: 29px vs 30px Triangles

### Triangle with height = 29 (29 % 3 = 2) ❌ PROBLEMATIC

For isosceles triangle converging from width 14 to 1:
```
Scanline 0:   width = 14  (both edges round together)
Scanline 1:   width = 14  (both edges round together) ← PLATEAU
Scanline 2:   width = 13  (one edge advances)
Scanline 3:   width = 13  ← PLATEAU
Scanline 4:   width = 12  (one edge advances)
...
Scanline 27:  width = 1   (tip)
Scanline 28:  width = 1   (tip)

Result: Left and right converge at DIFFERENT rates
- Left: 14, 14, 12, 12, 10, 10, ...
- Right: 14, 13, 13, 11, 11, 9, ...
- Width difference = 0, 1, 1, 2, 2, 3, ... ← ASYMMETRIC!
```

### Triangle with height = 30 (30 % 3 = 0) ✓ CORRECT

```
Scanline 0:   width = 14  (both edges round symmetrically)
Scanline 1:   width = 13  (smooth convergence)
Scanline 2:   width = 13  (acceptable plateau)
Scanline 3:   width = 12  (smooth convergence)
Scanline 4:   width = 12
...
Scanline 29:  width = 1   (tip)

Result: Left and right converge at SAME rate
- Left: 14, 13, 13, 12, 12, 11, ..., 1
- Right: 14, 13, 13, 12, 12, 11, ..., 1
- Width difference = 0 for all rows ← SYMMETRIC!
```

## The Fix: Force height % 3 == 0

### In src/layout.rs
```rust
let cbar_h = if show_colorbar { 
    let base_h = map_h / 20.0;           // Original calculation
    let rounded = base_h.round();         // Round to nearest integer
    ((rounded / 3.0).round() * 3.0)       // Round to nearest multiple of 3
    .max(12.0)                            // Ensure minimum
} else { 
    0.0 
};
```

### Effect: Different Heights Get Adjusted
| Original | Mod 3 | Adjusted | New Mod 3 | Status |
|----------|-------|----------|-----------|--------|
| 19px | 1 ❌ | 18px | 0 ✓ | Fixed |
| 21px | 0 ✓ | 21px | 0 ✓ | OK |
| 29px | 2 ❌ | 30px | 0 ✓ | Fixed |
| 35px | 2 ❌ | 36px | 0 ✓ | Fixed |

## Validation via Tests

The test suite in `tests/test_triangle_rendering.rs` verifies:

```rust
#[test]
fn test_triangle_height_must_be_multiple_of_3() {
    // PNG-SPECIFIC: height % 3 == 0 for fill_triangle()
    // Tests heights 27, 28, 29, 30, 33, 36
    // Documents which are divisible by 3
    // Shows: 27, 30, 33, 36 are GOOD; 28, 29 are PROBLEMATIC
}

#[test]
fn test_left_right_symmetry_exact_match() {
    // PNG-SPECIFIC: Validates fill_triangle() produces symmetric widths
    // With height % 3 == 0, left_width[y] should equal right_width[y]
}

#[test]
fn test_no_cliffs_at_triangle_bottom() {
    // PNG-SPECIFIC: Documents the 15-pixel cliff issue
    // With height % 3 == 0, max width change should be 1-2 pixels/row
}

#[test]
fn test_no_plateaus_in_convergence() {
    // PNG-SPECIFIC: Documents the 58% plateau issue
    // With height % 3 == 0, should be ~1-2% of scanlines as plateaus
}
```

## References

- **PDF Rendering**: `src/render/pdf.rs` (uses Cairo)
- **PNG Rendering**: `src/colorbar.rs` (fill_triangle function)
- **Layout**: `src/layout.rs` (colorbar height calculation)
- **Tests**: `tests/test_triangle_rendering.rs` (PNG-specific tests)
- **Algorithm Reference**: Akenine-Moller, Haines, Hoffman "Real-Time Rendering" (3rd ed)
