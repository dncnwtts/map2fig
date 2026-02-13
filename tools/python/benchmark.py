#!/usr/bin/env python3
"""
Comprehensive benchmark comparing map2fig, Cosmoglobe, and HEALPy.
Measures execution time, output quality, and memory usage.
"""

import os
import sys
import time
import subprocess
import json
from pathlib import Path
import numpy as np

try:
    import healpy as hp
    HEALPY_AVAILABLE = True
except ImportError:
    HEALPY_AVAILABLE = False

try:
    import cosmoglobe
    COSMOGLOBE_AVAILABLE = True
except ImportError:
    COSMOGLOBE_AVAILABLE = False

try:
    from astropy.io import fits
    ASTROPY_AVAILABLE = True
except ImportError:
    ASTROPY_AVAILABLE = False

# Check if map2png is available and has required libraries
MAP2PNG_PATH = "/home/dwatts/Cosmotools/src/cpp/utils/map2png"
MAP2PNG_AVAILABLE = False

try:
    # Try to run map2png with no args to check status
    result = subprocess.run([MAP2PNG_PATH], capture_output=True, text=True, timeout=2)
    # Check stdout/stderr for library error
    output = result.stdout + result.stderr
    if "cannot open shared object" in output or "libhealpix_cxx" in output:
        MAP2PNG_AVAILABLE = False
    else:
        # If no library error, map2png is available
        MAP2PNG_AVAILABLE = True
except FileNotFoundError:
    MAP2PNG_AVAILABLE = False
except subprocess.TimeoutExpired:
    # Binary ran long enough to timeout - likely OK
    MAP2PNG_AVAILABLE = True
except Exception:
    MAP2PNG_AVAILABLE = False

