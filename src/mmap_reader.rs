//! Memory-mapped FITS file reader
//!
//! Provides a memory-mapped implementation of the Read and BufRead traits for FITS files,
//! eliminating buffer copy overhead for large sequential reads.
//!
//! # Usage
//!
//! ```ignore
//! let file = File::open("data.fits")?;
//! let mmap = MmapFitsReader::new(file)?;
//! let fits = Fits::from_reader(mmap);
//! ```

use memmap2::Mmap;
use std::fs::File;
use std::io::{self, BufRead, Read, Seek, SeekFrom};
use std::path::Path;

/// Memory-mapped FITS file reader
///
/// Wraps memmap2::Mmap to provide Read, BufRead, and Seek interfaces for FITS files.
/// Eliminates buffer copies for large files by mapping the entire file to memory.
pub struct MmapFitsReader {
    mmap: Mmap,
    position: usize,
}

impl MmapFitsReader {
    /// Create a new memory-mapped FITS reader from a file
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or mapped.
    pub fn new(file: File) -> io::Result<Self> {
        // Safety: FITS files are read-only during parsing
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Self { mmap, position: 0 })
    }

    /// Create a memory-mapped FITS reader from a file path
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be opened or mapped.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = File::open(path)?;
        Self::new(file)
    }

    /// Get the current position in the file
    pub fn position(&self) -> usize {
        self.position
    }

    /// Get the total size of the mapped file
    pub fn len(&self) -> usize {
        self.mmap.len()
    }

    /// Check if the file is empty
    pub fn is_empty(&self) -> bool {
        self.mmap.is_empty()
    }
}

impl Read for MmapFitsReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let available = self.mmap.len() - self.position;
        let to_read = std::cmp::min(buf.len(), available);

        if to_read == 0 {
            return Ok(0); // EOF
        }

        // Copy from memory-mapped region to output buffer
        buf[..to_read].copy_from_slice(&self.mmap[self.position..self.position + to_read]);
        self.position += to_read;

        Ok(to_read)
    }
}

impl Seek for MmapFitsReader {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let new_position = match pos {
            SeekFrom::Start(offset) => offset as usize,
            SeekFrom::Current(offset) => {
                let current = self.position as i64;
                (current + offset) as usize
            }
            SeekFrom::End(offset) => {
                let end = self.mmap.len() as i64;
                (end + offset) as usize
            }
        };

        if new_position > self.mmap.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "seek position out of bounds",
            ));
        }

        self.position = new_position;
        Ok(self.position as u64)
    }
}

impl BufRead for MmapFitsReader {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        // Return a slice from current position to end of file
        // This allows callers to read without copying
        let available = &self.mmap[self.position..];
        if available.is_empty() {
            Ok(&[])
        } else {
            Ok(available)
        }
    }

    fn consume(&mut self, amt: usize) {
        // Move position forward by the consumed amount
        let amt = std::cmp::min(amt, self.mmap.len().saturating_sub(self.position));
        self.position += amt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmap_basic_read() {
        // Create a temporary file with known content
        let mut temp = tempfile::NamedTempFile::new().unwrap();
        use std::io::Write;
        temp.write_all(b"Hello, World!").unwrap();
        temp.flush().unwrap();

        let path = temp.path();
        let mut reader = MmapFitsReader::open(path).unwrap();

        let mut buf = [0u8; 5];
        assert_eq!(reader.read(&mut buf).unwrap(), 5);
        assert_eq!(&buf, b"Hello");

        assert_eq!(reader.position(), 5);
    }

    #[test]
    fn test_mmap_seek() {
        let mut temp = tempfile::NamedTempFile::new().unwrap();
        use std::io::Write;
        temp.write_all(b"0123456789").unwrap();
        temp.flush().unwrap();

        let path = temp.path();
        let mut reader = MmapFitsReader::open(path).unwrap();

        // Seek to position 5
        reader.seek(SeekFrom::Start(5)).unwrap();
        assert_eq!(reader.position(), 5);

        let mut buf = [0u8; 2];
        assert_eq!(reader.read(&mut buf).unwrap(), 2);
        assert_eq!(&buf, b"56");
    }

    #[test]
    fn test_mmap_size() {
        let mut temp = tempfile::NamedTempFile::new().unwrap();
        use std::io::Write;
        temp.write_all(b"test").unwrap();
        temp.flush().unwrap();

        let path = temp.path();
        let reader = MmapFitsReader::open(path).unwrap();
        assert_eq!(reader.len(), 4);
    }
}
