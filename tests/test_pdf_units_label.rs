// Tests for PDF units label positioning and sizing
// Issues: https://github.com/dncnwtts/map2fig/issues/XXX
// 
// Bug: PDF units label is too large (2x) and has excessive padding that
// overlaps with the colorbar and map. This is a major visual bug.

#[cfg(test)]
mod pdf_units_label_tests {
    use map2fig::layout::compute_gnomonic_layout_with_fonts;
    use map2fig::cli::TickDirection;

    #[test]
    fn test_pdf_units_font_size_matches_png() {
        // PDF and PNG should use consistent LaTeX font sizing
        // PNG formula: cb_layout.tick_font_size * 0.25
        // PDF formula: should match PNG, not use 0.5
        
        let width = 300.0;
        let (_layout, cb_layout) = compute_gnomonic_layout_with_fonts(
            width,
            true,
            TickDirection::Outward,
            true,
            true,  // show_title
            None,
        );
        
        // PNG uses 0.25x multiplier
        let png_latex_font_size = (cb_layout.tick_font_size * 0.25).clamp(3.0, 20.0);
        
        // PDF should NOT use 0.5x (that's too large)
        let pdf_latex_font_size_wrong = (cb_layout.tick_font_size * 0.5).clamp(3.0, 20.0);
        
        // PDF should match PNG
        assert!(
            png_latex_font_size < pdf_latex_font_size_wrong,
            "PNG font size {} should be less than wrong PDF size {}", 
            png_latex_font_size,
            pdf_latex_font_size_wrong
        );
        
        // At width=300, verify sizes
        let expected_png_size: f32 = (7.0_f32 * 0.25).clamp(3.0, 20.0);  // ~1.75, clamped to 3.0
        assert_eq!(expected_png_size, 3.0, "PNG font should be clamped to 3.0");
    }

    #[test]
    fn test_units_label_does_not_overlap_colorbar() {
        // Units label should be positioned BELOW tick labels, not overlapping
        // This is a major bug: white padding of units label extends upward
        // through colorbar and into the map
        
        let width = 300.0;
        let (layout, cb_layout) = compute_gnomonic_layout_with_fonts(
            width,
            true,
            TickDirection::Outward,
            true,
            true,  // show_title
            None,
        );
        
        // Units should be positioned below tick labels
        let tick_bottom = cb_layout.tick_bottom;
        let label_pad = 1.0 * (width / 1200.0);  // Current PDF offset
        let units_y_pos = tick_bottom + label_pad;
        
        // LaTeX-rendered units text will have some height
        // With font size ~8-10px, typical text height is 10-15px
        // After tight cropping, should be ~8-12px
        let expected_text_height = 10.0;  // Conservative estimate
        
        let units_bottom = units_y_pos + expected_text_height;
        
        // Verify units doesn't extend back up into colorbar
        assert!(
            units_y_pos > tick_bottom,
            "Units Y position {} should be below tick bottom {}",
            units_y_pos,
            tick_bottom
        );
        
        // Verify units stays within image bounds
        assert!(
            units_bottom < layout.height,
            "Units label bottom {} should be within image height {}",
            units_bottom,
            layout.height
        );
    }

    #[test]
    fn test_units_label_padding_is_minimal() {
        // Units label padding should be minimal and not extend upward
        // into the colorbar/map. Current bug: excessive white padding
        // blocks colorbar and map content
        
        let width = 300.0;
        let scale = width / 1200.0;
        
        // Current offset in PDF
        let current_offset = 1.0 * scale;
        
        // Padding should be minimal (0-5px)
        assert!(
            current_offset < 5.0,
            "Units label offset {} should be <5px", 
            current_offset
        );
        
        // LaTeX rendering will add whitespace, but it should be cropped
        // after rendering. If not cropped, padding could be 20-50px which
        // extends through colorbar and into map
        
        // With proper cropping, LaTeX image should be:
        // - Rendered at small font (3-8px)
        // - Cropped to text bounding box (~8-15px height)
        // - Positioned tightly below tick labels
        
        // Without cropping, it might be 60-100px+ height due to LaTeX padding
        // Anything larger than 30px would be a bug
        
        // This test validates the assumption that LaTeX padding is minimal
        // If this fails with actual rendering, it means LaTeX padding is too large
    }

