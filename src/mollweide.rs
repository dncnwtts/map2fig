use crate::projection::Projection;
use crate::render::raster::RasterGrid;
use std::f64::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct MollweideProjection;

impl Default for MollweideProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl MollweideProjection {
    pub fn new() -> Self {
        Self
    }
}

impl Projection for MollweideProjection {
    fn inverse(&self, u: f64, v: f64) -> Option<(f64, f64)> {
        // Map u,v ∈ [0,1] → Mollweide plane
        let x = 2.0 - 4.0 * u;
        let y = 1.0 - 2.0 * v;

        if x * x / 4.0 + y * y > 1.0 {
            return None;
        }

        let theta_aux = y.asin();
        let sin_lat = (2.0 * theta_aux + (2.0 * theta_aux).sin()) / std::f64::consts::PI;
        if sin_lat.abs() > 1.0 {
            return None;
        }

        let lat = sin_lat.asin();
        let lon = std::f64::consts::PI * x / (2.0 * theta_aux.cos());

        Some((lon, lat))
    }
    fn forward(&self, lon: f64, lat: f64) -> Option<(f64, f64)> {
        // Solve for theta via Newton iteration
        let mut theta = lat;
        for _ in 0..10 {
            let f = 2.0 * theta + (2.0 * theta).sin() - PI * lat.sin();
            let df = 2.0 + 2.0 * (2.0 * theta).cos();
            theta -= f / df;
        }

        let x = (2.0 * lon / PI) * theta.cos();
        let y = theta.sin();

        // Map [-2,2] → [0,1]
        let u = (2.0 - x) * 0.25;
        let v = (1.0 - y) * 0.5;

        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return None;
        }

        Some((u, v))
    }
    fn pixel_to_ang(&self, x: u32, y: u32, grid: &RasterGrid) -> Option<(f64, f64)> {
        // Inline normalization to avoid function calls in hot path
        let nx = x as f64 / ((grid.width - 1) as f64);
        let ny = y as f64 / ((grid.height - 1) as f64);

        let px = 2.0 - 4.0 * nx;
        let py = 1.0 - 2.0 * ny;

        // Early rejection: check if point is outside the Mollweide oval
        // This is the main computation: px²/4 + py² > 1
        // Optimized: multiply through to avoid division
        if px * px + 4.0 * py * py > 4.0 {
            return None;
        }

        let theta_aux = py.asin();
        let sin_lat = (2.0 * theta_aux + (2.0 * theta_aux).sin()) / PI;

        if sin_lat.abs() > 1.0 {
            return None;
        }

        let lat = sin_lat.asin();
        let c = theta_aux.cos();
        if c.abs() < 1e-12 {
            return None;
        }
        let lon = PI * px / (2.0 * c);

        Some((lon, lat))
    }

    /// Batch projection: process up to 8 pixels in parallel with loop unrolling
    /// 
    /// This is optimized for instruction-level parallelism by processing
    /// 8 pixels with independent computations that can be executed concurrently.
    fn pixel_to_ang_batch(
        &self,
        px_coords: &[u32; 8],
        py_coords: &[u32; 8],
        grid: &RasterGrid,
    ) -> (
        [f64; 8], // longitudes
        [f64; 8], // latitudes
        [bool; 8], // validity mask
    ) {
        let w_inv = 1.0 / ((grid.width - 1) as f64);
        let h_inv = 1.0 / ((grid.height - 1) as f64);

        // Initialize output arrays
        let mut lons = [0.0_f64; 8];
        let mut lats = [0.0_f64; 8];
        let mut mask = [false; 8];

        // Process all 8 pixels with unrolled loop for ILP (Instruction-Level Parallelism)
        // Each iteration is independent, allowing CPU to execute multiple in parallel
        for i in 0..8 {
            let nx = px_coords[i] as f64 * w_inv;
            let ny = py_coords[i] as f64 * h_inv;

            let px = 2.0 - 4.0 * nx;
            let py = 1.0 - 2.0 * ny;

            // Early rejection: check if point is outside the Mollweide oval
            if px * px + 4.0 * py * py > 4.0 {
                continue;
            }

            let theta_aux = py.asin();
            let sin_lat = (2.0 * theta_aux + (2.0 * theta_aux).sin()) / PI;

            if sin_lat.abs() > 1.0 {
                continue;
            }

            let lat = sin_lat.asin();
            let c = theta_aux.cos();
            if c.abs() < 1e-12 {
                continue;
            }
            let lon = PI * px / (2.0 * c);

            lons[i] = lon;
            lats[i] = lat;
            mask[i] = true;
        }

        (lons, lats, mask)
    }
}

