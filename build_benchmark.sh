#!/bin/bash
# build_benchmark.sh - Benchmark and profile Rust compilation times
# Usage: ./build_benchmark.sh [command]
# 
# Commands:
#   check       - Quick syntax check (fastest)
#   dev         - Build debug binary for development
#   release     - Full release build with maximum optimization
#   release-fast - Fast release build with moderate optimization
#   ci          - Simulate CI build (release-fast profile)
#   profile     - Profile build with detailed timings
#   compare     - Compare dev vs release-fast vs release builds
#   clean       - Clean build artifacts
#   help        - Show this help message

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_status() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_time() {
    echo -e "${YELLOW}⏱${NC}  $1"
}

benchmark_build() {
    local profile=$1
    local label=$2
    
    print_status "Benchmarking $label build..."
    echo ""
    
    local start=$(date +%s%N)
    
    if [ "$profile" = "dev" ]; then
        cargo build 2>&1 | tail -5
    elif [ "$profile" = "check" ]; then
        cargo check 2>&1 | tail -5
    else
        cargo build --profile "$profile" 2>&1 | tail -5
    fi
    
    local end=$(date +%s%N)
    local duration=$(( (end - start) / 1000000 ))  # Convert ns to ms
    local seconds=$(( duration / 1000 ))
    local ms=$(( duration % 1000 ))
    
    print_success "$label completed in ${seconds}.${ms}s"
    echo ""
}

show_help() {
    grep "^#" "$0" | grep -v "^#!/" | sed 's/^# //'
}

case "${1:-help}" in
    check)
        print_status "Running cargo check (syntax check only, no compilation)..."
        cargo check
        print_success "Syntax check complete"
        ;;
    
    dev)
        benchmark_build "dev" "Debug"
        ;;
    
    release)
        benchmark_build "release" "Release (maximum optimization, slowest compile)"
        ;;
    
    release-fast)
        benchmark_build "release-fast" "Release-fast (moderate optimization, faster compile)"
        ;;
    
    ci)
        print_status "Simulating CI build (release-fast profile)..."
        benchmark_build "release-fast" "CI build"
        ;;
    
    profile)
        print_status "Building with cargo --timings for profile analysis..."
        cargo build --release --timings
        print_success "Timings HTML report generated in target/cargo-timings/"
        
        # Show duplicate dependencies
        print_status "Checking for duplicate dependency versions..."
        cargo tree --duplicate | head -30 || echo "No duplicates found"
        
        # Show dependency count
        print_status "Dependency statistics..."
        echo "Total dependencies: $(cargo tree | wc -l)"
        ;;
    
    compare)
        print_status "Comparing build profiles (clean build)..."
        cargo clean
        echo ""
        
        print_status "1. Check (syntax only)..."
        time cargo check > /dev/null 2>&1
        
        print_status "2. Debug build..."
        cargo clean > /dev/null
        time cargo build > /dev/null 2>&1
        
        print_status "3. Release-fast build..."
        cargo clean > /dev/null
        time cargo build --profile release-fast > /dev/null 2>&1
        
        print_status "4. Release build (full optimization)..."
        cargo clean > /dev/null
        time cargo build --release > /dev/null 2>&1
        
        echo ""
        print_success "Comparison complete - check times above"
        ;;
    
    clean)
        print_status "Cleaning build artifacts..."
        cargo clean
        print_success "Clean complete"
        ;;
    
    help|--help|-h)
        show_help
        ;;
    
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        echo ""
        show_help
        exit 1
        ;;
esac
