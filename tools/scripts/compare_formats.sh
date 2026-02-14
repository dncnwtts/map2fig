#!/bin/bash

# Compare PNG vs PDF rendering performance from map2fig
# Tests whether output format (Cairo vs image crate) is the bottleneck

set -e

PROJECT_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}")/.." && pwd )"
BINARY="$PROJECT_ROOT/target/release/map2fig"
TEST_FILE="$PROJECT_ROOT/tests/data/class_dr1_40GHz_skymap_n128.fits"

if [ ! -f "$BINARY" ]; then
    echo "Building release binary..."
    cd "$PROJECT_ROOT"
    cargo build --release
fi

if [ ! -f "$TEST_FILE" ]; then
    echo "Error: Test file not found: $TEST_FILE"
    exit 1
fi

echo "========================================="
echo "Output Format Performance Comparison"
echo "========================================="
echo ""
echo "Test File: $(basename $TEST_FILE)"
echo "Binary: $(basename $BINARY)"
echo ""

# Run multiple times to reduce variance
RUNS=3

echo "Linear scale (3 runs):"
echo ""

total_pdf=0
total_png=0

for i in $(seq 1 $RUNS); do
    echo "Run $i:"
    
    echo -n "  PDF: "
    pdf_time=$( { time $BINARY -f "$TEST_FILE" -o /tmp/test_run${i}.pdf > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}')
    echo "$pdf_time"
    
    echo -n "  PNG: "
    png_time=$( { time $BINARY -f "$TEST_FILE" --format png -o /tmp/test_run${i}.png > /dev/null 2>&1; } 2>&1 | grep real | awk '{print $2}')
    echo "$png_time"
    echo ""
done

echo "========================================="
echo "Analysis"
echo "========================================="
echo ""
echo "If PDF and PNG times are within 5%, then:"
echo "  ✓ Output format is NOT the bottleneck"
echo "  ✓ Rasterization (Cairo/image) is balanced"
echo "  ✓ Focus optimization on projection/scaling math"
echo ""
echo "If PNG is significantly faster:"
echo "  ✗ Cairo rendering may be overhead"
echo "  ✗ Consider rasterization redesign instead"
echo ""

# Test file sizes
pdf_size=$(du -h /tmp/test_run1.pdf 2>/dev/null | cut -f1)
png_size=$(du -h /tmp/test_run1.png 2>/dev/null | cut -f1)

if [ -n "$pdf_size" ] && [ -n "$png_size" ]; then
    echo "Output file sizes:"
    echo "  PDF: $pdf_size"
    echo "  PNG: $png_size"
    echo ""
fi

echo "Run the full suite with:"
echo "  ./tools/scripts/profile.sh"
echo ""
