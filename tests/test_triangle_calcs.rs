/// Test specifically looking at the triangle vertices and edge calculations

#[test]
fn test_triangle_vertex_calculation() {
    // Replicate the exact calculation from render_colorbar_standalone
    let width = 400u32;
    let height = 200u32;
    let padding = 50u32;

    // From standalone render
    let cbar_height = (height as i32 - 2 * padding as i32).max(1) as u32; // 100
    let cbar_y = padding; // 50

    let image_center_x = (width as f64) / 2.0; // 200.0
    let mut cbar_width_f64 = width as f64 - 2.0 * padding as f64; // 300.0
    if (cbar_width_f64 as u32).is_multiple_of(2) {
        cbar_width_f64 -= 1.0; // 299.0
    }
    let cbar_width = cbar_width_f64 as u32; // 299
    let cbar_x_f64 = image_center_x - cbar_width_f64 / 2.0; // 200 - 149.5 = 50.5
    let cbar_x = cbar_x_f64.round() as u32; // 51 or 50?

    eprintln!("Colorbar position:");
    eprintln!("  cbar_x_f64 = {:.1}, cbar_x = {}", cbar_x_f64, cbar_x);
    eprintln!(
        "  cbar_width_f64 = {:.1}, cbar_width = {}",
        cbar_width_f64, cbar_width
    );

    // From draw_colorbar_extends
    let cbar_x_px = cbar_x_f64.round() as i32;
    let cbar_w_px = cbar_width_f64.round() as i32;
    let cbar_y_f64 = cbar_y as f64;
    let cbar_h_f64 = cbar_height as f64;

    eprintln!("Colorbar in i32:");
    eprintln!("  cbar_x_px = {}, cbar_w_px = {}", cbar_x_px, cbar_w_px);

    let cbar_left_x = cbar_x_px;
    let cbar_right_x = cbar_x_px + cbar_w_px - 1;

    eprintln!("Colorbar bounds:");
    eprintln!("  left = {}, right = {}", cbar_left_x, cbar_right_x);

    let tip_distance = (cbar_h_f64 * 0.5).round() as i32;
    eprintln!("Tip distance = {}", tip_distance);

    let cbar_y_truncated = cbar_y_f64 as i32; // 50
    let cbar_h_truncated = cbar_h_f64 as i32; // 100

    let gradient_top = cbar_y_truncated;
    let gradient_bottom = gradient_top + cbar_h_truncated - 1;

    eprintln!(
        "Gradient bounds: {} to {} (height {})",
        gradient_top, gradient_bottom, cbar_h_truncated
    );

    let gradient_center_f64 = (gradient_top as f64 + gradient_bottom as f64) / 2.0;
    let tip_y = gradient_center_f64.ceil() as i32;

    eprintln!(
        "Tip Y: gradient_center = {:.1}, tip_y = {}",
        gradient_center_f64, tip_y
    );

    let half_height = (gradient_bottom - gradient_top) as f64 / 2.0;
    let base_top_y = (tip_y as f64 - half_height).round() as i32;
    let base_bottom_y = (tip_y as f64 + half_height).round() as i32;

    eprintln!(
        "Base Y: half_height = {:.1}, base_top = {}, base_bottom = {}",
        half_height, base_top_y, base_bottom_y
    );

    // Triangle vertices
    let tip_x_left = cbar_left_x - tip_distance;
    let base_x_left = cbar_left_x;

    let tip_x_right = cbar_right_x + tip_distance;
    let base_x_right = cbar_right_x;

    eprintln!("\nLEFT triangle:");
    eprintln!("  tip = ({}, {})", tip_x_left, tip_y);
    eprintln!("  base_top = ({}, {})", base_x_left, base_top_y);
    eprintln!("  base_bottom = ({}, {})", base_x_left, base_bottom_y);

    eprintln!("\nRIGHT triangle:");
    eprintln!("  tip = ({}, {})", tip_x_right, tip_y);
    eprintln!("  base_top = ({}, {})", base_x_right, base_top_y);
    eprintln!("  base_bottom = ({}, {})", base_x_right, base_bottom_y);

    // Now test edge calculation at y=76 and y=77
    eprintln!("\n=== EDGE CALCULATIONS ===");

    for y_test in [76, 77, 78] {
        eprintln!("\ny = {}:", y_test);

        // LEFT triangle edges
        // Top edge: from (tip_x_left, tip_y) to (base_x_left, base_top_y)
        let p1_left = (tip_x_left as f64, tip_y as f64);
        let p2_left = (base_x_left as f64, base_top_y as f64);

        let t_left = (y_test as f64 - p1_left.1) / (p2_left.1 - p1_left.1);
        let x_left = p1_left.0 + t_left * (p2_left.0 - p1_left.0);

        // RIGHT triangle edges
        // Top edge: from (tip_x_right, tip_y) to (base_x_right, base_top_y)
        let p1_right = (tip_x_right as f64, tip_y as f64);
        let p2_right = (base_x_right as f64, base_top_y as f64);

        let t_right = (y_test as f64 - p1_right.1) / (p2_right.1 - p1_right.1);
        let x_right = p1_right.0 + t_right * (p2_right.0 - p1_right.0);

        eprintln!("  LEFT edge: t={:.6}, x={:.3}", t_left, x_left);
        eprintln!("  RIGHT edge: t={:.6}, x={:.3}", t_right, x_right);

        let x_min_f = x_left.min(x_right);
        let x_max_f = x_left.max(x_right);

        let x_min_i = x_min_f.round() as i32;
        let x_max_i = x_max_f.round() as i32;

        let x_min_clamped = x_min_i.max(0);
        let x_max_clamped = x_max_i.min(399);

        eprintln!(
            "  Range: [{:.1}..{:.1}], rounded: [{:}..{:}], clamped: [{:}..{:}]",
            x_min_f, x_max_f, x_min_i, x_max_i, x_min_clamped, x_max_clamped
        );

        if x_max_clamped >= x_min_clamped {
            let width = x_max_clamped - x_min_clamped + 1;
            eprintln!("  Width: {}", width);
        }
    }
}