class BenchmarkRunner:
    def __init__(self, output_dir="benchmark_results"):
        self.output_dir = Path(output_dir)
        self.output_dir.mkdir(exist_ok=True)
        self.results = {}
        
    def get_file_size(self, filepath):
        """Get file size in KB."""
        try:
            return os.path.getsize(filepath) / 1024
        except:
            return None
    
    def benchmark_map2fig(self, fits_file, output_file, extra_args=""):
        """Benchmark map2fig execution."""
        cmd = f"./target/release/map2fig -f {fits_file} -o {output_file} {extra_args}"
        
        start = time.time()
        result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
        elapsed = time.time() - start
        
        success = result.returncode == 0
        file_size = self.get_file_size(output_file)
        
        return {
            "time": elapsed,
            "success": success,
            "file_size_kb": file_size,
            "stderr": result.stderr if not success else ""
        }
    
    def benchmark_healpy(self, fits_file, output_file):
        """Benchmark HEALPy mollview."""
        if not HEALPY_AVAILABLE:
            return {"error": "HEALPy not installed"}
        
        try:
            start = time.time()
            m = hp.read_map(fits_file, verbose=False)
            
            import matplotlib
            matplotlib.use('Agg')
            import matplotlib.pyplot as plt
            
            fig = plt.figure(figsize=(12, 6), dpi=100)
            hp.mollview(m, xsize=1200, title='', cbar=True, fig=fig.number)
            plt.savefig(output_file, dpi=100, bbox_inches='tight')
            plt.close(fig)
            
            elapsed = time.time() - start
            file_size = self.get_file_size(output_file)
            
            return {
                "time": elapsed,
                "success": True,
                "file_size_kb": file_size
            }
        except Exception as e:
            return {
                "time": None,
                "success": False,
                "error": str(e)
            }
    
    def benchmark_cosmoglobe(self, fits_file, output_file):
        """Benchmark Cosmoglobe plotting."""
        if not COSMOGLOBE_AVAILABLE:
            return {"error": "Cosmoglobe not installed"}
        
        try:
            start = time.time()
            cosmoglobe.plot(fits_file, xsize=1024)
            
            import matplotlib.pyplot as plt
            plt.savefig(output_file)
            plt.close()
            
            elapsed = time.time() - start
            file_size = self.get_file_size(output_file)
            
            return {
                "time": elapsed,
                "success": True,
                "file_size_kb": file_size
            }
        except Exception as e:
            return {
                "time": None,
                "success": False,
                "error": str(e)
            }
    
    def benchmark_map2png(self, fits_file, output_file):
        """Benchmark map2png rendering."""
        if not MAP2PNG_AVAILABLE:
            return {"error": "map2png not available or missing libraries (libhealpix_cxx)"}
        
        try:
            # map2png syntax: map2png <input> [output] [options]
            cmd = f"{MAP2PNG_PATH} {fits_file} {output_file} -xsz 1024 -mollweide"
            
            start = time.time()
            result = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=30)
            elapsed = time.time() - start
            
            success = result.returncode == 0
            file_size = self.get_file_size(output_file) if success else None
            
            return {
                "time": elapsed,
                "success": success,
                "file_size_kb": file_size,
                "stderr": result.stderr if not success else ""
            }
        except subprocess.TimeoutExpired:
            return {
                "time": None,
                "success": False,
                "error": "Timeout (>30s)"
            }
        except Exception as e:
            return {
                "time": None,
                "success": False,
                "error": str(e)
            }
    
    def run_projection_benchmark(self, fits_file, base_name="projection_test"):
        """Benchmark different projections."""
        results = {}
        
        for proj in ["mollweide", "hammer", "gnomonic"]:
            output = self.output_dir / f"{base_name}_{proj}.pdf"
            
            extra = ""
            if proj != "mollweide":
                extra = f"--projection {proj}"
            
            results[proj] = self.benchmark_map2fig(fits_file, str(output), extra)
            print(f"  {proj:12} -> {results[proj]['time']:.3f}s ({results[proj]['file_size_kb']:.1f} KB)")
        
        return results
    
    def run_scaling_benchmark(self, fits_file, base_name="scaling_test"):
        """Benchmark different output widths."""
        results = {}
        widths = [600, 1200, 1800, 2400]
        
        for w in widths:
            output = self.output_dir / f"{base_name}_{w}.pdf"
            extra = f"-w {w}"
            
            results[f"width_{w}"] = self.benchmark_map2fig(fits_file, str(output), extra)
            print(f"  width {w:4}  -> {results[f'width_{w}']['time']:.3f}s ({results[f'width_{w}']['file_size_kb']:.1f} KB)")
        
        return results
    
    def run_feature_benchmark(self, fits_file, base_name="feature_test"):
        """Benchmark different feature combinations."""
        results = {}
        features = [
            ("baseline", ""),
            ("with_graticule", "--graticule"),
            ("with_latex", "--latex --units '$K_{CMB}$'"),
            ("log_scale", "--log --min 1e-6 --max 1e-3"),
            ("hist_eq", "--hist"),
            ("all_features", "--graticule --latex --units '$K_{CMB}$' -w 1200"),
        ]
        
        for name, args in features:
            output = self.output_dir / f"{base_name}_{name}.pdf"
            
            result = self.benchmark_map2fig(fits_file, str(output), args)
            results[name] = result
            print(f"  {name:20} -> {result['time']:.3f}s ({result['file_size_kb']:.1f} KB)")
        
        return results
    
    def run_format_benchmark(self, fits_file, base_name="format_test"):
        """Benchmark PNG vs PDF output."""
        results = {}
        
        for ext in ["pdf", "png"]:
            output = self.output_dir / f"{base_name}.{ext}"
            
            result = self.benchmark_map2fig(fits_file, str(output), "")
            results[ext] = result
            print(f"  {ext.upper():3}       -> {result['time']:.3f}s ({result['file_size_kb']:.1f} KB)")
        
        return results
    
    def run_comparison_benchmark(self, fits_file, base_name="comparison"):
        """Compare map2fig vs HEALPy vs Cosmoglobe."""
        results = {}
        
        print("\n  map2fig (Mollweide, 1200x600):")
        output_m2f = self.output_dir / f"{base_name}_map2fig.pdf"
        results["map2fig"] = self.benchmark_map2fig(fits_file, str(output_m2f), "-w 1200")
        print(f"    → {results['map2fig']['time']:.3f}s ({results['map2fig']['file_size_kb']:.1f} KB)")
        
        if HEALPY_AVAILABLE:
            print("  HEALPy (mollview, 1200x600):")
            output_hp = self.output_dir / f"{base_name}_healpy.png"
            results["healpy"] = self.benchmark_healpy(fits_file, str(output_hp))
            if results["healpy"].get("success"):
                print(f"    → {results['healpy']['time']:.3f}s ({results['healpy']['file_size_kb']:.1f} KB)")
            else:
                print(f"    → FAILED ({results['healpy'].get('error', 'unknown')})")
        
        if COSMOGLOBE_AVAILABLE:
            print("  Cosmoglobe (1024x512):")
            output_cg = self.output_dir / f"{base_name}_cosmoglobe.png"
            results["cosmoglobe"] = self.benchmark_cosmoglobe(fits_file, str(output_cg))
            if results["cosmoglobe"].get("success"):
                print(f"    → {results['cosmoglobe']['time']:.3f}s ({results['cosmoglobe']['file_size_kb']:.1f} KB)")
            else:
                print(f"    → FAILED ({results['cosmoglobe'].get('error', 'unknown')})")
        
        if MAP2PNG_AVAILABLE:
            print("  map2png (Mollweide, 1024x512):")
            output_m2p = self.output_dir / f"{base_name}_map2png.png"
            results["map2png"] = self.benchmark_map2png(fits_file, str(output_m2p))
            if results["map2png"].get("success"):
                print(f"    → {results['map2png']['time']:.3f}s ({results['map2png']['file_size_kb']:.1f} KB)")
            else:
                print(f"    → FAILED ({results['map2png'].get('error', 'unknown')})")
        else:
            print("  map2png (NOT AVAILABLE - missing libhealpix_cxx.so.4)")
        
        return results
    
    def save_results(self, filename="benchmark_results.json"):
        """Save all results to JSON."""
        output_path = self.output_dir / filename
        with open(output_path, 'w') as f:
            json.dump(self.results, f, indent=2, default=str)
        print(f"\nResults saved to: {output_path}")
    
    def print_summary(self):
        """Print summary statistics."""
        print("\n" + "="*60)
        print("BENCHMARK SUMMARY")
        print("="*60)
        
        for test_name, test_results in self.results.items():
            if isinstance(test_results, dict):
                print(f"\n{test_name.upper()}:")
                for name, result in test_results.items():
                    if isinstance(result, dict) and result.get("success"):
                        print(f"  {name:20} {result['time']:7.3f}s")

