use map2fig::{cli::Extend, colorbar::render_colorbar_standalone, get_colormap};

#[test]
fn test_colorbar_geometry() {
    println!("\n=== COLORBAR GEOMETRY ===");

    // With width=400, height=200, padding=50:
    // - Image center: x=200
    // - cbar_height = 200 - 2*50 = 100
    // - cbar_width_f64 = 400 - 2*50 = 300
    // - cbar_width becomes 299 (made odd)
    // - cbar_x_f64 = 200 - 299/2 = 200 - 149.5 = 50.5
    // - cbar_x rounds to 50 or 51

    let img = render_colorbar_standalone(400, 200, get_colormap("viridis"), 1.0, Extend::Both, 50);

    // Find the gradient bounds
    let mut grad_left = None;
    let mut grad_right = None;

    for x in 0..400 {
        let pix = img.get_pixel(x, 100);
        // Check if this pixel is part of the gradient (not pure white)
        if pix[0] < 254 && grad_left.is_none() {
            grad_left = Some(x);
        }
        if pix[0] < 254 {
            grad_right = Some(x);
        }
    }

    if let (Some(l), Some(r)) = (grad_left, grad_right) {
        println!("Gradient bounds: x={} to x={} (width={})", l, r, r - l + 1);
        println!("Gradient center: {}", (l as f64 + r as f64) / 2.0);
    }

    // The colorbar width is 299, so:
    // - cbar_x should be at 50 (if 50.5 rounds down) or 51 (if it rounds)
    // - cbar_right should be at cbar_x + 299 - 1 = 348 or 349

    // For extends with tip_distance = (100 * 0.5).round() = 50:
    // - LEFT tip: x = grad_left - 50
    // - RIGHT tip: x = grad_right + 50

    // Check for extends
    println!("\nLooking for extend triangles at y=100:");
    let mut left_extent = (399, 0); // (min_x, max_x)
    let mut right_extent = (399, 0);

    for x in 0..400 {
        let pix = img.get_pixel(x, 100);
        // Extend pixels might be slightly different from gradient
        let is_dark = pix[0] < 254;

        if is_dark && let (Some(gl), Some(gr)) = (grad_left, grad_right) {
            if (x as i32) < gl as i32 {
                left_extent.0 = left_extent.0.min(x);
                left_extent.1 = left_extent.1.max(x);
            } else if (x as i32) > gr as i32 {
                right_extent.0 = right_extent.0.min(x);
                right_extent.1 = right_extent.1.max(x);
            }
        }
    }

    if left_extent.1 > 0 {
        println!(
            "LEFT extend at y=100: x={} to x={} (width={})",
            left_extent.0,
            left_extent.1,
            left_extent.1 - left_extent.0 + 1
        );
    }

    if right_extent.1 > 0 {
        println!(
            "RIGHT extend at y=100: x={} to x={} (width={})",
            right_extent.0,
            right_extent.1,
            right_extent.1 - right_extent.0 + 1
        );
    }
}
