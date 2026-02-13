use image::Rgba;
use map2fig::colorbar::fill_triangle;

#[test]
fn test_base_vertices_fill() {
    println!("\n=== BASE VERTICES FILL TEST ===");

    // Simple vertical base triangle at y=50-150
    let mut img = image::RgbaImage::from_pixel(10, 200, Rgba([255, 255, 255, 255]));

    // Triangle with tip at (0, 100) and base at (9, 50) to (9, 150)
    let vertices = [
        (0i32, 100i32), // tip
        (9i32, 50i32),  // base left
        (9i32, 150i32), // base right
    ];
    fill_triangle(vertices, Rgba([0, 0, 0, 255]), &mut img);

    println!("Checking all Y levels from 40 to 160:");
    for y in 40..161 {
        let mut pixels = String::new();
        for x in 0..10 {
            let pix = img.get_pixel(x, y);
            if pix[0] == 0 {
                pixels.push('X');
            } else {
                pixels.push('.');
            }
        }
        if y == 50 || y == 100 || y == 150 {
            println!("y={:3}: {}", y, pixels);
        }
    }

    println!("\nExpected:");
    println!("y= 50: XXXXXXXXXX (tip at x=0, base at x=9)");
    println!("y=100: X......... (only tip)");
    println!("y=150: XXXXXXXXXX (tip at x=0, base at x=9)");
}
