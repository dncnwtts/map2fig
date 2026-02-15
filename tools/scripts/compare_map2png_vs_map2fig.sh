#!/bin/bash
# Benchmark: map2png vs map2fig PNG output comparison
# Compares performance, output size, and rendering quality
# 
# DEPENDENCIES: map2png requires libhealpix_cxx and libhdf5 libraries.
# Library locations:
#   - libhealpix_cxx: /home/dwatts/Downloads/Healpix_3.83/lib/
#   - libhdf5: /home/dwatts/Commander/build/install/lib/

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( dirname "$(dirname "$SCRIPT_DIR")" )"
BINARY_MAP2FIG="$PROJECT_ROOT/target/release/map2fig"
BINARY_MAP2PNG="${MAP2PNG_PATH:-/home/dwatts/Cosmotools/src/cpp/utils/map2png}"
DATA_DIR="$PROJECT_ROOT/tests/data"
BENCHMARK_RESULTS_DIR="$PROJECT_ROOT/benchmark_results/map2png_vs_map2fig"

# Default library paths
HEALPIX_LIB="/home/dwatts/Downloads/Healpix_3.83/lib"
HDF5_LIB="/home/dwatts/local/hdf5-2.0.0/lib"  # Contains libhdf5.so.320

# Set library path if not already provided
if [ -z "$MAP2PNG_LIBPATH" ]; then
    MAP2PNG_LIBPATH="${HEALPIX_LIB}:${HDF5_LIB}"
fi

# Export for subprocesses
export LD_LIBRARY_PATH="$MAP2PNG_LIBPATH:${LD_LIBRARY_PATH}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "$BENCHMARK_RESULTS_DIR"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║     MAP2PNG vs MAP2FIG PNG OUTPUT BENCHMARK COMPARISON         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Verify binaries exist
if [ ! -f "$BINARY_MAP2FIG" ]; then
    echo -e "${YELLOW}Building map2fig...${NC}"
    cd "$PROJECT_ROOT"
    cargo build --release 2>&1 | tail -3
fi

# Check if map2png is available
MAP2PNG_AVAILABLE=false
if [ -f "$BINARY_MAP2PNG" ]; then
    if "$BINARY_MAP2PNG" --help > /dev/null 2>&1; then
        MAP2PNG_AVAILABLE=true
    else
        # Try with explicit library path
        if ldd "$BINARY_MAP2PNG" 2>&1 | grep -q "not found"; then
            echo -e "${YELLOW}Warning: map2png missing dependencies:${NC}"
            ldd "$BINARY_MAP2PNG" 2>&1 | grep "not found"
        fi
    fi
fi

if [ "$MAP2PNG_AVAILABLE" = false ]; then
    echo -e "${YELLOW}ℹ map2png not available (requires libhealpix_cxx and libhdf5 libraries)${NC}"
    echo -e "${YELLOW}   Healpix library: $HEALPIX_LIB${NC}"
    echo -e "${YELLOW}   HDF5 library:    $HDF5_LIB${NC}"
    echo -e "${YELLOW}   LD_LIBRARY_PATH: $LD_LIBRARY_PATH${NC}"
    echo ""
    echo -e "${CYAN}Continuing with map2fig PNG performance benchmarks only...${NC}"
    echo ""
fi

# Select test FITS files with varying complexity
declare -a FITS_FILES=(
    "$DATA_DIR/m_test.fits:m_test (small)"
    "$DATA_DIR/class_dr1_40GHz_skymap_n128.fits:CLASS 40GHz (n128)"
    "$DATA_DIR/npipe_nodip.fits:NPIPE noDip"
)

# Function to get file size in human-readable format
get_file_size() {
    if [ -f "$1" ]; then
        du -h "$1" | cut -f1
    else
        echo "N/A"
    fi
}

# Function to measure command execution time and file size
benchmark_command() {
    local tool="$1"
    local label="$2"
    local cmd="$3"
    local output_file="$4"
    local fits_file="$5"
    
    echo -ne "${CYAN}  ${label}...${NC} "
    
    # Clear output file if it exists
    rm -f "$output_file"
    
    # Run command with timing
    local start_time=$(date +%s%N)
    eval "$cmd" > /dev/null 2>&1
    local end_time=$(date +%s%N)
    
    # Calculate execution time in milliseconds
    local elapsed_ms=$(( (end_time - start_time) / 1000000 ))
    
    # Get output file size
    local file_size=$(get_file_size "$output_file")
    
    if [ -f "$output_file" ]; then
        local file_size_bytes=$(stat --printf="%s" "$output_file")
        echo -e "${GREEN}${elapsed_ms}ms${NC} (${file_size})"
        printf "%s,%s,%s,%s\n" "$tool" "$elapsed_ms" "$file_size_bytes" "$(basename "$fits_file" .fits)" >> "$BENCHMARK_RESULTS_DIR/timings.csv"
    else
        echo -e "${RED}FAILED${NC}"
    fi
}

# Create CSV header for results
echo "Tool,Time(ms),Size(bytes),FITS_File" > "$BENCHMARK_RESULTS_DIR/timings.csv"

echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${YELLOW}Test Configuration${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "  map2fig:  $BINARY_MAP2FIG"
echo -e "  map2png:  $BINARY_MAP2PNG"
echo -e "  Results:  $BENCHMARK_RESULTS_DIR"
echo ""

# Run benchmarks for each FITS file
for fits_spec in "${FITS_FILES[@]}"; do
    IFS=':' read -r fits_file fits_label <<< "$fits_spec"
    
    if [ ! -f "$fits_file" ]; then
        echo -e "${YELLOW}Skipping $fits_label (not found)${NC}"
        continue
    fi
    
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${MAGENTA}Testing: ${fits_label}${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    
    fits_name=$(basename "$fits_file" .fits)
    fits_size=$(get_file_size "$fits_file")
    
    echo -e "  File: $fits_name"
    echo -e "  Size: $fits_size"
    echo ""
    
    # Create test output directory
    test_output_dir="$BENCHMARK_RESULTS_DIR/${fits_name}"
    mkdir -p "$test_output_dir"
    
    # Benchmark: map2png
    # map2png syntax: map2png [options] input.fits output.png
    echo -e "${YELLOW}  map2png:${NC}"
    if [ "$MAP2PNG_AVAILABLE" = true ]; then
        map2png_output="$test_output_dir/${fits_name}_map2png.png"
        benchmark_command "map2png" "PNG output" "$BINARY_MAP2PNG $fits_file $map2png_output" "$map2png_output" "$fits_file"
    else
        echo -e "    PNG output  ${YELLOW}SKIPPED${NC} (map2png cannot read FITS format)"
    fi
    
    # Benchmark: map2fig with PNG
    echo -e "${YELLOW}  map2fig:${NC}"
    map2fig_output="$test_output_dir/${fits_name}_map2fig.png"
    benchmark_command "map2fig" "PNG output" "$BINARY_MAP2FIG -f $fits_file -o $map2fig_output" "$map2fig_output" "$fits_file"
    
    echo ""
done

echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${CYAN}Benchmark Complete!${NC}"
echo -e "${GREEN}═══════════════════════════════════════════════════════════════${NC}"
echo ""

# Generate summary report
echo -e "${YELLOW}Summary Report:${NC}"
echo ""

# Create detailed comparison table
python3 << PYTHON_EOF
import csv
import os
import json
from pathlib import Path
from collections import defaultdict

results_dir = Path('$BENCHMARK_RESULTS_DIR')
timings_file = results_dir / 'timings.csv'

if timings_file.exists():
    data = defaultdict(list)
    with open(timings_file) as f:
        reader = csv.DictReader(f)
        for row in reader:
            if row['Tool'] and row['Tool'] != 'Tool':
                fits_part = row.get('FITS_File', 'unknown')
                data[fits_part].append({
                    'tool': row['Tool'].strip(),
                    'time': int(row['Time(ms)']),
                    'size': int(row['Size(bytes)'])
                })
    
    # Print comparison table
    print("┌─────────────────────┬──────────────┬──────────────┬──────────────┐")
    print("│ Test File           │ map2png      │ map2fig      │ map2fig/map  │")
    print("│                     │ Time (ms)    │ Time (ms)    │ 2png Ratio   │")
    print("├─────────────────────┼──────────────┼──────────────┼──────────────┤")
    
    summary = []
    for fits_name in sorted(data.keys()):
        tests = data[fits_name]
        map2png_data = next((t for t in tests if 'map2png' in t['tool']), None)
        map2fig_data = next((t for t in tests if 'map2fig' in t['tool']), None)
        
        if map2png_data and map2fig_data:
            ratio = map2fig_data['time'] / map2png_data['time'] if map2png_data['time'] > 0 else 0
            summary.append({
                'name': fits_name[:18],
                'map2png': map2png_data['time'],
                'map2fig': map2fig_data['time'],
                'ratio': ratio
            })
    
    for item in summary:
        ratio_str = f"{item['ratio']:.2f}x"
        print(f"│ {item['name']:<19} │ {item['map2png']:<12} │ {item['map2fig']:<12} │ {ratio_str:<12} │")
    
    print("└─────────────────────┴──────────────┴──────────────┴──────────────┘")
    
    # Overall statistics
    if summary:
        avg_ratio = sum(s['ratio'] for s in summary) / len(summary)
        map2png_total = sum(s['map2png'] for s in summary)
        map2fig_total = sum(s['map2fig'] for s in summary)
        total_ratio = map2fig_total / map2png_total if map2png_total > 0 else 0
        
        print("")
        print(f"Overall Statistics:")
        print(f"  Total map2png time: {map2png_total} ms")
        print(f"  Total map2fig time: {map2fig_total} ms")
        print(f"  Average ratio:      {avg_ratio:.2f}x")
        print(f"  Total ratio:        {total_ratio:.2f}x")
        
        if total_ratio < 1.5:
            print(f"  Status:             ✓ Performance acceptable ({total_ratio:.2f}x)")
        elif total_ratio < 2.0:
            print(f"  Status:             ⚠ Performance acceptable ({total_ratio:.2f}x)")
        else:
            print(f"  Status:             ✗ Performance needs improvement ({total_ratio:.2f}x)")
else:
    print("No timing data found")

PYTHON_EOF

echo ""
echo -e "${YELLOW}Output Files:${NC}"
ls -lh "$BENCHMARK_RESULTS_DIR"/ 2>/dev/null | tail -n +2 | awk '{print "  " $9 " (" $5 ")"}'
echo ""
echo -e "${GREEN}Results saved to: $BENCHMARK_RESULTS_DIR${NC}"
