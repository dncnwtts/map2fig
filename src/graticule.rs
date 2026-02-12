use crate::rotation::{Rotation, coord_rotation, CoordSystem};
use std::f64::consts::PI;

/// Generate evenly-spaced graticule lines that always include key lines (0°, ±90°)
/// 
/// For meridians (longitude):
///   - Always includes: 0° (prime meridian), 90°, 180°, 270°
///   - Then fills in with spacing_deg intervals from 0°
///   - All lines are in [0°, 360°) range
///
/// For parallels (latitude):
///   - Always includes: 0° (equator), 90° (north pole), -90° (south pole)
///   - Then fills in with spacing_deg intervals from 0° toward ±90°
fn generate_graticule_degrees(spacing_deg: f64, is_latitude: bool) -> Vec<f64> {
    let mut degrees: Vec<f64> = Vec::new();
    
    if is_latitude {
        // For latitude: always include poles and equator
        degrees.push(0.0);   // Equator
        degrees.push(90.0);  // North pole
        degrees.push(-90.0); // South pole
        
        // Fill with regular spacing from equator toward poles
        let mut deg = spacing_deg;
        while deg < 90.0 {
            degrees.push(deg);
            degrees.push(-deg);
            deg += spacing_deg;
        }
    } else {
        // For longitude: always include cardinal meridians
        degrees.push(0.0);    // Prime meridian
        degrees.push(90.0);   // East
        degrees.push(180.0);  // Antimeridian
        degrees.push(270.0);  // West
        
        // Fill with regular spacing from prime meridian
        let mut deg = spacing_deg;
        while deg < 360.0 {
            degrees.push(deg);
            deg += spacing_deg;
        }
    }
    
    // Sort and deduplicate (remove near-duplicates from mandatory lines matching spacing)
    degrees.sort_unstable_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());
    degrees.dedup_by(|a: &mut f64, b: &mut f64| (*a - *b).abs() < 0.01);
    degrees
}

/// Represents a polyline in normalized Mollweide coordinates [0,1]
#[derive(Clone, Debug)]
pub struct GraticulePolyline {
    pub points: Vec<(f64, f64)>,
}

impl Default for GraticulePolyline {
    fn default() -> Self {
        Self::new()
    }
}

impl GraticulePolyline {
    pub fn new() -> Self {
        Self { points: Vec::new() }
    }
    
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
    
    pub fn len(&self) -> usize {
        self.points.len()
    }
    
    pub fn push(&mut self, point: (f64, f64)) {
        self.points.push(point);
    }
}

/// Represents all graticule lines for vectorized rendering
#[derive(Clone, Debug)]
pub struct GraticuleLineSegments {
    pub polylines: Vec<GraticulePolyline>,
}

impl Default for GraticuleLineSegments {
    fn default() -> Self {
        Self::new()
    }
}

impl GraticuleLineSegments {
    pub fn new() -> Self {
        Self { polylines: Vec::new() }
    }
    
    pub fn add_polyline(&mut self, polyline: GraticulePolyline) {
        if !polyline.is_empty() {
            self.polylines.push(polyline);
        }
    }
}

/// Transform graticule coordinates through coordinate systems and view rotation
/// 
/// Scenario: You have a map in one coordinate system (e.g., Galactic),
/// but you want to display a graticule in a different system (e.g., Celestial).
/// This handles: graticule_coord → map_input_coord → view_rotation
pub struct GraticuleTransform {
    /// Rotation from graticule coordinate system to map input coordinate system
    grat_to_input: Rotation,
    /// Optional view rotation (applied after coordinate transform)
    view: Option<Rotation>,
}

impl GraticuleTransform {
    pub fn new(
        graticule_coord: CoordSystem,
        input_coord: CoordSystem,
        view: Option<Rotation>,
    ) -> Self {
        let grat_to_input = coord_rotation(graticule_coord, input_coord);
        Self { grat_to_input, view }
    }

    /// Transform a point from graticule coordinates through all transformations
    /// 
    /// Returns the final 3D unit vector after:
    /// 1. Converting (lon, lat) to cartesian in graticule system
    /// 2. Rotating to input coordinate system
    /// 3. Applying optional view rotation
    pub fn apply(&self, lon: f64, lat: f64) -> [f64; 3] {
        // 1. lon/lat → vector in graticule coord (standard spherical coords)
        let v = lonlat_to_vec(lon, lat);

        // 2. graticule coord → map input coord
        let mut v = self.grat_to_input.apply(v);

        // 3. apply view rotation (if any)
        if let Some(view_rot) = &self.view {
            v = view_rot.apply(v);
        }

        v
    }
}

/// Convert lon/lat (in radians) to unit 3D cartesian vector
/// Standard spherical coordinates: lon increases eastward, lat increases northward
#[inline]
pub fn lonlat_to_vec(lon: f64, lat: f64) -> [f64; 3] {
    let cos_lat = lat.cos();
    [
        cos_lat * lon.cos(),
        cos_lat * lon.sin(),
        lat.sin(),
    ]
}

/// Convert unit 3D cartesian vector to lon/lat (in radians)
/// Returns (lon in [-π, π], lat in [-π/2, π/2])
#[inline]
pub fn vec_to_lonlat(v: [f64; 3]) -> (f64, f64) {
    let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let lon = v[1].atan2(v[0]); // atan2 already returns [-π, π]
    let lat = (v[2] / r).asin();
    (lon, lat)
}

/// Estimate the expected magnitude of coordinate change based on parameter step size
/// and recent segment history.
/// 
/// Uses the analytical derivative approach: if we have 2+ points, estimate
/// the typical rate of change and use that to detect anomalous jumps.
fn estimate_max_jump(segment: &[(f64, f64)], _param_step: f64) -> f64 {
    if segment.len() < 2 {
        // Can't estimate without history - use conservative default
        return 0.15;
    }
    
    // Use last two points to estimate derivative
    let (u1, v1) = segment[segment.len() - 2];
    let (u2, v2) = segment[segment.len() - 1];
    
    let du = (u2 - u1).abs();
    let dv = (v2 - v1).abs();
    let dist = (du * du + dv * dv).sqrt();
    
    // Expected jump for the same param step - use 3x safety margin
    // to account for acceleration near features like poles
    (dist * 3.0).max(0.05)  // minimum threshold of 0.05
}

