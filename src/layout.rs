#[derive(Debug, Clone, Copy)]
pub struct MollweideLayout {
    pub width: f64,
    pub height: f64,

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


#[derive(Debug, Clone, Copy)]
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
}


pub fn compute_mollweide_layout(
    width: f64,
    show_colorbar: bool,
) -> (MollweideLayout, ColorbarLayout) {
    let outer_pad = 24.0;      // margin around everything
    let map_pad   = 8.0;       // breathing room for border AA
    let cbar_pad  = 16.0;      // space between map and colorbar
    let label_pad = 14.0;      // space for text descenders

    let map_w = width - 2.0 * outer_pad;
    let map_h = map_w / 2.0;
    let cbar_h = if show_colorbar { map_h / 20.0} else { 0.0 };
    let label_h = if show_colorbar { 18.0 } else { 0.0 };

    let height =
        outer_pad +
        map_h +
        map_pad +
        cbar_pad + 
        cbar_h +
        label_h +
        outer_pad;

    let map_x = outer_pad;
    let map_y = outer_pad;

    let cbar_y = map_y + map_h + map_pad + cbar_pad;

    (MollweideLayout {
        width,
        height,

        map_x,
        map_y,
        map_w: width - 2.0 * outer_pad,
        map_h,

        cbar_x: outer_pad,
        cbar_y,
        cbar_w: width - 2.0 * outer_pad,
        cbar_h,

        cbar_pad,
        border_width_px:  (width as f64 * 0.01).max(2.0),
    },
    compute_cbar_layout(outer_pad, cbar_y, width - 2.0 * outer_pad, cbar_h, label_pad)
    )
}


fn compute_cbar_layout(cbar_x: f64, cbar_y:f64, cbar_w:f64, cbar_h:f64, label_pad:f64) -> ColorbarLayout {
    ColorbarLayout {
        x: cbar_x,
        y: cbar_y,
        w: cbar_w,
        h: cbar_h,

        tick_bottom: cbar_y + cbar_h,
        major_tick_height: (cbar_h * 0.5).round().max(1.0),
        minor_tick_height: (cbar_h * 0.3).round().max(1.0),
        major_tick_width: (cbar_w * 0.002).round().max(1.0),
        minor_tick_width: (cbar_w * 0.001).round().max(1.0),

        tick_font_size: (cbar_h * 0.3).max(10.0),
        tick_label_pad: cbar_h + cbar_y + label_pad,
    }
}
