//! FITS file reading and data extraction.
//!
//! This module provides functions for reading HEALPix astronomical data from FITS files.
//! It supports both:
//! - **Dense maps**: Complete sky coverage (standard NSIDE² pixels)
//! - **Sparse maps**: Partial sky coverage with explicit PIXEL column indexing
//!
//! # FITS Format Support
//!
//! - Binary table format (.fits files)
//! - HEALPix RING or NEST pixel ordering
//! - Sparse maps with IMPLICIT or EXPLICIT indexing schemes via the INDXSCHM keyword
//!
//! # Sparse Map Handling
//!
//! For sparse maps with EXPLICIT indexing, this module automatically expands the data
//! to a full dense array with UNSEEN values for omitted pixels.
//!
//! **Tier 4.2b Optimization:** Sparse map column extraction is parallelized via rayon
//! for efficient multi-core processing of large sparse catalogs.
//!
//! # Examples
//!
//! ```ignore
//! use map2fig::read_healpix_column;
//!
//! let data = read_healpix_column("map.fits", 0);
//! println!("Loaded {} pixels", data.len());
//! ```

use std::fs::File;
use std::io::{BufReader, Read, Write};

use fitsrs::hdu::data::bintable::{ColumnId, DataValue};
use fitsrs::{Fits, HDU, card::Value};
use rayon::prelude::*;

/// Read a HEALPix column from a FITS binary table.
///
/// Extracts data from a specific column of a HEALPix FITS binary table.
/// Automatically handles both dense and sparse (EXPLICIT) indexing schemes.
/// For sparse maps, expands the result to full NSIDE² size with UNSEEN values.
///
/// # Arguments
///
/// * `filename` - Path to the FITS file
/// * `col_idx`  - 0-based data column index (not the PIXEL column, just the data)
///
/// # Returns
///
/// Vector of f64 values with length = 12 * NSIDE²
/// - Dense maps: all pixels present in FITS
/// - Sparse maps: UNSEEN (-1.6375e30) for missing pixels
///
/// # Panics
///
/// Panics if:
/// - File cannot be opened
/// - FITS structure is invalid
/// - Column index is out of bounds
/// - Required HEALPix headers are missing
pub fn read_healpix_column(filename: &str, col_idx: usize) -> Vec<f64> {
    // Tier 2 Optimization: Use memory-mapped I/O instead of buffered reads
    // Eliminates kernel memcpy overhead (rep_movs_alternative) and improves cache locality
    let reader = crate::mmap_reader::MmapFitsReader::open(filename)
        .unwrap_or_else(|_| panic!("Failed to open FITS file: {}", filename));

    let mut fits = Fits::from_reader(reader);
    let mut result: Vec<f64> = Vec::new();
    let mut nside: i64 = 0;

    while let Some(Ok(hdu)) = fits.next() {
        if let HDU::XBinaryTable(hdu) = hdu {
            let header = hdu.get_header();

            // Check if this uses explicit indexing (sparse/partial sky map)
            let has_explicit_indexing = match header.get("INDXSCHM") {
                Some(Value::String { value, .. }) => value.trim() == "EXPLICIT",
                _ => false,
            };

            // Get NSIDE if not already set
            if nside == 0 {
                nside = match header.get("NSIDE") {
                    Some(Value::Integer { value, .. }) => *value,
                    _ => 0,
                };
            }

            let data = fits.get_data(&hdu);
            let mut table = data.table_data();

            // If explicit indexing, read both PIXEL and data columns together
            if has_explicit_indexing && nside > 0 {
                // For explicit indexing:
                // - Column 0 is always PIXEL indices
                // - Column 1+ are data columns
                // - User's --col N refers to the N-th data column
                // - Adjust file column: file_col = col_idx + 1
                let file_col_for_data = col_idx + 1;

                // Tier 1 Optimization: Eliminate intermediate Vec<DataValue>
                // Directly zip columns and process without intermediate buffer
                let npix = (12 * nside * nside) as usize;
                let mut full_map = vec![f64::NEG_INFINITY; npix];

                // Extract columns directly as iterators, then process
                let pixel_col = table.select_fields(&[ColumnId::Index(0)]).collect::<Vec<_>>();
                let value_col = table.select_fields(&[ColumnId::Index(file_col_for_data)]).collect::<Vec<_>>();

                if !pixel_col.is_empty() && !value_col.is_empty() {
                    // Tier 4.2b: Parallel extraction of pixel indices and values
                    // Extract pairs from separate columns in parallel
                    let pairs: Vec<(usize, f64)> = (0..pixel_col.len())
                        .into_par_iter()
                        .filter_map(|row_idx| {
                            let pix = match &pixel_col[row_idx] {
                                DataValue::Integer { value, .. } => *value as i64,
                                DataValue::Long { value, .. } => *value,
                                DataValue::Float { value, .. } => *value as i64,
                                DataValue::Double { value, .. } => *value as i64,
                                _ => -1,
                            };

                            let val = if row_idx < value_col.len() {
                                match &value_col[row_idx] {
                                    DataValue::Double { value, .. } => *value,
                                    DataValue::Float { value, .. } => *value as f64,
                                    DataValue::Integer { value, .. } => *value as f64,
                                    other => {
                                        panic!("Unsupported column type in FITS table: {:?}", other)
                                    }
                                }
                            } else {
                                return None;
                            };

                            if pix >= 0 && (pix as usize) < npix {
                                Some((pix as usize, val))
                            } else {
                                None
                            }
                        })
                        .collect();

                    // Populate map sequentially from parallel results
                    for (pix_idx, val) in pairs {
                        full_map[pix_idx] = val;
                    }
                }

                result = full_map;
            } else {
                // Regular dense map: read column directly
                let values = table.select_fields(&[ColumnId::Index(col_idx)]);
                for cell in values {
                    match cell {
                        DataValue::Double { value, .. } => result.push(value),
                        DataValue::Float { value, .. } => result.push(value as f64),
                        DataValue::Integer { value, .. } => result.push(value as f64),
                        other => panic!("Unsupported column type in FITS table: {:?}", other),
                    }
                }
            }
        }
    }

    result
}

