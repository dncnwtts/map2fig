use image::Rgb;

const VIRIDIS: [[u8; 3]; 256] = include!("viridis_lut.in");

#[inline]
pub fn viridis(t: f64) -> Rgb<u8> {
    let x = (t * 255.0).round() as usize;
    let c = VIRIDIS[x.min(255)];
    Rgb(c)
}

