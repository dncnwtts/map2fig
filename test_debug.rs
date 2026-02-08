// Simple test to debug the vertical base rasterization
fn main() {
    let tip_x: f64 = 0.0;
    let tip_y: f64 = 100.0;
    let base_x: f64 = 9.0;
    let base_y_min: f64 = 50.0;
    let base_y_max: f64 = 150.0;
    
    let y: f64 = 50.0;  // At the base edge
    
    let distance_from_tip = (y - tip_y).abs();
    let distance_to_base = (base_y_max - tip_y).abs().max((base_y_min - tip_y).abs());
    
    println!("At y=50:");
    println!("  distance_from_tip = {}", distance_from_tip);
    println!("  distance_to_base = {}", distance_to_base);
    
    let t = if distance_to_base > 0.0 {
        (distance_from_tip / distance_to_base).min(1.0)
    } else {
        0.0
    };
    
    println!("  t = {}", t);
    
    let x_interp = tip_x + (base_x - tip_x) * t;
    println!("  x_interp = {} (from {} to {})", x_interp, tip_x, base_x);
    
    // At the base, we should reach x=9
    // t=0 at tip (100), t=1 at base (50 or 150)
    // distance_from_tip at y=50 is |50-100| = 50
    // distance_to_base is max(|50-100|, |150-100|) = max(50, 50) = 50
    // So t = 50/50 = 1.0
    // x_interp = 0 + (9 - 0) * 1.0 = 9.0
    // That's correct!
}