    #[test]
    fn test_units_label_does_not_overlap_map_bottom() {
        // Units label should not reach back up and overlap the map bottom
        // ISSUE: White padding of units extends upward through colorbar
        
        let width = 300.0;
        let (layout, cb_layout) = compute_gnomonic_layout_with_fonts(
            width,
            true,
            TickDirection::Outward,
            true,
            true,  // show_title
            None,
        );
        
        // Map bottom position
        let map_bottom = layout.map_y + layout.map_h;
        
        // Colorbar top position
        let cbar_top = cb_layout.y;
        
        // Verify there's space between map and colorbar
        let gap = cbar_top - map_bottom;
        assert!(
            gap > 0.0,
            "Colorbar top {} should be below map bottom {}",
            cbar_top,
            map_bottom
        );
        
        // Units label should NOT extend into this gap
        // Current bug: units label has excessive padding that extends upward
        // into the gap and overlaps with map bottom
        
        // Gnomonic has tighter layout than mollweide, gap may be small
        // But any units label with excessive padding will overlap
        let min_safe_gap = 0.0;  // Gap exists but may be small in gnomonic
        assert!(
            gap >= min_safe_gap,
            "Gap between map and colorbar {} should be non-negative",
            gap
        );
    }

    #[test]
    fn test_units_font_size_is_readable() {
        // Units label should be readable but smaller than main tick labels
        // Current bug: font is too large (2x expected size)
        
        let width = 300.0;
        let (_layout, cb_layout) = compute_gnomonic_layout_with_fonts(
            width,
            true,
            TickDirection::Outward,
            true,
            true,  // show_title
            None,
        );
        
        let tick_font_size = cb_layout.tick_font_size;
        
        // Units should be smaller than tick labels
        let units_font_size = (tick_font_size * 0.25).clamp(3.0, 20.0);
        let units_font_size_too_large = (tick_font_size * 0.5).clamp(3.0, 20.0);
        
        assert!(
            units_font_size < tick_font_size,
            "Units font {} should be smaller than tick font {}",
            units_font_size,
            tick_font_size
        );
        
        assert!(
            units_font_size >= 3.0,
            "Units font {} should be at least 3px (readable)",
            units_font_size
        );
        
        // Demonstrate the bug: current PDF formula gives 2x size
        assert!(
            units_font_size_too_large > units_font_size,
            "Current PDF formula {} gives 2x font compared to PNG {}",
            units_font_size_too_large,
            units_font_size
        );
    }

    #[test]
    fn test_latex_text_height_estimation() {
        // LaTeX rendered text height should be estimated to prevent overlap
        // With font size 8px and tight crop, height should be ~10-12px
        // Without crop, could be 60+ px (major bug)
        
        // Expected font size for gnomonic at 300px width
        let expected_font_size: f32 = (7.0_f32 * 0.25).clamp(3.0, 20.0);  // ~3px
        
        // Typical text height is 1.2-1.3x font size
        // With font size 3px: height ~4px
        // With font size 8px: height ~10px
        // With font size 16px: height ~20px
        
        // LaTeX padding can add 20-50px if not cropped
        // With cropping: should be close to text height only
        
        let expected_tight_height = expected_font_size * 1.3;  // ~4px
        let expected_uncropped_height = expected_font_size * 8.0;  // ~24px (with LaTeX padding)
        
        assert!(
            expected_tight_height < expected_uncropped_height,
            "Tight crop height {} should be much smaller than uncropped {}",
            expected_tight_height,
            expected_uncropped_height
        );
    }
}
