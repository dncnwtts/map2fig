use map2fig::{colorbar::render_colorbar_standalone, get_colormap, cli::Extend};

#[test]
fn test_visual_output() {
    println!("\n=== GENERATING VISUAL OUTPUT ===");
    
    // Generate a colorbar for visual inspection
    let img = render_colorbar_standalone(400, 200, get_colormap("viridis"), 1.0, Extend::Both, 50);
    
    // Save to file for visual inspection
    img.save("/tmp/colorbar_test.png").expect("Failed to save image");
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
            
            for i in 1..pixels.len() {
                if pixels[i] == end + 1 {
                    end = pixels[i];
                } else {
                    ranges.push((start, end));
                    start = pixels[i];
                    end = pixels[i];
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
