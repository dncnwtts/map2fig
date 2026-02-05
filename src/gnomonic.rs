use std::f64::consts::PI;
use crate::projection::Projection;
use crate::render::raster::RasterGrid;

/// Gnomonic projection with center and resolution parameters
/// 
/// Maps a square tangent plane to spherical coordinates using gnomonic (perspective)
/// projection from the center of the sphere onto a tangent plane.
#[derive(Debug, Clone, Copy)]
pub struct GnomonicProjection {
    /// Center longitude in radians
    pub lon_center: f64,
    /// Center latitude in radians
    pub lat_center: f64,
    /// Resolution in arcmin per pixel
    pub resolution_arcmin: f64,
}

impl GnomonicProjection {
    /// Create a new gnomonic projection
    /// 
    /// # Arguments
    /// * `lon_deg` - Center longitude in degrees
    /// * `lat_deg` - Center latitude in degrees
    /// * `resolution_arcmin` - Resolution in arcmin per pixel
    pub fn new(lon_deg: f64, lat_deg: f64, resolution_arcmin: f64) -> Self {
        Self {
            lon_center: lon_deg.to_radians(),
            lat_center: lat_deg.to_radians(),
            resolution_arcmin,
        }
    }

    /// Default projection (galactic center, 1 arcmin/pixel)
    pub fn default_center() -> Self {
        Self::new(0.0, 0.0, 1.0)
    }

    /// Compute CPAC Euler rotation matrix to align center with north pole
    /// CPAC: Make_CPAC_Euler_Matrix(lon0, -lat0, 0)
    fn compute_rotation(&self) -> [[f64; 3]; 3] {
        let lon0 = self.lon_center;
        let lat0 = self.lat_center;

        let cos_lon = lon0.cos();
        let sin_lon = lon0.sin();
        let cos_lat = (-lat0).cos();
        let sin_lat = (-lat0).sin();

        // Combined rotation matrix for CPAC Euler angles
        [
            [cos_lon * cos_lat, -sin_lon, cos_lon * sin_lat],
            [sin_lon * cos_lat, cos_lon, sin_lon * sin_lat],
            [-sin_lat, 0.0, cos_lat],
        ]
    }

    /// Apply rotation matrix to 3D vector
    fn apply_rotation(&self, mat: &[[f64; 3]; 3], p: &[f64; 3]) -> [f64; 3] {
        [
            mat[0][0] * p[0] + mat[0][1] * p[1] + mat[0][2] * p[2],
            mat[1][0] * p[0] + mat[1][1] * p[1] + mat[1][2] * p[2],
            mat[2][0] * p[0] + mat[2][1] * p[1] + mat[2][2] * p[2],
        ]
    }

    /// Convert 3D Cartesian vector to spherical (lon, lat) in radians
    fn vec_to_spherical(&self, v: &[f64; 3]) -> (f64, f64) {
        let r = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let lon = v[1].atan2(v[0]);
        let lat = (v[2] / r).asin();
        (lon, lat)
    }

    /// Convert spherical (lon, lat) to 3D Cartesian unit vector
    #[allow(dead_code)]
    fn spherical_to_vec(&self, lon: f64, lat: f64) -> [f64; 3] {
        let cos_lat = lat.cos();
        [cos_lat * lon.cos(), cos_lat * lon.sin(), lat.sin()]
    }
}

impl Projection for GnomonicProjection {
    /// Inverse projection:
    /// (u,v) ∈ [0,1]² → (lon,lat) on sphere
    /// This is the main path for rendering: pixel coordinates → sky coordinates
    fn inverse(&self, u: f64, v: f64) -> Option<(f64, f64)> {
        let x = 2.0 * u - 1.0;
        let y = 2.0 * v - 1.0;
    
        let rho2 = x * x + y * y;
    
        if rho2 == 0.0 {
            return Some((self.lon_center, self.lat_center));
        }
    
        let rho = rho2.sqrt();
        let c = rho.atan();
    
        let sin_c = c.sin();
        let cos_c = c.cos();
    
        let _lat = (y * sin_c / rho).asin();
        let _lon = (x * sin_c).atan2(rho * cos_c);
        let point = [1.0, -x, -y];
        let r = (point[0] * point[0] + point[1] * point[1] + point[2] * point[2]).sqrt();
        if r == 0.0 {
            return None;
        }
        let point_normalized = [point[0] / r, point[1] / r, point[2] / r];
        
        let rot = self.compute_rotation();
        let rotated = self.apply_rotation(&rot, &point_normalized);
        let (lon_celestial, lat_celestial) = self.vec_to_spherical(&rotated);
    
        Some((lon_celestial, lat_celestial))
    }

    /// Forward projection:
    /// (lon,lat) → (u,v) ∈ [0,1]²
    fn forward(&self, lon: f64, lat: f64) -> Option<(f64, f64)> {
        let sin_lat = lat.sin();
        let cos_lat = lat.cos();
        let sin_lon = lon.sin();
        let cos_lon = lon.cos();

        // Direction cosine toward tangent plane
        let z = cos_lat * cos_lon;
        if z <= 0.0 {
            return None; // behind the plane
        }

        let x = cos_lat * sin_lon / z;
        let y = sin_lat / z;

        // Map [-1,1] → [0,1]
        let u = 0.5 * (x + 1.0);
        let v = 0.5 * (y + 1.0);

        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return None;
        }

