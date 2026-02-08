#!/usr/bin/env python3
"""
Compare PDF and PNG outputs by converting PDF to PNG and computing differences.
"""
import subprocess
import sys
from pathlib import Path

def pdf_to_png(pdf_path, png_path, dpi=300):
    """Convert PDF to PNG using pdftoppm."""
    try:
        subprocess.run(
            ["pdftoppm", "-singlefile", "-png", "-r", str(dpi), pdf_path, png_path.with_suffix('')],
            check=True,
            capture_output=True
        )
        return True
    except subprocess.CalledProcessError as e:
        print(f"Error converting PDF to PNG: {e.stderr.decode()}")
        return False
    except FileNotFoundError:
        print("pdftoppm not found. Install poppler-utils.")
        return False

def compare_images(img1_path, img2_path, diff_path):
    """Compare two images using ImageMagick."""
    try:
        result = subprocess.run(
            ["compare", "-metric", "mae", img1_path, img2_path, diff_path],
            capture_output=True,
            text=True
        )
        # compare returns non-zero but still produces output
        output = result.stdout + result.stderr
        print(f"Image comparison result:\n{output}")
        return diff_path.exists()
    except FileNotFoundError:
        print("ImageMagick 'compare' not found. Install imagemagick.")
        return False

def main():
    if len(sys.argv) < 2:
        print("Usage: python compare_outputs.py <base_filename_without_extension>")
        print("Example: python compare_outputs.py test_final")
        print("         This will compare test_final.pdf and test_final.png")
        sys.exit(1)
    
    base = Path(sys.argv[1])
    pdf_path = base.with_suffix('.pdf')
    png_path = base.with_suffix('.png')
    
    if not pdf_path.exists():
        print(f"Error: {pdf_path} not found")
        sys.exit(1)
    
    if not png_path.exists():
        print(f"Error: {png_path} not found")
        sys.exit(1)
    
    print(f"Comparing {pdf_path} and {png_path}...")
    
    # Convert PDF to PNG for comparison
    pdf_as_png = base.with_name(f"{base.name}_pdf_converted.png")
    print(f"Converting PDF to PNG at 300 DPI...")
    if not pdf_to_png(str(pdf_path), pdf_as_png, dpi=300):
        sys.exit(1)
    
    # Compare
    diff_path = base.with_name(f"{base.name}_diff.png")
    print(f"Comparing images...")
    compare_images(str(pdf_as_png), str(png_path), diff_path)
    
    if diff_path.exists():
        print(f"Difference image saved to {diff_path}")
    
    print(f"PDF (converted):  {pdf_as_png}")
    print(f"PNG:              {png_path}")

if __name__ == "__main__":
    main()