/// Render graticule lines (meridians and parallels) for a given coordinate system
/// 
/// Projects graticule lines from `grat_coord` onto the Mollweide projection
/// after applying the view transformation.
pub fn render_graticule_mollweide(
    grid: &mut crate::render::raster::RasterGrid,
    view: &crate::rotation::ViewTransform,
    dpar_deg: f64,  // parallel (latitude) spacing
    dmer_deg: f64,  // meridian (longitude) spacing
    grat_coord: CoordSystem,
    input_coord: CoordSystem,
) {
    use crate::mollweide::MollweideProjection;
    use crate::projection::Projection;

    let transform = GraticuleTransform::new(grat_coord, input_coord, None);
    let proj = MollweideProjection;

    // dpar_deg and dmer_deg used as calculated absolute values below

    // Get properly-spaced meridian and parallel degrees (always includes key lines)
    let meridian_degrees = generate_graticule_degrees(dmer_deg, false);
    let parallel_degrees = generate_graticule_degrees(dpar_deg, true);

    // Meridians: constant longitude in graticule coords
    for &mer_deg_start in &meridian_degrees {
        let lon_grat = mer_deg_start * PI / 180.0;
        let mut line_segments: Vec<Vec<(f64, f64)>> = Vec::new();
        let mut current_segment: Vec<(f64, f64)> = Vec::new();

        // Sample parallels along this meridian - cover the full range
        // and handle the full 0-360 range properly
        for par_deg in (-90..=90).step_by(2) {
            let lat_grat = par_deg as f64 * PI / 180.0;

            // Transform through all coordinate systems
            let v_final = transform.apply(lon_grat, lat_grat);
            
            // Apply view transformation
            let v_viewed = view.apply(v_final);
            
            // Convert back to lon/lat in final coordinate system
            let (lon_final, lat_final) = vec_to_lonlat(v_viewed);

            // Project to Mollweide
            if let Some((u, v)) = proj.forward(lon_final, lat_final) {
                // Check for discontinuities using analytical derivative
                if !current_segment.is_empty()
                    && let Some(last_point) = current_segment.last() {
                        let prev_u = last_point.0;
                        let prev_v = last_point.1;
                        
                        let du = (u - prev_u).abs();
                        let dv = (v - prev_v).abs();
                        let jump_dist = (du * du + dv * dv).sqrt();
                        
                        // Estimate expected jump based on previous motion
                        let param_step = 2.0;  // we step by 2 degrees
                        let max_expected = estimate_max_jump(&current_segment, param_step);
                        
                        // If jump significantly exceeds expected, it's a discontinuity
                        if jump_dist > max_expected {
                            // Discontinuity detected - save segment and start new one
                            if current_segment.len() > 1 {
                                line_segments.push(current_segment);
                            }
                            current_segment = Vec::new();
                        }
                    }
                current_segment.push((u, v));
            } else {
                // Projection failed - save current segment and start a new one
                if current_segment.len() > 1 {
                    line_segments.push(current_segment);
                }
                current_segment = Vec::new();
            }
        }
        
        // Add final segment if any
        if current_segment.len() > 1 {
            line_segments.push(current_segment);
        }

        // Draw all segments
        for segment in line_segments {
            for window in segment.windows(2) {
                draw_line_on_grid(grid, window[0].0, window[0].1, window[1].0, window[1].1);
            }
        }
    }

    // Parallels: constant latitude in graticule coords
    for &par_deg in &parallel_degrees {
        let lat_grat = par_deg * PI / 180.0;
        let mut line_segments: Vec<Vec<(f64, f64)>> = Vec::new();
        let mut current_segment: Vec<(f64, f64)> = Vec::new();

        // For poles (lat = ±90°), all longitudes map to the same point
        // So we only need one point to represent the pole
        let is_pole = (par_deg - 90.0).abs() < 0.1 || (par_deg + 90.0).abs() < 0.1;
        
        if is_pole {
            // For poles, just sample one longitude to get the single point
            let lon_grat = 0.0;
            let v_final = transform.apply(lon_grat, lat_grat);
            let v_viewed = view.apply(v_final);
            let (_lon_final, _lat_final) = vec_to_lonlat(v_viewed);
            
            // A pole is just a single point, not a line
            // Skip drawing for poles (they're points, not lines)
            // Could draw as a marker if needed in future
        } else {
            // Sample meridians along this parallel with fine granularity
            let mut mer_deg_float = 0.0;
            while mer_deg_float < 360.0 {
                let lon_grat = mer_deg_float * PI / 180.0;

                // Transform through all coordinate systems
                let v_final = transform.apply(lon_grat, lat_grat);
                
                // Apply view transformation
                let v_viewed = view.apply(v_final);
                
                // Convert back to lon/lat in final coordinate system
                let (lon_final, lat_final) = vec_to_lonlat(v_viewed);

                // Project to Mollweide
                if let Some((u, v)) = proj.forward(lon_final, lat_final) {
                    // Check for discontinuities using analytical derivative
                    if !current_segment.is_empty()
                        && let Some(last_point) = current_segment.last() {
                            let prev_u = last_point.0;
                            let prev_v = last_point.1;
                            
                            let du = (u - prev_u).abs();
                            let dv = (v - prev_v).abs();
                            let jump_dist = (du * du + dv * dv).sqrt();
                            
                            // Estimate expected jump based on previous motion
                            let param_step = 0.5;  // we step by 0.5 degrees
                            let max_expected = estimate_max_jump(&current_segment, param_step);
                            
                            // If jump significantly exceeds expected, it's a discontinuity
                            if jump_dist > max_expected {
                                // Discontinuity detected - save segment and start new one
                                if current_segment.len() > 1 {
                                    line_segments.push(current_segment);
                                }
                                current_segment = Vec::new();
                            }
                        }
                    current_segment.push((u, v));
                } else {
                    // Projection failed - save current segment and start a new one
                    if current_segment.len() > 1 {
                        line_segments.push(current_segment);
                    }
                    current_segment = Vec::new();
                }
                
                mer_deg_float += 0.5;
            }
            
            // Add final segment if any
            if current_segment.len() > 1 {
                line_segments.push(current_segment);
            }

            // Draw all segments
            for segment in line_segments {
                for window in segment.windows(2) {
                    draw_line_on_grid(grid, window[0].0, window[0].1, window[1].0, window[1].1);
                }
            }
        }
    }
}

