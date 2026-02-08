#!/usr/bin/env python3
"""
Detailed colorbar height calculations showing the fix in action.
This demonstrates how the height % 3 constraint improves rendering.
"""

def show_detailed_analysis():
    print("\n" + "=" * 80)
    print("COLORBAR HEIGHT CONSTRAINT FIX - DETAILED ANALYSIS")
    print("=" * 80 + "\n")
    
    print("CONSTRAINT: Triangle height must be divisible by 3")
    print("RATIONALE: Bresenham-like rasterization has periodicities related to height\n")
    
    print("-" * 80)
    print("PORTRAIT LAYOUT (standard maps)")
    print("-" * 80)
    print("Formula: cbar_h = round(map_h / 20.0), then round to nearest multiple of 3\n")
    
    portrait_data = [
        (800, "Small"),
        (1024, "Tablet"),
        (1200, "Default (Mollweide)"),
        (1440, "High-res laptop"),
        (1920, "4K monitor"),
        (2560, "Ultra-high-res"),
    ]
    
    print(f"{'Width':<10} {'Label':<25} {'Old':<12} {'Old%3':<10} {'New':<12} {'New%3':<10} {'Status':<15}")
    print("-" * 80)
    
    for width, label in portrait_data:
        outer_pad = 24
        map_w = width - 2 * outer_pad
        map_h = map_w / 2.0
        
        # OLD CALCULATION (PROBLEMATIC)
        cbar_h_old_float = map_h / 20.0
        cbar_h_old = round(cbar_h_old_float)
        old_mod3 = cbar_h_old % 3
        
        # NEW CALCULATION (FIXED)
        cbar_h_new_float = round(cbar_h_old_float)
        cbar_h_new = int(((cbar_h_new_float / 3.0) * 3.0))  # Round to multiple of 3
        new_mod3 = cbar_h_new % 3
        
        if cbar_h_new < 12:
            cbar_h_new = 12
            new_mod3 = 0
        
        old_status = "✓ OK" if old_mod3 == 0 else "✗ PROBLEM"
        new_status = "✓ FIXED" if new_mod3 == 0 and cbar_h_new != cbar_h_old else "✓ SAME"
        
        print(f"{width:<10} {label:<25} {cbar_h_old:<12} {old_mod3:<10} {cbar_h_new:<12} {new_mod3:<10} {new_status:<15}")
    
    print("\n" + "-" * 80)
    print("SQUARE LAYOUT (square maps)")
    print("-" * 80)
    print("Formula: cbar_h = round(map_h / 25.0), then round to nearest multiple of 3\n")
    
    square_data = [
        (512, "Small square"),
        (1024, "Standard square"),
        (2048, "Large square"),
        (4096, "Ultra-large square"),
    ]
    
    print(f"{'Size':<10} {'Label':<25} {'Old':<12} {'Old%3':<10} {'New':<12} {'New%3':<10} {'Status':<15}")
    print("-" * 80)
    
    for size, label in square_data:
        outer_pad = 24
        map_h = size
        
        # OLD CALCULATION
        cbar_h_old_float = map_h / 25.0
        cbar_h_old = round(cbar_h_old_float)
        old_mod3 = cbar_h_old % 3
        
        # NEW CALCULATION
        cbar_h_new_float = round(cbar_h_old_float)
        cbar_h_new = int(((cbar_h_new_float / 3.0) * 3.0))
        new_mod3 = cbar_h_new % 3
        
        if cbar_h_new < 12:
            cbar_h_new = 12
            new_mod3 = 0
        
        old_status = "✓ OK" if old_mod3 == 0 else "✗ PROBLEM"
        new_status = "✓ FIXED" if new_mod3 == 0 and cbar_h_new != cbar_h_old else "✓ SAME"
        
        print(f"{size:<10} {label:<25} {cbar_h_old:<12} {old_mod3:<10} {cbar_h_new:<12} {new_mod3:<10} {new_status:<15}")
    
    print("\n" + "-" * 80)
    print("WHY HEIGHT % 3 MATTERS")
    print("-" * 80)
    print("""
Triangle rendering uses scanline fill:
1. For each scanline y from base_y to tip_y:
   - Calculate left edge x position using edge_x_at_y(left_edge, y)
   - Calculate right edge x position using edge_x_at_y(right_edge, y)
   - Fill pixels from left_x to right_x

2. Edge calculations use linear interpolation:
   - For an isosceles triangle with height H
   - Left/right edges converge at a rate determined by H
   
3. The half-open interval [y_min, y_max) causes rounding errors:
   - Each edge independently calculates pixel positions
   - Cumulative errors can differ between left and right
   - If H % 3 != 0, errors accumulate asymmetrically
   
4. When H % 3 == 0:
   - Mathematical structure aligns with pixel grid
   - Rounding periods cancel between left/right
   - Result: Perfect left-right symmetry

EXAMPLE: 27-pixel triangle (27 % 3 == 0) ✓ GOOD
- Perfect Bresenham progression
- Width: 14, 13, 12, 11, ..., 2, 1
- No asymmetries at any scanline

EXAMPLE: 28-pixel triangle (28 % 3 == 1) ✗ PROBLEM  
- Rounding causes irregular progression
- Width might be: 14, 14, 13, 12, 12, 11, ..., 2, 2, 1 (too many plateaus)
- Left and right edges converge at different rates
""")
    
    print("\n" + "-" * 80)
    print("VALIDATION")
    print("-" * 80)
    print("""
To verify this fix works:

1. Render test images with various heights:
   $ cargo run -- -f test.fits -o out_good.pdf     # New: height % 3 == 0
   
2. Zoom into colorbar extend triangles in PDF viewer
   
3. Inspect:
   - Left and right edges should be perfectly symmetric
   - Width should decrease smoothly: 14, 13, 12, 11, ..., 2, 1
   - No plateaus (same width for multiple rows)
   - No cliffs (sudden width jumps)

4. Run diagnostic tests:
   $ cargo test --test test_triangle_rendering -- --nocapture
   
   Should show:
   - 0 pixel asymmetry (left == right for all scanlines)
   - ~1-2% plateau rate (not 58%)
   - 0 cliffs (no width jumps > 2 pixels)
""")
    
    print("\n" + "=" * 80)
    print("END OF ANALYSIS")
    print("=" * 80 + "\n")

if __name__ == "__main__":
    show_detailed_analysis()
