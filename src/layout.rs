#[derive(Debug, Clone, Copy)] pub struct MollweideLayout {
    pub width: f64,
    pub height: f64,

    pub map_pad: f64,
    pub map_x: f64,
    pub map_y: f64,
    pub map_w: f64,
    pub map_h: f64,

    pub cbar_x: f64,
    pub cbar_y: f64,
    pub cbar_w: f64,
    pub cbar_h: f64,

    pub cbar_pad: f64,
    pub border_width_px: f64,
}


#[derive(Debug, Clone)]
pub struct ColorbarLayout {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,

    pub tick_bottom: f64,
    pub major_tick_height: f64,
    pub minor_tick_height: f64,
    pub major_tick_width: f64,
    pub minor_tick_width: f64,

    pub tick_font_size: f64,
    pub tick_label_pad: f64,
    pub tick_direction: crate::cli::TickDirection,
}


pub fn compute_mollweide_layout(
    width: f64,
    show_colorbar: bool,
    tick_direction: crate::cli::TickDirection,
) -> (MollweideLayout, ColorbarLayout) {
    compute_mollweide_layout_with_fonts(width, show_colorbar, tick_direction, None)
}

pub fn compute_mollweide_layout_with_fonts(
    width: f64,
    show_colorbar: bool,
    tick_direction: crate::cli::TickDirection,
    custom_tick_font_size: Option<f32>,
) -> (MollweideLayout, ColorbarLayout) {
    // Scale all dimensions with width (reference: 1200px)
    let scale = width / 1200.0;
    
    let outer_pad = 24.0 * scale;      // margin around everything
    let cbar_pad  = 16.0 * scale;      // space between map and colorbar
    let label_pad = 14.0 * scale;      // space for text descenders
    let units_pad = if show_colorbar { -4.0 * scale } else { 0.0 };  // negative to remove bottom whitespace

    let border_width_px = (width * 0.0025).max(2.0 * scale);
    let map_pad = border_width_px.ceil() + 2.0 * scale;

    let map_w = width - 2.0 * outer_pad;
    let map_h = map_w / 2.0;
    let cbar_h = if show_colorbar { 
        // Ensure colorbar height is a multiple of 3 for proper triangle rendering
        // Triangle rendering requires height % 3 == 0 to avoid asymmetries
        let base_h = map_h / 20.0;
        let rounded = base_h.round();
        // Round to nearest multiple of 3
        ((rounded / 3.0).round() * 3.0).max(12.0)
    } else { 
        0.0 
    };
    let label_h = if show_colorbar { 70.0 * scale } else { 0.0 };
    
    // Add extra padding for outward ticks: major ticks extend downward, taking up space
    let tick_overflow = match &tick_direction {
        crate::cli::TickDirection::Outward => (cbar_h * 0.5).max(6.0 * scale),
        crate::cli::TickDirection::Inward => 0.0,
    };

    let height =
        outer_pad +
        map_h +
        map_pad +
        cbar_pad + 
        cbar_h +
        label_h +
        tick_overflow +
        units_pad +
        outer_pad;

    let map_x = outer_pad;
    let map_y = outer_pad;

    let cbar_y = map_y + map_h + map_pad + cbar_pad;

    (MollweideLayout {
        width,
        height,

        map_pad,
        map_x,
        map_y,
        map_w: width - 2.0 * outer_pad,
        map_h,

        cbar_x: outer_pad,
        cbar_y,
        cbar_w: width - 2.0 * outer_pad,
        cbar_h,

        cbar_pad,
        border_width_px,
    },
    compute_cbar_layout(outer_pad, cbar_y, width - 2.0 * outer_pad, cbar_h, label_pad, width, tick_direction, custom_tick_font_size, 1.0)
    )
}


fn compute_cbar_layout(cbar_x: f64, cbar_y:f64, cbar_w:f64, cbar_h:f64, label_pad:f64, width: f64, tick_direction: crate::cli::TickDirection, custom_tick_font_size: Option<f32>, width_scale: f64) -> ColorbarLayout {
    // Scale tick font size using actual image width
    // Default: 12pt at width=800px, so scale is (width / 800.0)
    let tick_font_size = if let Some(custom) = custom_tick_font_size {
        custom as f64
    } else {
        (12.0 * (width / 800.0)).clamp(7.0, 24.0)
    };
    
    ColorbarLayout {
        x: cbar_x,
        y: cbar_y,
        w: cbar_w,
        h: cbar_h,

        tick_bottom: cbar_y + cbar_h,
        major_tick_height: (cbar_h * 0.5).round().max(1.0),
        minor_tick_height: (cbar_h * 0.3).round().max(1.0),
        major_tick_width: ((cbar_w * 0.002) * width_scale).round().max(1.0),
        minor_tick_width: ((cbar_w * 0.001) * width_scale).round().max(1.0),

        tick_font_size,
        tick_label_pad: cbar_h + cbar_y + label_pad,
        tick_direction,
    }
}