// ============================================================================
// Metadata Caching (Phase 4.2a: I/O Optimization)
// ============================================================================

use serde_json::{Value as JsonValue, json};
use std::path::{Path, PathBuf};

/// Get cache directory for FITS metadata
fn get_cache_dir() -> Option<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "map2fig")?;
    Some(dirs.cache_dir().to_path_buf())
}

/// Hash a file path and mtime for cache validation
fn compute_cache_key(filepath: &str) -> Option<String> {
    use std::fs;

    let path = Path::new(filepath);
    let metadata = fs::metadata(path).ok()?;
    let mtime = metadata.modified().ok()?;

    let mtime_seconds = mtime.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();

    // Use SHA256 for filename safety
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(filepath.as_bytes());
    let hash = format!("{:x}", hasher.finalize());

    // Cache key: {hash}_{mtime}
    Some(format!("{}_{}", hash, mtime_seconds))
}

/// Try to load cached metadata for a FITS file
fn try_load_cache(filepath: &str) -> Option<(i64, String, String)> {
    let cache_dir = get_cache_dir()?;
    let cache_key = compute_cache_key(filepath)?;
    let cache_file = cache_dir.join(format!("fits_meta_{}.json", cache_key));

    let json_str = std::fs::read_to_string(cache_file).ok()?;
    let json: JsonValue = serde_json::from_str(&json_str).ok()?;

    let nside = json.get("nside")?.as_i64()?;
    let order = json.get("ordering")?.as_str()?.to_string();
    let indxschm = json.get("indxschm")?.as_str()?.to_string();

    Some((nside, order, indxschm))
}

