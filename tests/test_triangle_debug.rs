use image::Rgba;
use map2fig::colorbar::fill_triangle;

#[test]
fn test_left_right_triangle_identity() {
    println!("\n=== TRIANGLE IDENTITY TEST ===");

    let img_left = {
        let mut img = image::RgbaImage::from_pixel(400, 200, Rgba([255, 255, 255, 255]));

        // LEFT triangle: tip at x=10, base at x=0
        let vertices = [
            (10i32, 100i32), // tip
            (0i32, 50i32),   // base top
            (0i32, 150i32),  // base bottom
        ];
        fill_triangle(vertices, Rgba([0, 0, 0, 255]), &mut img);
        img
    };

    let img_right = {
        let mut img = image::RgbaImage::from_pixel(400, 200, Rgba([255, 255, 255, 255]));

        // RIGHT triangle: tip at x=390, base at x=400
        let vertices = [
            (390i32, 100i32), // tip
            (400i32, 50i32),  // base top
            (400i32, 150i32), // base bottom
        ];
        fill_triangle(vertices, Rgba([0, 0, 0, 255]), &mut img);
        img
    };

    println!("\nComparing triangles at each Y level:");

    for y in 50..=150 {
        // Count black pixels
        let mut left_count = 0;
        for x in 0..20 {
            if img_left.get_pixel(x, y)[0] == 0 {
                left_count += 1;
            }
        }

        let mut right_count = 0;
        for x in 380..400 {
            if img_right.get_pixel(x, y)[0] == 0 {
                right_count += 1;
            }
        }

        if left_count != right_count {
            println!(
                "  y={}: left={} pixels, right={} pixels",
                y, left_count, right_count
            );
        }
    }
}
