use map2fig::{cli::Extend, colorbar::render_colorbar_standalone, get_colormap};

#[test]
fn test_visual_output() {
    println!("\n=== GENERATING VISUAL OUTPUT ===");

    // Generate a colorbar for visual inspection
    let img = render_colorbar_standalone(400, 200, get_colormap("viridis"), 1.0, Extend::Both, 50);

    // Save to file for visual inspection
    img.save("/tmp/colorbar_test.png")
        .expect("Failed to save image");
    println!("Saved to /tmp/colorbar_test.png");

    // Analyze the extends in detail
    println!("\nDetailed extend analysis:");

    // Find where the extends are
    for y in 60..141 {
        print!("y={:3}: ", y);

        let mut pixels = Vec::new();
        for x in 0..400 {
            let pix = img.get_pixel(x, y);
            // Check if pixel is not white (part of gradient or extends)
            if pix[0] < 254 || pix[1] < 254 || pix[2] < 254 {
                pixels.push(x);
            }
        }

        if !pixels.is_empty() {
            let mut ranges = Vec::new();
            let mut start = pixels[0];
            let mut end = pixels[0];

            for &v in pixels.iter().skip(1) {
                if v == end + 1 {
                    end = v;
                } else {
                    ranges.push((start, end));
                    start = v;
                    end = v;
                }
            }
            ranges.push((start, end));

            for (s, e) in ranges {
                let width = e - s + 1;
                print!("[{:3}..{:3}]={:3} ", s, e, width);
            }
        }
        println!();
    }
}

#[test]
fn test_tick_bottom_alignment() {
    println!("\n=== TICK BOTTOM ALIGNMENT TEST ===");

    // Render a colorbar to check tick positioning
    let img = render_colorbar_standalone(400, 200, get_colormap("viridis"), 1.0, Extend::Both, 50);

    // Colorbar parameters: padding=50, width=400, height=200
    // Colorbar Y range: 50..150 (100 pixels high)
    // Colorbar X range: 51..349 (299 pixels wide)

    let cbar_y_start = 50u32;
    let cbar_y_end = 149u32; // 0-indexed, so height-1

    println!("Colorbar Y range: {}..{}", cbar_y_start, cbar_y_end);

    // Find the bottommost colored pixel in the colorbar region
    let mut max_colored_y = 0u32;
    let mut colored_pixel_count = 0;

    // Scan the colorbar X range to find the bottom
    // Image height is 200, so valid Y indices are 0..199
    for y in cbar_y_start..200 {
        for x in 40..360 {
            // Scan around the colorbar region
            let pix = img.get_pixel(x, y);
            // Check if pixel is not white (part of gradient, ticks, or extends)
            if pix[0] < 254 || pix[1] < 254 || pix[2] < 254 {
                colored_pixel_count += 1;
                if y > max_colored_y {
                    max_colored_y = y;
                }
            }
        }
    }

    println!("Found {} colored pixels", colored_pixel_count);
    println!("Bottommost colored pixel at Y={}", max_colored_y);
    println!("Colorbar bottom should be at Y={}", cbar_y_end);

    // The bottommost colored pixel should not exceed the colorbar bounds
    // (allowing for small floating point errors in rasterization, maybe 1-2 pixels)
    let overhang = max_colored_y.saturating_sub(cbar_y_end);

    println!("Tick overhang: {} pixels", overhang);

    if overhang > 0 {
        println!("❌ FAIL: Ticks extend {} pixel(s) below colorbar", overhang);

        // Debug: show which Y rows have pixels outside the colorbar
        let max_y_to_check = (cbar_y_end + 1).min(199);
        for y in (cbar_y_end + 1)..=max_y_to_check {
            let mut pixels_in_row = 0;
            for x in 40..360 {
                let pix = img.get_pixel(x, y);
                if pix[0] < 254 || pix[1] < 254 || pix[2] < 254 {
                    pixels_in_row += 1;
                }
            }
            if pixels_in_row > 0 {
                println!("  Y={}: {} pixels", y, pixels_in_row);
            }
        }
    } else {
        println!("✓ PASS: Ticks are properly aligned with colorbar");
    }

    assert_eq!(
        overhang, 0,
        "Ticks should not extend below colorbar (overhang: {} pixels)",
        overhang
    );
}
