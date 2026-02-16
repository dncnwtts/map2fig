use super::mollweide::{
    _plot_mollweide_pdf_impl, _plot_mollweide_png_impl_projected, ProjectionType,
};
use crate::params::HammerParams;
use crate::params::MollweideParams;
use std::path::Path;

fn render_hammer_pixels(
    params: crate::params::RenderMollweideParams,
    layout: crate::layout::MollweideLayout,
    sink: &mut dyn crate::PixelSink,
    debug_overlay: Option<super::DebugOverlay>,
) {
    use crate::hammer::HammerProjection;
    use crate::render::raster::RasterGrid;
    let proj = HammerProjection::new();

    let mut grid = RasterGrid::new(layout.map_w as u32, layout.map_h as u32);

    if let Some(overlay) = debug_overlay
        && overlay.show_background
    {
        super::fill_grid_background(&mut grid);
    }

    super::render_projection_to_grid(
        crate::params::RenderGridParams {
            map: params.map,
            proj: &proj,
            scale: params.scale,
            cmap: params.cmap,
            scale_type: params.scale_type,
            neg_mode: params.neg_mode,
            gamma: params.gamma,
            bad_color: params.bad_color,
            meta: params.meta,
            hist_scale: params.hist_scale,
            view: params.view,
            mask: params.mask,
            scale_cache: params.scale_cache,
            underflow: (255, 0, 0),
            overflow: (0, 0, 255),
        },
        &mut grid,
    );

    // Draw debug overlay only if provided
    if let Some(overlay) = debug_overlay {
        super::draw_debug_overlay_raster(&mut grid, overlay);
    }

    super::blit_grid_to_sink(&grid, sink, 0, 0);
}

/// Plot a Hammer projection map as PNG.
pub fn plot_hammer_png(params: HammerParams) {
    let mollweide_params = MollweideParams {
        plot: params.plot,
        scale: params.scale,
        color: params.color,
        display: params.display,
        graticule: params.graticule,
        meta: params.meta,
        view: params.view,
    };
    _plot_mollweide_png_impl_projected(
        mollweide_params,
        render_hammer_pixels,
        ProjectionType::Hammer,
    );
}

/// Plot a Hammer projection map as PDF.
pub fn plot_hammer_pdf(params: HammerParams) {
    let mollweide_params = MollweideParams {
        plot: params.plot,
        scale: params.scale,
        color: params.color,
        display: params.display,
        graticule: params.graticule,
        meta: params.meta,
        view: params.view,
    };
    _plot_mollweide_pdf_impl(mollweide_params, render_hammer_pixels);
}

/// Automatically choose PNG or PDF based on file extension for Hammer projection.
pub fn plot_hammer_auto(params: HammerParams) {
    let ext = Path::new(params.plot.filename.as_str())
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "png" => plot_hammer_png(params),
        "pdf" => plot_hammer_pdf(params),
        _ => {
            panic!(
                "Unsupported output format: .{} (expected .png or .pdf)",
                ext
            );
        }
    }
}
