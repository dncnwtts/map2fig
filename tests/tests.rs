use std::str::FromStr;
use image::Rgba;

use healpix_plotter::{
    BadColor, NegMode, PixelValue,
    RgbaArg, generate_index_map, get_colormap,
    scale_value, plot_mollweide,
};

use healpix_plotter::plot::Scale; // <- Scale comes from plot module
use healpix_plotter::resolve_bad_color; // <- now public from lib.rs


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

/// ----------------------------
/// Test resolve_bad_color returns correct RGBA
/// ----------------------------
#[test]
fn test_resolve_bad_color() {
    let cmap = get_colormap("viridis");

    let auto_color = healpix_plotter::resolve_bad_color(Some(BadColor::Auto), cmap);
    assert_eq!(auto_color.0[3], 255);

    let gray_color = healpix_plotter::resolve_bad_color(Some(BadColor::Gray), cmap);
    assert_eq!(gray_color, Rgba([128,128,128,255]));

    let custom_color = BadColor::Rgba(10,20,30,40);
    let c = healpix_plotter::resolve_bad_color(Some(custom_color), cmap);
    assert_eq!(c, Rgba([10,20,30,40]));
}

/// ----------------------------
/// Test plotting with small map
/// ----------------------------
#[test]
fn test_plot_small_map() {
    let map = generate_index_map(1); // 12 pixels
    let cmap = get_colormap("viridis");
    let bad_color = Rgba([128,128,128,255]);
    let neg_mode = NegMode::Zero;

    // Should not panic
    plot_mollweide(
        &map,
        100,
        "test.png",
        None,
        None,
        cmap,
        true,
        false,
        true,
        1.0,
        false,
        false,
        false,
        0.0,
        1.0,
        neg_mode,
        bad_color,
    );
}

