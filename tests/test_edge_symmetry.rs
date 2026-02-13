use map2fig::{cli::Extend, colorbar::render_colorbar_standalone, get_colormap};

#[test]
fn test_edge_asymmetry_analysis() {
    println!("\n=== EDGE ASYMMETRY ANALYSIS ===");

    let cmap = get_colormap("viridis");
    let img = render_colorbar_standalone(400, 200, cmap, 1.0, Extend::Both, 50);

    // Focus on the asymmetric region
    let center_y = 100i32;

    for distance in 24..=30 {
        let y_top = center_y - distance;
        let y_bottom = center_y + distance;

        // Check pixels across the extend triangle width
        println!("\ndist={} y_top={} y_bottom={}", distance, y_top, y_bottom);

        // LEFT extend - check full width
        let mut top_pixels = Vec::new();
        let mut bot_pixels = Vec::new();

        for x in 0..30 {
            let pix_top = img.get_pixel(x, y_top as u32);
            let pix_bot = img.get_pixel(x, y_bottom as u32);

            let colored_top = pix_top[0] < 250 || pix_top[1] < 250 || pix_top[2] < 250;
            let colored_bot = pix_bot[0] < 250 || pix_bot[1] < 250 || pix_bot[2] < 250;

            if colored_top {
                top_pixels.push(x);
            }
            if colored_bot {
                bot_pixels.push(x);
            }
        }

        println!("  LEFT: y_top={:?}, y_bot={:?}", top_pixels, bot_pixels);

        if !top_pixels.is_empty() && !bot_pixels.is_empty() {
            let top_width = top_pixels[top_pixels.len() - 1] - top_pixels[0] + 1;
            let bot_width = bot_pixels[bot_pixels.len() - 1] - bot_pixels[0] + 1;
            let top_left = top_pixels[0];
            let bot_left = bot_pixels[0];

            println!(
                "    Width: top={}, bot={} (diff={}), Left edge: top={}, bot={} (diff={})",
                top_width,
                bot_width,
                (top_width as i32 - bot_width as i32),
                top_left,
                bot_left,
                (top_left as i32 - bot_left as i32)
            );
        }
    }
}
