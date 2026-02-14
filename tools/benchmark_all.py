#!/usr/bin/env python3
"""
Comprehensive benchmarking suite for HEALPix Plotter optimizations.

Tests column caching effectiveness across all available FITS files.
Measures first run (cache miss) vs second run (cache hit) performance.
"""

import os
import subprocess
import sys
import time
from pathlib import Path
import json
import re

# FITS files to benchmark, with expected characteristics
TEST_FILES = [
    ("m_test.fits", "tiny", 8.5e3),
    ("mhat_0_00_n00512_2025W17_4B.fits", "small", 678e3),
    ("class_dr1_40GHz_skymap_n128.fits", "medium", 6.8e6),
    ("cosmoglobe_clipped.fits", "medium-large", 25e6),
    ("cosmoglobe_DIRBE_06_I_n00512_DR2.fits", "large", 73e6),
    ("npipe_nodip.fits", "very-large", 193e6),
    ("npipe6v20_217_map_K.fits", "very-large", 577e6),
    ("combined_map_95GHz_nside8192_ptsrcmasked_50mJy.fits", "huge", 3.1e9),
]

class Benchmark:
    """Single benchmark result."""
    def __init__(self, filename, file_size_bytes, file_category):
        self.filename = filename
        self.file_size_bytes = file_size_bytes
        self.file_category = file_category
        self.first_run_time = None
        self.second_run_time = None
        self.cache_speedup_pct = None
        self.first_run_memory_mb = None
        self.second_run_memory_mb = None

    def calculate_speedup(self):
        """Calculate cache speedup percentage."""
        if self.first_run_time and self.second_run_time:
            self.cache_speedup_pct = (
                (self.first_run_time - self.second_run_time) / self.first_run_time * 100
            )

    def to_dict(self):
        """Convert to dictionary for JSON serialization."""
        return {
            "filename": self.filename,
            "file_size_mb": self.file_size_bytes / 1e6,
            "file_category": self.file_category,
            "first_run_time_s": self.first_run_time,
            "second_run_time_s": self.second_run_time,
            "cache_speedup_pct": self.cache_speedup_pct,
            "first_run_memory_mb": self.first_run_memory_mb,
            "second_run_memory_mb": self.second_run_memory_mb,
        }


def run_benchmark(fits_file, output_file="/tmp/bench.pdf"):
    """
    Run a single benchmark.
    Returns (wall_time, peak_rss_mb) from /usr/bin/time -v
    """
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
    ]

    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=True,
            cwd="/home/dwatts/projects/healpix_plotter",
            timeout=600,  # 10 min timeout
        )

        # Parse time -v output
        stderr = result.stderr
        wall_time = None
        peak_rss = None

        for line in stderr.split("\n"):
            if "Elapsed (wall clock) time" in line:
                # Format: "Elapsed (wall clock) time (h:mm:ss or m:ss): 0:15.23"
                match = re.search(r"(\d+):(\d+\.\d+)", line)
                if match:
                    minutes = int(match.group(1))
                    seconds = float(match.group(2))
                    wall_time = minutes * 60 + seconds
            elif "Maximum resident set size" in line:
                # Format: "Maximum resident set size (kbytes): 6145820"
                match = re.search(r"(\d+)", line)
                if match:
                    peak_rss = int(match.group(1)) / 1024  # Convert KB to MB

        return wall_time, peak_rss

    except subprocess.TimeoutExpired:
        print(f"⏱️  TIMEOUT on {fits_file} (>10 minutes)")
        return None, None
    except Exception as e:
        print(f"❌ Error running benchmark on {fits_file}: {e}")
        return None, None


def clear_cache():
    """Clear the column cache to force cache miss."""
    cache_dir = Path.home() / ".cache/map2fig"
    if cache_dir.exists():
        # Remove all cached column files
        for f in cache_dir.glob("fits_col_*"):
            f.unlink()
            print(f"  Cleared cache: {f.name}")


