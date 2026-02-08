#[test]
fn test_edge_calculation() {
    // Simulate the edge calculation at different Y levels
    
    // LEFT triangle: tip (-49, 100), base1 (1, 50), base2 (1, 150)
    let tip = (-49.0, 100.0);
    let base1 = (1.0, 50.0);
    let base2 = (1.0, 150.0);
    
    println!("\n=== EDGE CALCULATION ===");
    println!("LEFT triangle: tip {:?}, base1 {:?}, base2 {:?}", tip, base1, base2);
    
    for y in [75, 76, 77, 78] {
        let y_f = y as f64;
        
        // Edge 1: tip to base1
        let t1 = (y_f - tip.1) / (base1.1 - tip.1);
        let x1 = tip.0 + t1 * (base1.0 - tip.0);
        let x1_rounded = x1.round();
        let x1_floored = x1.floor();
        let x1_ceiled = x1.ceil();
        
        // Edge 2: tip to base2
        let t2 = (y_f - tip.1) / (base2.1 - tip.1);
        let x2 = tip.0 + t2 * (base2.0 - tip.0);
        let x2_rounded = x2.round();
        let x2_floored = x2.floor();
        let x2_ceiled = x2.ceil();
        
        let width_round = (x2_rounded.max(x1_rounded) as i32) - (x2_rounded.min(x1_rounded) as i32) + 1;
        let width_floor_ceil = ((x2_ceiled.max(x1_ceiled)) as i32) - ((x2_floored.min(x1_floored)) as i32) + 1;
        
        println!("y={}: x1={:.2} (round={:.0}, floor={:.0}, ceil={:.0}), x2={:.2} (round={:.0}, floor={:.0}, ceil={:.0})",
            y, x1, x1_rounded, x1_floored, x1_ceiled, x2, x2_rounded, x2_floored, x2_ceiled);
        println!("     width with round: {}, width with floor/ceil: {}", width_round, width_floor_ceil);
    }
    
    // Also check the actual colorbar's edges
    println!("\n=== CHECKING ACTUAL COLORBAR EDGES ===");
    
    // The gradient goes from x=1 to x=399, and we're using clamping
    // So the effective base vertices might be different
    
    let tip = (-49i32, 100i32);
    let grad_left = 1i32;
    let grad_right = 399i32;
    
    // For the LEFT extend (tip at -49, base at grad_left=1)
    let base1 = (grad_left, 50i32);
    let base2 = (grad_left, 150i32);
    
    // For the RIGHT extend (tip at 449, base at grad_right=399)
    let tip_r = (449i32, 100i32);
    let base1_r = (grad_right, 50i32);
    let base2_r = (grad_right, 150i32);
    
    println!("LEFT: tip {:?}, bases {:?}/{:?}", tip, base1, base2);
    println!("RIGHT: tip {:?}, bases {:?}/{:?}", tip_r, base1_r, base2_r);
    
    for y in [75, 76, 77, 78] {
        let y_f = y as f64;
        
        // LEFT edges
        let t1 = (y_f - tip.1 as f64) / (base1.1 as f64 - tip.1 as f64);
        let x1 = tip.0 as f64 + t1 * (base1.0 as f64 - tip.0 as f64);
        let t2 = (y_f - tip.1 as f64) / (base2.1 as f64 - tip.1 as f64);
        let x2 = tip.0 as f64 + t2 * (base2.0 as f64 - tip.0 as f64);
        
        let x_min = x1.min(x2).max(0.0);  // Clamp to 0
        let x_max = x2.max(x1).max(0.0);
        
        let left_width = (x_max.round() as i32) - (x_min.round() as i32) + 1;
        
        println!("y={}: x1={:.2}, x2={:.2}, width={}", y, x1, x2, left_width);
    }
}
