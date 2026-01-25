use crate::healpix::{HealpixMeta, ang2pix};
// use crate::rotation::{sph_to_vec, vec_to_sph, mat_vec};
// 
// pub struct HealpixSampler<'a> {
//     pub map: &'a [f64],
//     pub meta: &'a HealpixMeta,
//     pub rot: [[f64; 3]; 3], // view → map
// }
// 
// impl<'a> HealpixSampler<'a> {
//     #[inline(always)]
//     pub fn sample(&self, theta: f64, lon: f64) -> Option<f64> {
//         if !theta.is_finite() || !lon.is_finite() {
//             return None;
//         }
// 
//         // View → map rotation
//         let v = sph_to_vec(theta, lon);
//         let v = mat_vec(&self.rot, v);
//         let (theta, lon) = vec_to_sph(v);
// 
//         // Clamp / wrap exactly once
//         let theta = theta.clamp(0.0, std::f64::consts::PI);
//         let lon = lon.rem_euclid(2.0 * std::f64::consts::PI);
// 
//         let ipix = ang2pix(*self.meta, theta, lon) as usize;
//         self.map.get(ipix).copied()
//     }
// }
// 