def main():
    import argparse
    
    parser = argparse.ArgumentParser(description="Comprehensive benchmark suite for map2fig")
    parser.add_argument("fits_file", help="FITS file to benchmark with")
    parser.add_argument("--binary", default="./target/release/map2fig", help="Path to map2fig binary")
    parser.add_argument("--output-dir", default="benchmark_results", help="Output directory for results")
    parser.add_argument("--full", action="store_true", help="Run all benchmark suites")
    parser.add_argument("--projection-only", action="store_true", help="Only benchmark projections")
    parser.add_argument("--feature-only", action="store_true", help="Only benchmark features")
    parser.add_argument("--scaling-only", action="store_true", help="Only benchmark scaling")
    parser.add_argument("--format-only", action="store_true", help="Only benchmark formats")
    parser.add_argument("--comparison-only", action="store_true", help="Only compare with other tools")
    parser.add_argument("--map2png-only", action="store_true", help="Only benchmark map2png (comparison)")
    
    args = parser.parse_args()
    
    # Verify binary exists
    if not os.path.exists(args.binary):
        print(f"Error: Binary not found at {args.binary}")
        print("Build with: cargo build --release")
        sys.exit(1)
    
    # Verify FITS file exists
    if not os.path.exists(args.fits_file):
        print(f"Error: FITS file not found: {args.fits_file}")
        sys.exit(1)
        
    runner = BenchmarkRunner(args.output_dir)
    
    # Default: run all tests
    run_all = args.full or not any([
        args.projection_only, args.feature_only, 
        args.scaling_only, args.format_only, args.comparison_only, args.map2png_only
    ])
    
    print("="*60)
    print("MAP2FIG BENCHMARK SUITE")
    print("="*60)
    print(f"Binary: {args.binary}")
    print(f"FITS: {args.fits_file}")
    print(f"Output: {args.output_dir}")
    if MAP2PNG_AVAILABLE:
        print(f"map2png: Available ({MAP2PNG_PATH})")
    else:
        print(f"map2png: NOT AVAILABLE (missing dependencies)")
    print("="*60)
    
    if run_all or args.projection_only:
        print("\n📈 PROJECTION BENCHMARK")
        runner.results["projections"] = runner.run_projection_benchmark(args.fits_file)
    
    if run_all or args.scaling_only:
        print("\n📈 SCALING BENCHMARK (output width)")
        runner.results["scaling"] = runner.run_scaling_benchmark(args.fits_file)
    
    if run_all or args.feature_only:
        print("\n📈 FEATURE BENCHMARK")
        runner.results["features"] = runner.run_feature_benchmark(args.fits_file)
    
    if run_all or args.format_only:
        print("\n📈 FORMAT BENCHMARK (PNG vs PDF)")
        runner.results["formats"] = runner.run_format_benchmark(args.fits_file)
    
    if run_all or args.comparison_only or args.map2png_only:
        label = "map2png" if args.map2png_only else "HEALPy, Cosmoglobe, map2png"
        print(f"\n📈 COMPARISON BENCHMARK (vs {label})")
        runner.results["comparison"] = runner.run_comparison_benchmark(args.fits_file)
    
    runner.save_results()
    runner.print_summary()


if __name__ == "__main__":
    main()
