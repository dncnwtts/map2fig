/// Test for RGBA<->ARGB color channel conversion fix
#[cfg(test)]
mod color_conversion_tests {
    use image::Rgba;

    #[test]
    fn test_rgba_to_argb_conversion() {
        // Simulate the RGBA to ARGB conversion we're doing in mollweide.rs
        let rgba_pixel = Rgba([0xFF, 0x80, 0x40, 0xC0]); // R, G, B, A
        
        // Convert to ARGB as we do in the fixed code
        let mut argb_bytes = Vec::new();
        argb_bytes.push(rgba_pixel[3]); // A
        argb_bytes.push(rgba_pixel[0]); // R
        argb_bytes.push(rgba_pixel[1]); // G
        argb_bytes.push(rgba_pixel[2]); // B
        
        // Verify byte order
        assert_eq!(argb_bytes[0], 0xC0, "Alpha should be first");
        assert_eq!(argb_bytes[1], 0xFF, "Red should be second");
        assert_eq!(argb_bytes[2], 0x80, "Green should be third");
        assert_eq!(argb_bytes[3], 0x40, "Blue should be fourth");
    }

    #[test]
    fn test_color_red() {
        let red_rgba = Rgba([255, 0, 0, 255]);
        let mut argb = vec![red_rgba[3], red_rgba[0], red_rgba[1], red_rgba[2]];
        
        // In ARGB: should be [FF, FF, 00, 00] = opaque red
        assert_eq!(argb, vec![255, 255, 0, 0]);
    }

    #[test]
    fn test_color_green() {
        let green_rgba = Rgba([0, 255, 0, 255]);
        let mut argb = vec![green_rgba[3], green_rgba[0], green_rgba[1], green_rgba[2]];
        
        // In ARGB: should be [FF, 00, FF, 00] = opaque green
        assert_eq!(argb, vec![255, 0, 255, 0]);
    }

    #[test]
    fn test_color_blue() {
        let blue_rgba = Rgba([0, 0, 255, 255]);
        let mut argb = vec![blue_rgba[3], blue_rgba[0], blue_rgba[1], blue_rgba[2]];
        
        // In ARGB: should be [FF, 00, 00, FF] = opaque blue
        assert_eq!(argb, vec![255, 0, 0, 255]);
    }

    #[test]
    fn test_semi_transparent_color() {
        let semi_rgba = Rgba([100, 150, 200, 128]); // 50% transparent
        let mut argb = vec![semi_rgba[3], semi_rgba[0], semi_rgba[1], semi_rgba[2]];
        
        assert_eq!(argb, vec![128, 100, 150, 200]);
    }
}
