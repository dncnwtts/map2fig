#[test]
fn test_edge_jump_at_y77() {
    // Debug the specific Y coordinate where the jump happens
    
    // From colorbar dimensions: width=400, height=200, padding=50
    // Gradient: x=51..349 (299 pixels), y=50..150 (101 pixels)
    // Gradient center: x=200, y=100
    // Gradient left: x=51, right: x=349
    // Left extend tip: x = 51 - 50 = 1, y = 100 (tip_distance = 0.5 * 101 = 50.5)
    // Right extend tip: x = 349 + 50 = 399, y = 100
    // Base: y = 50 to 150
    
    // Triangle vertices (LEFT)
    // Tip: (1, 100) - wait, this doesn't match. The extends start outside the image
    // Let me recalculate...
    
    // With padding=50, colorbar positioning:
    // cbar_left_x = 51 (50 + 1 for gradient starting)
    // cbar_right_x = 350 (51 + 299, where 299 is gradient width)
    // tip_distance = 50 (approximately 0.5 * 100 for the colorbar height) 
    // LEFT tip_x = 51 - 50 = 1
    // RIGHT tip_x = 350 + 50 = 400 (outside image!)
    
    // But based on the test output, extends are visible at x=[1..399], so they must be:
    // LEFT tip at x=1, base at x=51
    // RIGHT tip at x=399, base at x=349
    
    // For LEFT triangle at y=77 (23 pixels from tip at y=100):
    // Base is at y=50..150
    // Distance from tip to y=77 is 100 - 77 = 23 pixels up from tip
    // T-parameter: t = 23 / 50 = 0.46
    
    let y = 77i32;
    let tip_y = 100i32;
    let base_top_y = 50i32;
    let _base_bottom_y = 150i32;
    
    // LEFT triangle
    let tip_x_left = 1i32;
    let base_x_left = 51i32;
    
    // RIGHT triangle
    let tip_x_right = 399i32;
    let base_x_right = 349i32;
    
    // Calculate edges at y=77
    // For the TOP edge (from tip to base_top):
    let t_to_base_top = (y as f64 - base_top_y as f64) / (tip_y as f64 - base_top_y as f64);
    let x_on_left_top = tip_x_left as f64 + t_to_base_top * (base_x_left as f64 - tip_x_left as f64);
    let x_on_right_top = tip_x_right as f64 + t_to_base_top * (base_x_right as f64 - tip_x_right as f64);
    
    eprintln!("y=77 (23 pixels from tip):");
    eprintln!("  Distance from base_top_y to y: {} - {} = {}", y, base_top_y, y - base_top_y);
    eprintln!("  Distance from base_top_y to tip_y: {} - {} = {}", tip_y, base_top_y, tip_y - base_top_y);
    eprintln!("  t (normalized position on edge) = {} / {} = {:.6}", y - base_top_y, tip_y - base_top_y, t_to_base_top);
    eprintln!("  LEFT top edge: x = {} + {:.6} * ({} - {}) = {:.3}", tip_x_left, t_to_base_top, base_x_left, tip_x_left, x_on_left_top);
    eprintln!("  RIGHT top edge: x = {} + {:.6} * ({} - {}) = {:.3}", tip_x_right, t_to_base_top, base_x_right, tip_x_right, x_on_right_top);
    
    // Check neighboring rows
    for y_test in 76..=78 {
        let t = (y_test as f64 - base_top_y as f64) / (tip_y as f64 - base_top_y as f64);
        let x_left = tip_x_left as f64 + t * (base_x_left as f64 - tip_x_left as f64);
        let x_right = tip_x_right as f64 + t * (base_x_right as f64 - tip_x_right as f64);
        
        let x_left_min_rounded = x_left.min(x_right).round() as i32;
        let x_right_max_rounded = x_left.max(x_right).round() as i32;
        
        // Clamp
        let x_min_clamped = x_left_min_rounded.max(0);
        let x_max_clamped = x_right_max_rounded.min(399);
        
        let width = if x_max_clamped >= x_min_clamped {
            x_max_clamped - x_min_clamped + 1
        } else {
            0
        };
        
        eprintln!("y={}: x_left={:.3}, x_right={:.3}, t={:.6}, rounded=[{},{}], clamped=[{},{}], width={}", 
            y_test, x_left, x_right, t, x_left.round(), x_right.round(), x_min_clamped, x_max_clamped, width);
    }
    
    // Now let's check why the BOTTOM edge calculations
    // For the BOTTOM edge (from tip to base_bottom):
    eprintln!("\n=== CHECKING IF THERE'S AN EDGE DISCONTINUITY ===");
    
    // The triangle has TWO edges: one to base_top and one to base_bottom
    // At y=77, we're using the top edge (closer to base_top at y=50)
    // But rounding might cause issues
    
    // Check what happens with floor vs ceil vs round:
    eprintln!("\nRounding comparison at y=77:");
    let x_left_frac = x_on_left_top;
    let x_right_frac = x_on_right_top;
    
    eprintln!("  Floor:  left={:.0}, right={:.0}", x_left_frac.floor(), x_right_frac.floor());
    eprintln!("  Ceil:   left={:.0}, right={:.0}", x_left_frac.ceil(), x_right_frac.ceil());
    eprintln!("  Round:  left={:.0}, right={:.0}", x_left_frac.round(), x_right_frac.round());
    eprintln!("  Actual: left={:.3}, right={:.3}", x_left_frac, x_right_frac);
    
    // Check BOTH edges (top and bottom) simultaneously:
    eprintln!("\n=== CHECKING TOP AND BOTTOM EDGES SIMULTANEOUSLY ===");
    for y_test in 76..=78 {
        // TOP edge: from tip (1, 100) to base (51, 50)
        let t_top = (y_test as f64 - 50.0) / (100.0 - 50.0);
        let x_left_top = 1.0 + t_top * (51.0 - 1.0);
        
        // BOTTOM edge: from tip (1, 100) to base (51, 150)
        let t_bottom = (y_test as f64 - 100.0) / (150.0 - 100.0);
        let x_left_bottom = 1.0 + t_bottom * (51.0 - 1.0);
        
        eprintln!("y={}: top_edge_x={:.3}, bottom_edge_x={:.3}", y_test, x_left_top, x_left_bottom);
    }
}
