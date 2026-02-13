use map2fig::cli::Extend;
use map2fig::colorbar::render_colorbar_standalone;
use map2fig::get_colormap;

#[test]
fn test_left_right_symmetry_detail() {
    println!("\n=== LEFT-RIGHT SYMMETRY DETAIL ===");

    // The standalone function creates an odd-width colorbar (299 pixels in a 400-pixel image)
    // This means left and right aren't perfectly symmetric about image center
    // But the SHAPES of the triangles should still be identical

    let img = render_colorbar_standalone(400, 200, get_colormap("viridis"), 1.0, Extend::Both, 50);

    // Find the actual colorbar bounds by looking for the gradient
    let mut cbar_left: i32 = 0;
    let mut cbar_right: i32 = 399;
    for x in 0..400 {
        let pix = img.get_pixel(x, 100);
        // Gradient pixels are not pure white/black
        if pix[0] < 250 {
            cbar_left = x as i32;
            break;
        }
    }
    for x in (0..400).rev() {
        let pix = img.get_pixel(x, 100);
        if pix[0] < 250 {
            cbar_right = x as i32;
            break;
        }
    }

    println!("Detected colorbar bounds: {} to {}", cbar_left, cbar_right);
    let cbar_center = (cbar_left as f64 + cbar_right as f64) / 2.0;
    println!("Colorbar center: {}", cbar_center);

    // For left-right symmetry of the TRIANGLE SHAPES, we need to check:
    // - At each distance d from the left base, the width should match the width at distance d from the right base

    println!("\nChecking triangle shape symmetry:");
    println!("(comparing distance from base rather than absolute position)");

    let mut asymmetries = Vec::new();

    // Scan from base outward
    for y in 63..139 {
        // Find left extent pixels
        let mut left_pixels = Vec::new();
        for x in 0..400 {
            let pix = img.get_pixel(x, y);
            if pix[0] < 250 || pix[1] < 250 || pix[2] < 250 {
                left_pixels.push(x as i32);
            }
        }

        // Find the portion that's from the left extend (pixels less than colorbar left)
        let left_extent_start = left_pixels
            .iter()
            .filter(|&&x| x < cbar_left)
            .min()
            .copied();
        let left_extent_end = cbar_left.min(
            left_pixels
                .iter()
                .filter(|&&x| x < cbar_left)
                .max()
                .copied()
                .unwrap_or(cbar_left - 1),
        );

        let left_width = if let Some(start) = left_extent_start {
            left_extent_end - start + 1
        } else {
            0
        };

        // Find the portion that's from the right extend (pixels greater than colorbar right)
        let right_extent_start = (cbar_right + 1).max(
            left_pixels
                .iter()
                .filter(|&&x| x > cbar_right)
                .min()
                .copied()
                .unwrap_or(cbar_right + 1),
        );
        let right_extent_end = left_pixels
            .iter()
            .filter(|&&x| x > cbar_right)
            .max()
            .copied()
            .unwrap_or(cbar_right);

        let right_width = if right_extent_start <= cbar_right {
            0
        } else if right_extent_start <= right_extent_end {
            right_extent_end - right_extent_start + 1
        } else {
            0
        };

        if left_width != right_width {
            asymmetries.push((y, left_width, right_width));
        }
    }

    if asymmetries.is_empty() {
        println!("✓ Triangle shapes are perfectly symmetric!");
    } else {
        println!("Found {} asymmetries:", asymmetries.len());
        for (y, lw, rw) in asymmetries.iter().take(10) {
            println!(
                "  y={}: LEFT extent width={}, RIGHT extent width={}",
                y, lw, rw
            );
        }
    }
}
