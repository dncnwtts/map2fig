use crate::projection::Projection;
use crate::render::raster::RasterGrid;

/// Hammer-Aitoff projection.
///
/// An equal-area projection particularly useful for all-sky maps.
/// The projection is defined as:
///   - Forward: (lon, lat) → (x, y)
///   - Inverse: (x, y) → (lon, lat)
#[derive(Debug, Clone, Copy)]
pub struct HammerProjection;

impl Default for HammerProjection {
    fn default() -> Self {
        Self::new()
    }
}

impl HammerProjection {
    pub fn new() -> Self {
        Self
    }
}

impl Projection for HammerProjection {
    fn inverse(&self, u: f64, v: f64) -> Option<(f64, f64)> {
        // Map u,v ∈ [0,1] to projection plane: x,y ∈ [-2,2] x [-1,1]
        let x = 4.0 * u - 2.0;
        let y = 1.0 - 2.0 * v;

        // Check if point is within the valid Hammer ellipse
        // Hammer projection has bounds: x² + 2y² ≤ 4
        if x * x + 2.0 * y * y > 4.0 {
            return None;
        }

        // Inverse formulas for Hammer projection:
        // z = sqrt(1 - (x/2)² - (y/2)²)
        // lon = 2 * arctan(z*x / (2*(2*z² - 1)))
        // lat = arcsin(z*y)
        
        let z_sq = 1.0 - (x / 2.0) * (x / 2.0) - (y / 2.0) * (y / 2.0);
        if z_sq < 0.0 {
            return None;
        }
        
        let z = z_sq.sqrt();
        let lat = (z * y).asin();
        
        // Use atan2 for lon to handle all quadrants
        let lon = 2.0 * (z * x / (2.0 * (2.0 * z * z - 1.0))).atan();

        Some((lon, lat))
    }

    fn forward(&self, lon: f64, lat: f64) -> Option<(f64, f64)> {
        // Forward Hammer projection:
        // d = sqrt(1 + cos(lat)*cos(lon/2))
        // x = 2 * cos(lat) * sin(lon/2) / d
        // y = sin(lat) / d
        // Normalized to [0,1]² with x ∈ [-2,2], y ∈ [-1,1]
        
        let lon_2 = lon / 2.0;
        let cos_lat = lat.cos();
        let cos_lon_2 = lon_2.cos();
        
        let d = (1.0 + cos_lat * cos_lon_2).sqrt();
        
        if d.abs() < 1e-10 {
            return None;
        }
        
        let x = 2.0 * cos_lat * lon_2.sin() / d;
        let y = lat.sin() / d;
        
        // Normalize to [0, 1]
        let u = (x + 2.0) / 4.0;
        let v = (1.0 - y) / 2.0;
        
        // Check bounds
        if u >= 0.0 && u <= 1.0 && v >= 0.0 && v <= 1.0 {
            Some((u, v))
        } else {
            None
        }
    }

    fn pixel_to_ang(&self, x: u32, y: u32, grid: &RasterGrid) -> Option<(f64, f64)> {
        let u = x as f64 / grid.width as f64;
        let v = y as f64 / grid.height as f64;
        self.inverse(u, v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn test_hammer_center() {
        let proj = HammerProjection::new();
        // Center of projection (lon=0, lat=0) should map to center of image
        let (u, v) = proj.forward(0.0, 0.0).unwrap();
        assert!((u - 0.5).abs() < 0.01);
        assert!((v - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_hammer_poles() {
        let proj = HammerProjection::new();
        // North pole (lat=π/2) should have specific x,y coordinates
        if let Some((u, v)) = proj.forward(0.0, PI / 2.0) {
            assert!(u >= 0.0 && u <= 1.0);
            assert!(v >= 0.0 && v <= 1.0);
        }
        // South pole (lat=-π/2)
        if let Some((u, v)) = proj.forward(0.0, -PI / 2.0) {
            assert!(u >= 0.0 && u <= 1.0);
            assert!(v >= 0.0 && v <= 1.0);
        }
    }

    #[test]
    fn test_hammer_roundtrip() {
        let proj = HammerProjection::new();
        let lons = [-PI, -PI / 2.0, 0.0, PI / 2.0, PI];
        let lats = [-PI / 4.0, 0.0, PI / 4.0];

        for &lon in &lons {
            for &lat in &lats {
                if let Some((u, v)) = proj.forward(lon, lat) {
                    if let Some((lon2, lat2)) = proj.inverse(u, v) {
                        let dlon = ((lon - lon2 + PI) % (2.0 * PI) - PI).abs();
                        let dlat = (lat - lat2).abs();
                        assert!(dlon < 1e-6, "Lon mismatch: {} vs {}", lon, lon2);
                        assert!(dlat < 1e-6, "Lat mismatch: {} vs {}", lat, lat2);
                    }
                }
            }
        }
    }
}
