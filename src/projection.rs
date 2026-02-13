use crate::render::raster::RasterGrid;
/// A map projection between spherical coordinates and normalized device coords.
///
/// All coordinates are normalized to `[0, 1]` unless otherwise stated.
pub trait Projection: Send + Sync {
    /// Inverse projection:
    /// Maps normalized pixel coordinates (u, v) ∈ `[0,1]²`
    /// to spherical coordinates (lon, lat) in radians.
    ///
    /// Returns None if (u, v) lies outside the projection.
    fn inverse(&self, u: f64, v: f64) -> Option<(f64, f64)>;

    /// Forward projection:
    /// Maps spherical coordinates (lon, lat) in radians
    /// to normalized device coordinates (u, v) ∈ `[0,1]`².
    ///
    /// Returns None if (lon, lat) is outside the projection domain.
    fn forward(&self, lon: f64, lat: f64) -> Option<(f64, f64)>;

    /// Maps normalized pixel coordinates to spherical coordinates in radians
    ///
    /// Takes existing grid as additional argument
    fn pixel_to_ang(&self, x: u32, y: u32, grid: &RasterGrid) -> Option<(f64, f64)>;
}
