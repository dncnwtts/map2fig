use map2fig::cli::TickDirection;
use map2fig::layout::compute_gnomonic_layout;

#[test]
fn test_gnomonic_layout_colorbar_alignment_with_text() {
    // When show_text = true, the map should be shifted right by left_text_pad
    // and the colorbar should also be shifted right to stay aligned with the map

    let map_size = 1200.0;
    let show_colorbar = true;
    let show_text = true;
    let tick_direction = TickDirection::Inward;

    let (layout, _cb_layout) = compute_gnomonic_layout(
        map_size,
        show_colorbar,
        tick_direction,
        show_text,
        true, // show_title
    );

    // With show_text = true, left_text_pad = 50.0 * scale, where scale = 1200/1200 = 1.0
    // So left_text_pad = 50.0
    let expected_left_text_pad = 50.0;
    let outer_pad = 24.0;

    // Map should be shifted right by left_text_pad
    let expected_map_x = outer_pad + expected_left_text_pad;
    assert_eq!(
        layout.map_x, expected_map_x,
        "Map X position should include left_text_pad offset when show_text=true"
    );

    // Colorbar should also be shifted right by the same amount (aligned with map)
    let expected_cbar_x = outer_pad + expected_left_text_pad;
    assert_eq!(
        layout.cbar_x, expected_cbar_x,
        "Colorbar X position should be aligned with map (shifted by left_text_pad)"
    );

    // Map and colorbar should have the same X position
    assert_eq!(
        layout.map_x, layout.cbar_x,
        "Map and colorbar should be horizontally aligned when show_text=true"
    );
}

#[test]
fn test_gnomonic_layout_colorbar_alignment_without_text() {
    // When show_text = false, neither map nor colorbar should have extra left padding

    let map_size = 1200.0;
    let show_colorbar = true;
    let show_text = false;
    let tick_direction = TickDirection::Inward;

    let (layout, _cb_layout) = compute_gnomonic_layout(
        map_size,
        show_colorbar,
        tick_direction,
        show_text,
        true, // show_title
    );

    let outer_pad = 24.0;
    let expected_x = outer_pad; // No left_text_pad when show_text = false

    // Both map and colorbar should be at the same position
    assert_eq!(
        layout.map_x, expected_x,
        "Map X position should be at outer_pad when show_text=false"
    );

    assert_eq!(
        layout.cbar_x, expected_x,
        "Colorbar X position should be at outer_pad when show_text=false"
    );

    assert_eq!(
        layout.map_x, layout.cbar_x,
        "Map and colorbar should be aligned when show_text=false"
    );
}

#[test]
fn test_gnomonic_layout_map_and_colorbar_always_aligned() {
    // Regardless of show_text or show_colorbar settings, map and colorbar
    // should always be horizontally aligned

    for show_text in &[true, false] {
        for show_colorbar in &[true, false] {
            let (layout, _cb_layout) = compute_gnomonic_layout(
                1200.0,
                *show_colorbar,
                TickDirection::Inward,
                *show_text,
                true, // show_title
            );

            assert_eq!(
                layout.map_x, layout.cbar_x,
                "Map and colorbar should be aligned for show_text={}, show_colorbar={}",
                show_text, show_colorbar
            );
        }
    }
}

#[test]
fn test_gnomonic_layout_text_pad_scales_with_map_size() {
    // The left_text_pad should scale with the map size
    // At reference size 1200px: left_text_pad = 50.0
    // At size 600px: left_text_pad = 25.0 (scale = 0.5)
    // At size 2400px: left_text_pad = 100.0 (scale = 2.0)

    let outer_pad = 24.0;
    // base_text_pad = 50.0 at reference scale

    let test_cases = vec![
        (600.0, 25.0),   // scale = 0.5
        (1200.0, 50.0),  // scale = 1.0
        (2400.0, 100.0), // scale = 2.0
    ];

    for (map_size, expected_text_pad) in test_cases {
        let (layout, _cb_layout) = compute_gnomonic_layout(
            map_size,
            true,
            TickDirection::Inward,
            true, // show_text = true
            true, // show_title
        );

        let expected_x = outer_pad * (map_size / 1200.0) + expected_text_pad;

        assert!(
            (layout.map_x - expected_x).abs() < 0.1,
            "At map_size={}, expected map_x={}, got {}",
            map_size,
            expected_x,
            layout.map_x
        );

        assert_eq!(
            layout.map_x, layout.cbar_x,
            "Map and colorbar should be aligned at map_size={}",
            map_size
        );
    }
}
