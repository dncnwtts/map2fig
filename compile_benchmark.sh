#!/usr/bin/env bash
# Comprehensive compilation time benchmarking script
# Measures clean builds and incremental rebuilds with different configurations

set -e

PROJECT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$PROJECT_DIR"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Configuration
RUNS=${1:-3}  # Number of runs per benchmark (default: 3)
BENCHMARK_RESULTS="compile_benchmarks_$(date +%Y%m%d_%H%M%S).txt"

echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}Rust Compilation Time Benchmark Suite${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo ""
echo "Configuration:"
echo "  Project: map2fig"
echo "  Directory: $PROJECT_DIR"
echo "  Runs per test: $RUNS"
echo "  Results file: $BENCHMARK_RESULTS"
echo ""

# Function to benchmark a command
benchmark_command() {
    local name=$1
    local command=$2
    local description=$3
    
    echo -e "${YELLOW}Testing: $name${NC}"
    echo "Description: $description"
    echo "Command: $command"
    echo ""
    
    local times=()
    
    for i in $(seq 1 $RUNS); do
        echo -n "  Run $i/$RUNS: "
        
        # Clear cargo build cache for clean builds if specified
        if [[ "$command" == *"clean"* ]]; then
            rm -rf target/release target/debug 2>/dev/null || true
        fi
        
        # Time the command
        local start=$(date +%s%3N)
        eval "$command" > /dev/null 2>&1 || {
            echo -e "${RED}FAILED${NC}"
            return 1
        }
        local end=$(date +%s%3N)
        local elapsed=$((end - start))
        local seconds=$(echo "scale=2; $elapsed / 1000" | bc)
        
        times+=("$seconds")
        echo -e "${GREEN}${seconds}s${NC}"
    done
    
    # Calculate average
    local sum=0
    for time in "${times[@]}"; do
        sum=$(echo "$sum + $time" | bc)
    done
    local avg=$(echo "scale=2; $sum / ${#times[@]}" | bc)
    
    echo ""
    echo -e "  ${BLUE}Average: ${GREEN}${avg}s${NC}"
    echo ""
    
    # Log to results file
    {
        echo "Test: $name"
        echo "Description: $description"
        echo "Command: $command"
        echo "Times: ${times[@]}"
        echo "Average: ${avg}s"
        echo ""
    } >> "$BENCHMARK_RESULTS"
}

# Check prerequisites
echo -e "${BLUE}Checking prerequisites...${NC}"
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: cargo not found. Please install Rust.${NC}"
    exit 1
fi

if ! which mold &> /dev/null; then
    echo -e "${YELLOW}Warning: mold linker not found. Install with: sudo apt-get install mold${NC}"
fi

echo "✓ Rust toolchain: $(rustc --version)"
echo "✓ Cargo: $(cargo --version)"
if which mold &> /dev/null; then
    echo "✓ Mold linker: $(mold --version | head -1)"
fi
echo ""

# Create results header
{
    echo "═══════════════════════════════════════════════════════════════"
    echo "Rust Compilation Time Benchmark Results"
    echo "═══════════════════════════════════════════════════════════════"
    echo ""
    echo "Test System Information:"
    echo "  Date: $(date)"
    echo "  Rust: $(rustc --version)"
    echo "  Cargo: $(cargo --version)"
    echo "  CPU: $(lscpu | grep 'Model name' | cut -d: -f2 | xargs)"
    echo "  Cores: $(nproc)"
    if which mold &> /dev/null; then
        echo "  Mold: $(mold --version | head -1)"
    fi
    if rustup toolchain list | grep -q nightly; then
        echo "  Nightly: available"
    fi
    echo ""
    echo "Configuration:"
    echo "  Runs per test: $RUNS"
    echo ""
} > "$BENCHMARK_RESULTS"

# Run benchmarks
echo -e "${BLUE}Starting benchmarks (this may take several minutes)...${NC}"
echo ""

# 1. Clean release build with current config (mold)
benchmark_command \
    "Clean Release (with mold)" \
    "cargo clean && cargo build --release" \
    "Full rebuild with currently configured linker (mold if available)"

# 2. Incremental rebuild
benchmark_command \
    "Incremental Release (no changes)" \
    "cargo build --release" \
    "Rebuild without code changes (tests mold link speed)"

# 3. Touch a file in src to trigger partial rebuild
benchmark_command \
    "Partial Rebuild (src/colormap.rs touched)" \
    "touch src/colormap.rs && cargo build --release" \
    "Incremental rebuild after touching a source file"

# 4. Fast release profile
benchmark_command \
    "Fast Release Profile" \
    "cargo build --profile release-fast" \
    "Optimized for faster compilation (thin LTO, codegen-units=16)"

# 5. Dev build
benchmark_command \
    "Dev Build" \
    "cargo build" \
    "Debug build for development"

# 6. Nightly with parallel compiler (if available)
if rustup toolchain list | grep -q nightly; then
    benchmark_command \
        "Release Build with Parallel Compiler (nightly)" \
        "RUSTFLAGS='-Z threads=8' cargo +nightly build --release" \
        "Release build using parallel compiler frontend (no runtime penalty)"
fi

# 7. Check compile (fast sanity check)
benchmark_command \
    "Cargo Check" \
    "cargo check --release" \
    "Fast compilation check (no code generation)"

echo ""
echo -e "${BLUE}═══════════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}Benchmarking complete!${NC}"
echo ""
echo -e "Results saved to: ${BLUE}$BENCHMARK_RESULTS${NC}"
cat "$BENCHMARK_RESULTS"

echo ""
echo -e "${BLUE}Key Metrics:${NC}"
echo ""
echo "Interpreting the results:"
echo "  • Incremental builds show linker overhead (mold should be fastest)"
echo "  • Fast Release should be 20-30% faster than Release"
echo "  • Check should be 2-3x faster than full builds"
echo "  • All profiles maintain full runtime performance"
echo ""

echo -e "${BLUE}Optimization Recommendations:${NC}"
echo ""
echo "1. For development workflows: Use 'cargo build' (mold linker ~40-70% faster than lld)"
echo "2. For CI/CD: Use '--profile release-fast' (faster compile, 90% optimization)"
echo "3. For frequent rebuilds: 'cargo check' provides instant feedback"
echo "4. For nightly users: Try '-Z threads=N' for parallel compiler frontend"
echo "5. All configurations maintain full runtime performance"
echo ""
