//! Graticule rendering for gnomonic projections
//!
//! Supports two modes:
//! 1. Local grid: Constant lat/lon lines in the tangent plane coordinate system
//! 2. Sky coordinates: Constant lat/lon lines from a specified celestial coordinate system

use std::f64::consts::PI;
use crate::gnomonic::GnomonicProjection;
use crate::rotation::{CoordSystem, ViewTransform, coord_rotation};
use crate::render::raster::RasterGrid;
use image::Rgba;

/// Mode of graticule operation for gnomonic projections
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GnomonicGraticuleMode {
    /// Local grid: Lines of constant lat/lon in the tangent plane
    /// This is the natural coordinate system centered on the gnomonic center
    LocalGrid,
    /// Sky coordinate overlay: Lines from a specified celestial coordinate system
    SkyCoordinate,
}

/// Gnomonic graticule configuration
#[derive(Debug, Clone, Copy)]
pub struct GnomonicGraticule {
    /// Rendering mode
    pub mode: GnomonicGraticuleMode,
    /// Spacing for meridians (longitude) in degrees
    pub dlon_deg: f64,
    /// Spacing for parallels (latitude) in degrees
    pub dlat_deg: f64,
    /// For sky coordinate mode: which coordinate system to overlay
    pub coord_system: Option<CoordSystem>,
    /// For sky coordinate mode: the input coordinate system of the map
    pub input_coord: Option<CoordSystem>,
}

impl GnomonicGraticule {
    /// Create a local grid graticule with given spacing
    pub fn local_grid(dlon_deg: f64, dlat_deg: f64) -> Self {
        Self {
            mode: GnomonicGraticuleMode::LocalGrid,
            dlon_deg,
            dlat_deg,
            coord_system: None,
            input_coord: None,
        }
    }

    /// Create a sky coordinate overlay with given spacing
    pub fn sky_coordinate(
        grat_coord: CoordSystem,
        input_coord: CoordSystem,
        dlon_deg: f64,
        dlat_deg: f64,
    ) -> Self {
        Self {
            mode: GnomonicGraticuleMode::SkyCoordinate,
            dlon_deg,
            dlat_deg,
            coord_system: Some(grat_coord),
            input_coord: Some(input_coord),
        }
    }
}

/// Generate evenly-spaced graticule lines for gnomonic (local tangent plane) coordinates
/// These are in local spherical coords centered on the gnomonic projection center
fn generate_graticule_degrees(spacing_deg: f64, is_latitude: bool) -> Vec<f64> {
    let mut degrees: Vec<f64> = Vec::new();
    
    if is_latitude {
        // For latitude: include equator, and lines up to ±60 degrees
        degrees.push(0.0);   // Equator
        let mut deg = spacing_deg;
        while deg <= 60.0 {
            degrees.push(deg);
            degrees.push(-deg);
            deg += spacing_deg;
        }
    } else {
        // For longitude: lines from -60 to +60 degrees (stay away from horizon)
        let mut deg = 0.0;
        while deg <= 60.0 {
            degrees.push(deg);
            if deg > 0.0 {
                degrees.push(-deg);
            }
            deg += spacing_deg;
        }
    }
    
    // Sort
    degrees.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    degrees
}

/// Generate evenly-spaced graticule lines for sky coordinates (full sphere)
/// Returns meridian lines (constant longitude) or parallel lines (constant latitude)
fn generate_sky_graticule_degrees(spacing_deg: f64, is_latitude: bool) -> Vec<f64> {
    let mut degrees: Vec<f64> = Vec::new();
    
    if is_latitude {
        // For latitude/declination: from -90° to +90°, including equator/celestial equator
        degrees.push(0.0);   // Equator
        let mut deg = spacing_deg;
        while deg <= 90.0 {
            degrees.push(deg);
            degrees.push(-deg);
            deg += spacing_deg;
        }
    } else {
        // For longitude/right ascension: full circle from 0° to 360°
        let mut deg = 0.0;
        while deg < 360.0 {
            degrees.push(deg);
            deg += spacing_deg;
        }
    }
    
    // Sort
    degrees.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    degrees
}

/// Render local grid graticule on gnomonic projection
pub fn render_gnomonic_local_grid(
    grid: &mut RasterGrid,
    proj: &GnomonicProjection,
    dlon_deg: f64,
    dlat_deg: f64,
) {
    render_gnomonic_local_grid_colored(grid, proj, dlon_deg, dlat_deg, Rgba([0u8, 0u8, 0u8, 255u8]));
}

