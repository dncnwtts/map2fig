use map2fig::colorbar::fill_triangle;
use image::Rgba;

#[test]
fn test_symmetric_triangles() {
    println!("\n=== EXACT COLORBAR GEOMETRY TEST ===");
    
    let mut img = image::RgbaImage::from_pixel(400, 200, Rgba([255, 255, 255, 255]));
    
    // Exact colorbar geometry from render_colorbar_standalone
    // colorbar spans x=0 to x=399 (width 400)
    // tip_distance = (100 * 0.5).round() = 50
    // LEFT extends: tip_x = 0-50=-50, base_x = 0
    // RIGHT extends: tip_x = 399+50=449, base_x = 399
    
    let tip_distance = 50i32;
    let cbar_left_x = 0i32;
    let cbar_right_x = 399i32;
    let _cbar_h = 100i32;
    let tip_y = 100i32;
    let base_top_y = 50i32;
    let base_bottom_y = 150i32;
    
    // LEFT triangle
    let vertices_left = [
        (cbar_left_x - tip_distance, tip_y),
        (cbar_left_x, base_top_y),
        (cbar_left_x, base_bottom_y),
    ];
    println!("LEFT triangle: {:?}", vertices_left);
    fill_triangle(vertices_left, Rgba([0, 0, 0, 255]), &mut img);
    
    // RIGHT triangle
    let vertices_right = [
        (cbar_right_x + tip_distance, tip_y),
        (cbar_right_x, base_top_y),
        (cbar_right_x, base_bottom_y),
    ];
    println!("RIGHT triangle: {:?}", vertices_right);
    fill_triangle(vertices_right, Rgba([100, 100, 100, 255]), &mut img);
    
    println!("\nPixel analysis:");
    println!("At y=100 (tip):");
    let mut left_range = (None, None);
    for x in 0..400 {
        if img.get_pixel(x, 100)[0] == 0 {
            if left_range.0.is_none() { left_range.0 = Some(x); }
            left_range.1 = Some(x);
        }
    }
    
    let mut right_range = (None, None);
    for x in 0..400 {
        if img.get_pixel(x, 100)[0] == 100 {
            if right_range.0.is_none() { right_range.0 = Some(x); }
            right_range.1 = Some(x);
        }
    }
    
    if let (Some(l0), Some(l1)) = left_range {
        println!("  LEFT: x={:3} to x={:3} (width={})", l0, l1, l1 - l0 + 1);
    }
    if let (Some(r0), Some(r1)) = right_range {
        println!("  RIGHT: x={:3} to x={:3} (width={})", r0, r1, r1 - r0 + 1);
    }
    
    println!("\nAt y=50 (base):");
    let mut left_range = (None, None);
    for x in 0..400 {
        if img.get_pixel(x, 50)[0] == 0 {
            if left_range.0.is_none() { left_range.0 = Some(x); }
            left_range.1 = Some(x);
        }
    }
    
    let mut right_range = (None, None);
    for x in 0..400 {
        if img.get_pixel(x, 50)[0] == 100 {
            if right_range.0.is_none() { right_range.0 = Some(x); }
            right_range.1 = Some(x);
        }
    }
    
    if let (Some(l0), Some(l1)) = left_range {
        println!("  LEFT: x={:3} to x={:3} (width={})", l0, l1, l1 - l0 + 1);
    }
    if let (Some(r0), Some(r1)) = right_range {
        println!("  RIGHT: x={:3} to x={:3} (width={})", r0, r1, r1 - r0 + 1);
    }
    
    println!("\nChecking all Y levels for asymmetry:");
    let mut asymmetries = 0;
    for y in 50..=150 {
        let mut left_width = 0;
        let mut left_range = (399i32, 0i32);
        for x in 0..400 {
            if img.get_pixel(x, y)[0] == 0 {
                left_width += 1;
                left_range.0 = left_range.0.min(x as i32);
                left_range.1 = left_range.1.max(x as i32);
            }
        }
        
        let mut right_width = 0;
        let mut right_range = (399i32, 0i32);
        for x in 0..400 {
            if img.get_pixel(x, y)[0] == 100 {
                right_width += 1;
                right_range.0 = right_range.0.min(x as i32);
                right_range.1 = right_range.1.max(x as i32);
            }
        }
        
        if left_width != right_width {
            asymmetries += 1;
        }
    }
    
    if asymmetries == 0 {
        println!("✓ Perfect left-right symmetry!");
    } else {
        println!("❌ Found {} asymmetries", asymmetries);
    }
}