/// Compute layout for gnomonic projection with optional colorbar
pub fn compute_gnomonic_layout(
    map_size: f64,
    show_colorbar: bool,
    tick_direction: crate::cli::TickDirection,
    show_text: bool,
    show_title: bool,
) -> (MollweideLayout, ColorbarLayout) {
    compute_gnomonic_layout_with_fonts(map_size, show_colorbar, tick_direction, show_text, show_title, None)
}

pub fn compute_gnomonic_layout_with_fonts(
    map_size: f64,
    show_colorbar: bool,
    tick_direction: crate::cli::TickDirection,
    show_text: bool,
    show_title: bool,
    custom_tick_font_size: Option<f32>,
) -> (MollweideLayout, ColorbarLayout) {
    // Scale all dimensions with map size (reference: 1200px)
    let scale = map_size / 1200.0;
    
    let outer_pad = 24.0 * scale;      // margin around everything
    // Title padding: add space at top for title when show_title is true
    // Increased to 32.0 to accommodate title text height and prevent clipping
    let title_pad = if show_title { 
        32.0 * scale  // increased from 20.0 to prevent text clipping
    } else { 
        0.0 
    };
    // Space for rotated text label on left: after 90° CCW rotation, the text height (50px) extends to the left
    // Original surface: 200 wide x 50 tall
    // After 90° CCW: 50 wide x 200 tall
    // We position at start_x, and rotated_x = src_y means rotated_x ranges from 0 to surface_height
    // Text extends from 0 to rotated_extent
    // Whitespace from rotated_extent to 1.25*rotated_extent (which is 0.25*rotated_extent)
    // Map should start at 1.25*rotated_extent
    let left_text_pad = if show_text { 
        50.0 * scale  // left margin for rotated text label
    } else { 
        0.0 
    };
    let cbar_pad  = 8.0 * scale;       // minimal gap between map and colorbar
    let label_pad = 4.0 * scale;       // minimal space for text descenders (reduced from 14.0)
    let units_pad = if show_colorbar { -4.0 * scale } else { 0.0 };  // negative to remove bottom whitespace

    let border_width_px = (map_size * 0.0025).max(2.0 * scale);
    let map_pad = border_width_px.ceil() + 2.0 * scale;

    let map_w = map_size;
    let map_h = map_size;
    let cbar_h = if show_colorbar { 
        // Ensure colorbar height is a multiple of 3 for proper triangle rendering
        // Triangle rendering requires height % 3 == 0 to avoid asymmetries
        let base_h = map_h / 25.0;
        let rounded = base_h.round();
        // Round to nearest multiple of 3
        ((rounded / 3.0).round() * 3.0).max(12.0)
    } else { 
        0.0 
    };
    // Increased from 70 to 120 to accommodate units label with extra vertical spacing (12pt * scale)
    let label_h = if show_colorbar { 120.0 * scale } else { 0.0 };
    
    // Add extra padding for outward ticks: major ticks extend downward, taking up space
    let tick_overflow = match &tick_direction {
        crate::cli::TickDirection::Outward => (cbar_h * 0.5).max(6.0 * scale),
        crate::cli::TickDirection::Inward => 0.0,
    };

    let height =
        outer_pad +
        title_pad +
        map_h +
        cbar_pad +
        cbar_h +
        label_h +
        tick_overflow +
        units_pad +
        outer_pad;

    let map_x = outer_pad + (if show_text { left_text_pad } else { 0.0 });
    let map_y = outer_pad + title_pad;
    let cbar_y = map_y + map_h + cbar_pad;
    let width = map_w + outer_pad + (if show_text { left_text_pad } else { 0.0 }) + outer_pad;

    let width_scale = map_size / 1200.0;

    (MollweideLayout {
        width,
        height,

        map_pad,
        map_x,
        map_y,
        map_w,
        map_h,

        cbar_x: map_x,  // Colorbar should be aligned with map
        cbar_y,
        cbar_w: map_w,
        cbar_h,

        cbar_pad,
        border_width_px,
    },
    compute_cbar_layout(map_x, cbar_y, map_w, cbar_h, label_pad, width, tick_direction, custom_tick_font_size, width_scale)
    )
}
