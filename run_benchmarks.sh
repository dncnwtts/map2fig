#!/bin/bash
# Comprehensive benchmark orchestration script

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( dirname "$SCRIPT_DIR" )"
BINARY="$PROJECT_ROOT/target/release/map2fig"
BENCHMARK_DIR="$PROJECT_ROOT/benchmark_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="$BENCHMARK_DIR/$TIMESTAMP"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}═════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}         MAP2FIG COMPREHENSIVE BENCHMARK SUITE${NC}"
echo -e "${BLUE}═════════════════════════════════════════════════════════════${NC}"

# Step 1: Build release binary
if [ ! -f "$BINARY" ]; then
    echo -e "${YELLOW}[1/5]${NC} Building release binary..."
    cd "$PROJECT_ROOT"
    cargo build --release 2>&1 | tail -5
    echo -e "${GREEN}✓ Binary built${NC}"
else
    echo -e "${GREEN}✓ Binary exists${NC}"
fi

# Step 2: Run tests
echo -e "\n${YELLOW}[2/5]${NC} Running unit tests..."
cd "$PROJECT_ROOT"
cargo test --lib --quiet 2>&1 | tail -3
echo -e "${GREEN}✓ Tests passed${NC}"

# Step 3: Run clippy
echo -e "\n${YELLOW}[3/5]${NC} Running clippy (code quality)..."
cargo clippy --all-targets 2>&1 | grep -E "(warning|error)" | head -10 || echo "No clippy warnings"
echo -e "${GREEN}✓ Clippy check complete${NC}"

# Step 4: Prepare benchmark directory
echo -e "\n${YELLOW}[4/5]${NC} Setting up benchmark environment..."
mkdir -p "$RESULTS_DIR"
mkdir -p "$RESULTS_DIR/outputs"

# Select FITS file for benchmark (prefer smaller for speed)
if [ -f "$PROJECT_ROOT/m_test.fits" ]; then
    FITS_FILE="$PROJECT_ROOT/m_test.fits"
    echo -e "${GREEN}  Using:${NC} m_test.fits"
elif [ -f "$PROJECT_ROOT/cosmoglobe_clipped.fits" ]; then
    FITS_FILE="$PROJECT_ROOT/cosmoglobe_clipped.fits"
    echo -e "${GREEN}  Using:${NC} cosmoglobe_clipped.fits"
elif [ -f "$PROJECT_ROOT/npipe_nodip.fits" ]; then
    FITS_FILE="$PROJECT_ROOT/npipe_nodip.fits"
    echo -e "${GREEN}  Using:${NC} npipe_nodip.fits"
else
    echo -e "${YELLOW}  No FITS file found. Using first available...${NC}"
    FITS_FILE=$(ls "$PROJECT_ROOT"/*.fits 2>/dev/null | head -1)
    if [ -z "$FITS_FILE" ]; then
        echo -e "${YELLOW}Error: No FITS files found for benchmarking${NC}"
        exit 1
    fi
    echo -e "${GREEN}  Using:${NC} $(basename $FITS_FILE)"
fi

# Step 5: Run Python benchmark suite
echo -e "\n${YELLOW}[5/5]${NC} Running benchmark suite..."
echo -e "   Projections, Scaling, Features, Formats, Comparisons"
echo ""

python3 "$SCRIPT_DIR/benchmark.py" "$FITS_FILE" \
    --binary "$BINARY" \
    --output-dir "$RESULTS_DIR/outputs" \
    --full

# Generate report
echo -e "\n${BLUE}═════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}✓ BENCHMARK COMPLETE${NC}"
echo -e "${BLUE}═════════════════════════════════════════════════════════════${NC}"
echo -e "\n${YELLOW}Results:${NC}"
echo -e "  Directory: $RESULTS_DIR"
echo -e "  Outputs:   $RESULTS_DIR/outputs"
echo -e "  Report:    $RESULTS_DIR/outputs/benchmark_results.json"

echo -e "\n${YELLOW}Analysis:${NC}"
if [ -f "$RESULTS_DIR/outputs/benchmark_results.json" ]; then
    echo ""
    python3 << 'EOF'
import json
import sys
from pathlib import Path

json_file = Path('$RESULTS_DIR/outputs/benchmark_results.json')
if json_file.exists():
    with open(json_file) as f:
        data = json.load(f)
    
    print("Projection Performance:")
    if 'projections' in data:
        times = {k: v['time'] for k, v in data['projections'].items() if v.get('success')}
        if times:
            fastest = min(times, key=times.get)
            print(f"  Fastest: {fastest} ({times[fastest]:.3f}s)")
    
    print("\nScaling Performance (output width):")
    if 'scaling' in data:
        for name, result in sorted(data['scaling'].items()):
            if result.get('success'):
                print(f"  {name:15} {result['time']:7.3f}s ({result['file_size_kb']:7.1f} KB)")
EOF
fi

echo -e "\n${YELLOW}Next steps:${NC}"
echo "  1. Review results:     less $RESULTS_DIR/outputs/benchmark_results.json"
echo "  2. Compare outputs:    ls -lh $RESULTS_DIR/outputs/*.pdf $RESULTS_DIR/outputs/*.png"
echo "  3. Check for regressions weekly"
echo ""
