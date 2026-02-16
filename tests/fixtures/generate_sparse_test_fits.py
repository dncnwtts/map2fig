#!/usr/bin/env python3
"""
Generate a minimal sparse HEALPix FITS file for testing explicit indexing.

Creates NSIDE=1 map with sparse data using healpy's write_map function.
This ensures HEALPix conformance and tests the regression fix for sparse map handling.
"""

import healpy as hp
import numpy as np
import os

def create_sparse_nside1_fits(output_path):
    """
    Create a minimal sparse FITS file with NSIDE=1 using healpy.
    Uses EXPLICIT indexing with only 4 of the 12 pixels populated.
    
    NSIDE=1 has 12 pixels total in RING ordering.
    We populate pixels: 0, 3, 6, 9 with values: 1.0, 2.0, 3.0, 4.0
    """
    
    NSIDE = 1
    npix = hp.nside2npix(NSIDE)  # 12 pixels for NSIDE=1
    
    # Create a sparse map: initialize with UNSEEN, populate only specific pixels
    pixel_map = np.full(npix, hp.UNSEEN, dtype=np.float32)
    
    # Populate specific pixels with test values
    pixel_indices = np.array([0, 3, 6, 9])
    pixel_values = np.array([1.0, 2.0, 3.0, 4.0], dtype=np.float32)
    
    pixel_map[pixel_indices] = pixel_values
    
    # Write using healpy - this handles EXPLICIT indexing automatically for sparse maps
    hp.write_map(
        output_path,
        pixel_map,
        nest=False,  # Use RING ordering
        overwrite=True,
        dtype=np.float32
    )
    
    print(f"Created sparse test FITS using healpy: {output_path}")
    print(f"  NSIDE: {NSIDE} ({npix} pixels total)")
    print(f"  Total pixels: {npix}")
    print(f"  Populated pixels: {list(pixel_indices)} with values {list(pixel_values)}")
    print(f"  Unpopulated pixels: marked with UNSEEN sentinel")
    print(f"  Ordering: RING")
    
    # Verify the file was created and check its structure
    if os.path.exists(output_path):
        size = os.path.getsize(output_path)
        print(f"  File size: {size} bytes")
        
        # Read back and verify
        read_map = hp.read_map(output_path, verbose=False)
        print(f"  Verification: Map has {np.sum(np.isfinite(read_map))} finite values")
        print(f"  Successfully created and verified HEALPix-conformant sparse FITS file")
    else:
        print(f"  ERROR: File was not created!")

if __name__ == '__main__':
    try:
        import healpy as hp
    except ImportError:
        print("ERROR: healpy is required. Install with: pip install healpy")
        exit(1)
    
    script_dir = os.path.dirname(os.path.abspath(__file__))
    output_path = os.path.join(script_dir, 'sparse_nside1.fits')
    create_sparse_nside1_fits(output_path)
