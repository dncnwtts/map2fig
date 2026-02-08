#!/usr/bin/env python3
"""
Compare PDF and PNG outputs visually.

This script converts the PDF to PNG using pdftoppm and then compares
the two images to identify any visual differences.
"""

import subprocess
import sys
from pathlib import Path
from PIL import Image
import numpy as np

def convert_pdf_to_png(pdf_path, dpi=150):
    """Convert PDF to PNG using pdftoppm."""
    print(f"Converting {pdf_path} to PNG at {dpi} DPI...")
    
    # Remove .pdf extension and append _converted
    output_base = pdf_path.parent / (pdf_path.stem + '_converted')
    
    result = subprocess.run(
        ['pdftoppm', '-png', '-r', str(dpi), str(pdf_path), str(output_base)],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"Error converting PDF: {result.stderr}")
        return None
    
    # pdftoppm adds -1 suffix for single page PDFs
    pdf_png = output_base.parent / (output_base.name + '-1.png')
    if pdf_png.exists():
        return pdf_png
    
    # Try without suffix
    plain_png = output_base.parent / (output_base.name + '.png')
    if plain_png.exists():
        return plain_png
    
    print(f"Converted PNG not found")
    return None


def compare_images(img1_path, img2_path, output_dir=None):
    """Compare two images and report differences."""
    print(f"\nComparing:")
    print(f"  Image 1: {img1_path}")
    print(f"  Image 2: {img2_path}")
    
    img1 = Image.open(img1_path).convert('RGB')
    img2 = Image.open(img2_path).convert('RGB')
    
    # Resize if needed
    if img1.size != img2.size:
        print(f"Size mismatch: {img1.size} vs {img2.size}")
        # Resize img2 to match img1
        img2 = img2.resize(img1.size, Image.Resampling.LANCZOS)
        print(f"Resized image 2 to {img1.size}")
    
    # Convert to numpy arrays
    arr1 = np.array(img1)
    arr2 = np.array(img2)
    
    # Compute difference
    diff = np.abs(arr1.astype(float) - arr2.astype(float))
    max_diff = np.max(diff)
    mean_diff = np.mean(diff)
    
    print(f"\nImage Comparison Results:")
    print(f"  Max pixel difference: {max_diff:.1f}")
    print(f"  Mean pixel difference: {mean_diff:.1f}")
    
    # Count pixels that differ significantly
    threshold = 5  # 5 out of 255
    different_pixels = np.sum(np.max(diff, axis=2) > threshold)
    total_pixels = arr1.shape[0] * arr1.shape[1]
    pct_different = 100.0 * different_pixels / total_pixels
    
    print(f"  Pixels differing by >{threshold}: {different_pixels} ({pct_different:.2f}%)")
    
    # If requested, create a visualization
    if output_dir:
        output_dir = Path(output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)
        
        # Create difference image
        diff_img = Image.fromarray((diff / np.max(diff) * 255).astype(np.uint8).max(axis=2))
        diff_path = output_dir / 'difference.png'
        diff_img.save(diff_path)
        print(f"  Difference visualization saved to: {diff_path}")
        
        # Create side-by-side comparison
        combined = Image.new('RGB', (img1.width * 2, img1.height))
        combined.paste(img1, (0, 0))
        combined.paste(img2, (img1.width, 0))
        combined_path = output_dir / 'comparison.png'
        combined.save(combined_path)
        print(f"  Side-by-side comparison saved to: {combined_path}")
    
    return max_diff < 1.0  # Consider them identical if max diff < 1


def main():
    if len(sys.argv) < 3:
        print("Usage: compare_pdf_png.py <pdf_file> <png_file> [output_dir]")
        sys.exit(1)
    
    pdf_file = Path(sys.argv[1])
    png_file = Path(sys.argv[2])
    output_dir = Path(sys.argv[3]) if len(sys.argv) > 3 else Path('/tmp/pdf_png_compare')
    
    if not pdf_file.exists():
        print(f"PDF file not found: {pdf_file}")
        sys.exit(1)
    
    if not png_file.exists():
        print(f"PNG file not found: {png_file}")
        sys.exit(1)
    
    # Convert PDF to PNG
    pdf_converted = convert_pdf_to_png(pdf_file)
    
    if not pdf_converted:
        print("Failed to convert PDF")
        sys.exit(1)
    
    # Compare
    identical = compare_images(pdf_converted, png_file, output_dir)
    
    if identical:
        print("\n✓ Images are essentially identical!")
    else:
        print("\n✗ Images have notable differences")


if __name__ == '__main__':
    main()
