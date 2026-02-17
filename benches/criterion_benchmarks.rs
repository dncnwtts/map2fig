use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use map2fig::healpix::{ang2pix_nest, ang2pix_ring, pix2ang_nest, pix2ang_ring};
use std::f64::consts::PI;

/// Benchmark coordinate system conversions - the core hotspot
///
/// These functions are called billions of times during rendering,
/// making them key targets for optimization (SIMD, caching, etc.)
fn coordinate_conversions(c: &mut Criterion) {
    let mut group = c.benchmark_group("coordinates");
    group.sample_size(1000); // More samples for stable measurements

    // Test different nside values
    for nside_log2 in [8, 10, 12, 14] {
        let nside = 1i64 << nside_log2;
        let _npix = (12 * nside * nside) as u64;

        group.bench_with_input(
            BenchmarkId::new("pix2ang_ring", nside),
            &nside,
            |b, &nside| {
                b.iter(|| {
                    for pix in
                        (0..1000).map(|i| black_box((i * 1000) as i64 % (12 * nside * nside)))
                    {
                        let (theta, phi) = pix2ang_ring(nside, pix);
                        black_box((theta, phi));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("pix2ang_nest", nside),
            &nside,
            |b, &nside| {
                b.iter(|| {
                    for pix in
                        (0..1000).map(|i| black_box((i * 1000) as i64 % (12 * nside * nside)))
                    {
                        let (theta, phi) = pix2ang_nest(nside, pix);
                        black_box((theta, phi));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("ang2pix_ring", nside),
            &nside,
            |b, &nside| {
                b.iter(|| {
                    for i in 0..1000 {
                        let theta = black_box(PI * i as f64 / 1000.0);
                        let phi = black_box(2.0 * PI * i as f64 / 1000.0);
                        let pix = ang2pix_ring(nside, theta, phi);
                        black_box(pix);
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("ang2pix_nest", nside),
            &nside,
            |b, &nside| {
                b.iter(|| {
                    for i in 0..1000 {
                        let theta = black_box(PI * i as f64 / 1000.0);
                        let phi = black_box(2.0 * PI * i as f64 / 1000.0);
                        let pix = ang2pix_nest(nside, theta, phi);
                        black_box(pix);
                    }
                });
            },
        );
    }

    group.finish();
}

/// Benchmark the downgrade operation at different scales
///
/// This is the target for adaptive chunking optimization.
/// We benchmark it separately to track improvements from parallelization.
fn downgrade_operation(c: &mut Criterion) {
    let mut group = c.benchmark_group("downgrade");
    group.sample_size(10); // Lower samples since downgrade is expensive

    // Test small downsampling (fast)
    group.bench_function("downgrade_nside256_to_128", |b| {
        use map2fig::healpix::{HealpixOrdering, downgrade_healpix_map};

        let nside = 256i64;
        let npix = (12 * nside * nside) as usize;
        let map = vec![1.0f64; npix];

        b.iter(|| {
            let _result = downgrade_healpix_map(
                black_box(&map),
                black_box(nside),
                black_box(128i64),
                black_box(HealpixOrdering::Ring),
            );
        });
    });

    // Test medium downsampling
    group.bench_function("downgrade_nside512_to_256", |b| {
        use map2fig::healpix::{HealpixOrdering, downgrade_healpix_map};

        let nside = 512i64;
        let npix = (12 * nside * nside) as usize;
        let map = vec![1.0f64; npix];

        b.iter(|| {
            let _result = downgrade_healpix_map(
                black_box(&map),
                black_box(nside),
                black_box(256i64),
                black_box(HealpixOrdering::Ring),
            );
        });
    });

    group.finish();
}

criterion_group!(benches, coordinate_conversions, downgrade_operation);
criterion_main!(benches);