/// Render local grid graticule on gnomonic projection with custom color
fn render_gnomonic_local_grid_colored(
    grid: &mut RasterGrid,
    proj: &GnomonicProjection,
    dlon_deg: f64,
    dlat_deg: f64,
    color: Rgba<u8>,
) {
    let meridian_degrees = generate_graticule_degrees(dlon_deg, false);
    let parallel_degrees = generate_graticule_degrees(dlat_deg, true);

    // Precompute resolution in radians and grid center
    let delta = proj.resolution_arcmin * PI / (180.0 * 60.0);
    let xsize = grid.width as f64;
    let ysize = grid.height as f64;
    let center_x = xsize / 2.0;
    let center_y = ysize / 2.0;

    // Render meridians (constant longitude in local frame)
    for &lon_deg in &meridian_degrees {
        let lon_rad = lon_deg * PI / 180.0;
        
        // Sample from -45° to +45° latitude (sufficient for local grid)
        let mut points = Vec::new();
        for lat_deg_int in -45..=45 {
            let lat_rad = lat_deg_int as f64 * PI / 180.0;
            
            // Convert local spherical coords to tangent plane (gnomonic projection)
            if let Some((px, py)) = gnomonic_local_to_pixel_abs(lon_rad, lat_rad, delta, center_x, center_y) {
                points.push((px, py));
            }
        }
        
        // Draw line segments
        for window in points.windows(2) {
            draw_line_on_grid_colored(grid, window[0].0, window[0].1, window[1].0, window[1].1, color);
        }
    }

    // Render parallels (constant latitude in local frame)
    for &lat_deg in &parallel_degrees {
        let lat_rad = lat_deg * PI / 180.0;
        
        // Sample longitude range with finer steps
        let mut points = Vec::new();
        for lon_step in -60..60 {
            let lon_rad = lon_step as f64 * PI / 180.0;
            
            if let Some((px, py)) = gnomonic_local_to_pixel_abs(lon_rad, lat_rad, delta, center_x, center_y) {
                points.push((px, py));
            }
        }
        
        // Draw line segments
        for window in points.windows(2) {
            draw_line_on_grid_colored(grid, window[0].0, window[0].1, window[1].0, window[1].1, color);
        }
    }
}

/// Convert local spherical coordinates (lon, lat) to pixel coordinates
/// Using the gnomonic projection: tangent plane at z=1
fn gnomonic_local_to_pixel_abs(
    lon_local: f64,
    lat_local: f64,
    delta: f64,
    center_x: f64,
    center_y: f64,
) -> Option<(f64, f64)> {
    // Gnomonic projection formulas (for a tangent plane at z=1):
    //   x = tan(lon_local)
    //   y = tan(lat_local) / cos(lon_local)
    // These give coordinates on the tangent plane in the same scale as angles (radians)
    
    let cos_lat = lat_local.cos();
    let cos_lon = lon_local.cos();
    
    // Check if point is in front of the tangent plane
    if cos_lat * cos_lon <= 0.0 {
        return None;
    }
    
    // Compute tangent plane coordinates
    let x_tangent = lon_local.tan();
    let y_tangent = lat_local.tan() / cos_lon;
    
    // Convert from tangent plane coordinates (radians) to pixel coordinates
    // pixel_offset = tangent_offset / delta
    // pixel_coordinate = center + pixel_offset
    let px = center_x - x_tangent / delta;  // Note: x offset is negated because pixel x increases rightward
    let py = center_y - y_tangent / delta;  // Note: y offset is negated because pixel y increases downward
    
    Some((px, py))
}

