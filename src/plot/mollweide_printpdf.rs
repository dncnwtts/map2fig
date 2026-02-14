use crate::PngSink;
use crate::healpix::is_seen;
use crate::layout::compute_mollweide_layout;
/// Minimal printpdf-based PDF rendering for benchmarking
/// This version focuses on fast image embedding without vector overlays
use crate::params::MollweideParams;
use crate::render::printpdf_backend::PrintpdfBackend;
use crate::scale::{HistogramRange, Scale, build_histogram_scale, unsafe_float_cmp};
use image::RgbaImage;

/// Fast PDF rendering using printpdf
/// This is a minimal implementation that embeds pre-rendered images
/// without vector overlays (graticule, colorbar, labels)
///
/// Purpose: Benchmark whether printpdf image embedding is faster than
/// Cairo's PDF reconstruction and compression
pub fn plot_mollweide_pdf_printpdf(params: MollweideParams) {
    let map = params.plot.map;
    let width = params.plot.width;
    let filename = params.plot.filename;
    let minv = params.scale.minv;
    let maxv = params.scale.maxv;
    let cmap = params.color.cmap;
    let transparent = params.display.transparent;
    let gamma = params.scale.gamma;
    let scale = params.scale.scale;
    let neg_mode = params.scale.neg_mode;
    let bad_color = params.color.bad_color;
    let meta = params.meta;
    let view = params.view;
    let mask = params.display.mask.as_ref();

    // Compute layout (just for dimensions, not for vector overlays)
    let (layout, _cb_layout) = compute_mollweide_layout(
        width as f64,
        false, // No colorbar in minimal version
        params.display.tick_direction.clone(),
    );

    // Get values for scaling
    let mut values: Vec<f64> = map.iter().filter(|&v| is_seen(*v)).copied().collect();
    if values.is_empty() {
        panic!("Map contains no valid HEALPix values");
    }
    values.sort_unstable_by(unsafe_float_cmp);

    // Compute scale
    let scale_params =
        crate::plot::mollweide::compute_mollweide_scale(map, minv, maxv, gamma, scale);

    let hist_scale_opt = if scale == Scale::Histogram {
        let range = match (minv, maxv) {
            (Some(minv), Some(maxv)) => HistogramRange::Explicit {
                min: minv,
                max: maxv,
            },
            _ => HistogramRange::Full,
        };
        Some(build_histogram_scale(map, range, 1024))
    } else {
        None
    };

    // Phase 2B: Image pre-rendering (same as Cairo version)
    let scale_cache = crate::scale::ScaleCache::new(scale_params.minv, scale_params.maxv, scale);
    let map_w_int = (layout.map_w + 2.0 * layout.map_pad) as u32;
    let map_h_int = (layout.map_h + 2.0 * layout.map_pad) as u32;
    let mut pixel_buffer = RgbaImage::new(map_w_int, map_h_int);

    // Clear buffer background
    let bg_color = if transparent {
        image::Rgba([0, 0, 0, 0])
    } else {
        image::Rgba([255, 255, 255, 255])
    };
    for pixel in pixel_buffer.pixels_mut() {
        *pixel = bg_color;
    }

    // Render pixels to buffer
    let mut sink = PngSink {
        img: &mut pixel_buffer,
        x0: 0,
        y0: 0,
    };

    let debug_overlay = if cfg!(feature = "debug_overlay") {
        Some(crate::plot::DebugOverlay::grid_only())
    } else {
        None
    };

    crate::plot::render_mollweide_pixels(
        crate::params::RenderMollweideParams {
            map,
            scale: &scale_params,
            cmap,
            gamma,
            scale_type: scale,
            neg_mode,
            bad_color,
            meta,
            hist_scale: hist_scale_opt.as_ref(),
            view,
            mask,
            scale_cache: Some(&scale_cache),
        },
        layout,
        &mut sink,
        debug_overlay,
    );

    // Embed image in printpdf (the core optimization)
    let mut backend = PrintpdfBackend::new(layout.width, layout.height);

    if let Err(e) = backend.embed_image_buffer(
        &pixel_buffer,
        layout.map_x,
        layout.map_y,
        layout.map_w,
        layout.map_h,
    ) {
        eprintln!("Failed to embed image: {}", e);
        return;
    }

    // Save PDF
    if let Err(e) = backend.save(filename) {
        eprintln!("Failed to save PDF: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minimal_pdf_rendering() {
        // Just verify the function signature is correct
        // Actual benchmarking happens in integration tests
    }
}
