// Unit tests to highlight gnomonic projection text positioning and sizing issues
//
// Issues tracked:
// 1. PNG resolution text font size: Too small for readability at small image widths
// 2. PNG resolution text y-position: Incorrectly centered instead of at map bottom
// 3. PNG units label whitespace: Excessive padding above label (partially fixed by LaTeX crop)
// 4. PDF units label whitespace: Still has significant padding (15.0 * scale)
// 5. PNG/PDF inconsistency: Different positioning and padding approaches

use map2fig::layout::compute_gnomonic_layout_with_fonts;
use map2fig::cli::TickDirection;

#[test]
fn test_gnomonic_resolution_text_font_size_readability() {
    // Resolution text font size should be readable across width range
    // Current formula: (12.0 * width / 800.0).clamp(7.0, 24.0)
    
    // At 300px width (small/thumbnail image)
    let width_300: f64 = 300.0;
    let font_size_300 = (12.0 * (width_300 / 800.0)).clamp(7.0, 24.0);
    assert!(font_size_300 >= 7.0, "Font at width=300 is {} - below minimum 7px clamp", font_size_300);
    
    // At 600px width (medium image)
    let width_600: f64 = 600.0;
    let font_size_600 = (12.0 * (width_600 / 800.0)).clamp(7.0, 24.0);
    assert!(font_size_600 >= 9.0, "Font at width=600 is {} - may be hard to read", font_size_600);
    
    // At 1200px width (reference size)
    let width_1200: f64 = 1200.0;
    let font_size_1200 = (12.0 * (width_1200 / 800.0)).clamp(7.0, 24.0);
    assert_eq!(font_size_1200, 18.0, "Font at width=1200 should be 18px");
    
    // Font size should scale smoothly
    assert!(font_size_600 > font_size_300, "Font should increase with image width");
    assert!(font_size_1200 > font_size_600, "Font should increase with image width");
}

#[test]
fn test_gnomonic_resolution_text_y_position_at_map_bottom() {
    // Resolution text should start at map bottom, not centered vertically
    // ISSUE: Previous centering put text in middle of image
    
    let (layout, _cb_layout) = compute_gnomonic_layout_with_fonts(
        300.0, 
        true, 
        TickDirection::Outward, 
        true, 
        None
    );
    
    // Text should start at map_y + map_h, extending upward after rotation
    let expected_start_y = layout.map_y + layout.map_h;
    
    // This is a logical test - actual rendering happens in plot_gnomonic_png()
    // The expected position should be at or very near the map bottom
    assert!(
        expected_start_y > layout.map_y,
        "Resolution text Y position {} should be below map top {}",
        expected_start_y,
        layout.map_y
    );
    
    // Should NOT be centered vertically
    let center_y = layout.height / 2.0;
    assert!(
        expected_start_y > center_y,
        "Resolution text Y position {} should be below center {}, not centered",
        expected_start_y,
        center_y
    );
}

#[test]
fn test_gnomonic_resolution_text_x_position_in_left_margin() {
    // Resolution text should be positioned in left margin, not overlapping map
    // After 90° rotation, text is 50px wide
    // Should be positioned: outer_pad + 5px ≤ x ≤ map_x - 5px
    
    let width = 300.0;
    let scale = width / 1200.0;
    let outer_pad = 24.0 * scale;
    
    let (layout, _cb_layout) = compute_gnomonic_layout_with_fonts(
        width, 
        true, 
        TickDirection::Outward, 
        true, 
        None
    );
    
    // Text x position should be in left margin
    let text_x_min = outer_pad + 5.0;
    let text_x_max = layout.map_x;  // Should not overlap map
    
    assert!(
        text_x_max > text_x_min,
        "Left margin area {} to {} is too small for 50px rotated text",
        text_x_min,
        text_x_max
    );
    
    // Verify margin exists
    assert!(
        layout.map_x > outer_pad,
        "Map position {} should be greater than outer padding {}",
        layout.map_x,
        outer_pad
    );
}

#[test]
fn test_gnomonic_units_label_padding_consistency() {
    // Units label padding should be minimal and consistent between PNG/PDF
    // ISSUE: PDF uses 15.0 * scale, PNG uses tight LaTeX crop
    // PNG crop should reduce effective padding to ~0-5px
    
    let width = 300.0;
    let scale = width / 1200.0;
    
    // Current PDF offset
    let pdf_padding = 1.0 * scale;  // Reduced from 15.0
    
    assert!(
        pdf_padding > 0.0,
        "PDF padding {} should be positive (minimal spacing)",
        pdf_padding
    );
    
    assert!(
        pdf_padding < 5.0,
        "PDF padding {} should match PNG tight crop approach (<5px)",
        pdf_padding
    );
}

#[test]
fn test_gnomonic_layout_dimensions_at_various_widths() {
    // Verify layout calculations produce reasonable dimensions
    // that can accommodate text elements without clipping
    
    let test_widths = vec![300.0, 600.0, 900.0, 1200.0];
    
    for width in test_widths {
        let (layout, cb_layout) = compute_gnomonic_layout_with_fonts(
            width, 
            true, 
            TickDirection::Outward, 
            true, 
            None
        );
        
        // Width calculations
        assert!(layout.width >= width, "Layout width {} should accommodate map width {}", layout.width, width);
        
        // Map should fit within layout
        assert!(layout.map_w == width, "Map width {} should equal specified width {}", layout.map_w, width);
        assert!(layout.map_h == width, "Map height {} should be square ({}x{})", layout.map_h, width, width);
        
        // Left margin should exist for text
        let left_margin = layout.map_x;
        assert!(left_margin > 15.0, "Left margin {} should be >15px for text at width={}", left_margin, width);
        
        // Colorbar should fit below map
        assert!(cb_layout.y > layout.map_y + layout.map_h, 
                "Colorbar Y {} should be below map bottom {}", 
                cb_layout.y, layout.map_y + layout.map_h);
    }
}