/// Render sky coordinate overlay on gnomonic projection
pub fn render_gnomonic_sky_overlay(
    grid: &mut RasterGrid,
    proj: &GnomonicProjection,
    view: &ViewTransform,
    dlon_deg: f64,
    dlat_deg: f64,
    grat_coord: CoordSystem,
    input_coord: CoordSystem,
    color: Rgba<u8>,
) {
    // Generate graticule lines in the overlay coordinate system
    // Sample the full range to ensure all lines are attempted
    let parallel_degrees = generate_sky_graticule_degrees(dlat_deg, true);
    
    // For meridians, generate lines at regular spacing around the full circle
    let mut meridian_degrees: Vec<f64> = Vec::new();
    let mut lon_deg = 0.0;
    while lon_deg < 360.0 {
        meridian_degrees.push(lon_deg);
        lon_deg += dlon_deg;
    }

    // Transform path: grat_coord → input_coord (what the view expects)
    let grat_to_input = coord_rotation(grat_coord, input_coord);

    // Precompute projection parameters
    let delta = proj.resolution_arcmin * PI / (180.0 * 60.0);
    let xsize = grid.width as f64;
    let ysize = grid.height as f64;
    let center_x = xsize / 2.0;
    let center_y = ysize / 2.0;
    
    // Adaptive sampling: step size in degrees based on grid resolution
    // Aim for roughly 1-2 pixels per sample to avoid oversampling
    // With very high resolution inputs, we can still have reasonable sampling with larger steps
    let step_deg = (2.0 * delta * 180.0 / PI).max(5.0); // At least 5° steps

    // Render meridians in the overlay coordinate system
    for &lon_deg in &meridian_degrees {
        let lon_rad = lon_deg * PI / 180.0;

        // Sample parallels with adaptive steps, covering full range including poles
        let mut points = Vec::new();
        let mut lat_deg = -90.0;
        while lat_deg <= 90.0 {
            let lat_rad = lat_deg * PI / 180.0;

            // Point in graticule coordinate system (3D unit vector)
            let cos_lat = lat_rad.cos();
            let x_grat = cos_lat * lon_rad.cos();
            let y_grat = cos_lat * lon_rad.sin();
            let z_grat = lat_rad.sin();

            // Transform to input coordinate system (what the view expects)
            let v_input = grat_to_input.apply([x_grat, y_grat, z_grat]);

            // Apply view transformation (coordinate system + camera rotation)
            let v_view = view.apply(v_input);

            // Project to gnomonic local coordinates
            if let Some((px, py)) = project_to_gnomonic_local(v_view, proj, delta, center_x, center_y) {
                points.push((px, py));
            }
            
            lat_deg += step_deg;
        }

        // Draw line segments, handling discontinuities
        let mut current_segment: Vec<(f64, f64)> = Vec::new();
        for point in points {
            if current_segment.is_empty() {
                current_segment.push(point);
            } else {
                let last = current_segment[current_segment.len() - 1];
                let dist = ((point.0 - last.0).powi(2) + (point.1 - last.1).powi(2)).sqrt();
                // If discontinuity detected (large jump), start new segment
                if dist > 50.0 {
                    for window in current_segment.windows(2) {
                        draw_line_on_grid_colored(grid, window[0].0, window[0].1, window[1].0, window[1].1, color);
                    }
                    current_segment.clear();
                    current_segment.push(point);
                } else {
                    current_segment.push(point);
                }
            }
        }
        // Draw remaining segment
        for window in current_segment.windows(2) {
            draw_line_on_grid_colored(grid, window[0].0, window[0].1, window[1].0, window[1].1, color);
        }
    }

    // Render parallels in the overlay coordinate system
    for &lat_deg in &parallel_degrees {
        let lat_rad = lat_deg * PI / 180.0;

        // Sample meridians with adaptive steps, covering full range
        let mut points = Vec::new();
        let mut lon_deg = 0.0;
        while lon_deg < 360.0 {
            let lon_rad = lon_deg * PI / 180.0;

            // Point in graticule coordinate system (3D unit vector)
            let cos_lat = lat_rad.cos();
            let x_grat = cos_lat * lon_rad.cos();
            let y_grat = cos_lat * lon_rad.sin();
            let z_grat = lat_rad.sin();

            // Transform to input coordinate system (map coordinates)
            let v_input = grat_to_input.apply([x_grat, y_grat, z_grat]);

            // Apply view rotation
            let v_view = view.apply(v_input);

            // Project to gnomonic local coordinates
            if let Some((px, py)) = project_to_gnomonic_local(v_view, proj, delta, center_x, center_y) {
                points.push((px, py));
            }
            
            lon_deg += step_deg;
        }

        // Draw line segments, handling discontinuities
        let mut current_segment: Vec<(f64, f64)> = Vec::new();
        for point in points {
            if current_segment.is_empty() {
                current_segment.push(point);
            } else {
                let last = current_segment[current_segment.len() - 1];
                let dist = ((point.0 - last.0).powi(2) + (point.1 - last.1).powi(2)).sqrt();
                // If discontinuity detected (large jump), start new segment
                if dist > 50.0 {
                    for window in current_segment.windows(2) {
                        draw_line_on_grid_colored(grid, window[0].0, window[0].1, window[1].0, window[1].1, color);
                    }
                    current_segment.clear();
                    current_segment.push(point);
                } else {
                    current_segment.push(point);
                }
            }
        }
        // Draw remaining segment
        for window in current_segment.windows(2) {
            draw_line_on_grid_colored(grid, window[0].0, window[0].1, window[1].0, window[1].1, color);
        }
    }
}

