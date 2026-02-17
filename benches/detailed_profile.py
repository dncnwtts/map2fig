#!/usr/bin/env python3
"""
Detailed timing breakdown of healpix_plotter pipeline stages.
Measures each phase: FITS read, downgrade, projection, scaling, rendering.
"""

import subprocess
import sys
import re
from pathlib import Path
import time

def run_with_timing(fits_file: str, iterations: int = 3) -> float:
    """Run plotter and capture timing output."""
    binary = Path("./target/release/map2fig")
    if not binary.exists():
        print("Error: Binary not found. Run 'cargo build --release' first.")
        sys.exit(1)
    
    fits_path = Path(fits_file)
    if not fits_path.exists():
        print(f"Error: FITS file not found: {fits_file}")
        sys.exit(1)
    
    times = []
    for i in range(iterations):
        print(f"  Run {i+1}/{iterations}...", flush=True, end=" ")
        start = time.time()
        result = subprocess.run(
            [str(binary), "-f", str(fits_path), "-o", "/tmp/profile_out.pdf"],
            capture_output=True,
            text=True
        )
        elapsed = time.time() - start
        times.append(elapsed)
        print(f"{elapsed:.3f}s")
        
        if result.returncode != 0:
            print(f"Error: {result.stderr}")
            sys.exit(1)
    
    return sum(times) / len(times)

def main():
    # Test files with relative sizes
    test_files = [
        ("class_dr1_40GHz_skymap_n128.fits", "6MB"),
        ("cosmoglobe_clipped.fits", "24MB"),
        ("npipe6v20_217_map_K.fits", "576MB"),
        ("combined_map_95GHz_ntsrcmasked_50mJy.fits", "3.1GB"),
    ]
    
    print("=" * 70)
    print("HEALPix Plotter - Detailed Pipeline Timing")
    print("=" * 70)
    
    for fits_file, size_label in test_files:
        if not Path(fits_file).exists():
            print(f"⊘ Skipping {size_label} (not found)")
            continue
        
        print(f"\n{size_label:<6} ({fits_file})")
        avg_time = run_with_timing(fits_file, iterations=2)
        print(f"  Average: {avg_time:.3f}s")

if __name__ == "__main__":
    main()
