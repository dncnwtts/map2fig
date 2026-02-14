#!/bin/bash

# Profiling script for map2fig
# Runs comprehensive performance tests and generates a report

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( cd "$SCRIPT_DIR/../.." && pwd )"
cd "$PROJECT_ROOT"

# Get version from Cargo.toml
VERSION=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*"\([^"]*\)".*/\1/')
REPORT_FILE="perf_report_v${VERSION}.txt"
FLAMEGRAPH_FILE="flamegraph_v${VERSION}.svg"

echo "======================================"
echo "map2fig Performance Profiling v${VERSION}"
echo "======================================"
echo ""

# Ensure release build exists
if [ ! -f "$PROJECT_ROOT/target/release/map2fig" ]; then
    echo "Building release binary..."
    cargo build --release
fi

BINARY="$PROJECT_ROOT/target/release/map2fig"

# Test files
SMALL_MAP="$PROJECT_ROOT/tests/data/class_dr1_40GHz_skymap_n128.fits"
MEDIUM_MAP="$PROJECT_ROOT/tests/data/cosmoglobe_DIRBE_06_I_n00512_DR2.fits"

if [ ! -f "$SMALL_MAP" ]; then
    echo "Error: Test data not found. Expected: tests/data/class_dr1_40GHz_skymap_n128.fits"
    exit 1
fi

# Create report
{
    echo "HEALPix Plotter Performance Report"
    echo "Version: ${VERSION}"
    echo "Date: $(date)"
    echo "System: $(uname -s) $(uname -r)"
    echo "Architecture: $(uname -m)"
    echo ""
    echo "====== BASELINE TIMINGS ======"
    echo ""
    
    echo "1. Small Map (Clipped Cosmoglobe) - Default Linear Scale"
    { time $BINARY -f "$SMALL_MAP" -o /tmp/test.pdf > /dev/null 2>&1; } 2>&1 | grep real
    echo ""
    
    echo "2. Small Map - Log Scale"
    { time $BINARY -f "$SMALL_MAP" --log -o /tmp/test_log.pdf > /dev/null 2>&1; } 2>&1 | grep real
    echo ""
    
    echo "3. Small Map - Histogram Equalization"
    { time $BINARY -f "$SMALL_MAP" --hist -o /tmp/test_hist.pdf > /dev/null 2>&1; } 2>&1 | grep real
    echo ""
    
    echo "4. Small Map - Asinh Scale"
    { time $BINARY -f "$SMALL_MAP" --asinh -o /tmp/test_asinh.pdf > /dev/null 2>&1; } 2>&1 | grep real
    echo ""
    
    echo "5. Small Map - Symlog Scale"
    { time $BINARY -f "$SMALL_MAP" --symlog -o /tmp/test_symlog.pdf > /dev/null 2>&1; } 2>&1 | grep real
    echo ""
    
    if [ -f "$MEDIUM_MAP" ]; then
        echo "6. Medium Map (n=512) - Default Linear Scale"
        { time $BINARY -f "$MEDIUM_MAP" -o /tmp/test_medium.pdf > /dev/null 2>&1; } 2>&1 | grep real
        echo ""
    fi
    
    echo ""
    echo "====== PROFILING RECOMMENDATIONS ======"
    echo ""
    echo "To generate flamegraph (Linux only):"
    echo "  cargo flamegraph --bin map2fig -- -f tests/data/class_dr1_40GHz_skymap_n128.fits -o /tmp/test.pdf"
    echo "  open flamegraph.svg"
    echo ""
    
} | tee "$REPORT_FILE"

echo ""
echo "Report saved to: $REPORT_FILE"
echo ""

# Check if flamegraph is available (Linux)
if command -v cargo 2>/dev/null && cargo install --list | grep -q flamegraph; then
    read -p "Generate flamegraph? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Generating flamegraph (this may take a minute)..."
        cd "$PROJECT_ROOT"
        cargo flamegraph --bin map2fig -- -f "$SMALL_MAP" -o /tmp/test_profile.pdf
        mv flamegraph.svg "$FLAMEGRAPH_FILE"
        echo "Flamegraph saved to: $FLAMEGRAPH_FILE"
    fi
fi

echo ""
echo "✓ Profiling complete!"
echo ""
