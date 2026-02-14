use crate::render::raster::RasterGrid;
/// A map projection between spherical coordinates and normalized device coords.
///
/// All coordinates are normalized to `[0, 1]` unless otherwise stated.
pub trait Projection {
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

    /// Batch projection: process up to 8 pixels in parallel
    ///
    /// Input:
    ///   - px: array of x pixel coordinates [x0, x1, ..., x7]
    ///   - py: array of y pixel coordinates [y0, y1, ..., y7]
    ///   - grid: rendering grid with dimensions
    ///
    /// Output:
    ///   - lons: array of longitudes [lon0, lon1, ..., lon7]
    ///   - lats: array of latitudes [lat0, lat1, ..., lat7]
    ///   - mask: array of booleans [valid0, valid1, ..., valid7]
    ///
    /// All pixels are returned (valid and invalid), with validity indicated by mask.
    /// Invalid pixels (outside projection) return (0.0, 0.0) with mask bit = false.
    fn pixel_to_ang_batch(
        &self,
        px: &[u32; 8],
        py: &[u32; 8],
        grid: &RasterGrid,
    ) -> (
        [f64; 8], // longitudes
        [f64; 8], // latitudes
        [bool; 8], // validity mask
    ) {
        // Default implementation: just call scalar version 8 times
        let mut lons = [0.0_f64; 8];
        let mut lats = [0.0_f64; 8];
        let mut mask = [false; 8];

        for i in 0..8 {
            if let Some((lon, lat)) = self.pixel_to_ang(px[i], py[i], grid) {
                lons[i] = lon;
                lats[i] = lat;
                mask[i] = true;
            }
        }

        (lons, lats, mask)
    }
}