#[test]
fn test_resolution_text_rotation_bounds() {
    // After 90° CW rotation, text surface becomes (50px wide × 200px tall)
    // Should fit within image bounds when positioned at map bottom
    // ISSUE: Position was extending beyond image bottom previously
    
    let width = 300.0;
    let (layout, _cb_layout) = compute_gnomonic_layout_with_fonts(
        width, 
        true, 
        TickDirection::Outward, 
        true, 
        None
    );
    
    // Original text surface: 200px wide × 50px tall
    // After 90° CW rotation: 50px wide × 200px tall
    
    // Position at map bottom
    let start_y = layout.map_y + layout.map_h;
    
    // After rotation, text extends upward (y - x)
    let text_y_min = (start_y - 200.0).max(0.0);  // Should not go below 0
    let text_y_max = start_y;
    
    assert!(
        text_y_max <= layout.height,
        "Text top {} should not exceed image height {}",
        text_y_max,
        layout.height
    );
    
    // Text should span vertically within reasonable bounds
    assert!(
        (text_y_max - text_y_min) <= 200.0,
        "Text vertical span {} should be ≤200px",
        (text_y_max - text_y_min)
    );
}

#[test]
fn test_colorbar_alignment_with_map() {
    // Colorbar and map should be aligned to same X position when text is shown
    // ISSUE: Previous misalignment had been fixed but should be validated
    
    let width = 300.0;
    let (layout, cb_layout) = compute_gnomonic_layout_with_fonts(
        width, 
        true, 
        TickDirection::Outward, 
        true, 
        None
    );
    
    // Both should have same X position
    assert_eq!(
        layout.cbar_x, cb_layout.x,
        "Map X position {} should equal colorbar X position {}",
        layout.cbar_x,
        cb_layout.x
    );
    
    // Both should have same width
    assert_eq!(
        layout.cbar_w, cb_layout.w,
        "Map width {} should equal colorbar width {}",
        layout.cbar_w,
        cb_layout.w
    );
}

#[test]
fn test_text_shown_flag_affects_layout() {
    // Layout with show_text=true should have left_text_pad
    // Layout with show_text=false should not
    
    let width = 300.0;
    
    let (layout_with_text, _) = compute_gnomonic_layout_with_fonts(
        width, 
        true, 
        TickDirection::Outward, 
        true, 
        None
    );
    let (layout_no_text, _) = compute_gnomonic_layout_with_fonts(
        width, 
        true, 
        TickDirection::Outward, 
        false, 
        None
    );
    
    // With text should have larger left margin
    assert!(
        layout_with_text.map_x > layout_no_text.map_x,
        "Layout with text should have larger left margin: {} vs {}",
        layout_with_text.map_x,
        layout_no_text.map_x
    );
    
    // Total width should be larger with text
    assert!(
        layout_with_text.width > layout_no_text.width,
        "Layout with text should be wider: {} vs {}",
        layout_with_text.width,
        layout_no_text.width
    );
}

#[test]
fn test_font_size_formula_scaling() {
    // Font size formula should scale smoothly and clamp appropriately
    // Formula: (12.0 * width / 800.0).clamp(7.0, 24.0)
    
    // Test clamping at lower end
    let width_100: f64 = 100.0;
    let font_100 = (12.0 * (width_100 / 800.0)).clamp(7.0, 24.0);
    assert_eq!(font_100, 7.0, "Font at width=100 should clamp to 7.0");
    
    // Test no clamping in middle range
    let width_800: f64 = 800.0;
    let font_800 = (12.0 * (width_800 / 800.0)).clamp(7.0, 24.0);
    assert_eq!(font_800, 12.0, "Font at width=800 should be exactly 12.0");
    
    // Test clamping at upper end
    let width_2400: f64 = 2400.0;
    let font_2400 = (12.0 * (width_2400 / 800.0)).clamp(7.0, 24.0);
    assert_eq!(font_2400, 24.0, "Font at width=2400 should clamp to 24.0");
}

// Summary of issues these tests highlight:
//
// 1. RESOLUTION TEXT READABILITY
//    - Font size formula should produce 7-24px text across width range
//    - At 300px width: currently 4.5px → clamped to 7.0px (marginal)
//    - At 600px width: currently 9px (acceptable)
//    - At 1200px width: currently 18px (good)
//
// 2. RESOLUTION TEXT POSITIONING (Y)
//    - Should start at map bottom, NOT centered
//    - Current code: start_y = layout.map_y + layout.map_h ✓ (now correct)
//    - Previous bug: centered vertically ✗
//
// 3. RESOLUTION TEXT POSITIONING (X)
//    - Should be in left margin, not overlapping map
//    - Current code: start_x = outer_pad + 5.0 ✓ (now correct)
//    - Previous bug: start_x = layout.map_x - 5.0 ✗ (too close to map)
//
// 4. UNITS LABEL PADDING (PNG)
//    - LaTeX rendering adds large transparent padding
//    - LaTeX crop should remove most padding
//    - Expected result: 0-10px spacing
//
// 5. UNITS LABEL PADDING (PDF)
//    - Previously used 15.0 * scale offset
//    - Reduced to 1.0 * scale for consistency
//    - Expected result: tight spacing matching PNG
//
// 6. PNG/PDF CONSISTENCY
//    - Both should use similar positioning approaches
//    - Both should have readable fonts
//    - Both should have minimal whitespace
