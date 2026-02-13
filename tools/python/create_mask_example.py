#!/usr/bin/env python3
"""
Create example FITS mask files for use with healpix_plotter.

This script demonstrates how to create binary mask FITS files that can be
applied to HEALPix maps using the --mask-file option.

Mask File Format:
  - Binary FITS table with a single column of pixel data
  - Data values: 0 or near-0 = masked (hidden), >0.5 = valid (shown)
  - Size: 12 * nside^2 pixels
  - Supported data types: float, double, integer

Examples:
  python3 create_mask_example.py --nside 128 --mask-galactic-plane 10 -o mask.fits
  python3 create_mask_example.py --nside 256 --mask-disk 266.5 -28.9 30
  python3 create_mask_example.py --nside 512 --mask-uniform 0.5
"""

import numpy as np
import argparse
from astropy.io import fits
from pathlib import Path


def nside_to_pixel_count(nside):
    """Get pixel count from NSIDE."""
    return 12 * nside * nside


def validate_nside(nside):
    """Check that NSIDE is a valid power of 2."""
    return (nside & (nside - 1)) == 0 and nside > 0


def create_simple_mask_galactic_plane(nside, latitude_range=10):
    """
    Create mask hiding pixels near Galactic plane (simple version).
    
    This is a basic implementation that masks pixels without healpy.
    For precise pixel-by-pixel masking, use healpy (requires installation).
    
    Args:
        nside: HEALPix resolution parameter
        latitude_range: Hide pixels within ±latitude_range degrees of equator
    
    Returns:
        numpy array of mask values (0 = masked, 1 = valid)
    """
    print(f"Creating Galactic plane mask (±{latitude_range}°, simplified)...")
    npix = nside_to_pixel_count(nside)
    
    # Simple approximation: pixels are arranged in rings by latitude
    # This is not perfectly accurate but works for demonstrations
    mask = np.ones(npix)
    
    # Approximate: southern hemisphere first ~latitude_range*2/180 pixels per ring
    # This is a rough approximation without full HEALPix math
    pixels_per_band = int(npix * latitude_range / 90)
    
    # Mask central bands (approximate Galactic equator)
    start_idx = (npix // 2) - pixels_per_band // 2
    end_idx = start_idx + pixels_per_band
    mask[start_idx:end_idx] = 0
    
    return mask


def create_random_mask(nside, fraction=0.1):
    """
    Create mask with random pixels (for testing).
    
    Args:
        nside: HEALPix resolution
        fraction: Fraction of pixels to mask (0.0 to 1.0)
    
    Returns:
        numpy array of mask values
    """
    print(f"Creating random mask ({fraction*100:.1f}% masked)...")
    npix = nside_to_pixel_count(nside)
    mask = np.ones(npix)
    
    # Randomly select pixels to mask
    n_masked = int(npix * fraction)
    masked_indices = np.random.choice(npix, n_masked, replace=False)
    mask[masked_indices] = 0
    
    return mask


def create_uniform_mask(nside, fill_fraction=0.5):
    """
    Create mask with uniform masking pattern (for testing).
    
    Args:
        nside: HEALPix resolution
        fill_fraction: Fraction of pixels to keep valid (0.0 to 1.0)
    
    Returns:
        numpy array of mask values
    """
    print(f"Creating uniform mask ({fill_fraction*100:.1f}% valid)...")
    npix = nside_to_pixel_count(nside)
    mask = np.ones(npix)
    
    # Mask alternating regions
    mask[int(npix * (1 - fill_fraction)):] = 0
    
    return mask


def create_patch_mask(nside, center_lon=266.5, center_lat=-28.9, radius=30):
    """
    Create mask hiding a circular patch (simple distance-based).
    
    Args:
        nside: HEALPix resolution
        center_lon: Patch center longitude (degrees)
        center_lat: Patch center latitude (degrees)
        radius: Patch radius (degrees)
    
    Returns:
        numpy array of mask values
    
    Note: This is a simplified implementation. For precise masking,
    use healpy for accurate HEALPix pixel coordinates.
    """
    print(f"Creating patch mask at ({center_lon}°, {center_lat}°) "
          f"with radius {radius}° (simplified)...")
    npix = nside_to_pixel_count(nside)
    
    # Simplified: approximate pixel positions without healpy
    mask = np.ones(npix)
    
    # Approximate disk masking (not accurate without pixel coordinates)
    approx_pixels_in_disk = int(npix * (radius / 180)**2)
    start_idx = max(0, (npix // 2) - (approx_pixels_in_disk // 2))
    end_idx = min(npix, start_idx + approx_pixels_in_disk)
    mask[start_idx:end_idx] = 0
    
    return mask


def write_mask_fits(mask, output_file, nside):
    """
    Write mask array to FITS binary table.
    
    Args:
        mask: numpy array of mask values
        output_file: Path for output FITS file
        nside: HEALPix resolution (stored in header)
    """
    # Create binary table HDU
    col = fits.Column(
        name='mask',
        format='D',  # Double (64-bit float)
        array=mask
    )
    hdu = fits.BinTableHDU.from_columns([col])
    
    # Add header keywords for metadata
    hdu.header['NSIDE'] = nside
    hdu.header['PIXTYPE'] = 'HEALPIX'
    hdu.header['ORDERING'] = 'RING'
    
    # Write to file
    hdu.writeto(output_file, overwrite=True)
    print(f"Wrote mask to {output_file}")


def main():
    parser = argparse.ArgumentParser(
        description='Create FITS mask files for healpix_plotter'
    )
    parser.add_argument('--nside', type=int, default=128,
                        help='HEALPix resolution: 1, 2, 4, 8, 16, 32, 64, 128, '
                             '256, 512, 1024, 2048, 4096, 8192 (default: 128)')
    parser.add_argument('-o', '--output', default=None,
                        help='Output FITS file (auto-named if not specified)')
    
    # Mask type options
    parser.add_argument('--mask-galactic-plane', type=float, metavar='DEGREES',
                        help='Hide pixels within ±N degrees of Galactic plane '
                             '(simplified implementation)')
    parser.add_argument('--mask-uniform', type=float, metavar='FILL_FRACTION',
                        help='Hide pixels uniformly (0.0-1.0, fraction to keep)')
    parser.add_argument('--mask-random', type=float, metavar='MASK_FRACTION',
                        help='Randomly mask fraction of pixels (0.0-1.0)')
    parser.add_argument('--mask-patch', nargs=3, type=float,
                        metavar=('LON', 'LAT', 'RADIUS'),
                        help='Hide circular patch region (simplified, lon lat radius in degrees)')
    
    args = parser.parse_args()
    
    # Validate NSIDE
    if not validate_nside(args.nside):
        print(f"ERROR: NSIDE={args.nside} is not a power of 2")
        print("Valid values: 1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192")
        return 1
    
    # Validate arguments
    mask_types = sum([
        args.mask_galactic_plane is not None,
        args.mask_uniform is not None,
        args.mask_random is not None,
        args.mask_patch is not None,
    ])
    
    if mask_types == 0:
        print("ERROR: Specify a mask type:")
        print("  --mask-galactic-plane N    (hide ±N degrees from Galactic equator)")
        print("  --mask-uniform FRACTION    (uniformly mask pixels)")
        print("  --mask-random FRACTION     (randomly mask fraction of pixels)")
        print("  --mask-patch LON LAT R     (hide circular patch)")
        return 1
    
    if mask_types > 1:
        print("ERROR: Specify only one mask type")
        return 1
    
    # Generate mask
    if args.mask_galactic_plane is not None:
        mask = create_simple_mask_galactic_plane(args.nside, args.mask_galactic_plane)
        if not args.output:
            args.output = f"mask_galactic_plane_{args.mask_galactic_plane:g}deg_n{args.nside:05d}.fits"
    
    elif args.mask_uniform is not None:
        if not 0 <= args.mask_uniform <= 1:
            print(f"ERROR: Fill fraction must be 0.0-1.0, got {args.mask_uniform}")
            return 1
        mask = create_uniform_mask(args.nside, args.mask_uniform)
        if not args.output:
            args.output = f"mask_uniform_{args.mask_uniform:.0%}_n{args.nside:05d}.fits"
    
    elif args.mask_random is not None:
        if not 0 <= args.mask_random <= 1:
            print(f"ERROR: Mask fraction must be 0.0-1.0, got {args.mask_random}")
            return 1
        mask = create_random_mask(args.nside, args.mask_random)
        if not args.output:
            args.output = f"mask_random_{args.mask_random:.0%}_n{args.nside:05d}.fits"
    
    elif args.mask_patch is not None:
        lon, lat, radius = args.mask_patch
        mask = create_patch_mask(args.nside, lon, lat, radius)
        if not args.output:
            args.output = f"mask_patch_r{radius:g}deg_n{args.nside:05d}.fits"
    
    # Write output
    write_mask_fits(mask, args.output, args.nside)
    
    # Print statistics
    npix_masked = np.count_nonzero(mask == 0)
    npix_total = len(mask)
    pct_masked = 100 * npix_masked / npix_total
    print(f"\nMask statistics:")
    print(f"  NSIDE: {args.nside}")
    print(f"  Total pixels: {npix_total:,}")
    print(f"  Masked pixels: {npix_masked:,} ({pct_masked:.1f}%)")
    print(f"  Valid pixels: {npix_total - npix_masked:,} ({100 - pct_masked:.1f}%)")
    
    # Usage example
    print(f"\nUsage with healpix_plotter:")
    print(f"  cargo run -- -f data.fits --mask-file {args.output} -o output.pdf")
    
    return 0


if __name__ == '__main__':
    exit(main())

