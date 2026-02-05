use std::str::FromStr;

use map2fig::{
    cli::InputColor, NegMode, PixelValue,
    RgbaArg, generate_index_map,
};

use map2fig::scale::{Scale, scale_value};




/// ----------------------------
/// Test RGBAArg parsing
/// ----------------------------
#[test]
fn test_rgbaarg_from_str() {
    let rgba = RgbaArg::from_str("10,20,30,40").unwrap();
    assert_eq!((rgba.r, rgba.g, rgba.b, rgba.a), (10,20,30,40));

    assert!(RgbaArg::from_str("10,20,30").is_err());
    assert!(RgbaArg::from_str("a,b,c,d").is_err());
}

/// ----------------------------
/// Test InputColor parsing
/// ----------------------------
#[test]
fn test_bad_color_parse() {
    assert!(matches!(InputColor::from_str("gray").unwrap(), InputColor::Gray));
    assert!(matches!(InputColor::from_str("grey").unwrap(), InputColor::Gray));
    assert!(matches!(InputColor::from_str("255,128,0,255").unwrap(),
        InputColor::Rgba(255,128,0,255)));
}

/// ----------------------------
/// Test HEALPix map generation
/// ----------------------------
#[test]
fn test_generate_index_map() {
    let map = generate_index_map(2);
    assert_eq!(map.len(), 12 * 2 * 2); // nside=2 -> 48 pixels
}

/// ----------------------------
/// Test scale_value transformations
/// ----------------------------
#[test]
fn test_scale_value_transformations() {
    let min = 1.0;
    let max = 100.0;

    // Linear scale
    let t = scale_value(50.0, min, max, Scale::Linear, NegMode::Zero, None);
    match t {
        PixelValue::Color(c) => assert!((c - 0.4949).abs() < 1e-3),
        _ => panic!(),
    }

    // Log scale
    let t = scale_value(10.0, min, max, Scale::Log, NegMode::Zero, None);
    match t {
        PixelValue::Color(c) => assert!((c - 0.5).abs() < 1e-3),
        _ => panic!(),
    }

    // Asinh scale
    let t = scale_value(50.0, min, max, Scale::Asinh { scale: 10.0 }, NegMode::Zero, None);
    match t {
        PixelValue::Color(c) => assert!(c > 0.0 && c < 1.0),
        _ => panic!(),
    }
}

/// ----------------------------
/// Test neg_mode handling
/// ----------------------------
#[test]
fn test_neg_mode_behavior() {
    let min = 1.0;
    let max = 10.0;

    // Negative value with Zero mode
    let t = scale_value(-5.0, min, max, Scale::Linear, NegMode::Zero, None);
    match t {
        PixelValue::Color(c) => assert_eq!(c, 0.0),
        _ => panic!(),
    }

}

#[test]
fn test_plot_smoke() {
    use map2fig::healpix::{HealpixMeta, HealpixOrdering};
    use map2fig::rotation::{CoordSystem,ViewTransform};
    let map = map2fig::generate_index_map(1);
    let cmap = map2fig::get_colormap("viridis");
    let meta = HealpixMeta { ordering: HealpixOrdering::Ring, nside: 1, coord: CoordSystem::G};
    let input = CoordSystem::G;
    let output = CoordSystem::G;
    let rot = None;
    let view = ViewTransform::new(input,output,rot);

    map2fig::plot_mollweide_png(
        &map,
        32,
        "smoke.png",
        None,
        None,
        cmap,
        false,
        true,
        false,
        1.0,
        map2fig::scale::Scale::Linear,
        map2fig::NegMode::Zero,
        image::Rgba([0, 0, 0, 0]),
        image::Rgba([0, 0, 0, 0]),
        meta,
        false,
        Some("str"),
        &view,
        false,  // show_graticule
        None,   // grat_coord
        15.0,   // dpar_deg
        15.0,   // dmer_deg
    );
}