        Some((u, v))
    }

    /// Pixel-space helper: converts pixel coordinates to sky coordinates
    /// This is the main entry point for rendering
    fn pixel_to_ang(&self, x: u32, y: u32, grid: &RasterGrid) -> Option<(f64, f64)> {
        let xsize = grid.width as f64;
        
        // Convert resolution from arcmin to radians
        let delta = self.resolution_arcmin * PI / (180.0 * 60.0);
        
        // Starting offset (center of image at pixel (xsize/2, xsize/2))
        let start = -(xsize / 2.0) * delta;
        
        // Offset in tangent plane coordinates (radians)
        let dx = start + (x as f64) * delta;
        let dy = start + (y as f64) * delta;
        
        // Construct 3D point on tangent plane: (1, -dx, -dy) before rotation
        let point = [1.0, -dx, -dy];
        
        // Normalize to unit vector
        let r = (point[0] * point[0] + point[1] * point[1] + point[2] * point[2]).sqrt();
        if r == 0.0 {
            return None;
        }
        let point_normalized = [point[0] / r, point[1] / r, point[2] / r];
        
        // Apply rotation to transform from tangent plane to celestial sphere
        let rot = self.compute_rotation();
        let rotated = self.apply_rotation(&rot, &point_normalized);
        
        // Convert to spherical coordinates
        let (lon, lat) = self.vec_to_spherical(&rotated);
        
        Some((lon, lat))
    }
}


impl Default for GnomonicProjection {
    fn default() -> Self {
        Self::default_center()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gnomonic_inverse_center() {
        let p = GnomonicProjection::new(0.0, 0.0, 1.0);
        let (lon, lat) = p.inverse(0.5, 0.5).unwrap();

        assert!(lon.abs() < 1e-6, "lon={}", lon);
        assert!(lat.abs() < 1e-6, "lat={}", lat);
    }

    #[test]
    fn gnomonic_rejects_behind_plane() {
        let p = GnomonicProjection::new(0.0, 0.0, 1.0);

        // 90° away from center → z ≈ 0
        assert!(p.forward(PI / 2.0, 0.0).is_none());
        assert!(p.forward(-PI / 2.0, 0.0).is_none());
    }

    #[test]
    fn pixel_to_ang_center() {
        let p = GnomonicProjection::new(0.0, 0.0, 1.0);
        let grid = RasterGrid::new(512, 512);

        let (lon, lat) = p.pixel_to_ang(256, 256, &grid).unwrap();
        
        // Center of image should map to center of projection
        assert!(lon.abs() < 0.05, "Center lon too far: {}", lon);
        assert!(lat.abs() < 0.05, "Center lat too far: {}", lat);
    }

    #[test]
    fn pixel_to_ang_with_offset_center() {
        let p = GnomonicProjection::new(45.0, 30.0, 1.0);
        let grid = RasterGrid::new(512, 512);

        let (lon, lat) = p.pixel_to_ang(256, 256, &grid).unwrap();
        
        let lon_expected = 45.0_f64.to_radians();
        let lat_expected = 30.0_f64.to_radians();
        
        assert!(
            (lon - lon_expected).abs() < 0.05,
            "Offset lon: got {}, expected {}",
            lon,
            lon_expected
        );
        assert!(
            (lat - lat_expected).abs() < 0.05,
            "Offset lat: got {}, expected {}",
            lat,
            lat_expected
        );
    }

    #[test]
    fn pixel_to_ang_center_returns_correct_coords() {
        let p = GnomonicProjection::new(0.0, 0.0, 1.0);
        let grid = RasterGrid::new(256, 256);

        let (lon, lat) = p.pixel_to_ang(128, 128, &grid).unwrap();

        // Center should map close to origin
        assert!(lon.abs() < 0.1, "Center lon should be near 0: {}", lon);
        assert!(lat.abs() < 0.1, "Center lat should be near 0: {}", lat);
    }

    #[test]
    fn pixel_to_ang_returns_lon_lat_not_theta_lon() {
        let p = GnomonicProjection::new(0.0, 0.0, 1.0);
        let grid = RasterGrid::new(512, 512);

        let (lon, lat) = p.pixel_to_ang(256, 256, &grid).unwrap();

        // At center, both should be near 0
        assert!(lon.abs() < 0.05, "Center lon should be near 0: {}", lon);
        assert!(lat.abs() < 0.05, "Center lat should be near 0: {}", lat);

        // Test north vs south (lat should change with y)
        let (_lon_n, lat_north) = p.pixel_to_ang(256, 100, &grid).unwrap();
        let (_lon_s, lat_south) = p.pixel_to_ang(256, 412, &grid).unwrap();

        // North should have positive lat, south should have negative
        assert!(lat_north > 0.0, "North should have positive lat: {}", lat_north);
        assert!(lat_south < 0.0, "South should have negative lat: {}", lat_south);
    }
}


