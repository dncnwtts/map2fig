#[test]
fn test_triangle_vertices() {
    // Simulate extend triangle creation

    // Colorbar parameters
    let cbar_y: f64 = 50.0;
    let cbar_h: f64 = 100.0;
    let cbar_y_truncated = (cbar_y as u32) as f64;
    let cbar_h_truncated = (cbar_h as u32) as i32;

    let gradient_top = cbar_y_truncated as i32;
    let gradient_bottom = gradient_top + cbar_h_truncated - 1;
    let gradient_center_f64 = (gradient_top as f64 + gradient_bottom as f64) / 2.0;
    let tip_y = gradient_center_f64.ceil() as i32;

    let cbar_left_x = 100i32;
    let tip_distance = (cbar_h * 0.5).round() as i32;

    // LEFT triangle vertices
    let tip_x = cbar_left_x - tip_distance;
    let base_x = cbar_left_x;
    let base_top_y = gradient_top;
    let base_bottom_y = gradient_bottom;

    let v0 = (tip_x, tip_y);
    let v1 = (base_x, base_top_y);
    let v2 = (base_x, base_bottom_y);

    println!("Triangle vertices:");
    println!("  v0 (tip): {:?}", v0);
    println!("  v1 (base-top): {:?}", v1);
    println!("  v2 (base-bottom): {:?}", v2);

    // Check distances
    let dist2 = |dx, dy| dx * dx + dy * dy;
    let v01_dist = dist2(v1.0 - v0.0, v1.1 - v0.1);
    let v12_dist = dist2(v2.0 - v1.0, v2.1 - v1.1);
    let v20_dist = dist2(v0.0 - v2.0, v0.1 - v2.1);

    println!("\nEdge distances (squared):");
    println!("  v0-v1: {}", v01_dist);
    println!("  v1-v2: {}", v12_dist);
    println!("  v2-v0: {}", v20_dist);

    // Determine tip
    if v01_dist == v20_dist && v01_dist != v12_dist {
        println!("\nTip detected at v0 (correct)");
    } else if v12_dist == v01_dist && v12_dist != v20_dist {
        println!("\nTip detected at v1");
    } else if v20_dist == v12_dist && v20_dist != v01_dist {
        println!("\nTip detected at v2");
    } else {
        println!("\nNo isosceles tip detected!");
        println!("This triangle won't use the fast isosceles path!");
    }
}