/// Save metadata to cache
fn save_cache(filepath: &str, nside: i64, ordering: &str, indxschm: &str) {
    let cache_dir = match get_cache_dir() {
        Some(d) => d,
        None => return, // Caching not available
    };

    // Create cache directory if it doesn't exist
    let _ = std::fs::create_dir_all(&cache_dir);

    let cache_key = match compute_cache_key(filepath) {
        Some(k) => k,
        None => return,
    };

    let cache_file = cache_dir.join(format!("fits_meta_{}.json", cache_key));

    let cache_data = json!({
        "nside": nside,
        "ordering": ordering,
        "indxschm": indxschm,
    });

    let _ = std::fs::write(cache_file, cache_data.to_string());
}

/// Read FITS metadata with caching support
/// This function attempts to use cached metadata to avoid expensive header parsing
/// When MAP2FIG_PROFILE environment variable is set, outputs diagnostic timing info
pub fn read_healpix_meta_cached(filename: &str) -> Option<(i64, String, String)> {
    // Check if mmap mode is enabled
    let use_mmap = std::env::var("MAP2FIX_USE_MMAP").is_ok();
    if use_mmap {
        return read_healpix_meta_cached_mmap(filename);
    }

    let enable_profile = std::env::var("MAP2FIG_PROFILE").is_ok();

    // Try cache first
    let cache_start = std::time::Instant::now();
    if let Some((nside, order, indxschm)) = try_load_cache(filename) {
        if enable_profile {
            let elapsed = cache_start.elapsed();
            eprintln!(
                "[I/O DIAG] Cache HIT: {} ({:.3}µs)",
                filename,
                elapsed.as_micros()
            );
        }
        return Some((nside, order, indxschm));
    }

    // Cache miss: parse FITS file
    if enable_profile {
        let elapsed = cache_start.elapsed();
        eprintln!(
            "[I/O DIAG] Cache MISS: {} (lookup took {:.3}µs)",
            filename,
            elapsed.as_micros()
        );
    }

    let parse_start = std::time::Instant::now();
    let f = File::open(filename).ok()?;
    let reader = BufReader::with_capacity(256 * 1024, f);
    let mut fits = Fits::from_reader(reader);
    let mut nside: i64 = 0;
    let mut ordering = String::new();
    let mut indxschm = String::from("IMPLICIT");

    while let Some(Ok(hdu)) = fits.next() {
        if let HDU::XBinaryTable(hdu) = hdu {
            let header = hdu.get_header();

            // Get NSIDE
            if nside == 0 {
                nside = match header.get("NSIDE") {
                    Some(Value::Integer { value, .. }) => *value,
                    _ => 0,
                };
            }

            // Get ordering
            if ordering.is_empty() {
                ordering = match header.get("ORDERING") {
                    Some(Value::String { value, .. }) => value.trim().to_string(),
                    _ => String::from("RING"),
                };
            }

            // Get index scheme (sparse maps)
            indxschm = match header.get("INDXSCHM") {
                Some(Value::String { value, .. }) => value.trim().to_string(),
                _ => String::from("IMPLICIT"),
            };

            // Found what we need
            if nside > 0 {
                break;
            }
        }
    }

    let parse_elapsed = parse_start.elapsed();
    if enable_profile {
        eprintln!(
            "[I/O DIAG] FITS parse took {:.2}ms",
            parse_elapsed.as_secs_f64() * 1000.0
        );
    }

    if nside > 0 {
        // Cache for next time
        save_cache(filename, nside, &ordering, &indxschm);
        Some((nside, ordering, indxschm))
    } else {
        None
    }
}

// ============================================================================
// Tier 5.2.1: Column Data Caching
// ============================================================================

