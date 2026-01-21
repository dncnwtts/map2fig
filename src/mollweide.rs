use std::f64::consts::PI;
use crate::projection::Projection;
use crate::render::raster::RasterGrid;

#[derive(Debug, Clone, Copy)]
pub struct MollweideProjection;

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

        // Map [-1,1] → [0,1]
        let u = (2.0 - x) * 0.25;
        let v = (1.0 - y) * 0.5;


        if !(0.0..=1.0).contains(&u) || !(0.0..=1.0).contains(&v) {
            return None;
        }

        Some((u, v))
    }
    fn pixel_to_ang(
        &self,
        x: u32,
        y: u32,
        grid: &RasterGrid,
    ) -> Option<(f64, f64)> {
        let nx = grid.norm_x(x);
        let ny = grid.norm_y(y);

        let px = 2.0 - 4.0 * nx;
        let py = 1.0 - 2.0 * ny;

        if px * px / 4.0 + py * py > 1.0 {
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

        let theta = PI / 2.0 - lat;
        if !(0.0..=PI).contains(&theta) {
            return None;
        }

        Some((theta, lon))
    }
}



#[inline]
pub fn mollweide_inside_oval(x: f64, y: f64) -> bool {
    (x * x) / 4.0 + y * y <= 1.0
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

