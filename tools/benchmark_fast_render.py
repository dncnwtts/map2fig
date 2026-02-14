#!/usr/bin/env python3
"""
Fast-Render Mode Benchmark
Tests rendering performance with and without vector overlays.
"""

import subprocess
import sys
import time
from pathlib import Path
import re

def parse_time_output(stderr):
    """Parse /usr/bin/time -v output."""
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


def benchmark_render_mode(fits_file, fast_render, width=512, format_ext=".pdf"):
    """Benchmark a specific render mode."""
    
    mode_name = "fast-render" if fast_render else "normal"
    output_file = f"/tmp/bench_{mode_name}_{int(time.time())}{format_ext}"
    
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
    
    if fast_render:
        cmd.append("--fast-render")
    
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            cwd="/home/dwatts/projects/healpix_plotter",
            timeout=300,
        )
        
        wall_time, peak_rss = parse_time_output(result.stderr)
        return wall_time, peak_rss
        
    except subprocess.TimeoutExpired:
        return None, None
    except Exception as e:
        print(f"Error: {e}")
        return None, None


def main():
    project_root = Path("/home/dwatts/projects/healpix_plotter")
    fits_file = project_root / "combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits"
    
    if not fits_file.exists():
        print(f"❌ Test file not found: {fits_file}")
        return 1
    
    print("\n" + "=" * 80)
    print("Fast-Render Mode Benchmark")
    print("=" * 80 + "\n")
    
    print(f"File: {fits_file.name}")
    print(f"Size: 3.1 GB\n")
    
    # Pre-warm cache
    print("📖 Pre-warming column cache...")
    subprocess.run(
        ["cargo", "run", "--release", "--", "-f", str(fits_file), "-o", "/tmp/warmup.pdf", "-w", "512"],
        capture_output=True,
        cwd=project_root,
        timeout=300,
    )
    print("   ✓ Cache warmed\n")
    
    # Benchmark parameters
    widths = [512, 1200]
    formats = [("pdf", ".pdf"), ("png", ".png")]
    modes = [
        ("normal", False),
        ("fast-render", True),
    ]
    
    results_by_format = {}
    
    for format_name, ext in formats:
        print(f"🎯 Output format: {format_name.upper()}\n")
        results_by_format[format_name] = {}
        
        for width in widths:
            print(f"  Resolution: {width}px")
            results_by_format[format_name][width] = {}
            
            for mode_name, fast_render in modes:
                print(f"    {mode_name.upper()}...", end="", flush=True)
                
                wall_time, peak_rss = benchmark_render_mode(
                    str(fits_file), 
                    fast_render, 
                    width, 
                    ext
                )
                
                if wall_time is None:
                    print(" ❌ Failed")
                    continue
                
                print(f" ✓ {wall_time:.2f}s, {peak_rss:.0f}MB")
                results_by_format[format_name][width][mode_name] = {
                    "time": wall_time,
                    "memory": peak_rss,
                }
            
            print()
    
    # Summary
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80 + "\n")
    
    for format_name in ["pdf", "png"]:
        if format_name not in results_by_format:
            continue
        
        print(f"{format_name.upper()} Format:")
        print("-" * 70)
        
        for width in [512, 1200]:
            if width not in results_by_format[format_name]:
                continue
            
            data = results_by_format[format_name][width]
            
            if "normal" in data and "fast-render" in data:
                normal_time = data["normal"]["time"]
                fast_time = data["fast-render"]["time"]
                
                speedup = (normal_time - fast_time) / normal_time * 100 if normal_time > 0 else 0
                
                print(f"  {width}px:")
                print(f"    Normal:      {normal_time:.2f}s")
                print(f"    Fast-render: {fast_time:.2f}s")
                print(f"    Speedup:     {speedup:.1f}% faster ⚡")
                print()
    
    # Analysis
    print("=" * 80)
    print("ANALYSIS")
    print("=" * 80 + "\n")
    
    # Calculate average speedups
    speedups = []
    
    for format_name in ["pdf", "png"]:
        if format_name not in results_by_format:
            continue
        
        for width in [512, 1200]:
            if width not in results_by_format[format_name]:
                continue
            
            data = results_by_format[format_name][width]
            if "normal" in data and "fast-render" in data:
                speedup = (data["normal"]["time"] - data["fast-render"]["time"]) / data["normal"]["time"] * 100
                speedups.append(speedup)
    
    if speedups:
        avg_speedup = sum(speedups) / len(speedups)
        max_speedup = max(speedups)
        min_speedup = min(speedups)
        
        print(f"Overall Performance Gains:")
        print(f"  Average speedup: {avg_speedup:.1f}%")
        print(f"  Range: {min_speedup:.1f}% - {max_speedup:.1f}%")
        print()
        
        if avg_speedup > 10:
            print(f"✅ Fast-render mode provides significant speedup ({avg_speedup:.1f}%)")
            print("   Use case: Iterative map refinement and quick previews")
        else:
            print(f"⚠️  Fast-render speedup is modest ({avg_speedup:.1f}%)")
            print("   Suggests graticule/colorbar aren't the main bottleneck")
    
    print("\n" + "=" * 80 + "\n")
    
    return 0


if __name__ == "__main__":
    sys.exit(main())
