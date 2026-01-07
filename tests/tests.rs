use std::str::FromStr;

use healpix_plotter::{
    BadColor, NegMode, PixelValue,
    RgbaArg, generate_index_map, 
    scale_value
};

use healpix_plotter::plot::Scale; // <- Scale comes from plot module




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
/// Test BadColor parsing
/// ----------------------------
#[test]
fn test_bad_color_parse() {
    assert!(matches!(BadColor::from_str("auto").unwrap(), BadColor::Auto));
    assert!(matches!(BadColor::from_str("gray").unwrap(), BadColor::Gray));
    assert!(matches!(BadColor::from_str("grey").unwrap(), BadColor::Gray));
    assert!(matches!(BadColor::from_str("255,128,0,255").unwrap(),
        BadColor::Rgba(255,128,0,255)));
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
    let t = scale_value(50.0, min, max, Scale::Linear, NegMode::Zero, 1.0);
    match t {
        PixelValue::Color(c) => assert!((c - 0.4949).abs() < 1e-3),
        _ => panic!(),
    }

    // Log scale
    let t = scale_value(10.0, min, max, Scale::Log, NegMode::Zero, 1.0);
    match t {
        PixelValue::Color(c) => assert!((c - 0.5).abs() < 1e-3),
        _ => panic!(),
    }

    // Asinh scale
    let t = scale_value(50.0, min, max, Scale::Asinh { scale: 10.0 }, NegMode::Zero, 1.0);
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
    let t = scale_value(-5.0, min, max, Scale::Linear, NegMode::Zero, 1.0);
    match t {
        PixelValue::Color(c) => assert_eq!(c, 0.0),
        _ => panic!(),
    }

    // Negative value with Unseen mode
    let t = scale_value(-5.0, min, max, Scale::Linear, NegMode::Unseen, 1.0);
    assert!(matches!(t, PixelValue::Bad));
}

#[test]
fn test_plot_smoke() {
    let map = healpix_plotter::generate_index_map(1);
    let cmap = healpix_plotter::get_colormap("viridis");

    healpix_plotter::plot_mollweide(
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
        healpix_plotter::plot::Scale::Linear,
        healpix_plotter::NegMode::Zero,
        image::Rgba([0, 0, 0, 0]),
    );
}

