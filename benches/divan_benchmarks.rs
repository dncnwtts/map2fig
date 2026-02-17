use map2fig::healpix::{ang2pix_ring, pix2ang_ring};
use std::f64::consts::PI;

fn main() {
    divan::main();
}

#[divan::bench]
fn bench_pix2ang_ring_bulk() {
    let nside = divan::black_box(512i64);
    let mut sum = 0.0f64;

    for pix in 0..1000 {
        let pix = pix % (12 * nside * nside);
        let (theta, phi) = pix2ang_ring(nside, pix);
        sum += theta + phi;
    }

    divan::black_box(sum);
}

#[divan::bench]
fn bench_ang2pix_ring_bulk() {
    let nside = divan::black_box(512i64);
    let mut sum = 0i64;

    for i in 0..1000 {
        let theta = PI * i as f64 / 1000.0;
        let phi = 2.0 * PI * i as f64 / 1000.0;
        sum += ang2pix_ring(nside, theta, phi);
    }

    divan::black_box(sum);
}

#[divan::bench]
fn bench_pix2ang_ring_scattered() {
    let nside = divan::black_box(8192i64);
    let mut sum = 0.0f64;

    // Simulate scattered access pattern (cache-hostile)
    for i in 0..100 {
        let pix = (i * 31337) % (12 * nside * nside);
        let (theta, phi) = pix2ang_ring(nside, pix);
        sum += theta + phi;
    }

    divan::black_box(sum);
}

#[divan::bench]
fn bench_ang2pix_ring_scattered() {
    let nside = divan::black_box(8192i64);
    let mut sum = 0i64;

    // Wide range of angles to avoid branch prediction bias
    for i in 0..100 {
        let angle_varied = (i as f64 * 0.0432) % 1.0; // Irrational multiplier
        let theta = PI * angle_varied;
        let phi = 2.0 * PI * angle_varied.sin().abs();
        sum += ang2pix_ring(nside, theta, phi);
    }

    divan::black_box(sum);
}
