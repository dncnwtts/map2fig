use map2fig::colorbar::render_colorbar_standalone;
use map2fig::get_colormap;
use map2fig::cli::Extend;

#[test]
fn test_check_actual_base_symmetry() {
    println!("\n=== CHECK ACTUAL BASE SYMMETRY ===");
    
    // Create a test image with extend triangles
    let img = render_colorbar_standalone(400, 200, get_colormap("viridis"), 1.0, Extend::Both, 50);
    
    // The colorbar should be from y=50 to y=149 (100 pixels)
    // Tip should be at center: y=99.5, ceiled to 100
    // For isosceles, base should be symmetric around tip
    
    // Find the extent of the left triangle by checking which pixels are colored
    println!("\nLEFT extend triangle:");
    
    for y in 50..150 {
        let mut pixels = Vec::new();
        for x in 0..40 {
            let pix = img.get_pixel(x as u32, y as u32);
            if pix[0] < 250 || pix[1] < 250 || pix[2] < 250 {
                pixels.push(x);
            }
        }
        
        if !pixels.is_empty() {
            let width = pixels[pixels.len()-1] - pixels[0] + 1;
            if y <= 55 || y >= 145 || width <= 5 {
                println!("  y={}: pixels {}..{} (width={})", 
                    y, pixels[0], pixels[pixels.len()-1], width);
            }
        }
    }
    
    // Check for plateaus - where width doesn't decrease
    println!("\nPlateau detection:");
    let mut last_width = 999;
    let mut plateau_count = 0;
    
    for y in 50..150 {
        let mut pixels = Vec::new();
        for x in 0..40 {
            let pix = img.get_pixel(x as u32, y as u32);
            if pix[0] < 250 || pix[1] < 250 || pix[2] < 250 {
                pixels.push(x);
            }
        }
        
        if !pixels.is_empty() {
            let width = pixels[pixels.len()-1] - pixels[0] + 1;
            if width == last_width && width > 1 {
                plateau_count += 1;
                if plateau_count <= 3 {
                    println!("  Plateau at y={}: width={} (same as y={})", y, width, y-1);
                }
            }
            last_width = width;
        } else {
            last_width = 999;
        }
    }
    
    if plateau_count == 0 {
        println!("  ✓ No plateaus detected!");
    } else {
        println!("  ❌ {} plateau levels detected", plateau_count);
    }
}