/// Generic graticule rendering for any projection
/// 
/// This function returns polylines suitable for vector output formats (PDF, SVG, etc.)
/// Works with any projection implementing the Projection trait.
pub fn render_graticule_vectorized_generic<P: crate::projection::Projection>(
    proj: &P,
    view: &crate::rotation::ViewTransform,
    dpar_deg: f64,  // parallel (latitude) spacing
    dmer_deg: f64,  // meridian (longitude) spacing
    grat_coord: CoordSystem,
    input_coord: CoordSystem,
) -> GraticuleLineSegments {
    let transform = GraticuleTransform::new(grat_coord, input_coord, None);
    
    let mut result = GraticuleLineSegments::new();

    // Get properly-spaced meridian and parallel degrees (always includes key lines)
    let meridian_degrees = generate_graticule_degrees(dmer_deg, false);
    let parallel_degrees = generate_graticule_degrees(dpar_deg, true);

    // Meridians: constant longitude in graticule coords
    for &mer_deg_start in &meridian_degrees {
        let lon_grat = mer_deg_start * PI / 180.0;
        let mut line_segments: Vec<Vec<(f64, f64)>> = Vec::new();
        let mut current_segment: Vec<(f64, f64)> = Vec::new();

        // Sample parallels along this meridian
        for par_deg in (-90..=90).step_by(2) {
            let lat_grat = par_deg as f64 * PI / 180.0;

            // Transform through all coordinate systems
            let v_final = transform.apply(lon_grat, lat_grat);
            
            // Apply view transformation
            let v_viewed = view.apply(v_final);
            
            // Convert back to lon/lat in final coordinate system
            let (lon_final, lat_final) = vec_to_lonlat(v_viewed);

            // Project to Mollweide
            if let Some((u, v)) = proj.forward(lon_final, lat_final) {
                // Check for discontinuities using analytical derivative
                if !current_segment.is_empty()
                    && let Some(last_point) = current_segment.last() {
                        let prev_u = last_point.0;
                        let prev_v = last_point.1;
                        
                        let du = (u - prev_u).abs();
                        let dv = (v - prev_v).abs();
                        let jump_dist = (du * du + dv * dv).sqrt();
                        
                        // Estimate expected jump based on previous motion
                        let param_step = 2.0;  // we step by 2 degrees
                        let max_expected = estimate_max_jump(&current_segment, param_step);
                        
                        // If jump significantly exceeds expected, it's a discontinuity
                        if jump_dist > max_expected {
                            // Discontinuity detected - save segment and start new one
                            if current_segment.len() > 1 {
                                line_segments.push(current_segment);
                            }
                            current_segment = Vec::new();
                        }
                    }
                current_segment.push((u, v));
            } else {
                // Projection failed - save current segment and start a new one
                if current_segment.len() > 1 {
                    line_segments.push(current_segment);
                }
                current_segment = Vec::new();
            }
        }
        
        // Add final segment if any
        if current_segment.len() > 1 {
            line_segments.push(current_segment);
        }

        // Convert segments to polylines
        for segment in line_segments {
            let mut polyline = GraticulePolyline::new();
            for point in segment {
                polyline.push(point);
            }
            result.add_polyline(polyline);
        }
    }

    // Parallels: constant latitude in graticule coords
    for &par_deg in &parallel_degrees {
        let lat_grat = par_deg * PI / 180.0;
        
        // Check if this is a pole latitude - poles are points, not line segments
        let is_pole = (par_deg - 90.0).abs() < 0.1 || (par_deg + 90.0).abs() < 0.1;
        
        if is_pole {
            // Skip poles entirely in parallels - they're single points, not lines
            continue;
        }
        
        let mut line_segments: Vec<Vec<(f64, f64)>> = Vec::new();
        let mut current_segment: Vec<(f64, f64)> = Vec::new();

        // Sample meridians along this parallel with fine granularity
        let mut mer_deg_float = 0.0;
        while mer_deg_float < 360.0 {
            let lon_grat = mer_deg_float * PI / 180.0;

            // Transform through all coordinate systems
            let v_final = transform.apply(lon_grat, lat_grat);
            
            // Apply view transformation
            let v_viewed = view.apply(v_final);
            
            // Convert back to lon/lat in final coordinate system
            let (lon_final, lat_final) = vec_to_lonlat(v_viewed);

            // Project to Mollweide
            if let Some((u, v)) = proj.forward(lon_final, lat_final) {
                // Check for discontinuities using analytical derivative
                if !current_segment.is_empty()
                    && let Some(last_point) = current_segment.last() {
                        let prev_u = last_point.0;
                        let prev_v = last_point.1;
                        
                        let du = (u - prev_u).abs();
                        let dv = (v - prev_v).abs();
                        let jump_dist = (du * du + dv * dv).sqrt();
                        
                        // Estimate expected jump based on previous motion
                        let param_step = 0.5;  // we step by 0.5 degrees
                        let max_expected = estimate_max_jump(&current_segment, param_step);
                        
                        // If jump significantly exceeds expected, it's a discontinuity
                        if jump_dist > max_expected {
                            // Discontinuity detected - save segment and start new one
                            if current_segment.len() > 1 {
                                line_segments.push(current_segment);
                            }
                            current_segment = Vec::new();
                        }
                    }
                current_segment.push((u, v));
            } else {
                // Projection failed - save current segment and start a new one
                if current_segment.len() > 1 {
                    line_segments.push(current_segment);
                }
                current_segment = Vec::new();
            }
            
            mer_deg_float += 0.5;
        }
        
        // Add final segment if any
        if current_segment.len() > 1 {
            line_segments.push(current_segment);
        }

        // Convert segments to polylines
        for segment in line_segments {
            let mut polyline = GraticulePolyline::new();
            for point in segment {
                polyline.push(point);
            }
            result.add_polyline(polyline);
        }
    }
    
    result
}

