#[test]
fn test_rounding() {
    let v1 = 50.5_f64;
    let v2 = 149.5_f64;
    eprintln!("50.5.round() = {}", v1.round());
    eprintln!("149.5.round() = {}", v2.round());

    // Also test 50.51 and 50.49
    let v3 = 50.51_f64;
    let v4 = 50.49_f64;
    eprintln!("50.51.round() = {}", v3.round());
    eprintln!("50.49.round() = {}", v4.round());
}
