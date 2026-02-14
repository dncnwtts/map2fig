#!/usr/bin/env python3
"""
Column Data I/O Profiler for HEALPix Plotter

Profiles the column data reading performance to identify I/O bottlenecks.
Measures the time spent reading FITS column data vs. other operations.

Usage:
    export MAP2FIG_PROFILE=1
    python3 profile_columns.py <fits_file> [--runs=N] [--col=INDEX]
"""

import subprocess
import sys
import time
import re
from pathlib import Path
from statistics import mean, stdev

def extract_timing_from_run(fits_file: str, col_idx: int = 0, output_res: int = 512) -> dict:
    """
    Run a single render and extract performance timing.
    """
    try:
        # Run the program with time prefix to capture timing
        result = subprocess.run(
            ['time', '-v', 'cargo', 'run', '--release', '--',
             '-f', fits_file,
             '-o', f'/tmp/profile_col_out_{int(time.time())}.pdf',
             '--width', str(output_res),
             '-i', str(col_idx)],
            capture_output=True,
            text=True,
            timeout=300,
            cwd='/home/dwatts/projects/healpix_plotter',
            env={**dict(__import__('os').environ), 'MAP2FIG_PROFILE': '1'}
        )
        
        output_text = result.stderr + result.stdout
        
        # Extract relevant timings
        data = {
            'returncode': result.returncode,
            'stdout': result.stdout,
            'stderr': result.stderr,
            'user_time': 0.0,
            'wall_time': 0.0,
            'max_rss': 0,
            'cache_ops': {'hits': 0, 'misses': 0},
            'fits_parse_ms': 0.0,
        }
        
        # Try to parse timing information
        for line in output_text.split('\n'):
            # Parse time -v output
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
            
            # Parse diagnostics
            if '[I/O DIAG] Cache HIT' in line:
                data['cache_ops']['hits'] += 1
            elif '[I/O DIAG] Cache MISS' in line:
                data['cache_ops']['misses'] += 1
            elif 'FITS parse took' in line:
                match = re.search(r'took\s+([\d.]+)ms', line)
                if match:
                    data['fits_parse_ms'] = float(match.group(1))
        
        return data
    except subprocess.TimeoutExpired:
        print(f"ERROR: Timeout (>300s)", file=sys.stderr)
        return None
    except Exception as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return None

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 profile_columns.py <fits_file> [--runs=N] [--col=INDEX]")
        print("\nExample:")
        print("  python3 profile_columns.py combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits --runs=2")
        sys.exit(1)
    
    fits_file = sys.argv[1]
    num_runs = 1
    col_idx = 0
    
    for arg in sys.argv[2:]:
        if arg.startswith('--runs='):
            num_runs = int(arg.split('=')[1])
        elif arg.startswith('--col='):
            col_idx = int(arg.split('=')[1])
    
    # Resolve path
    fits_path = Path(fits_file)
    if not fits_path.exists():
        fits_path = Path('/home/dwatts/projects/healpix_plotter') / fits_file
        if not fits_path.exists():
            print(f"ERROR: FITS file not found: {fits_file}", file=sys.stderr)
            sys.exit(1)
    
    print(f"\n=== Column Data I/O Profile ===")
    print(f"File:      {fits_path}")
    print(f"Column:    {col_idx}")
    print(f"Runs:      {num_runs}")
    print(f"\nRunning profiling (includes compilation on first run)...\n")
    
    results = []
    for i in range(num_runs):
        print(f"[{i+1}/{num_runs}] Profiling...", end='', flush=True)
        start = time.time()
        result = extract_timing_from_run(str(fits_path), col_idx, 512)
        elapsed = time.time() - start
        
        if result is None:
            print(" FAILED")
            continue
        
        if result['returncode'] != 0:
            print(f" FAILED")
            continue
        
        results.append(result)
        print(f" OK ({elapsed:.1f}s wall, {result['user_time']:.1f}s CPU, RSS: {result['max_rss']//1024}MB)")
        
        # Show diagnostics
        cache_total = result['cache_ops']['hits'] + result['cache_ops']['misses']
        if cache_total > 0:
            hit_pct = 100.0 * result['cache_ops']['hits'] / cache_total
            print(f"     Cache: {result['cache_ops']['hits']} hits, {result['cache_ops']['misses']} misses ({hit_pct:.0f}%)")
        if result['fits_parse_ms'] > 0:
            print(f"     FITS parse: {result['fits_parse_ms']:.2f}ms")
    
    if not results:
        print("ERROR: No successful profiles", file=sys.stderr)
        sys.exit(1)
    
    # Analysis
    print(f"\n=== Analysis ({len(results)} runs) ===\n")
    
    wall_times = [r['wall_time'] for r in results if r['wall_time'] > 0]
    user_times = [r['user_time'] for r in results if r['user_time'] > 0]
    
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
        if len(user_times) > 1:
            print(f"  StdDev: {stdev(user_times):8.2f}s")
    
    # Memory analysis
    max_rss = [r['max_rss'] for r in results]
    if max_rss:
        rss_mb = [m / 1024 for m in max_rss]
        print(f"\nMemory Usage (MB):")
        print(f"  Min:    {min(rss_mb):8.1f}MB")
        print(f"  Max:    {max(rss_mb):8.1f}MB")
        print(f"  Mean:   {mean(rss_mb):8.1f}MB")
    
    # Cache effectiveness
    all_hits = sum(r['cache_ops']['hits'] for r in results)
    all_misses = sum(r['cache_ops']['misses'] for r in results)
    total_cache_ops = all_hits + all_misses
    
    if total_cache_ops > 0:
        hit_rate = 100.0 * all_hits / total_cache_ops
        print(f"\nCache Operations: {total_cache_ops} ({all_hits} hits, {all_misses} misses, {hit_rate:.0f}% hit rate)")
        
        if hit_rate > 50 and all_misses > 0:
            first_run_wall = wall_times[0] if wall_times else 0
            cached_run_wall = min(wall_times) if len(wall_times) > 1 else wall_times[0] if wall_times else 0
            
            if first_run_wall > 0 and cached_run_wall > 0:
                cache_benefit = (first_run_wall - cached_run_wall) / first_run_wall * 100
                print(f"Cache Benefit:    {cache_benefit:.1f}% speedup on cached runs")
    
    print(f"\n=== Interpretation ===\n")
    
    if wall_times and max(wall_times) > 30:
        print("⚠ Render time is significant (>30s)")
        print("  Architecture breakdown likely: I/O ~14%, compute ~38%, rendering ~48%")
        print("  Main optimization opportunities:")
        print("    1. PDF rendering (48%) - largest bottleneck")
        print("    2. Pixel operations (38%) - SIMD already optimized")
        print("    3. I/O (14%) - metadata cached, column I/O efficient")
    
    if hit_rate > 90:
        print("\n✓ Metadata caching is very effective")
        print("  Further I/O optimization should focus on:")
        print("    - Column data layout/access patterns")
        print("    - Sparse map expansion efficiency")
        print("    - Binary table streaming (if supported by fitsrs)")

if __name__ == '__main__':
    main()
