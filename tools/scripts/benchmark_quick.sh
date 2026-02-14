#!/bin/bash
# Quick performance benchmark script for map2fig
# Compares current branch against main across standard test configurations
# 
# Usage: ./benchmark_quick.sh [branch_name] [fits_file] [note]
#   branch_name: git branch to benchmark (optional, defaults to current)
#   fits_file: FITS file to use (optional, defaults to cosmoglobe_clipped.fits)
#   note: optional label for results (e.g., "Phase 5.2", "Streaming I/O")

set -e

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$( dirname "$(dirname "$SCRIPT_DIR")" )"
BINARY="$PROJECT_ROOT/target/release/map2fig"

# Configuration
BRANCH_TO_TEST="${1:-}"
FITS_FILE="${2:-$PROJECT_ROOT/cosmoglobe_clipped.fits}"
NOTE="${3:-}"

# Default to current branch if not specified
if [ -z "$BRANCH_TO_TEST" ]; then
    BRANCH_TO_TEST=$(cd "$PROJECT_ROOT" && git rev-parse --abbrev-ref HEAD)
fi

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Verify binary exists
if [ ! -f "$BINARY" ]; then
    echo -e "${RED}Error: Binary not found at $BINARY${NC}"
    echo "Run: cargo build --release"
    exit 1
fi

# Verify FITS file exists
if [ ! -f "$FITS_FILE" ]; then
    echo -e "${RED}Error: FITS file not found: $FITS_FILE${NC}"
    exit 1
fi

FITS_NAME=$(basename "$FITS_FILE")
FITS_SIZE=$(du -h "$FITS_FILE" | cut -f1)

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC} MAP2FIG QUICK PERFORMANCE BENCHMARK${NC}${BLUE}                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${CYAN}Configuration:${NC}"
echo -e "  Branch:   ${YELLOW}$BRANCH_TO_TEST${NC}"
echo -e "  FITS:     ${YELLOW}$FITS_NAME${NC} (${YELLOW}$FITS_SIZE${NC})"
echo -e "  Binary:   ${YELLOW}$(basename $BINARY)${NC}"
if [ -n "$NOTE" ]; then
    echo -e "  Note:     ${YELLOW}$NOTE${NC}"
fi
echo ""

# Define test configurations
declare -a TESTS=(
    "Linear 512|512|--width 512"
    "Linear 1200|1200|--width 1200"
    "Log 512|512|--width 512 --log --min 0.1"
    "Log 1200|1200|--width 1200 --log --min 0.1"
)

# Run tests and collect results
declare -A RESULTS
echo -e "${CYAN}Running benchmarks...${NC}"
echo ""

for test_spec in "${TESTS[@]}"; do
    IFS='|' read -r TEST_NAME WIDTH FLAGS <<< "$test_spec"
    
    # Run benchmark
    OUTPUT=$( { time "$BINARY" -f "$FITS_FILE" -o /tmp/bench_$WIDTH.pdf $FLAGS 2>&1; } 2>&1)
    
    # Extract time (looking for "real" line from time command)
    REAL_TIME=$(echo "$OUTPUT" | grep -oP 'real\s+0m\K[0-9]+\.[0-9]+')
    
    if [ -z "$REAL_TIME" ]; then
        REAL_TIME="ERROR"
    fi
    
    RESULTS["$TEST_NAME"]="$REAL_TIME"
    
    # Determine status indicator
    if [ "$REAL_TIME" = "ERROR" ]; then
        STATUS="${RED}✗${NC}"
    else
        STATUS="${GREEN}✓${NC}"
    fi
    
    printf "  %s ${CYAN}%-20s${NC} %s seconds\n" "$STATUS" "$TEST_NAME:" "$REAL_TIME"
done

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║${NC} RESULTS${NC}${BLUE}                                                       ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo "Copy these results into PERFORMANCE_TRACKING.md:"
echo ""

# Generate markdown table row
echo "| \`$BRANCH_TO_TEST\` | \`$FITS_NAME\` | ${RESULTS["Linear 512"]} | ${RESULTS["Linear 1200"]} | ${RESULTS["Log 512"]} | ${RESULTS["Log 1200"]} |"
if [ -n "$NOTE" ]; then
    echo "| | | | | *Note: $NOTE* |"
fi

echo ""
echo -e "${CYAN}Output files:${NC}"
echo "  /tmp/bench_512.pdf (linear and log 512)"
echo "  /tmp/bench_1200.pdf (linear and log 1200)"