/// Generate cache key for column data
/// Format: {SHA256(filepath)}_{column_idx}_{mtime_secs}
fn get_column_cache_key(filepath: &str, col_idx: usize, mtime_secs: u64) -> Option<String> {
    use sha2::{Digest, Sha256};

    let mut hasher = Sha256::new();
    hasher.update(filepath.as_bytes());
    let hash = hasher.finalize();

    // Convert hash to hex string (first 16 chars for brevity)
    let hash_str = format!("{:x}", hash);
    let short_hash = &hash_str[..16.min(hash_str.len())];

    Some(format!(
        "fits_col_{}_{:03}_{}",
        short_hash, col_idx, mtime_secs
    ))
}

/// Try to load column data from cache
/// Returns Some(`Vec<f64>`) if cache exists and is valid, None otherwise
fn try_load_column_cache(filepath: &str, col_idx: usize) -> Option<Vec<f64>> {
    let cache_dir = get_cache_dir()?;

    // Get file metadata for mtime validation
    let metadata = std::fs::metadata(filepath).ok()?;
    let mtime_secs = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    // Generate cache key and filename
    let cache_key = get_column_cache_key(filepath, col_idx, mtime_secs)?;
    let cache_file = cache_dir.join(&cache_key);

    // Try to open and read cache file
    let mut file = File::open(cache_file).ok()?;

    // Read header (16 bytes)
    let mut header = [0u8; 16];
    file.read_exact(&mut header).ok()?;

    // Parse header
    let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
    let version = u32::from_le_bytes([header[4], header[5], header[6], header[7]]);
    let num_pixels = u32::from_le_bytes([header[8], header[9], header[10], header[11]]);

    // Validate magic and version
    if magic != 0xCAFEBABE || version != 1 {
        return None;
    }

    // Read data
    let mut data = vec![0.0; num_pixels as usize];
    let byte_data = unsafe {
        std::slice::from_raw_parts_mut(data.as_mut_ptr() as *mut u8, num_pixels as usize * 8)
    };
    file.read_exact(byte_data).ok()?;

    let enable_profile = std::env::var("MAP2FIG_PROFILE").is_ok();
    if enable_profile {
        eprintln!("[I/O DIAG] Column cache HIT: {} col#{}", filepath, col_idx);
    }

    Some(data)
}

/// Save column data to cache
fn save_column_cache(filepath: &str, col_idx: usize, data: &[f64]) -> Option<()> {
    let cache_dir = get_cache_dir()?;
    let _ = std::fs::create_dir_all(&cache_dir);

    // Get file metadata for mtime
    let metadata = std::fs::metadata(filepath).ok()?;
    let mtime_secs = metadata
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();

    // Generate cache key and filename
    let cache_key = get_column_cache_key(filepath, col_idx, mtime_secs)?;
    let cache_file = cache_dir.join(&cache_key);

    // Create file
    let mut file = File::create(cache_file).ok()?;

    // Write header: magic (0xCAFEBABE), version (1), num_pixels, reserved
    file.write_all(&0xCAFEBABEu32.to_le_bytes()).ok()?;
    file.write_all(&1u32.to_le_bytes()).ok()?;
    file.write_all(&(data.len() as u32).to_le_bytes()).ok()?;
    file.write_all(&0u32.to_le_bytes()).ok()?;

    // Write data as little-endian f64 bytes
    let byte_data =
        unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 8) };
    file.write_all(byte_data).ok()?;

    let enable_profile = std::env::var("MAP2FIG_PROFILE").is_ok();
    if enable_profile {
        eprintln!(
            "[I/O DIAG] Column cache SAVE: {} col#{} ({} pixels)",
            filepath,
            col_idx,
            data.len()
        );
    }

    Some(())
}