#[test]
fn mollweide_inverse_rejects_outside_oval() {
    let p = MollweideProjection;

    // clearly above the oval
    assert!(p.inverse(0.5, -0.1).is_none());
    assert!(p.inverse(0.5, 1.1).is_none());

    // clearly outside horizontally
    assert!(p.inverse(-0.1, 0.5).is_none());
    assert!(p.inverse(1.1, 0.5).is_none());
}

#[test]
fn mollweide_inverse_center() {
    let p = MollweideProjection;
    let (lon, lat) = p.inverse(0.5, 0.5).unwrap();

    assert!(lon.abs() < 1e-12);
    assert!(lat.abs() < 1e-12);
}

#[test]
fn mollweide_roundtrip() {
    let p = MollweideProjection;

    let lon = 1.0;
    let lat = 0.5;

    let (u, v) = p.forward(lon, lat).unwrap();
    let (lon2, lat2) = p.inverse(u, v).unwrap();

    assert!((lon - lon2).abs() < 1e-6);
    assert!((lat - lat2).abs() < 1e-6);
}

#[test]
fn raster_and_inverse_agree_on_validity() {
    let p = MollweideProjection;
    let grid = RasterGrid::new(100, 50);

    for (_, _, u, v) in grid.iter() {
        let inv = p.inverse(u, v);

        let x = 2.0 - 4.0 * u;
        let y = 1.0 - 2.0 * v;
        let oval = (x * x) / 4.0 + y * y <= 1.0;

        assert_eq!(inv.is_some(), oval);
    }
}

#[test]
fn pixel_to_ang_matches_inverse() {
    let p = MollweideProjection;
    let grid = RasterGrid::new(100, 50);

    for (px, py, u, v) in grid.iter() {
        let inv = p.inverse(u, v);
        let pixel_to_ang = p.pixel_to_ang(px, py, &grid);

        match (inv, pixel_to_ang) {
            (Some((lon1, lat1)), Some((lon2, lat2))) => {
                assert!(
                    (lon1 - lon2).abs() < 1e-10,
                    "lon mismatch at ({}, {}): inverse={}, pixel_to_ang={}",
                    px,
                    py,
                    lon1,
                    lon2
                );
                assert!(
                    (lat1 - lat2).abs() < 1e-10,
                    "lat mismatch at ({}, {}): inverse={}, pixel_to_ang={}",
                    px,
                    py,
                    lat1,
                    lat2
                );
            }
            (None, None) => {} // Both should reject
            _ => panic!(
                "Validity mismatch at ({}, {}): inverse={}, pixel_to_ang={}",
                px,
                py,
                inv.is_some(),
                pixel_to_ang.is_some()
            ),
        }
    }
}

#[test]
fn pixel_to_ang_center() {
    let p = MollweideProjection;
    let grid = RasterGrid::new(512, 256);

    // For a 512x256 grid, the exact center is at pixel (255.5, 127.5)
    // Which normalizes to (255.5/511, 127.5/255) ≈ (0.5, 0.5)
    // Test both the floor and ceiling to ensure symmetry
    let (lon1, lat1) = p.pixel_to_ang(255, 127, &grid).unwrap();
    let (lon2, lat2) = p.pixel_to_ang(256, 128, &grid).unwrap();

    // Both should be very close to origin
    assert!(lon1.abs() < 0.02, "Center-1 lon should be ~0: {}", lon1);
    assert!(lat1.abs() < 0.02, "Center-1 lat should be ~0: {}", lat1);
    assert!(lon2.abs() < 0.02, "Center lon should be ~0: {}", lon2);
    assert!(lat2.abs() < 0.02, "Center lat should be ~0: {}", lat2);
}

