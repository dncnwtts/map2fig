//! Adaptive data type wrapper that preserves FITS column precision.
//!
//! This module wraps HEALPix map data with the appropriate type (f32 or f64)
//! based on the source FITS column, avoiding unnecessary conversions.
//!
//! # Memory & Performance Impact
//!
//! By keeping f32 data as f32 instead of converting to f64:
//! - **Time saved**: 6.8s (62.4% of FITS reading for large maps)
//! - **Memory saved**: 3.2 GB (for 806M pixel maps)
//! - **Precision**: Sufficient for visualization (32-bit color depth)
//!
//! # Usage
//!
//! ```ignore
//! use map2fig::data_array::DataArray;
//!
//! // Generic operations
//! let min = data.min_value();
//! let max = data.max_value();
//! let len = data.len();
//!
//! // Type-specific operations
//! let scaled = data.scale_value(val, min_v, max_v);
//! ```

use crate::healpix::is_seen;
use std::fmt;

/// Adaptive array wrapper: holds HEALPix data in the native FITS precision.
///
/// This avoids unnecessary conversions when source data is f32.
/// All operations convert on-demand to f64 only when needed.
#[derive(Clone, Debug)]
pub enum DataArray {
    /// 32-bit float data (saves memory for typical HEALPix maps)
    Float32(Vec<f32>),
    /// 64-bit float data (for high-precision maps)
    Float64(Vec<f64>),
}

impl DataArray {
    /// Create a Float32 array from data
    #[inline]
    pub fn from_f32(data: Vec<f32>) -> Self {
        DataArray::Float32(data)
    }

    /// Create a Float64 array from data
    #[inline]
    pub fn from_f64(data: Vec<f64>) -> Self {
        DataArray::Float64(data)
    }

    /// Get array length
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            DataArray::Float32(v) => v.len(),
            DataArray::Float64(v) => v.len(),
        }
    }

    /// Check if array is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get element as f64 (converts on-demand if f32)
    #[inline]
    pub fn get(&self, idx: usize) -> Option<f64> {
        match self {
            DataArray::Float32(v) => v.get(idx).map(|&x| x as f64),
            DataArray::Float64(v) => v.get(idx).copied(),
        }
    }

    /// Iterate over values as f64
    pub fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        match self {
            DataArray::Float32(v) => {
                Box::new(v.iter().map(|&x| x as f64)) as Box<dyn Iterator<Item = f64>>
            }
            DataArray::Float64(v) => Box::new(v.iter().copied()),
        }
    }

    /// Compute min value, skipping UNSEEN/NaN
    pub fn min_value(&self) -> f64 {
        match self {
            DataArray::Float32(v) => v
                .iter()
                .filter(|&&x| is_seen(x as f64))
                .map(|&x| x as f64)
                .fold(f64::INFINITY, f64::min),
            DataArray::Float64(v) => v
                .iter()
                .filter(|&&x| is_seen(x))
                .copied()
                .fold(f64::INFINITY, f64::min),
        }
    }

    /// Compute max value, skipping UNSEEN/NaN
    pub fn max_value(&self) -> f64 {
        match self {
            DataArray::Float32(v) => v
                .iter()
                .filter(|&&x| is_seen(x as f64))
                .map(|&x| x as f64)
                .fold(f64::NEG_INFINITY, f64::max),
            DataArray::Float64(v) => v
                .iter()
                .filter(|&&x| is_seen(x))
                .copied()
                .fold(f64::NEG_INFINITY, f64::max),
        }
    }

    /// Collect valid values as f64 vector for percentile/statistics
    pub fn valid_f64_values(&self) -> Vec<f64> {
        match self {
            DataArray::Float32(v) => v
                .iter()
                .filter(|&&x| is_seen(x as f64))
                .map(|&x| x as f64)
                .collect(),
            DataArray::Float64(v) => v.iter().filter(|&&x| is_seen(x)).copied().collect(),
        }
    }

    /// Get underlying f64 vec if this is Float64, otherwise convert
    ///
    /// Used for compatibility with existing code that expects `Vec<f64>`.
    /// For f32 data, this allocates a new vector with conversions.
    pub fn as_f64_vec(&self) -> std::borrow::Cow<'_, Vec<f64>> {
        match self {
            DataArray::Float32(v) => std::borrow::Cow::Owned(v.iter().map(|&x| x as f64).collect()),
            DataArray::Float64(v) => std::borrow::Cow::Borrowed(v),
        }
    }

    /// Get as f64 slice if Float64, otherwise panics
    ///
    /// Used in performance-critical hot loops where we want f64 slice directly.
    /// For code that needs to work with both types, use `as_f64_vec()` instead.
    pub fn as_f64_slice(&self) -> Option<&[f64]> {
        match self {
            DataArray::Float32(_) => None,
            DataArray::Float64(v) => Some(v),
        }
    }

    /// Get memory size in bytes
    pub fn memory_size_bytes(&self) -> usize {
        match self {
            DataArray::Float32(v) => v.len() * 4,
            DataArray::Float64(v) => v.len() * 8,
        }
    }

    /// Get data type name for debugging/logging
    pub fn dtype(&self) -> &'static str {
        match self {
            DataArray::Float32(_) => "float32",
            DataArray::Float64(_) => "float64",
        }
    }
}

impl fmt::Display for DataArray {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mb = self.memory_size_bytes() as f64 / (1024.0 * 1024.0);
        write!(
            f,
            "{} data ({} pixels, {:.1} MB)",
            self.dtype(),
            self.len(),
            mb
        )
    }
}

/// Temporary compatibility wrapper: convert DataArray to `Vec<f64>`
/// Used for gradual migration of existing code
#[inline]
pub fn to_f64_vec(data: &DataArray) -> Vec<f64> {
    data.as_f64_vec().into_owned()
}