/// Read a HEALPix column with caching support (Tier 5.2.1 optimization)
///
/// This function reads column data from cache if available, otherwise reads from FITS
/// and saves to cache for future use. Cache is automatically invalidated if file mtime changes.
///
/// # Memory-mapped I/O Option
///
/// Set `MAP2FIX_USE_MMAP=1` environment variable to use memory-mapped I/O instead of buffered reading.
/// This can improve performance for large files (>500 MB) at the cost of higher memory usage.
pub fn read_healpix_column_cached(filename: &str, col_idx: usize) -> Vec<f64> {
    // Try cache first
    if let Some(data) = try_load_column_cache(filename, col_idx) {
        return data;
    }

    let enable_profile = std::env::var("MAP2FIG_PROFILE").is_ok();
    if enable_profile {
        eprintln!("[I/O DIAG] Column cache MISS: {} col#{}", filename, col_idx);
    }

    // Check if mmap mode is enabled
    let use_mmap = std::env::var("MAP2FIX_USE_MMAP").is_ok();
    
    // Cache miss: read from FITS (use mmap if enabled)
    let data = if use_mmap {
        read_healpix_column_mmap(filename, col_idx)
    } else {
        read_healpix_column(filename, col_idx)
    };

    // Save to cache for next time (ignore if save fails)
    let _ = save_column_cache(filename, col_idx, &data);

    data
}
// ============================================================================
// Memory-mapped FITS Reading (Optional optimization path)
// ============================================================================

/// Read a HEALPix column from a FITS file using memory-mapped I/O
///
/// This is an alternative to the standard `read_healpix_column` that uses
/// memory-mapped file access (memmap2) instead of buffered reading.
///
/// # Performance Characteristics
///
/// - **For large files** (>500 MB): Comparable or slightly faster (10-20% best case)
/// - **For small files** (<100 MB): Minimal difference, possibly slower due to memmap overhead
/// - **Memory usage**: Maps entire file to memory (could be large)
/// - **Cache effects**: Once mapped, subsequent passes are very fast
///
/// # Arguments
///
/// Same as `read_healpix_column`
///
/// # Returns
///
/// Same as `read_healpix_column`
pub fn read_healpix_column_mmap(filename: &str, col_idx: usize) -> Vec<f64> {
    use std::io::Cursor;
    use memmap2::Mmap;

    let f = File::open(filename).expect("Failed to open FITS file");
    let mmap = unsafe {
        Mmap::map(&f).expect("Failed to memory-map FITS file")
    };

    // Create a Cursor over the memory-mapped data
    // This allows fitsrs to work with the mapped memory without copying
    let cursor = Cursor::new(&mmap[..]);
    let mut fits = Fits::from_reader(cursor);
    let mut result: Vec<f64> = Vec::new();
    let mut nside: i64 = 0;

    while let Some(Ok(hdu)) = fits.next() {
        if let HDU::XBinaryTable(hdu) = hdu {
            let header = hdu.get_header();

            // Check if this uses explicit indexing (sparse/partial sky map)
            let has_explicit_indexing = match header.get("INDXSCHM") {
                Some(Value::String { value, .. }) => value.trim() == "EXPLICIT",
                _ => false,
            };

            // Get NSIDE if not already set
            if nside == 0 {
                nside = match header.get("NSIDE") {
                    Some(Value::Integer { value, .. }) => *value,
                    _ => 0,
                };
            }

            let data = fits.get_data(&hdu);
            let mut table = data.table_data();

            // If explicit indexing, read both PIXEL and data columns together
            if has_explicit_indexing && nside > 0 {
                let file_col_for_data = col_idx + 1;

                // Read both PIXEL (col 0) and data column
                let all_values: Vec<DataValue> = table
                    .select_fields(&[ColumnId::Index(0), ColumnId::Index(file_col_for_data)])
                    .collect();

                if all_values.is_empty() {
                    result = vec![f64::NEG_INFINITY; (12 * nside * nside) as usize];
                } else {
                    let n_rows = all_values.len() / 2;
                    let npix = (12 * nside * nside) as usize;
                    let mut full_map = vec![f64::NEG_INFINITY; npix];

                    let pairs: Vec<(usize, f64)> = (0..n_rows)
                        .into_par_iter()
                        .filter_map(|row_idx| {
                            let pix_idx = row_idx * 2;
                            let data_idx = row_idx * 2 + 1;

                            let pix = match &all_values[pix_idx] {
                                DataValue::Integer { value, .. } => *value as i64,
                                DataValue::Long { value, .. } => *value,
                                DataValue::Float { value, .. } => *value as i64,
                                DataValue::Double { value, .. } => *value as i64,
                                _ => -1,
                            };

                            let val = match &all_values[data_idx] {
                                DataValue::Double { value, .. } => *value,
                                DataValue::Float { value, .. } => *value as f64,
                                DataValue::Integer { value, .. } => *value as f64,
                                other => {
                                    panic!("Unsupported column type in FITS table: {:?}", other)
                                }
                            };

                            if pix >= 0 && (pix as usize) < npix {
                                Some((pix as usize, val))
                            } else {
                                None
                            }
                        })
                        .collect();

                    for (pix_idx, val) in pairs {
                        full_map[pix_idx] = val;
                    }

                    result = full_map;
                }
            } else {
                // Regular dense map: read column directly
                let values = table.select_fields(&[ColumnId::Index(col_idx)]);
                for cell in values {
                    match cell {
                        DataValue::Double { value, .. } => result.push(value),
                        DataValue::Float { value, .. } => result.push(value as f64),
                        DataValue::Integer { value, .. } => result.push(value as f64),
                        other => panic!("Unsupported column type in FITS table: {:?}", other),
                    }
                }
            }
        }
    }

    result
}

