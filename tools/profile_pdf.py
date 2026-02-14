#!/usr/bin/env python3
"""
PDF Streaming Optimization Profiler

Measures memory and time breakdown during PDF rendering to identify optimization opportunities.
Profiles the Cairo PDF pipeline to understand where buffering occurs.

Usage:
    python3 profile_pdf.py <fits_file> [--runs=N] [--output=WIDTHxHEIGHT]
"""

import subprocess
import sys
import re
import time
from pathlib import Path
from statistics import mean, stdev

def extract_timing_from_run(fits_file: str, output_res: int = 512) -> dict:
    """
    Run one profile iteration with /usr/bin/time -v for detailed resource usage.
    """
    try:
        # Build the command
        cmd = ['time', '-v', 'cargo', 'run', '--release', '--',
               '-f', fits_file,
               '-o', f'/tmp/profile_pdf_{int(time.time())}.pdf',
               '--width', str(output_res)]
        
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            timeout=300,
            cwd='/home/dwatts/projects/healpix_plotter',
        )
        
        output_text = result.stderr + result.stdout
        
        data = {
            'returncode': result.returncode,
            'user_time': 0.0,
            'wall_time': 0.0,
            'max_rss': 0,
            'io_input_reads': 0,
            'io_output_writes': 0,
        }
        
        # Parse time -v output
        for line in output_text.split('\n'):
            if 'User time (seconds):' in line:
                try:
                    data['user_time'] = float(line.split(':')[1].strip())
                except:
                    pass
            elif 'Elapsed (wall clock) time' in line:
                parts = line.split(':')[1].strip().split(':')
                try:
                    if len(parts) == 3:  # MM:SS.SS
                        m, s = int(parts[0]), float(parts[1] + '.' + parts[2])
                        data['wall_time'] = m * 60 + s
                    elif len(parts) == 2:  # SS.SS
                        data['wall_time'] = float(parts[1])
                except:
                    pass
            elif 'Maximum resident set size' in line:
                try:
                    data['max_rss'] = int(line.split(':')[1].strip())
                except:
                    pass
            elif 'File system inputs:' in line:
                try:
                    data['io_input_reads'] = int(line.split(':')[1].strip())
                except:
                    pass
            elif 'File system outputs:' in line:
                try:
                    data['io_output_writes'] = int(line.split(':')[1].strip())
                except:
                    pass
        
        return data
    except subprocess.TimeoutExpired:
        print(f"ERROR: Timeout (>300s)", file=sys.stderr)
        return None
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return None

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 profile_pdf.py <fits_file> [--runs=N] [--widths=512,1200]")
        print("\nExample:")
        print("  python3 profile_pdf.py combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits --runs=3 --widths=512,1200")
        sys.exit(1)
    
    fits_file = sys.argv[1]
    num_runs = 1
    widths = [512]
    
    # Parse additional arguments
    for arg in sys.argv[2:]:
        if arg.startswith('--runs='):
            num_runs = int(arg.split('=')[1])
        elif arg.startswith('--widths='):
            widths = [int(w) for w in arg.split('=')[1].split(',')]
    
    # Resolve path
    fits_path = Path(fits_file)
    if not fits_path.exists():
        fits_path = Path('/home/dwatts/projects/healpix_plotter') / fits_file
        if not fits_path.exists():
            print(f"ERROR: FITS file not found: {fits_file}", file=sys.stderr)
            sys.exit(1)
    
    file_size_mb = fits_path.stat().st_size / (1024**2)
    
    print(f"\n=== PDF Rendering Performance Profile ===")
    print(f"File:         {fits_path.name} ({file_size_mb:.1f} MB)")
    print(f"Runs:         {num_runs}")
    print(f"Resolutions:  {widths}")
    print(f"\nProfileing PDF rendering (includes compilation on first run)...\n")
    
    results_by_width = {}
    
    for width in widths:
        print(f"\n[Width: {width}px]")
        results = []
        
        for i in range(num_runs):
            print(f"  [{i+1}/{num_runs}] Profiling...", end='', flush=True)
            start = time.time()
            result = extract_timing_from_run(str(fits_path), width)
            elapsed = time.time() - start
            
            if result is None:
                print(" FAILED")
                continue
            
            if result['returncode'] != 0:
                print(f" FAILED")
                continue
            
            results.append(result)
            rss_gb = result['max_rss'] / (1024**2)
            print(f" OK ({elapsed:.1f}s wall, {result['user_time']:.1f}s CPU, Peak RSS: {rss_gb:.2f}GB)")
        
        results_by_width[width] = results
    
    # Analysis per width
    for width in widths:
        results = results_by_width[width]
        if not results:
            continue
        
        print(f"\n=== Analysis ({width}px, {len(results)} runs) ===\n")
        
        wall_times = [r['wall_time'] for r in results if r['wall_time'] > 0]
        user_times = [r['user_time'] for r in results if r['user_time'] > 0]
        max_rss = [r['max_rss'] for r in results]
        
        if wall_times:
            print(f"Wall Clock Time (seconds):")
            print(f"  Min:    {min(wall_times):8.2f}s")
            print(f"  Max:    {max(wall_times):8.2f}s")
            print(f"  Mean:   {mean(wall_times):8.2f}s")
            if len(wall_times) > 1:
                print(f"  StdDev: {stdev(wall_times):8.2f}s")
        
        if user_times:
            print(f"\nCPU Time (seconds):")
            print(f"  Min:    {min(user_times):8.2f}s")
            print(f"  Max:    {max(user_times):8.2f}s")
            print(f"  Mean:   {mean(user_times):8.2f}s")
        
        if max_rss:
            rss_gb = [m / (1024**2) for m in max_rss]
            print(f"\nPeak Memory (GB):")
            print(f"  Min:    {min(rss_gb):8.2f}GB")
            print(f"  Max:    {max(rss_gb):8.2f}GB")
            print(f"  Mean:   {mean(rss_gb):8.2f}GB")
        
        # Estimate breakdown
        if wall_times:
            avg_time = mean(wall_times)
            print(f"\n=== Performance Notes ===\n")
            print(f"At {width}px output resolution:")
            print(f"  - Estimated render time: {avg_time:.1f}s")
            print(f"  - Peak memory: {mean(rss_gb):.2f}GB")
            print(f"  - Memory per 100px width: {mean(rss_gb) / (width / 100):.2f}GB")
            
            if width >= 1200:
                # For large outputs, PDF overhead becomes visible
                print(f"  ⓘ Large output: PDF overhead may be significant")
            

if __name__ == '__main__':
    main()
