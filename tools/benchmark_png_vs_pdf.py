#!/usr/bin/env python3
"""
PNG vs PDF Rendering Benchmark
Tests rendering output format performance to identify rendering bottlenecks.
"""

import subprocess
import sys
import time
from pathlib import Path
import re

def parse_time_output(stderr):
    """Parse /usr/bin/time -v output for wall clock time and memory."""
    wall_time = None
    peak_rss = None
    
    for line in stderr.split("\n"):
        if "Elapsed (wall clock) time" in line:
            match = re.search(r"(\d+):(\d+\.\d+)", line)
            if match:
                minutes = int(match.group(1))
                seconds = float(match.group(2))
                wall_time = minutes * 60 + seconds
        elif "Maximum resident set size" in line:
            match = re.search(r"(\d+)", line)
            if match:
                peak_rss = int(match.group(1)) / 1024  # KB to MB

    return wall_time, peak_rss


def benchmark_format(fits_file, output_format, width=512, num_runs=2):
    """
    Benchmark a specific output format.
    
    Returns list of (wall_time, peak_rss_mb) tuples for each run.
    """
    results = []
    extension = ".png" if output_format == "png" else ".pdf"
    
    print(f"  📊 {output_format.upper()} format ({width}px):")
    
    for run_num in range(1, num_runs + 1):
        output_file = f"/tmp/bench_{output_format}_{run_num}{extension}"
        
        cmd = [
            "/usr/bin/time",
            "-v",
            "cargo",
            "run",
            "--release",
            "--",
            "-f",
            fits_file,
            "-o",
            output_file,
            "-w",
            str(width),
        ]
        
        print(f"    Run {run_num}...", end="", flush=True)
        
        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                cwd="/home/dwatts/projects/healpix_plotter",
                timeout=300,
            )
            
            wall_time, peak_rss = parse_time_output(result.stderr)
            
            if wall_time is None:
                print(" ❌ Failed to parse output")
                return None
            
            print(f" ✓ {wall_time:.2f}s, {peak_rss:.0f}MB")
            results.append((wall_time, peak_rss))
            
        except subprocess.TimeoutExpired:
            print(" ⏱️  TIMEOUT")
            return None
        except Exception as e:
            print(f" ❌ Error: {e}")
            return None
    
    return results


def main():
    project_root = Path("/home/dwatts/projects/healpix_plotter")
    fits_file = project_root / "combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits"
    
    if not fits_file.exists():
        print(f"❌ Test file not found: {fits_file}")
        return 1
    
    print("\n" + "=" * 80)
    print("PNG vs PDF Rendering Benchmark")
    print("=" * 80 + "\n")
    
    print(f"File: {fits_file.name}")
    print(f"Size: 3.1 GB\n")
    
    # Test parameters
    widths = [512, 1200]
    formats = ["png", "pdf"]
    
    all_results = {}
    
    for width in widths:
        print(f"🎯 Output width: {width}px\n")
        all_results[width] = {}
        
        for fmt in formats:
            results = benchmark_format(str(fits_file), fmt, width, num_runs=2)
            
            if results is None:
                print(f"  ⚠️  {fmt.upper()} benchmarking failed\n")
                all_results[width][fmt] = None
                continue
            
            all_results[width][fmt] = results
            
        print()
    
    # Summary
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80 + "\n")
    
    for width in widths:
        print(f"Resolution: {width}px")
        print("-" * 60)
        
        row_data = {}
        for fmt in formats:
            if all_results[width][fmt] is None:
                continue
            
            times = [t for t, _ in all_results[width][fmt]]
            avg_time = sum(times) / len(times)
            row_data[fmt] = avg_time
            
            print(f"  {fmt.upper():<6} Run 1: {times[0]:>6.2f}s  |  Run 2: {times[1]:>6.2f}s  |  Avg: {avg_time:.2f}s")
        
        if "png" in row_data and "pdf" in row_data:
            png_time = row_data["png"]
            pdf_time = row_data["pdf"]
            
            if png_time < pdf_time:
                speedup = (pdf_time - png_time) / pdf_time * 100
                print(f"\n  ⚡ PNG is {speedup:.1f}% faster than PDF")
            else:
                overhead = (png_time - pdf_time) / pdf_time * 100
                print(f"\n  📄 PNG is {overhead:.1f}% slower than PDF")
        
        print()
    
    # Analysis
    print("=" * 80)
    print("ANALYSIS")
    print("=" * 80 + "\n")
    
    has_png = any(all_results[w].get("png") is not None for w in widths)
    has_pdf = any(all_results[w].get("pdf") is not None for w in widths)
    
    if has_png and has_pdf:
        # Compare average times
        png_times = []
        pdf_times = []
        
        for width in widths:
            if all_results[width]["png"]:
                png_times.extend([t for t, _ in all_results[width]["png"]])
            if all_results[width]["pdf"]:
                pdf_times.extend([t for t, _ in all_results[width]["pdf"]])
        
        if png_times and pdf_times:
            avg_png = sum(png_times) / len(png_times)
            avg_pdf = sum(pdf_times) / len(pdf_times)
            
            print("Key Insights:")
            print("-" * 60)
            
            if avg_png < avg_pdf * 0.9:
                pct_faster = (avg_pdf - avg_png) / avg_pdf * 100
                print(f"✅ PNG is significantly faster ({pct_faster:.1f}%)")
                print("   → Rendering bottleneck is NOT image rasterization")
                print("   → Bottleneck is PDF-specific (Cairo PDF backend)")
                print("   → Future optimization: Could offer PNG for iterative work")
            elif avg_png > avg_pdf * 1.1:
                pct_slower = (avg_png - avg_pdf) / avg_pdf * 100
                print(f"⚠️  PNG is slower ({pct_slower:.1f}%)")
                print("   → Rendering bottleneck is likely image rasterization")
                print("   → Both formats hit same wall; optimization focus elsewhere")
            else:
                print(f"≈ PNG and PDF similar performance ({abs(avg_png - avg_pdf) / avg_pdf * 100:.1f}% difference)")
                print("   → Shows that base rasterization is the cost")
                print("   → Suggests both use similar image pipeline")
    
    print("\n" + "=" * 80 + "\n")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