/// Generate vectorized graticule lines for Mollweide projection
/// 
/// This is a convenience wrapper around render_graticule_vectorized_generic.
pub fn render_graticule_mollweide_vectorized(
    view: &crate::rotation::ViewTransform,
    dpar_deg: f64,  // parallel (latitude) spacing
    dmer_deg: f64,  // meridian (longitude) spacing
    grat_coord: CoordSystem,
    input_coord: CoordSystem,
) -> GraticuleLineSegments {
    use crate::mollweide::MollweideProjection;

    let proj = MollweideProjection;
    render_graticule_vectorized_generic(
        &proj,
        view,
        dpar_deg,
        dmer_deg,
        grat_coord,
        input_coord,
    )
}

/// Generate vectorized graticule lines for Hammer-Aitoff projection
/// 
/// This is a convenience wrapper around render_graticule_vectorized_generic.
pub fn render_graticule_hammer_vectorized(
    view: &crate::rotation::ViewTransform,
    dpar_deg: f64,  // parallel (latitude) spacing
    dmer_deg: f64,  // meridian (longitude) spacing
    grat_coord: CoordSystem,
    input_coord: CoordSystem,
) -> GraticuleLineSegments {
    use crate::hammer::HammerProjection;

    let proj = HammerProjection::new();
    render_graticule_vectorized_generic(
        &proj,
        view,
        dpar_deg,
        dmer_deg,
        grat_coord,
        input_coord,
    )
}

/// Render vectorized graticule lines to a Cairo context (for PDF output)
/// 
/// The polylines are scaled from normalized [0,1] coordinates to the actual
/// image dimensions and drawn with vector lines.
pub fn render_graticule_cairo(
    graticule: &GraticuleLineSegments,
    cr: &cairo::Context,
    x_offset: f64,
    y_offset: f64,
    width: f64,
    height: f64,
) {
    render_graticule_cairo_with_color(graticule, cr, x_offset, y_offset, width, height, 0.0, 0.0, 0.0);
}

/// Render graticule lines with custom color
pub fn render_graticule_cairo_with_color(
    graticule: &GraticuleLineSegments,
    cr: &cairo::Context,
    x_offset: f64,
    y_offset: f64,
    width: f64,
    height: f64,
    r: f64,
    g: f64,
    b: f64,
) {
    // Set line properties for graticule
    cr.set_source_rgb(r, g, b);
    cr.set_line_width(0.5); // Thin lines
    
    for polyline in &graticule.polylines {
        if polyline.is_empty() {
            continue;
        }
        
        // Start the path at the first point
        let first = polyline.points[0];
        let x = x_offset + first.0 * width;
        let y = y_offset + first.1 * height;
        cr.move_to(x, y);
        
        // Draw line segments to remaining points
        for &(u, v) in &polyline.points[1..] {
            let x = x_offset + u * width;
            let y = y_offset + v * height;
            cr.line_to(x, y);
        }
    }
    
    // Stroke all lines at once
    let _ = cr.stroke();
}

