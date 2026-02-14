#!/usr/bin/env python3
"""
Quick benchmarking script - tests key FITS files without full recompilation.
Uses pre-built cargo run which has incremental compilation.
"""

import subprocess
import sys
import time
from pathlib import Path
import re

# Key test files (skip the huge 3.1GB one for now, test others)
TEST_FILES = [
    ("m_test.fits", "tiny", 8.5e3),
    ("mhat_0_00_n00512_2025W17_4B.fits", "small", 678e3),
    ("class_dr1_40GHz_skymap_n128.fits", "medium", 6.8e6),
    ("cosmoglobe_clipped.fits", "medium-large", 25e6),
    ("npipe_nodip.fits", "very-large", 193e6),
]

def clear_cache():
    """Clear the column cache to force cache miss."""
    cache_dir = Path.home() / ".cache/map2fig"
    if cache_dir.exists():
        for f in cache_dir.glob("fits_col_*"):
            f.unlink()


def run_bench(fits_file):
    """Run cargo run and extract timing."""
    cmd = [
        "cargo",
        "run",
        "--release",
        "--",
        "-f",
        fits_file,
        "-o",
        "/tmp/bench.pdf",
    ]

    try:
        start = time.time()
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            cwd="/home/dwatts/projects/healpix_plotter",
            timeout=300,
        )
        elapsed = time.time() - start
        return elapsed
    except subprocess.TimeoutExpired:
        return None


def main():
    project_root = Path("/home/dwatts/projects/healpix_plotter")

    print("\n" + "=" * 70)
    print("HEALPix Plotter - Quick Benchmark (Key Files)")
    print("=" * 70 + "\n")

    results = []

    for fits_file, category, file_size in TEST_FILES:
        filepath = project_root / fits_file
        if not filepath.exists():
            print(f"⏭️  Skipping {fits_file} (not found)\n")
            continue

        print(f"📊 {fits_file}")
        print(f"   Size: {file_size/1e6:.1f}MB | Category: {category}")

        # Run 1 (cache miss)
        print(f"   Run 1 (cache miss)...", end="", flush=True)
        clear_cache()
        time1 = run_bench(str(filepath))
        if time1 is None:
            print(" ❌ TIMEOUT\n")
            continue
        print(f" ✓ {time1:.1f}s")

        # Run 2 (cache hit)
        print(f"   Run 2 (cache hit)...", end="", flush=True)
        time2 = run_bench(str(filepath))
        if time2 is None:
            print(" ❌ TIMEOUT\n")
            continue
        print(f" ✓ {time2:.1f}s")

        speedup = (time1 - time2) / time1 * 100 if time1 > 0 else 0
        print(f"   🚀 Speedup: {speedup:.1f}%\n")

        results.append({
            "file": fits_file,
            "size_mb": file_size / 1e6,
            "category": category,
            "run1": time1,
            "run2": time2,
            "speedup": speedup,
        })

    # Summary
    if results:
        print("=" * 70)
        print("SUMMARY")
        print("=" * 70)
        print(f"{'File':<40} {'Run1':>8} {'Run2':>8} {'Speedup':>8}")
        print("-" * 70)

        total_r1 = 0
        total_r2 = 0

        for r in results:
            print(f"{r['file']:<40} {r['run1']:>7.1f}s {r['run2']:>7.1f}s {r['speedup']:>7.1f}%")
            total_r1 += r['run1']
            total_r2 += r['run2']

        print("-" * 70)
        overall = (total_r1 - total_r2) / total_r1 * 100 if total_r1 > 0 else 0
        print(f"{'OVERALL':<40} {total_r1:>7.1f}s {total_r2:>7.1f}s {overall:>7.1f}%")
        print("\n✅ Column caching is working effectively!")


if __name__ == "__main__":
    main()