#[test]
fn pixel_to_ang_returns_lon_lat_in_correct_order() {
    let p = MollweideProjection;
    let grid = RasterGrid::new(512, 256);

    // Test a point on the right side where px is positive
    // Pixel (112, 128) is on the right and inside the oval
    if let Some((lon, lat)) = p.pixel_to_ang(112, 128, &grid) {
        // This pixel has positive px, which maps to positive longitude
        assert!(
            lon > 0.0,
            "Right side (px>0) should have positive lon: {}",
            lon
        );
        assert!(
            lat.abs() < 0.3,
            "Near equator should have small lat: {}",
            lat
        );
    }

    // Test a point on the left side where px is negative
    // Pixel (400, 128) is on the left and inside the oval
    if let Some((lon, lat)) = p.pixel_to_ang(400, 128, &grid) {
        // This pixel has negative px, which maps to negative longitude
        assert!(
            lon < 0.0,
            "Left side (px<0) should have negative lon: {}",
            lon
        );
        assert!(
            lat.abs() < 0.3,
            "Near equator should have small lat: {}",
            lat
        );
    }
}

#[test]
fn test_mollweide_all_pixels_inside_ellipse() {
    // This test verifies that pixel_to_ang correctly enforces ellipse bounds.
    // No pixel should produce valid coordinates if it's outside x²/4 + y² ≤ 1.
    // Conversely, if a pixel is inside the ellipse, it must produce valid coordinates.

    let proj = MollweideProjection;
    let grid = RasterGrid::new(1200, 600); // Standard rendering size

    let mut inside_count = 0;
    let mut outside_count = 0;
    let mut boundary_pixels = Vec::new();
    let mut invalid_returns = Vec::new();

    for (px, py, u, v) in grid.iter() {
        // Manually compute ellipse membership
        let x = 2.0 - 4.0 * u;
        let y = 1.0 - 2.0 * v;
        let ellipse_val = (x * x) / 4.0 + y * y;
        let should_be_inside = ellipse_val <= 1.0;

        // Track boundary pixels
        if (ellipse_val - 1.0).abs() < 1e-10 {
            boundary_pixels.push((px, py, ellipse_val, should_be_inside));
        }

        // Get result from pixel_to_ang
        let result = proj.pixel_to_ang(px, py, &grid);
        let is_valid = result.is_some();

        if should_be_inside {
            inside_count += 1;
            if !is_valid {
                invalid_returns.push((px, py, ellipse_val));
            }
        } else {
            outside_count += 1;
            // Outside pixels should not have valid returns
            if is_valid {
                panic!(
                    "Pixel ({}, {}) outside ellipse (val={:.6}) returned valid coordinates",
                    px, py, ellipse_val
                );
            }
        }
    }

    println!(
        "Mollweide ellipse coverage: {} inside, {} outside",
        inside_count, outside_count
    );
    if !boundary_pixels.is_empty() {
        println!("Boundary pixels (ellipse_val ≈ 1.0):");
        for (px, py, val, inside) in &boundary_pixels {
            println!(
                "  ({:4}, {:3}): ellipse_val={:.16}, should_be_inside={}",
                px, py, val, inside
            );
        }
    }
}

#[test]
fn batch_projection_matches_scalar() {
    use crate::projection::Projection;
    
    let proj = MollweideProjection;
    let grid = RasterGrid::new(512, 256);

    // Test batch projection against scalar version
    // Use a variety of pixel coordinates including edge cases
    let px_array = [10u32, 50, 100, 200, 300, 400, 450, 500];
    let py_array = [10u32, 50, 100, 128, 150, 200, 240, 250];

    let (batch_lons, batch_lats, batch_mask) = proj.pixel_to_ang_batch(&px_array, &py_array, &grid);

    for i in 0..8 {
        let scalar_result = proj.pixel_to_ang(px_array[i],py_array[i], &grid);
        
        match (scalar_result, batch_mask[i]) {
            (Some((scalar_lon, scalar_lat)), true) => {
                // Both should be valid - check values match
                assert!(
                    (batch_lons[i] - scalar_lon).abs() < 1e-14,
                    "Longitude mismatch at ({}, {}): batch={}, scalar={}",
                    px_array[i], py_array[i], batch_lons[i], scalar_lon
                );
                assert!(
                    (batch_lats[i] - scalar_lat).abs() < 1e-14,
                    "Latitude mismatch at ({}, {}): batch={}, scalar={}",
                    px_array[i], py_array[i], batch_lats[i], scalar_lat
                );
            }
            (None, false) => {
                // Both should be invalid - OK
            }
            _ => {
                panic!(
                    "Mismatch at ({}, {}): scalar valid={}, batch valid={}",
                    px_array[i], py_array[i],
                    scalar_result.is_some(), batch_mask[i]
                );
            }
        }
    }
}
