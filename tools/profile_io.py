#!/usr/bin/env python3
"""
I/O Profile Analyzer for HEALPix Plotter

Measures cache effectiveness and I/O bottleneck contribution by analyzing
the output of the diagnostic profiling system.

Usage:
    export MAP2FIG_PROFILE=1
    python3 profile_io.py <fits_file> [--runs=N] [--output=resolution]
"""

import subprocess
import sys
import re
import time
from pathlib import Path
from statistics import mean, stdev

def run_profile(fits_file: str, output_resolution: int = 512) -> dict:
    """Run one profile iteration and extract timing information."""
    try:
        result = subprocess.run(
            [
                'cargo', 'run', '--release', '--',
                '-f', fits_file,
                '-o', f'/tmp/profile_out_{int(time.time())}.pdf',
                '--width', str(output_resolution),
            ],
            capture_output=True,
            text=True,
            timeout=300,
            cwd='/home/dwatts/projects/healpix_plotter',
            env={**dict(__import__('os').environ), 'MAP2FIG_PROFILE': '1'}
        )
        
        # Parse diagnostic output
        cache_hits = 0
        cache_misses = 0
        fits_parse_time_ms = 0.0
        
        stderr_lines = result.stderr.split('\n')
        for line in stderr_lines:
            if 'Cache HIT' in line:
                cache_hits += 1
            elif 'Cache MISS' in line:
                cache_misses += 1
            elif 'FITS parse took' in line:
                # Extract time from: "[I/O DIAG] FITS parse took 12.345ms"
                match = re.search(r'took\s+([\d.]+)ms', line)
                if match:
                    fits_parse_time_ms = float(match.group(1))
        
        return {
            'cache_hits': cache_hits,
            'cache_misses': cache_misses,
            'fits_parse_time_ms': fits_parse_time_ms,
            'stdout': result.stdout,
            'stderr': result.stderr,
            'returncode': result.returncode
        }
    except subprocess.TimeoutExpired:
        print(f"ERROR: Timeout running profile (>300s)", file=sys.stderr)
        return None
    except Exception as e:
        print(f"ERROR: Failed to run profile: {e}", file=sys.stderr)
        return None

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 profile_io.py <fits_file> [--runs=N] [--output=WIDTHxHEIGHT]")
        print("\nExample:")
        print("  python3 profile_io.py cosmoglobe_clipped.fits --runs=3 --output=512")
        sys.exit(1)
    
    fits_file = sys.argv[1]
    num_runs = 1
    output_res = 512
    
    # Parse additional arguments
    for arg in sys.argv[2:]:
        if arg.startswith('--runs='):
            num_runs = int(arg.split('=')[1])
        elif arg.startswith('--output='):
            output_res = int(arg.split('=')[1])
    
    # Verify file exists
    if not Path(fits_file).exists():
        fits_file_resolved = Path('/home/dwatts/projects/healpix_plotter') / fits_file
        if fits_file_resolved.exists():
            fits_file = str(fits_file_resolved)
        else:
            print(f"ERROR: FITS file not found: {fits_file}", file=sys.stderr)
            sys.exit(1)
    
    print(f"\n=== I/O Performance Profiling ===")
    print(f"File:      {fits_file}")
    print(f"Runs:      {num_runs}")
    print(f"Resolution: {output_res}px")
    print(f"\nRunning profiling ({num_runs} iterations; first hit = cache miss)...\n")
    
    results = []
    for i in range(num_runs):
        print(f"[{i+1}/{num_runs}] Running profile...", end='', flush=True)
        start = time.time()
        result = run_profile(fits_file, output_res)
        elapsed = time.time() - start
        
        if result is None:
            print(" FAILED")
            continue
        
        if result['returncode'] != 0:
            print(f" FAILED (exit code {result['returncode']})")
            print("STDERR:", result['stderr'][:500])
            continue
        
        results.append(result)
        print(f" OK ({elapsed:.2f}s)")
        
        # Display cache stats for this run
        hit_rate = (result['cache_hits'] / (result['cache_hits'] + result['cache_misses']) * 100) \
                   if (result['cache_hits'] + result['cache_misses']) > 0 else 0
        print(f"     Cache: {result['cache_hits']} hits, {result['cache_misses']} misses ({hit_rate:.1f}%)")
        if result['fits_parse_time_ms'] > 0:
            print(f"     FITS parse: {result['fits_parse_time_ms']:.1f}ms")
    
    if not results:
        print("ERROR: No successful profiles", file=sys.stderr)
        sys.exit(1)
    
    # Analyze results
    print(f"\n=== Summary Statistics ({len(results)} runs) ===\n")
    
    # Cache analysis
    total_hits = sum(r['cache_hits'] for r in results)
    total_misses = sum(r['cache_misses'] for r in results)
    total_cache_ops = total_hits + total_misses
    
    if total_cache_ops > 0:
        hit_rate = 100.0 * total_hits / total_cache_ops
        print(f"Cache Operations:  {total_cache_ops:4d} (Hits: {total_hits:2d}, Misses: {total_misses:2d})")
        print(f"Cache Hit Rate:    {hit_rate:6.1f}%")
    
    # FITS parsing time
    parse_times = [r['fits_parse_time_ms'] for r in results if r['fits_parse_time_ms'] > 0]
    if parse_times:
        print(f"\nFITS Parse Time:")
        print(f"  Min:    {min(parse_times):8.2f}ms")
        print(f"  Max:    {max(parse_times):8.2f}ms")
        print(f"  Mean:   {mean(parse_times):8.2f}ms")
        if len(parse_times) > 1:
            print(f"  StdDev: {stdev(parse_times):8.2f}ms")
    
    print(f"\n=== Interpretation ===\n")
    
    if hit_rate > 90:
        print("✓ Cache is highly effective (>90% hit rate)")
        print("  → FITS metadata is being cached successfully")
        print("  → I/O optimization should focus on column data, not metadata")
    elif hit_rate > 50:
        print("✓ Cache is moderately effective (>50% hit rate)")
        print("  → Most files are being cached, some misses on new files")
        print("  → Consider expanding cache scope to column metadata")
    else:
        print("⚠ Cache has low effectiveness (<50% hit rate)")
        print("  → Check if cache directory is writable: ~/.cache/map2fig/")
        print("  → May be hitting permission or storage issues")
    
    if parse_times and parse_times[0] > 100:  # First run (miss) > 100ms
        print(f"\n⚠ FITS parsing takes significant time ({parse_times[0]:.1f}ms)")
        print("  → Consider: mmap for lazy loading, column metadata caching")
        print("  → Profile column extraction separately for more insight")

if __name__ == '__main__':
    main()