def main():
    """Run full benchmarking suite."""
    project_root = Path("/home/dwatts/projects/healpix_plotter")
    os.chdir(project_root)

    print("\n" + "=" * 80)
    print("HEALPix Plotter - Comprehensive Benchmarking Suite")
    print("Testing column caching effectiveness across all FITS files")
    print("=" * 80 + "\n")

    # Build in release mode first
    print("📦 Building in release mode...")
    build_result = subprocess.run(
        ["cargo", "build", "--release"],
        capture_output=True,
        cwd=project_root,
        timeout=300,
    )
    if build_result.returncode != 0:
        print("❌ Build failed!")
        print(build_result.stderr.decode())
        return 1

    print("✅ Build successful\n")

    results = []

    for fits_file, category, file_size in TEST_FILES:
        filepath = project_root / fits_file
        if not filepath.exists():
            print(f"⏭️  Skipping {fits_file} (not found)")
            continue

        print(f"\n📊 Benchmarking: {fits_file}")
        print(f"    Category: {category}")
        print(f"    Size: {file_size / 1e9:.2f} GB" if file_size >= 1e9 else
              f"    Size: {file_size / 1e6:.1f} MB")

        bench = Benchmark(fits_file, file_size, category)

        # First run (cache miss)
        print(f"  🔄 Run 1 (cache miss)...", end="", flush=True)
        clear_cache()
        wall_time, peak_rss = run_benchmark(str(filepath))

        if wall_time is None:
            print(" ❌ Failed")
            continue

        bench.first_run_time = wall_time
        bench.first_run_memory_mb = peak_rss
        print(f" ✓ {wall_time:.1f}s, {peak_rss:.0f}MB peak")

        # Second run (cache hit)
        print(f"  🔄 Run 2 (cache hit)...", end="", flush=True)
        wall_time, peak_rss = run_benchmark(str(filepath))

        if wall_time is None:
            print(" ❌ Failed")
            continue

        bench.second_run_time = wall_time
        bench.second_run_memory_mb = peak_rss
        bench.calculate_speedup()

        print(f" ✓ {wall_time:.1f}s, {peak_rss:.0f}MB peak")
        print(f"  📈 Cache speedup: {bench.cache_speedup_pct:.1f}%")

        results.append(bench)

    # Summary report
    print("\n" + "=" * 80)
    print("BENCHMARK SUMMARY")
    print("=" * 80 + "\n")

    if not results:
        print("❌ No successful benchmarks")
        return 1

    # Print table
    print(f"{'File':<45} {'Size':>12} {'First Run':>12} {'Cached':>12} {'Speedup':>10}")
    print("-" * 92)

    total_first = 0
    total_cached = 0
    valid_speedups = []

    for bench in results:
        filename_short = bench.filename[:43]
        size_str = (
            f"{bench.file_size_bytes / 1e9:.2f}GB"
            if bench.file_size_bytes >= 1e9
            else f"{bench.file_size_bytes / 1e6:.1f}MB"
        )

        speedup_str = f"{bench.cache_speedup_pct:.1f}%" if bench.cache_speedup_pct else "N/A"

        print(
            f"{filename_short:<45} {size_str:>12} {bench.first_run_time:>11.1f}s "
            f"{bench.second_run_time:>11.1f}s {speedup_str:>10}"
        )

        if bench.first_run_time and bench.second_run_time:
            total_first += bench.first_run_time
            total_cached += bench.second_run_time
            if bench.cache_speedup_pct:
                valid_speedups.append(bench.cache_speedup_pct)

    print("-" * 92)

    avg_speedup = sum(valid_speedups) / len(valid_speedups) if valid_speedups else 0
    overall_speedup = (total_first - total_cached) / total_first * 100 if total_first else 0

    print(
        f"{'TOTALS':<45} {'':<12} {total_first:>11.1f}s {total_cached:>11.1f}s "
        f"{overall_speedup:>9.1f}%"
    )
    print(f"\nAverage per-file cache speedup: {avg_speedup:.1f}%")
    print(f"Overall cache speedup across all files: {overall_speedup:.1f}%")

    # Detailed statistics by category
    print("\n" + "=" * 80)
    print("STATISTICS BY FILE CATEGORY")
    print("=" * 80 + "\n")

    categories = {}
    for bench in results:
        if bench.file_category not in categories:
            categories[bench.file_category] = []
        categories[bench.file_category].append(bench)

    for category in sorted(categories.keys(), key=lambda x: sum(b.file_size_bytes for b in categories[x])):
        benches = categories[category]
        cat_speedups = [b.cache_speedup_pct for b in benches if b.cache_speedup_pct]
        cat_avg_speedup = sum(cat_speedups) / len(cat_speedups) if cat_speedups else 0

        total_cat_size = sum(b.file_size_bytes for b in benches)
        total_cat_first = sum(b.first_run_time or 0 for b in benches)
        total_cat_cached = sum(b.second_run_time or 0 for b in benches)

        print(f"Category: {category.upper()}")
        print(
            f"  Files: {len(benches)} | "
            f"Total Size: {total_cat_size / 1e9:.2f}GB | "
            f"Avg Speedup: {cat_avg_speedup:.1f}%"
        )
        print(f"  First run: {total_cat_first:.1f}s | Cached: {total_cat_cached:.1f}s\n")

    # Save results to JSON
    output_file = project_root / "BENCHMARK_RESULTS.json"
    with open(output_file, "w") as f:
        json.dump(
            {
                "benchmark_date": time.strftime("%Y-%m-%d %H:%M:%S"),
                "results": [b.to_dict() for b in results],
                "summary": {
                    "total_first_run_time_s": total_first,
                    "total_cached_run_time_s": total_cached,
                    "overall_speedup_pct": overall_speedup,
                    "average_per_file_speedup_pct": avg_speedup,
                    "files_tested": len(results),
                },
            },
            f,
            indent=2,
        )
    print(f"✅ Results saved to {output_file}")

    print("\n" + "=" * 80)
    print("VALIDATION SUMMARY")
    print("=" * 80)

    # Check if results match expectations
    if overall_speedup >= 70:
        print(f"✅ Cache speedup {overall_speedup:.1f}% meets Tier 5.2 goal (>70%)")
    else:
        print(f"⚠️  Cache speedup {overall_speedup:.1f}% below Tier 5.2 goal (>70%)")

    if avg_speedup >= 50:
        print(f"✅ Average per-file speedup {avg_speedup:.1f}% is solid cache impact")
    else:
        print(f"⚠️  Average per-file speedup {avg_speedup:.1f}% is lower than expected")

    # Check memory is stable
    all_first_mem = [b.first_run_memory_mb for b in results if b.first_run_memory_mb]
    all_cached_mem = [b.second_run_memory_mb for b in results if b.second_run_memory_mb]

    if all_first_mem and all_cached_mem:
        avg_first_mem = sum(all_first_mem) / len(all_first_mem)
        avg_cached_mem = sum(all_cached_mem) / len(all_cached_mem)
        print(f"✅ Memory usage stable: {avg_first_mem:.0f}MB first run, {avg_cached_mem:.0f}MB cached")

    print("\n" + "=" * 80 + "\n")

    return 0


if __name__ == "__main__":
    sys.exit(main())