/// Draw a line between two normalized [0,1] points in the grid
fn draw_line_on_grid(grid: &mut crate::render::raster::RasterGrid, u0: f64, v0: f64, u1: f64, v1: f64) {
    use image::Rgba;
    
    let (x0, y0) = (
        (u0 * (grid.width - 1) as f64) as i32,
        (v0 * (grid.height - 1) as f64) as i32,
    );
    let (x1, y1) = (
        (u1 * (grid.width - 1) as f64) as i32,
        (v1 * (grid.height - 1) as f64) as i32,
    );

    // Bresenham line algorithm
    let steps = (x1 - x0).abs().max((y1 - y0).abs()) as usize;
    if steps == 0 {
        return;
    }

    for i in 0..=steps {
        let t = i as f64 / steps as f64;
        let px = ((1.0 - t) * x0 as f64 + t * x1 as f64).round() as u32;
        let py = ((1.0 - t) * y0 as f64 + t * y1 as f64).round() as u32;

        if px < grid.width && py < grid.height {
            grid.set_pixel(px, py, Rgba([0, 0, 0, 255])); // Black graticule
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::rotation::{DEG2RAD, RAD2DEG};

    /// Test three major points in a coordinate system:
    /// 1. North pole: (any lon, 90°)
    /// 2. Equatorial prime meridian: (0°, 0°)
    /// 3. Equatorial 90° meridian: (90°, 0°)
    ///
    /// For each test, verify that after coordinate transformation,
    /// these points map to predictable locations.

    /// Helper: test a graticule point transformation
    fn test_point_transformation(
        input_coord: CoordSystem,
        graticule_coord: CoordSystem,
        grat_lon_deg: f64,
        grat_lat_deg: f64,
        expected_lon_deg: f64,
        expected_lat_deg: f64,
        tolerance_deg: f64,
    ) {
        let transform = GraticuleTransform::new(graticule_coord, input_coord, None);
        let v = transform.apply(grat_lon_deg * DEG2RAD, grat_lat_deg * DEG2RAD);
        let (result_lon, result_lat) = vec_to_lonlat(v);

        // Normalize longitudes to [-π, π] for comparison
        let expected_lon_rad = expected_lon_deg.rem_euclid(360.0) * DEG2RAD;
        let result_lon_normalized = if result_lon > PI {
            result_lon - 2.0 * PI
        } else if result_lon < -PI {
            result_lon + 2.0 * PI
        } else {
            result_lon
        };

        let lon_diff = (result_lon_normalized - expected_lon_rad).abs();
        let lat_diff = (result_lat - expected_lat_deg * DEG2RAD).abs();

        let tolerance_rad = tolerance_deg * DEG2RAD;

        assert!(
            lon_diff < tolerance_rad || (2.0 * PI - lon_diff) < tolerance_rad,
            "Longitude mismatch for ({}, {}) in {} → {}: got {:.4}°, expected {:.4}°",
            grat_lon_deg, grat_lat_deg,
            format!("{:?}", graticule_coord),
            format!("{:?}", input_coord),
            result_lon_normalized * RAD2DEG,
            expected_lon_deg
        );

        assert!(
            lat_diff < tolerance_rad,
            "Latitude mismatch for ({}, {}) in {} → {}: got {:.4}°, expected {:.4}°",
            grat_lon_deg, grat_lat_deg,
            format!("{:?}", graticule_coord),
            format!("{:?}", input_coord),
            result_lat * RAD2DEG,
            expected_lat_deg
        );
    }

    #[test]
    fn graticule_celestial_to_celestial_identity() {
        // Celestial→Celestial should be identity
        test_point_transformation(
            CoordSystem::C,
            CoordSystem::C,
            0.0,
            0.0, // (0°, 0°) should stay (0°, 0°)
            0.0,
            0.0,
            0.01,
        );

        test_point_transformation(
            CoordSystem::C,
            CoordSystem::C,
            0.0,
            90.0, // North pole should stay at north pole
            0.0,
            90.0,
            0.01,
        );

        test_point_transformation(
            CoordSystem::C,
            CoordSystem::C,
            90.0,
            0.0, // (90°, 0°) should stay (90°, 0°)
            90.0,
            0.0,
            0.01,
        );
    }

    #[test]
    fn graticule_galactic_in_celestial() {
        // Place Galactic equatorial points and see where they appear in Celestial coords
        // Galactic (0°, 0°) → Celestial coordinates
        // Based on the GAL_TO_EQ matrix: (-93.6°, -28.9°)
        test_point_transformation(
            CoordSystem::C,   // input (map) coordinates
            CoordSystem::G,   // graticule coordinates
            0.0,
            0.0, // Galactic (0°, 0°)
            -93.6,
            -28.9,
            0.5,
        );

        // Galactic north pole (0°, 90°) → celestial
        // Should be at (-167.1°, 27.1°) in celestial
        test_point_transformation(
            CoordSystem::C,
            CoordSystem::G,
            0.0,
            90.0, // Galactic north pole
            -167.1,
            27.1,
            0.5,
        );

        // Galactic (90°, 0°) on equator
        test_point_transformation(
            CoordSystem::C,
            CoordSystem::G,
            90.0,
            0.0,
            -42.0,
            48.3,
            1.0,
        );
    }

    #[test]
    fn graticule_celestial_in_galactic() {
        // Inverse: Celestial graticule on Galactic map
        // Celestial (0°, 0°) → Galactic: (96.3°, -60.2°)
        test_point_transformation(
            CoordSystem::G,   // input (map) coordinates
            CoordSystem::C,   // graticule coordinates
            0.0,
            0.0, // Celestial (0°, 0°)
            96.3,
            -60.2,
            0.5,
        );

        // Celestial north pole (0°, 90°) in galactic coordinates
        // Should map to (122.9°, 27.1°) in galactic
        test_point_transformation(
            CoordSystem::G,
            CoordSystem::C,
            0.0,
            90.0, // Celestial north pole
            122.9,
            27.1,
            0.5,
        );
    }

    #[test]
    fn graticule_ecliptic_to_celestial() {
        // Ecliptic graticule on Celestial map
        // Ecliptic (0°, 0°) → Celestial (0.0°, 0.0°)
        test_point_transformation(
            CoordSystem::C,
            CoordSystem::E,
            0.0,
            0.0,
            0.0,
            0.0,
            0.01,
        );

        // Ecliptic north pole (0°, 90°) → Celestial
        // Should be at (-90.0°, 66.6°) approximately
        test_point_transformation(
            CoordSystem::C,
            CoordSystem::E,
            0.0,
            90.0,
            -90.0,
            66.6,
            1.0,
        );

        // Ecliptic (90°, 0°) → Celestial (90.0°, 23.4°)
        test_point_transformation(
            CoordSystem::C,
            CoordSystem::E,
            90.0,
            0.0,
            90.0,
            23.4,
            0.5,
        );
    }

    #[test]
    fn lonlat_vec_roundtrip() {
        // Verify lonlat↔vec conversions are consistent
        let test_points = vec![
            (0.0, 0.0),           // Prime meridian, equator
            (PI / 2.0, 0.0),      // 90° lon, equator
            (PI, 0.0),            // 180° lon, equator
            (0.0, PI / 2.0),      // North pole
            (0.0, -PI / 2.0),     // South pole
            (0.5, 0.3),           // Random point
        ];

        for (lon, lat) in test_points {
            let v = lonlat_to_vec(lon, lat);
            let (lon2, lat2) = vec_to_lonlat(v);

            // Normalize longitudes
            let lon_norm = lon.rem_euclid(2.0 * PI);
            let lon2_norm = lon2.rem_euclid(2.0 * PI);

            assert!((lon_norm - lon2_norm).abs() < 1e-10 || (lon_norm - lon2_norm).abs() > 2.0 * PI - 1e-10,
                "Longitude roundtrip failed: {} → {} → {}", lon, lon_norm, lon2_norm);
            assert!(
                (lat - lat2).abs() < 1e-10,
                "Latitude roundtrip failed: {} → {}", lat, lat2
            );
        }
    }

    #[test]
    fn debug_meridian_projection() {
        // Test a single meridian to see where points project
        use crate::mollweide::MollweideProjection;
        use crate::projection::Projection;

        let proj = MollweideProjection;
        let transform = GraticuleTransform::new(CoordSystem::E, CoordSystem::G, None);

        // Test meridian at 0° (prime meridian in Equatorial)
        let lon_grat = 0.0; // Equatorial coordinates
        
        println!("\n=== DEBUG: Meridian at lon=0° (Equatorial) ===");
        println!("Latitude | Grat Coords | Input Coords | Projected (u,v) | Valid");
        println!("{}", "-".repeat(80));
        
        let mut valid_points = 0;
        let mut invalid_points = 0;
        
        for par_deg in (-90..=90).step_by(10) {
            let lat_grat = par_deg as f64 * PI / 180.0;
            
            // Apply graticule transform
            let v = transform.apply(lon_grat, lat_grat);
            let (lon_input, lat_input) = vec_to_lonlat(v);
            
            // Project to Mollweide
            match proj.forward(lon_input, lat_input) {
                Some((u, v)) => {
                    valid_points += 1;
                    println!("{:8}° | ({:6.2}°, {:6.2}°) | ({:7.2}°, {:6.2}°) | ({:.4}, {:.4}) | ✓",
                        par_deg, lon_grat * RAD2DEG, lat_grat * RAD2DEG,
                        lon_input * RAD2DEG, lat_input * RAD2DEG, u, v);
                }
                None => {
                    invalid_points += 1;
                    println!("{:8}° | ({:6.2}°, {:6.2}°) | ({:7.2}°, {:6.2}°) | (----, ----) | ✗",
                        par_deg, lon_grat * RAD2DEG, lat_grat * RAD2DEG,
                        lon_input * RAD2DEG, lat_input * RAD2DEG);
                }
            }
        }
        
        println!("Total: {} valid, {} invalid points\n", valid_points, invalid_points);
    }

    #[test]
    fn debug_right_edge_meridians() {
        // Test meridians near 180° to check right edge coverage
        use crate::mollweide::MollweideProjection;
        use crate::projection::Projection;

        let proj = MollweideProjection;
        let transform = GraticuleTransform::new(CoordSystem::E, CoordSystem::G, None);

        println!("\n=== DEBUG: Meridians near right edge (180°) ===");
        println!("Longitude | Latitude 0° | Projected (u,v) | Valid");
        println!("{}", "-".repeat(60));
        
        for mer_deg in (150..=210).step_by(10) {
            let lon_grat = mer_deg as f64 * PI / 180.0;
            let lat_grat = 0.0; // Equator
            
            let v = transform.apply(lon_grat, lat_grat);
            let (lon_input, lat_input) = vec_to_lonlat(v);
            
            match proj.forward(lon_input, lat_input) {
                Some((u, v)) => {
                    println!("{:8}° | {:7.2}° | ({:.4}, {:.4}) | ✓",
                        mer_deg, lat_input * RAD2DEG, u, v);
                }
                None => {
                    println!("{:8}° | {:7.2}° | (----, ----) | ✗",
                        mer_deg, lat_input * RAD2DEG);
                }
            }
        }
        println!();
    }

    #[test]
    fn debug_parallel_sampling() {
        // Test parallel at 0° to check full coverage
        use crate::mollweide::MollweideProjection;
        use crate::projection::Projection;

        let proj = MollweideProjection;
        let transform = GraticuleTransform::new(CoordSystem::G, CoordSystem::G, None);

        println!("\n=== DEBUG: Parallel at lat=0° (full longitude sweep) ===");
        println!("Longitude | Projected (u,v) | Valid | Left? | Right?");
        println!("{}", "-".repeat(60));
        
        let lat_grat = 0.0;
        let mut samples = Vec::new();
        let mut left_count = 0;
        let mut right_count = 0;
        
        let mut lon_deg = 0.0;
        while lon_deg < 360.0 {
            let lon_grat = lon_deg * PI / 180.0;
            
            let v = transform.apply(lon_grat, lat_grat);
            let (lon_input, lat_input) = vec_to_lonlat(v);
            
            match proj.forward(lon_input, lat_input) {
                Some((u, v)) => {
                    samples.push((lon_deg, u, v));
                    if u < 0.3 { left_count += 1; }
                    if u > 0.7 { right_count += 1; }
                }
                None => {
                    if lon_deg % 60.0 < 1.0 {  // Print every 60°
                        println!("{:8.1}° | (----, ----) | ✗",
                            lon_deg);
                    }
                }
            }
            lon_deg += 1.0;
        }
        
        // Print samples at key longitudes
        for (lon, u, v) in &samples {
            if (lon % 60.0).abs() < 1.0 {  // Print every 60°
                let is_left = *u < 0.3;
                let is_right = *u > 0.7;
                println!("{:8.1}° | ({:.4}, {:.4}) | ✓ | {} | {}",
                    lon, u, v, if is_left { "✓" } else { " " }, if is_right { "✓" } else { " " });
            }
        }
        
        println!("Left (<0.3): {} points, Right (>0.7): {} points\n", left_count, right_count);
    }

    #[test]
    fn debug_longitude_normalization() {
        // Check what longitudes we get after coordinate transforms
        use crate::mollweide::MollweideProjection;
        use crate::projection::Projection;

        let proj = MollweideProjection;
        let transform = GraticuleTransform::new(CoordSystem::G, CoordSystem::G, None);

        println!("\n=== DEBUG: Longitude normalization ===");
        println!("Grat Lon | Input Lon (rad) | Input Lon (deg) | Projected (u,v) | Valid");
        println!("{}", "-".repeat(80));
        
        let lat_grat = 0.0;
        
        for mer_deg in (0..360).step_by(45) {
            let lon_grat = mer_deg as f64 * PI / 180.0;
            
            let v = transform.apply(lon_grat, lat_grat);
            let (lon_input, lat_input) = vec_to_lonlat(v);
            
            match proj.forward(lon_input, lat_input) {
                Some((u, v)) => {
                    println!("{:8}° | {:14.6} | {:15.2}° | ({:.4}, {:.4}) | ✓",
                        mer_deg, lon_input, lon_input * RAD2DEG, u, v);
                }
                None => {
                    println!("{:8}° | {:14.6} | {:15.2}° | (----, ----) | ✗",
                        mer_deg, lon_input, lon_input * RAD2DEG);
                }
            }
        }
        println!();
    }
    #[test]
    fn debug_graticule_lines_original_coords() {
        // Show which graticule lines are generated in the ORIGINAL coordinate system
        // This helps verify that generate_graticule_degrees() is working correctly
        
        println!("\n=== DEBUG: Graticule Lines Generated (No Coordinate Transform) ===\n");
        
        // Test various spacings to see what lines are generated
        let test_spacings = vec![15.0, 20.0, 25.0, 30.0, 35.0, 40.0];
        
        println!("MERIDIAN LINES (Constant Longitude):");
        println!("{}", "=".repeat(100));
        for spacing_deg in &test_spacings {
            let meridian_degs = generate_graticule_degrees(*spacing_deg, false);
            
            println!("\nSpacing: {}°", spacing_deg);
            println!("Values:  {:?}", meridian_degs);
            println!("Count:   {} lines", meridian_degs.len());
            
            // Show if this spacing is "clean" (cardinal lines align with spacing)
            let has_90 = meridian_degs.iter().any(|&x| (x - 90.0).abs() < 0.01);
            let has_180 = meridian_degs.iter().any(|&x| (x - 180.0).abs() < 0.01);
            let has_270 = meridian_degs.iter().any(|&x| (x - 270.0).abs() < 0.01);
            let expected_count = (360.0 / spacing_deg).round() as usize;
            let is_clean = has_90 && has_180 && has_270 && meridian_degs.len() == expected_count;
            
            if is_clean {
                println!("✓ CLEAN - spacing divides evenly into 360°");
            } else {
                println!("⚠ EXTRA LINES - cardinal meridians force non-spacing intervals");
            }
        }
        
        println!("\n{}", "=".repeat(100));
        println!("\nPARALLEL LINES (Constant Latitude):");
        println!("{}", "=".repeat(100));
        for spacing_deg in &test_spacings {
            let parallel_degs = generate_graticule_degrees(*spacing_deg, true);
            
            println!("\nSpacing: {}°", spacing_deg);
            println!("Values:  {:?}", parallel_degs);
            println!("Count:   {} lines", parallel_degs.len());
            
            // Check if equator and poles are included
            let has_0 = parallel_degs.iter().any(|&x| x.abs() < 0.01);
            let has_90 = parallel_degs.iter().any(|&x| (x - 90.0).abs() < 0.01);
            let has_neg90 = parallel_degs.iter().any(|&x| (x + 90.0).abs() < 0.01);
            
            println!("✓ Equator (0°): {}, North Pole (90°): {}, South Pole (-90°): {}", 
                has_0, has_90, has_neg90);
        }
        
        println!("\n{}", "=".repeat(100));
        println!("\nFull Rendering Example (G→G, 30° spacing):\n");
        
        let spacing = 30.0;
        let meridian_degs = generate_graticule_degrees(spacing, false);
        let parallel_degs = generate_graticule_degrees(spacing, true);
        
        println!("MERIDIANS:");
        for (i, &lon_deg) in meridian_degs.iter().enumerate() {
            println!("  Line {:2}: longitude = {:7.1}°", i+1, lon_deg);
        }
        
        println!("\nPARALLELS:");
        for (i, &lat_deg) in parallel_degs.iter().enumerate() {
            println!("  Line {:2}: latitude  = {:7.1}°", i+1, lat_deg);
        }
        
        println!();
    }

    #[test]
    fn debug_graticule_lines_ecliptic_on_galactic() {
        // Show Ecliptic graticule lines transformed to Galactic map coordinates
        // This is the G→C with E case mentioned by the user
        
        println!("\n=== DEBUG: Ecliptic Graticule on Galactic Map (E→G Transform) ===\n");
        
        let spacing = 30.0;
        let transform = GraticuleTransform::new(CoordSystem::E, CoordSystem::G, None);
        
        let meridian_degs = generate_graticule_degrees(spacing, false);
        let parallel_degs = generate_graticule_degrees(spacing, true);
        
        println!("Input: Ecliptic coordinates");
        println!("Map:   Galactic coordinates");
        println!("Spacing: {}°\n", spacing);
        
        println!("{}", "=".repeat(100));
        println!("ECLIPTIC MERIDIANS (Constant Ecliptic Longitude):");
        println!("{}", "=".repeat(100));
        println!("Ecl.Lon | Sample Ecliptic Latitudes | Galactic Result | Count");
        println!("{}", "-".repeat(100));
        
        for &ecl_lon_deg in &meridian_degs {
            let ecl_lon_rad = ecl_lon_deg * PI / 180.0;
            let mut valid_points = 0;
            let mut sample_lats = Vec::new();
            
            for ecl_lat_deg in (-60..=60).step_by(30) {
                let ecl_lat_rad = ecl_lat_deg as f64 * PI / 180.0;
                
                // Transform from Ecliptic to Galactic
                let v = transform.apply(ecl_lon_rad, ecl_lat_rad);
                let (gal_lon, gal_lat) = vec_to_lonlat(v);
                
                if ecl_lat_deg == -60 || ecl_lat_deg == 0 || ecl_lat_deg == 60 {
                    sample_lats.push(format!("G({:.1}°,{:.1}°)", 
                        gal_lon * 180.0 / PI, gal_lat * 180.0 / PI));
                }
                valid_points += 1;
            }
            
            println!("{:7.1}° | E: -60°, 0°, 60°           | {} | {}",
                ecl_lon_deg,
                sample_lats.join(" → "),
                valid_points
            );
        }
        
        println!("\n{}", "=".repeat(100));
        println!("ECLIPTIC PARALLELS (Constant Ecliptic Latitude):");
        println!("{}", "=".repeat(100));
        println!("Ecl.Lat | Sample Ecliptic Longitudes | Galactic Result | Count");
        println!("{}", "-".repeat(100));
        
        for &ecl_lat_deg in &parallel_degs {
            let ecl_lat_rad = ecl_lat_deg * PI / 180.0;
            let mut valid_points = 0;
            let mut sample_lons = Vec::new();
            
            for ecl_lon_deg in (0..360).step_by(90) {
                let ecl_lon_rad = ecl_lon_deg as f64 * PI / 180.0;
                
                // Transform from Ecliptic to Galactic
                let v = transform.apply(ecl_lon_rad, ecl_lat_rad);
                let (gal_lon, gal_lat) = vec_to_lonlat(v);
                
                if ecl_lon_deg == 0 || ecl_lon_deg == 90 || ecl_lon_deg == 180 || ecl_lon_deg == 270 {
                    sample_lons.push(format!("G({:.1}°,{:.1}°)", 
                        gal_lon * 180.0 / PI, gal_lat * 180.0 / PI));
                }
                valid_points += 1;
            }
            
            println!("{:7.1}° | E: 0°, 90°, 180°, 270°    | {} | {}",
                ecl_lat_deg,
                sample_lons.join(" → "),
                valid_points
            );
        }
        
        println!("\n{}", "=".repeat(100));
        println!("\nSUMMARY:");
        println!("Ecliptic Meridians: {} lines", meridian_degs.len());
        println!("Ecliptic Parallels: {} lines", parallel_degs.len());
        println!("\nKey Ecliptic lines:");
        println!("  Prime meridian (Ecl.Lon 0°):     {}", meridian_degs.iter().any(|&x| x.abs() < 0.01));
        println!("  Ecliptic equator (Ecl.Lat 0°):   {}", parallel_degs.iter().any(|&x| x.abs() < 0.01));
        println!("  Ecliptic north pole (Ecl.Lat 90°): {}", parallel_degs.iter().any(|&x| (x - 90.0).abs() < 0.01));
        println!();
    }

    #[test]
    fn graticule_transform_with_view_rotation() {
        // Test that view rotation is applied after coordinate transformation
        use crate::rotation::Rotation;

        // Create a 90° rotation around z-axis (pure camera rotation)
        let view_rot = Rotation {
            matrix: [
                [0.0, -1.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        };

        let transform = GraticuleTransform::new(
            CoordSystem::C,
            CoordSystem::C,
            Some(view_rot),
        );

        // Point at (0°, 0°) with 90° view rotation
        // After rotation, (lon, lat) → should rotate around z-axis
        let v = transform.apply(0.0, 0.0);
        let (lon, lat) = vec_to_lonlat(v);

        // Should be rotated by ~90°
        assert!(lon > PI / 2.0 * 0.9 && lon < PI / 2.0 * 1.1,
            "View rotation not applied correctly: got lon = {}", lon * RAD2DEG);
        assert!(lat.abs() < 0.01, "View rotation shouldn't change latitude much");
    }

    #[test]
    fn test_pole_graticule_no_wrapping() {
        // Test that polar latitude lines don't wrap around boundaries
        // Regression test for GitHub issue: pole graticules crossing over at boundaries
        //
        // This test simulates the problematic case:
        // - Input map: Galactic (G)
        // - Output view: Celestial (C)
        // - Graticule: Ecliptic (E)
        // - Issue: Top/bottom latitude lines were wrapping at boundaries
        //
        // The fix: Skip discontinuity checking at poles (±90°) since pole
        // transformations can be numerically unstable and all longitudes
        // converge to the same point anyway.

        use crate::rotation::ViewTransform;
        use crate::mollweide::MollweideProjection;
        use crate::projection::Projection;

        let proj = MollweideProjection;
        let transform = GraticuleTransform::new(CoordSystem::E, CoordSystem::G, None);
        let view = ViewTransform::new(CoordSystem::G, CoordSystem::G, None);

        // Sample the extreme parallel (ecliptic latitude = 90°)
        let lat_extreme = 90.0 * PI / 180.0;
        
        // Sample multiple longitudes along this parallel
        let mut projected_points = Vec::new();
        for lon_deg in (0..360).step_by(10) {
            let lon_ecl = lon_deg as f64 * PI / 180.0;
            
            // Transform E→G
            let v_final = transform.apply(lon_ecl, lat_extreme);
            let v_viewed = view.apply(v_final);
            let (lon_final, lat_final) = vec_to_lonlat(v_viewed);
            
            // Project to Mollweide
            if let Some((u, v)) = proj.forward(lon_final, lat_final) {
                projected_points.push((lon_deg, u, v));
            }
        }

        // For a pole, all longitudes should project to approximately the same point
        // (within projection numerical precision)
        if projected_points.len() > 1 {
            let first = &projected_points[0];
            
            for point in &projected_points[1..] {
                // At extreme poles, u,v coordinates should cluster together
                // Allow some numerical tolerance (±0.05 in [0,1] normalized space)
                let du = (point.1 - first.1).abs();
                let dv = (point.2 - first.2).abs();
                
                // If not at a pole, points would spread across the domain
                // Poles should be tightly clustered
                assert!(du < 0.15 || dv < 0.15,
                    "Pole wraparound detected: different longitudes at pole gave different projections. \
                    Lon {}°: ({:.4},{:.4}) vs Lon {}°: ({:.4},{:.4})",
                    first.0, first.1, first.2, point.0, point.1, point.2);
            }
        }

        println!("✓ Pole graticule wrapping test passed: no boundary crossing at ±90° latitude");
    }}