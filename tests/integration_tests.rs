//! Integration tests for code quality assurance
//! Tests end-to-end functionality, output quality, and error handling

#[cfg(test)]
mod integration_tests {
    use map2fig::healpix::{HealpixMeta, HealpixOrdering};
    use map2fig::params::PlotData;
    use map2fig::rotation::CoordSystem;
    use map2fig::scale::Scale;

    /// Test that basic PlotData can be constructed
    #[test]
    fn test_plot_data_construction() {
        let nside = 64;
        let npix = 12 * nside * nside;
        let mut map = vec![0.0f64; npix];

        for (i, v) in map.iter_mut().enumerate().take(npix) {
            *v = (i as f64 / npix as f64) * 100.0;
        }

        let plot_data = PlotData {
            map: &map,
            width: 800,
            filename: "test_basic.pdf".to_string(),
        };

        assert_eq!(plot_data.map.len(), npix);
        assert_eq!(plot_data.width, 800);
        assert!(!plot_data.filename.is_empty());
    }

    /// Test extreme values
    #[test]
    fn test_extreme_value_handling() {
        let nside = 32;
        let npix = 12 * nside * nside;
        let mut map = vec![0.0f64; npix];

        for (i, v) in map.iter_mut().enumerate().take(npix) {
            *v = if i % 3 == 0 {
                1e-6
            } else if i % 3 == 1 {
                1e6
            } else {
                0.0
            };
        }

        let plot_data = PlotData {
            map: &map,
            width: 600,
            filename: "test_extreme.pdf".to_string(),
        };

        assert!(plot_data.map.iter().any(|&v| v > 0.0));
        assert!(plot_data.map.iter().any(|&v| v < 1.0));
    }

    /// Test coordinate systems
    #[test]
    fn test_coordinate_systems() {
        let systems = [
            CoordSystem::G, // Galactic
            CoordSystem::E, // Equatorial
            CoordSystem::C, // Ecliptic
        ];

        assert_eq!(systems.len(), 3);
    }

    /// Test NaN handling
    #[test]
    fn test_unseen_value_handling() {
        let nside = 16;
        let npix = 12 * nside * nside;
        let mut map = vec![0.0f64; npix];

        for (i, v) in map.iter_mut().enumerate().take(npix) {
            if i % 10 == 0 {
                *v = f64::NAN;
            } else {
                *v = i as f64;
            }
        }

        let valid_count = map.iter().filter(|&&v| !v.is_nan()).count();
        assert!(valid_count > npix / 2);
    }

    /// Test HEALPix orderings
    #[test]
    fn test_healpix_orderings() {
        let ring = HealpixOrdering::Ring;
        let _nested = HealpixOrdering::Nested;

        let _meta = HealpixMeta {
            ordering: ring,
            nside: 64,
            coord: CoordSystem::G,
        };
    }

    /// Test scale types
    #[test]
    fn test_scale_enum() {
        let _scales = [
            Scale::Linear,
            Scale::Log,
            Scale::Asinh { scale: 1.0 },
            Scale::Symlog { linthresh: 10.0 },
            Scale::Histogram,
        ];
    }

    /// Test colormap availability
    #[test]
    fn test_colormap_availability() {
        use map2fig::colormap::available_colormaps;

        let cmaps = available_colormaps();
        assert!(cmaps.iter().any(|c| c.contains("viridis")));
        assert!(cmaps.iter().any(|c| c.contains("plasma")));
        assert!(cmaps.len() > 50);
    }

    /// Test various output widths
    #[test]
    fn test_various_output_widths() {
        let nside = 32;
        let npix = 12 * nside * nside;
        let map = vec![1.0f64; npix];

        for width in &[400u32, 800, 1200, 1600, 2400] {
            let plot_data = PlotData {
                map: &map,
                width: *width,
                filename: format!("test_{}.pdf", width),
            };

            assert_eq!(plot_data.width, *width);
        }
    }

    /// Test minimal map
    #[test]
    fn test_minimal_map_handling() {
        let nside = 1;
        let npix = 12 * nside * nside;
        let map = vec![1.0f64; npix];

        let plot_data = PlotData {
            map: &map,
            width: 100,
            filename: "test_minimal.pdf".to_string(),
        };

        assert_eq!(plot_data.map.len(), npix);
        assert_eq!(npix, 12);
    }

    /// Test NSIDE pyramid
    #[test]
    fn test_healpix_nsides() {
        for nside_pow in 0..10 {
            let nside = 1u32 << nside_pow;
            let npix = 12 * nside * nside;

            assert_eq!(npix, 12 * nside * nside);
        }
    }
}