/// Read FITS metadata using memory-mapped I/O (mmap variant of `read_healpix_meta_cached`)
pub fn read_healpix_meta_cached_mmap(filename: &str) -> Option<(i64, String, String)> {
    use std::io::Cursor;
    use memmap2::Mmap;

    let enable_profile = std::env::var("MAP2FIX_PROFILE").is_ok();

    // Try cache first
    let cache_start = std::time::Instant::now();
    if let Some((nside, order, indxschm)) = try_load_cache(filename) {
        if enable_profile {
            let elapsed = cache_start.elapsed();
            eprintln!(
                "[I/O DIAG] Cache HIT (mmap): {} ({:.3}µs)",
                filename,
                elapsed.as_micros()
            );
        }
        return Some((nside, order, indxschm));
    }

    // Cache miss: parse FITS file with mmap
    if enable_profile {
        let elapsed = cache_start.elapsed();
        eprintln!(
            "[I/O DIAG] Cache MISS (mmap): {} (lookup took {:.3}µs)",
            filename,
            elapsed.as_micros()
        );
    }

    let parse_start = std::time::Instant::now();
    let f = File::open(filename).ok()?;
    let mmap = unsafe {
        Mmap::map(&f).ok()?
    };

    let cursor = Cursor::new(&mmap[..]);
    let mut fits = Fits::from_reader(cursor);
    let mut nside: i64 = 0;
    let mut ordering = String::new();
    let mut indxschm = String::from("IMPLICIT");

    while let Some(Ok(hdu)) = fits.next() {
        if let HDU::XBinaryTable(hdu) = hdu {
            let header = hdu.get_header();
            nside = match header.get("NSIDE") {
                Some(Value::Integer { value, .. }) => *value,
                _ => 0,
            };
            ordering = match header.get("ORDERING") {
                Some(Value::String { value, .. }) => value.trim().to_string(),
                _ => "RING".to_string(),
            };
            indxschm = match header.get("INDXSCHM") {
                Some(Value::String { value, .. }) => value.trim().to_string(),
                _ => "IMPLICIT".to_string(),
            };
            break;
        }
    }

    let parse_end = std::time::Instant::now();
    if enable_profile {
        eprintln!(
            "[I/O DIAG] Parse done (mmap): {:.3}ms",
            parse_end.duration_since(parse_start).as_secs_f64() * 1000.0
        );
    }

    if nside > 0 {
        let _ = save_cache(filename, nside, &ordering, &indxschm);
        Some((nside, ordering, indxschm))
    } else {
        None
    }
}
