#!/bin/bash
# Local benchmarking guide for HEALPix Plotter
# 
# This script provides easy commands to run different benchmark suites locally
# without needing to remember the full cargo commands.

set -e

BOLD='\033[1m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'  # No Color

show_usage() {
    cat << EOF
${BOLD}HEALPix Plotter - Local Benchmarking Guide${NC}

Usage: ./benches/run_benchmarks.sh [command]

${BOLD}Commands:${NC}

  all              Run all benchmark suites (comprehensive)
  e2e              Run end-to-end Hyperfine benchmarks only
  criterion        Run Criterion micro-benchmarks with HTML reports
  divan            Run Divan quick benchmarks (fastest)
  
${BOLD}Examples:${NC}

  # Run all benchmarks to get comprehensive view
  ./benches/run_benchmarks.sh all
  
  # Run just the end-to-end tests (10-15 minutes)
  ./benches/run_benchmarks.sh e2e
  
  # Run quick micro-benchmarks (1-2 minutes)
  ./benches/run_benchmarks.sh criterion
  
  # Run lightning-fast micro-benchmarks (30 seconds)
  ./benches/run_benchmarks.sh divan

${BOLD}Output Locations:${NC}

  End-to-end results:  /tmp/hyperfine_results.{json,md}
  Criterion reports:   target/criterion/report/index.html
  Divan output:        stdout (cycle-accurate metrics)

${BOLD}Performance Book Reference:${NC}

  See https://nnethercote.github.io/perf-book/ for:
  - When to use which benchmarking tool
  - How to interpret results
  - Statistical analysis of performance data
  - Avoiding common pitfalls

EOF
}

if [ -z "$1" ] || [ "$1" == "help" ]; then
    show_usage
    exit 0
fi

# Check if binary is built
if [ ! -f "target/release/map2fig" ]; then
    echo "Building release binary..."
    cargo build --release
fi

case "$1" in
    all)
        echo -e "${BOLD}Running comprehensive benchmark suite...${NC}\n"
        
        echo -e "${BLUE}1. Divan micro-benchmarks (fast)${NC}"
        cargo bench --bench divan_benchmarks --release
        echo ""
        
        echo -e "${BLUE}2. Criterion micro-benchmarks (detailed)${NC}"
        cargo bench --bench criterion_benchmarks || true
        echo ""
        
        echo -e "${BLUE}3. Hyperfine end-to-end benchmarks (statistical)${NC}"
        bash benches/hyperfine_benchmarks.sh
        echo ""
        
        echo -e "${GREEN}✓ All benchmarks complete!${NC}"
        echo -e "\nResults available at:"
        echo "  - Hyperfine: /tmp/hyperfine_results.md"
        echo "  - Criterion: target/criterion/report/index.html"
        ;;
        
    e2e)
        echo -e "${BOLD}Running end-to-end Hyperfine benchmarks...${NC}\n"
        bash benches/hyperfine_benchmarks.sh
        echo -e "\n${GREEN}✓ Results saved to /tmp/hyperfine_results.md${NC}"
        ;;
        
    criterion)
        echo -e "${BOLD}Running Criterion micro-benchmarks...${NC}\n"
        cargo bench --bench criterion_benchmarks
        echo -e "\n${GREEN}✓ HTML report: target/criterion/report/index.html${NC}"
        ;;
        
    divan)
        echo -e "${BOLD}Running Divan quick benchmarks...${NC}\n"
        cargo bench --bench divan_benchmarks --release
        echo -e "\n${GREEN}✓ Complete (check output for cycle counts)${NC}"
        ;;
        
    *)
        echo "Unknown command: $1"
        show_usage
        exit 1
        ;;
esac
