#[test]
fn test_which_path() {
    println!("\n=== WHICH RASTERIZATION PATH ===");
    
    // LEFT triangle
    let v0 = (10i32, 100i32);
    let v1 = (0i32, 50i32);
    let v2 = (0i32, 150i32);
    
    // Compute distances
    let dist2 = |dx: i32, dy: i32| dx * dx + dy * dy;
    let v01_dist = dist2(v1.0 - v0.0, v1.1 - v0.1);
    let v12_dist = dist2(v2.0 - v1.0, v2.1 - v1.1);
    let v20_dist = dist2(v0.0 - v2.0, v0.1 - v2.1);
    
    println!("LEFT triangle vertices: {:?}, {:?}, {:?}", v0, v1, v2);
    println!("  v01_dist = {} (should be 100 + 2500 = 2600)", v01_dist);
    println!("  v12_dist = {} (should be 0 + 10000 = 10000)", v12_dist);
    println!("  v20_dist = {} (should be 100 + 2500 = 2600)", v20_dist);
    
    let is_isosceles_left = v01_dist == v20_dist && v01_dist != v12_dist;
    println!("  Is isosceles? {}", is_isosceles_left);
    println!("  Tip should be at v0: {}", v01_dist == v20_dist);
    
    // RIGHT triangle
    let v0 = (390i32, 100i32);
    let v1 = (400i32, 50i32);
    let v2 = (400i32, 150i32);
    
    let v01_dist = dist2(v1.0 - v0.0, v1.1 - v0.1);
    let v12_dist = dist2(v2.0 - v1.0, v2.1 - v1.1);
    let v20_dist = dist2(v0.0 - v2.0, v0.1 - v2.1);
    
    println!("\nRIGHT triangle vertices: {:?}, {:?}, {:?}", v0, v1, v2);
    println!("  v01_dist = {} (should be 100 + 2500 = 2600)", v01_dist);
    println!("  v12_dist = {} (should be 0 + 10000 = 10000)", v12_dist);
    println!("  v20_dist = {} (should be 100 + 2500 = 2600)", v20_dist);
    
    let is_isosceles_right = v01_dist == v20_dist && v01_dist != v12_dist;
    println!("  Is isosceles? {}", is_isosceles_right);
    println!("  Tip should be at v0: {}", v01_dist == v20_dist);
}
