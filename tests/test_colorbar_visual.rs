use map2fig::cli::Extend;
use map2fig::colorbar::render_colorbar_standalone;
use map2fig::colormap;

#[test]
fn test_colorbar_visual_output() {
    // Render a colorbar and save it for visual inspection
    let width = 400u32;
    let height = 200u32;
    let padding = 50u32;

    let img = render_colorbar_standalone(
        width,
        height,
        &colormap::VIRIDIS,
        1.0,
        Extend::Both,
        padding,
    );

    // Save for inspection
    img.save("/tmp/colorbar_test.png")
        .expect("Failed to save colorbar");
    eprintln!("Saved colorbar to /tmp/colorbar_test.png");

    // Also analyze the pixels to find the extends
    eprintln!("\n=== PIXEL ANALYSIS ===");

    // Scan each row to find colored pixels (extends)
    for y in 0..height {
        let mut first_x: Option<u32> = None;
        let mut last_x: Option<u32> = None;

        for x in 0..width {
            let pixel = img.get_pixel(x, y);
            // Check if pixel is colored (not white, not part of gradient area)
            if pixel[3] > 200 {
                // Alpha > 200 means colored
                // Check if it's not white
                let is_white = pixel[0] > 240 && pixel[1] > 240 && pixel[2] > 240;
                if !is_white {
                    if first_x.is_none() {
                        first_x = Some(x);
                    }
                    last_x = Some(x);
                }
            }
        }

        if let (Some(first), Some(last)) = (first_x, last_x) {
            let width_px = last - first + 1;
            eprintln!("y={:3}: [{}..{}]={} pixels", y, first, last, width_px);
        }
    }
}
