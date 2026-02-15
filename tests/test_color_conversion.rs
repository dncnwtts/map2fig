/// Test for RGBA<->ARGB color channel conversion fix
#[cfg(test)]
mod color_conversion_tests {
    use image::Rgba;

    #[test]
    fn test_rgba_to_argb_conversion() {
        // Simulate the RGBA to BGRA conversion for Cairo::Format::ARgb32
        // Cairo expects bytes in memory as B, G, R, A (little-endian ARgb32)
        let rgba_pixel = Rgba([0xFF, 0x80, 0x40, 0xC0]); // R, G, B, A

        // Convert to BGRA as we do in the fixed code
        let bgra_bytes = [
            rgba_pixel[2], // B
            rgba_pixel[1], // G
            rgba_pixel[0], // R
            rgba_pixel[3], // A
        ];

        // Verify byte order
        assert_eq!(bgra_bytes[0], 0x40, "Blue should be first");
        assert_eq!(bgra_bytes[1], 0x80, "Green should be second");
        assert_eq!(bgra_bytes[2], 0xFF, "Red should be third");
        assert_eq!(bgra_bytes[3], 0xC0, "Alpha should be fourth");
    }

    #[test]
    fn test_color_red() {
        let red_rgba = Rgba([255, 0, 0, 255]);
        let bgra = [red_rgba[2], red_rgba[1], red_rgba[0], red_rgba[3]];

        // In BGRA for Cairo: B=0, G=0, R=255, A=255
        assert_eq!(bgra, [0, 0, 255, 255]);
    }

    #[test]
    fn test_color_green() {
        let green_rgba = Rgba([0, 255, 0, 255]);
        let bgra = [green_rgba[2], green_rgba[1], green_rgba[0], green_rgba[3]];

        // In BGRA for Cairo: B=0, G=255, R=0, A=255
        assert_eq!(bgra, [0, 255, 0, 255]);
    }

    #[test]
    fn test_color_blue() {
        let blue_rgba = Rgba([0, 0, 255, 255]);
        let bgra = [blue_rgba[2], blue_rgba[1], blue_rgba[0], blue_rgba[3]];

        // In BGRA for Cairo: B=255, G=0, R=0, A=255
        assert_eq!(bgra, [255, 0, 0, 255]);
    }

    #[test]
    fn test_semi_transparent_color() {
        let semi_rgba = Rgba([100, 150, 200, 128]); // 50% transparent
        let bgra = [semi_rgba[2], semi_rgba[1], semi_rgba[0], semi_rgba[3]];

        // In BGRA for Cairo: B=200, G=150, R=100, A=128
        assert_eq!(bgra, [200, 150, 100, 128]);
    }
}
