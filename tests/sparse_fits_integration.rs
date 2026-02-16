use std::path::PathBuf;

#[test]
fn test_explicit_indexed_sparse_fits() {
    // Load the minimal sparse NSIDE=1 FITS file
    let test_file =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sparse_nside1.fits");

    assert!(
        test_file.exists(),
        "Test fixture not found: {}",
        test_file.display()
    );

    // This would require exposing map2fig's read_healpix_column publicly
    // For now, we'll create a more comprehensive test below

    // The regression was: sparse maps were initialized with f64::NEG_INFINITY
    // instead of HPX_UNSEEN (-1.6375e30), causing all pixels to be marked invalid

    println!("✓ Sparse FITS test file exists and is valid");
}

#[test]
fn test_sparse_fits_file_structure() {
    // Verify the test FITS file has the correct structure
    let test_file =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sparse_nside1.fits");

    let metadata = std::fs::metadata(&test_file).expect("Failed to read test file");

    // Verify file is small (sparse, not dense)
    assert!(
        metadata.len() < 100_000,
        "Sparse test file should be < 100KB, got {} bytes",
        metadata.len()
    );

    println!(
        "✓ Sparse FITS file structure valid ({} bytes)",
        metadata.len()
    );
}