/// Project a 3D unit vector to gnomonic local coordinates
/// 
/// Takes a 3D unit vector in the view frame and projects it to the gnomonic tangent plane.
fn project_to_gnomonic_local(
    v: [f64; 3],
    _proj: &GnomonicProjection,
    delta: f64,
    center_x: f64,
    center_y: f64,
) -> Option<(f64, f64)> {
    // Extract vector components (in view frame where projection center is at north pole)
    let x = v[0];
    let y = v[1];
    let z = v[2];

    // Check if point is in front of the tangent plane (z > 0)
    if z <= 0.0 {
        return None;
    }

    // Project to tangent plane (gnomonic formula: divide by z component)
    // Tangent plane is at z=1 after normalization
    let x_tangent = x / z;
    let y_tangent = y / z;

    // Convert from tangent plane coordinates (radians) to pixel coordinates
    let px = center_x - x_tangent / delta;
    let py = center_y - y_tangent / delta;

    Some((px, py))
}

/// Draw a line segment on the raster grid using Bresenham's algorithm
fn draw_line_on_grid_colored(grid: &mut RasterGrid, x0: f64, y0: f64, x1: f64, y1: f64, color: Rgba<u8>) {
    // Input coordinates are already in pixel space (not normalized)
    let px0 = x0 as i32;
    let py0 = y0 as i32;
    let px1 = x1 as i32;
    let py1 = y1 as i32;
    
    bresenham_line(grid, px0, py0, px1, py1, color);
}

/// Bresenham's line algorithm for drawing lines on raster grid
fn bresenham_line(grid: &mut RasterGrid, x0: i32, y0: i32, x1: i32, y1: i32, color: Rgba<u8>) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = (dx as f64 - dy as f64) / 2.0;
    
    let mut x = x0;
    let mut y = y0;
    
    loop {
        if x >= 0 && x < grid.width as i32 && y >= 0 && y < grid.height as i32 {
            grid.set_pixel(x as u32, y as u32, color);
        }
        
        if x == x1 && y == y1 {
            break;
        }
        
        let e2 = err;
        if e2 > -dx as f64 {
            err -= dy as f64;
            x += sx;
        }
        if e2 < dy as f64 {
            err += dx as f64;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_grid_generation() {
        let lons = generate_graticule_degrees(30.0, false);
        let lats = generate_graticule_degrees(30.0, true);
        
        // Function generates lines within ±60° to avoid horizon distortion in gnomonic projection
        // Meridians within that range
        assert!(lons.contains(&0.0));
        assert!(lons.contains(&30.0));
        assert!(lons.contains(&-30.0));
        assert!(lons.contains(&60.0));
        assert!(lons.contains(&-60.0));
        
        // Should include equator and parallels within range
        assert!(lats.contains(&0.0));
        assert!(lats.contains(&30.0));
        assert!(lats.contains(&-30.0));
        assert!(lats.contains(&60.0));
        assert!(lats.contains(&-60.0));
    }

    #[test]
    fn gnomonic_center_projects_to_center() {
        let proj = GnomonicProjection::new(0.0, 0.0, 1.0);
        
        // Center of tangent plane (0°, 0°) should project close to center
        // With resolution 1.0 arcmin/pixel, the center should be at (center_x, center_y)
        let delta = proj.resolution_arcmin * PI / (180.0 * 60.0);
        let center = 1248.0 / 2.0;  // Assuming 1248 pixel width
        
        if let Some((px, py)) = gnomonic_local_to_pixel_abs(0.0, 0.0, delta, center, center) {
            // At (0, 0), tangent plane coords are (0, 0), so pixel coords should be at center
            assert!((px - center).abs() < 10.0, "px should be near center {}, got {}", center, px);
            assert!((py - center).abs() < 10.0, "py should be near center {}, got {}", center, py);
        }
    }
}
